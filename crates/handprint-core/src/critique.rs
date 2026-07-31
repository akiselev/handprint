//! Phase 5b — the machine contract for the agent-critic loop.
//!
//! [`CritiqueReport`] is a versioned JSON document, shaped after Vale and
//! textlint: namespaced rule ids, severities, byte spans, and — the part that
//! makes it usable as an objective function — an observed value, a target band
//! and a direction of change for every finding. handprint scores; the agent
//! rewrites; nothing in this crate calls a model.
//!
//! # The loop, and why it is built to resist itself
//!
//! An optimizer pointed at a scorer will optimize the scorer. Four mechanisms
//! push back, all of them cheap:
//!
//! 1. **Score everything, report a few.** Every dimension is measured; only the
//!    top [`CritiqueConfig::max_findings`] offenders are shown. The agent fixes
//!    what it can see; the gate checks what it cannot.
//! 2. **A canary family is scored and never reported.** By default the
//!    surprisal family — a distributional measure that word-swapping does not
//!    move. Reported features improving while the canary worsens is the
//!    signature of feedback overfitting, and it sets `guards.canary_ok = false`.
//! 3. **The pass gate is holistic.** It is `p_same_author >= threshold` against
//!    the calibrated same-author distribution — never "all findings cleared".
//!    Clearing every reported finding does not pass anything by itself.
//! 4. **Per-dimension contributions are capped.** No single dimension may
//!    account for more than [`CritiqueConfig::max_dim_share`] of the distance
//!    magnitude when the gate is evaluated, so one heavily-tuned dimension
//!    cannot buy a pass.
//!
//! # Which metric a mode needs
//!
//! [`Metric::CosineDelta`] is the right default for
//! *comparing two texts against a shared background*, and it is wrong for
//! corpus-mimic. When the reference is fitted on the very corpus being matched,
//! a typical document of that corpus has a z-score of roughly zero on every
//! dimension — it *is* the mean — so its vector is small noise around the
//! origin, where an angle is undefined. Two typical documents then sit at
//! cosine distance ≈ 1 from each other, the same as a document that shares
//! nothing with the corpus.
//!
//! The one-class question "how far from typical is this draft?" is a question
//! about **magnitude**, not angle, so [`Critic::corpus_mimic`] uses
//! [`Metric::BurrowsDelta`] — mean absolute
//! z-difference, which against a corpus-typical target reduces to "how many
//! standard deviations from ordinary, averaged over dimensions". Contrast mode
//! keeps the reference's own metric, because there it really is a two-class
//! comparison. Override either through [`CritiqueConfig::metric`].
//!
//! # The contract's value sets are open
//!
//! [`CONTRACT_VERSION`] pins the *shape* of the JSON: which keys exist, what
//! types they hold, what they mean. It does not pin the set of values three of
//! those keys can take.
//!
//! * [`Finding::family`] and the keys of [`DocInfo::confidence`] draw from
//!   [`Family`], which is `#[non_exhaustive]` and gains variants as feature
//!   families are added. A consumer will see `"register"`, `"rhythm"`,
//!   `"device"`, `"syntax"`, `"verse"` and whatever comes next.
//! * [`Fix::kind`] is a free `String` and always has been.
//!
//! **Consumers must tolerate unknown values in all three.** Adding a family is
//! JSON-additive and Rust-source-compatible, so it is not a contract break and
//! does not bump [`CONTRACT_VERSION`]. The one thing that *will* break is an
//! external deserializer with a closed enum over family names and
//! deny-unknown-variants semantics; this note is the disclaimer for that case.
//! Match on the families you handle, pass the rest through.
//!
//! A fifth guard is about the text rather than the score: **drift**. A rewrite
//! that deletes half the draft or changes its subject will score beautifully.
//! [`Guards::drift_ok`] compares content-word and character-n-gram overlap
//! against the original draft. It is deliberately crude — real meaning
//! preservation belongs in the eval harness with a sentence embedder, not as a
//! core dependency.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::compare::{Contribution, DistanceMetric, Metric};
use crate::error::{Error, Result};
use crate::explain::ContrastReport;
use crate::feature::lexicon::Severity;
use crate::feature::{Confidence, Family, Fitted, Unit};
use crate::reference::{BandWidth, LengthTier, Profile, Reference};
use crate::text::Document;
use crate::vector::Symbol;

/// Version of the JSON contract. Changes to it are semver-major for the crate.
///
/// Pins the document *shape*, not the value sets of `family`, `confidence` keys
/// and `fix.kind` — those are open, and new values are additive. See the
/// module docs.
pub const CONTRACT_VERSION: &str = "1";

/// Which question the critique is answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// "Does this draft fall inside the reference corpus's own variation?"
    #[serde(rename = "corpus-mimic")]
    CorpusMimic,
    /// "Move this draft away from reference A and toward reference B."
    #[serde(rename = "contrast")]
    Contrast,
}

/// Which way a dimension needs to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Observed value is above the target band.
    Reduce,
    /// Observed value is below the target band.
    Increase,
}

/// A suggested repair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fix {
    /// What kind of repair this is, e.g. `"consider_replace"`.
    pub kind: String,
    /// Candidate replacements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

/// One actionable finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    /// Namespaced rule id: `family.rule`, or `family.pack.term` for lexicon
    /// hits. Stable across versions.
    pub id: String,
    /// The feature family it came from.
    pub family: Family,
    /// How much it matters.
    pub severity: Severity,
    /// The document's value.
    pub observed: f64,
    /// The unit `observed` and `target_band` are in.
    pub unit: String,
    /// The reference's central interval for this dimension.
    pub target_band: [f64; 2],
    /// The document's z-score against the reference corpus.
    pub z: f64,
    /// Which way to move.
    pub direction: Direction,
    /// Byte spans in the submitted text, when the profile tracked them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spans: Vec<[usize; 2]>,
    /// A human- and model-readable instruction.
    pub message: String,
    /// A concrete repair, where one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

/// Which reference the critique was run against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceRef {
    /// Reference name.
    pub name: String,
    /// Artifact version.
    pub version: String,
    /// What it models.
    pub kind: String,
}

/// What was measured about the document itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocInfo {
    /// Lexical tokens.
    pub tokens: usize,
    /// The graded length tier.
    pub length_tier: LengthTier,
    /// Per-family confidence at this length, keyed by family name. **Read this
    /// before trusting any finding**: a `none` family's findings are noise at
    /// this length.
    pub confidence: std::collections::BTreeMap<String, Confidence>,
    /// Dimensions that were undefined at this length and imputed to the corpus
    /// mean.
    pub imputed_dims: usize,
}

/// The pass/fail decision and the numbers behind it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerdictBlock {
    /// Whether the draft passes.
    ///
    /// `null` means the gate could not be evaluated — almost always because the
    /// reference carries no calibration. That is a configuration error, not a
    /// failure, and it maps to exit code 2.
    pub pass: Option<bool>,
    /// A description of the gate that was applied.
    pub gate: String,
    /// `P(distance ≥ d | same author)`. High is good.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_same_author: Option<f64>,
    /// `P(distance ≤ d | different authors)`. **Not** the probability the
    /// authors differ.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_value_vs_unrelated: Option<f64>,
    /// Raw distance to the target.
    pub distance: f64,
    /// Distance after per-dimension capping, which is what the gate uses.
    pub capped_distance: f64,
    /// The metric.
    pub metric: String,
}

/// Loop integrity checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guards {
    /// False when the reported features improved while the unreported canary
    /// family got worse — the signature of optimizing the feedback rather than
    /// the style.
    pub canary_ok: bool,
    /// False when the rewrite has drifted too far from the original draft's
    /// content.
    pub drift_ok: bool,
    /// Tokenizer attacks and generation residue found in the text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_flags: Vec<String>,
    /// Anything else the caller should know.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Content overlap with the original draft, in `[0, 1]`; `null` on the
    /// first iteration, which *is* the original.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_overlap: Option<f64>,
}

/// The full report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CritiqueReport {
    /// Version of handprint that produced this.
    pub handprint: String,
    /// Version of this JSON contract.
    pub contract: String,
    /// Which reference was used.
    pub reference: ReferenceRef,
    /// Which question was asked.
    pub mode: Mode,
    /// Loop iteration, starting at 1.
    pub iteration: usize,
    /// Facts about the submitted document.
    pub doc: DocInfo,
    /// The decision.
    pub verdict: VerdictBlock,
    /// The top offenders, most severe first.
    pub findings: Vec<Finding>,
    /// How many findings there were before truncation. Everything is scored;
    /// only `findings` is reported.
    pub findings_total: usize,
    /// Loop integrity checks.
    pub guards: Guards,
}

impl CritiqueReport {
    /// Vale-style exit code: 0 pass, 1 findings, 2 error.
    pub fn exit_code(&self) -> i32 {
        match self.verdict.pass {
            None => 2,
            Some(true) => 0,
            Some(false) => 1,
        }
    }

    /// Serialize to pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("CritiqueReport is always serializable")
    }

    /// A compact human-readable summary.
    pub fn summary(&self) -> String {
        let verdict = match self.verdict.pass {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "UNEVALUATED",
        };
        let mut out = format!(
            "{verdict}  iteration {}  {} tokens  distance {:.4}",
            self.iteration, self.doc.tokens, self.verdict.distance
        );
        if let Some(p) = self.verdict.p_same_author {
            out.push_str(&format!("  p_same_author {p:.3}"));
        }
        out.push_str(&format!(
            "\n{} of {} finding(s) shown",
            self.findings.len(),
            self.findings_total
        ));
        if !self.guards.canary_ok {
            out.push_str("\n  ! canary tripped: reported features improved while the held-out family worsened");
        }
        if !self.guards.drift_ok {
            out.push_str(
                "\n  ! content drift: the rewrite has moved too far from the original draft",
            );
        }
        for finding in &self.findings {
            out.push_str(&format!(
                "\n  [{}] {}: {}",
                finding.severity.as_str(),
                finding.id,
                finding.message
            ));
        }
        out
    }
}

/// Preset threshold profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdProfile {
    /// Wide bands, a low pass bar. For a first pass over a rough draft.
    Lenient,
    /// The default.
    #[default]
    Balanced,
    /// Narrow bands, a high pass bar. For close mimicry, and slow to converge.
    Strict,
}

impl ThresholdProfile {
    /// The configuration this profile implies.
    pub fn config(self) -> CritiqueConfig {
        let base = CritiqueConfig::default();
        match self {
            ThresholdProfile::Lenient => CritiqueConfig {
                pass_threshold: 0.02,
                band: BandWidth::P05P95,
                max_findings: 5,
                min_severity: Severity::Medium,
                ..base
            },
            ThresholdProfile::Balanced => base,
            ThresholdProfile::Strict => CritiqueConfig {
                pass_threshold: 0.25,
                band: BandWidth::P25P75,
                max_findings: 10,
                min_severity: Severity::Low,
                ..base
            },
        }
    }
}

/// How to run a critique.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CritiqueConfig {
    /// Which question to answer.
    pub mode: Mode,
    /// How many findings to report per iteration.
    ///
    /// Everything is still scored. Reporting a handful is what keeps the agent
    /// working on style rather than on the report.
    pub max_findings: usize,
    /// Drop findings below this severity.
    pub min_severity: Severity,
    /// The `p_same_author` value at or above which a draft passes.
    pub pass_threshold: f64,
    /// The family that is scored but never reported. `None` disables the
    /// canary, which is not recommended for an automated loop.
    pub canary: Option<Family>,
    /// Which central interval target bands span.
    pub band: BandWidth,
    /// Largest share of the total contribution magnitude any one dimension may
    /// hold when the gate is evaluated.
    pub max_dim_share: f64,
    /// Minimum content overlap with the original draft before
    /// [`Guards::drift_ok`] goes false.
    pub min_content_overlap: f64,
    /// Metric override; `None` uses the reference's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<Metric>,
}

impl Default for CritiqueConfig {
    fn default() -> Self {
        CritiqueConfig {
            mode: Mode::CorpusMimic,
            max_findings: 7,
            min_severity: Severity::Low,
            pass_threshold: 0.10,
            canary: Some(Family::Surprisal),
            band: BandWidth::P05P95,
            max_dim_share: 0.25,
            min_content_overlap: 0.5,
            metric: None,
        }
    }
}

/// A hashed sketch of a document's content, for the drift guard.
///
/// Hashes rather than strings so a long draft costs a bounded amount of memory,
/// and so the sketch carries no recoverable text.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContentFingerprint {
    words: HashSet<u64>,
    ngrams: HashSet<u64>,
}

impl ContentFingerprint {
    /// Build a fingerprint from text under a reference's tokenizer.
    pub fn of(reference: &Reference, text: &str) -> ContentFingerprint {
        use crate::feature::mfw::FUNCTION_WORDS;
        let doc = Document::new(text);
        let analysis = doc.analyze(reference.tokenizer());
        let words: HashSet<u64> = analysis
            .tokens()
            .lexical()
            .map(|(_, f)| f)
            // Function words carry no content, so including them would make
            // every English document look similar to every other.
            .filter(|f| !FUNCTION_WORDS.contains(f) && f.chars().count() > 2)
            .map(hash)
            .collect();
        let scoring = analysis.scoring().as_str().to_lowercase();
        let chars: Vec<char> = scoring.chars().collect();
        let ngrams: HashSet<u64> = chars
            .windows(4)
            .map(|w| hash(&w.iter().collect::<String>()))
            .collect();
        ContentFingerprint { words, ngrams }
    }

    /// Jaccard overlap with another fingerprint, in `[0, 1]`.
    ///
    /// The two components are averaged: content words catch topic replacement,
    /// character n-grams catch wholesale deletion and paraphrase collapse.
    pub fn overlap(&self, other: &ContentFingerprint) -> f64 {
        (jaccard(&self.words, &other.words) + jaccard(&self.ngrams, &other.ngrams)) / 2.0
    }
}

fn jaccard(a: &HashSet<u64>, b: &HashSet<u64>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Family scores from one iteration, kept so the next one can detect
/// feedback overfitting.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Scores {
    reported: f64,
    canary: f64,
}

/// A stateful critique session.
///
/// State is what makes the guards possible: the canary compares against the
/// previous iteration, and drift compares against the first. Construct one
/// [`Critic`] per rewrite loop, not one per call.
#[derive(Debug)]
pub struct Critic<'a> {
    reference: &'a Reference,
    config: CritiqueConfig,
    targets: Vec<Profile>,
    away: Option<Profile>,
    baseline: Option<ContentFingerprint>,
    previous: Option<Scores>,
    iteration: usize,
    canary: Option<Family>,
    canary_note: Option<String>,
}

impl<'a> Critic<'a> {
    /// Critique drafts against a corpus: "does this belong here yet?"
    ///
    /// `targets` are profiles of **representative documents** from the corpus,
    /// and the reported distance is the median across them.
    ///
    /// Passing a single centroid profile instead is a trap worth spelling out.
    /// A centroid is the corpus mean, so its z-score vector is the zero vector;
    /// under any angle-based metric the distance to the origin is undefined and
    /// the result is noise. Individual documents also match what the
    /// calibration measures — it samples document-to-document pairs — so the
    /// resulting `p_same_author` is read off the distribution it was actually
    /// estimated from.
    pub fn corpus_mimic(reference: &'a Reference, targets: Vec<Profile>) -> Critic<'a> {
        Critic::with_config(
            reference,
            targets,
            None,
            CritiqueConfig {
                mode: Mode::CorpusMimic,
                // Magnitude, not angle: see the module docs.
                metric: Some(Metric::BurrowsDelta),
                ..Default::default()
            },
        )
    }

    /// Critique drafts as "move away from `away`, toward `toward`".
    ///
    /// Both profiles must come from the **same** reference — one fitted on the
    /// union of both corpora. See [`ContrastReport::classify`] for why.
    pub fn contrast(reference: &'a Reference, away: Profile, toward: Profile) -> Critic<'a> {
        Critic::with_config(
            reference,
            vec![toward],
            Some(away),
            CritiqueConfig {
                mode: Mode::Contrast,
                ..Default::default()
            },
        )
    }

    /// Build a critic with an explicit configuration.
    pub fn with_config(
        reference: &'a Reference,
        targets: Vec<Profile>,
        away: Option<Profile>,
        config: CritiqueConfig,
    ) -> Critic<'a> {
        let (canary, canary_note) = resolve_canary(reference, config.canary);
        Critic {
            reference,
            config,
            targets,
            away,
            baseline: None,
            previous: None,
            iteration: 0,
            canary,
            canary_note,
        }
    }

    /// Replace the configuration mid-loop.
    pub fn configure(&mut self, config: CritiqueConfig) {
        let (canary, note) = resolve_canary(self.reference, config.canary);
        self.canary = canary;
        self.canary_note = note;
        self.config = config;
    }

    /// The configuration in force.
    pub fn config(&self) -> &CritiqueConfig {
        &self.config
    }

    /// Which family is being held out as the canary, if any.
    pub fn canary_family(&self) -> Option<Family> {
        self.canary
    }

    /// Review a draft, advancing the loop by one iteration.
    pub fn review(&mut self, text: &str) -> Result<CritiqueReport> {
        self.iteration += 1;
        let metric = self
            .config
            .metric
            .unwrap_or_else(|| self.reference.metric());
        let doc = Document::new(text);
        let profile = self.reference.profile_tracked(&doc);
        if self.targets.is_empty() {
            return Err(Error::InvalidConfig {
                what: "Critic::targets",
                detail: "no target profiles; corpus-mimic needs at least one".into(),
            });
        }
        // Distance to the median-distance target, so that one unusual corpus
        // document neither flatters nor punishes the draft.
        let mut scored: Vec<(f64, usize)> = Vec::with_capacity(self.targets.len());
        for (i, target) in self.targets.iter().enumerate() {
            scored.push((
                self.reference
                    .compare_with(&profile, target, metric)?
                    .distance,
                i,
            ));
        }
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let representative = scored[scored.len() / 2].1;
        let target = &self.targets[representative];
        let comparison = self.reference.compare_with(&profile, target, metric)?;
        let contributions = comparison.contributions();

        // The gate runs on a capped distance so that no single dimension can
        // buy a pass by being tuned into exact agreement.
        let capped = cap_distance(
            &contributions,
            comparison.offset(),
            self.config.max_dim_share,
        );
        let assessment = self
            .reference
            .calibration()
            .filter(|c| c.metric() == metric)
            .map(|c| c.assess(capped, profile.tokens(), target.tokens()));

        let mut notes: Vec<String> = Vec::new();
        if let Some(note) = &self.canary_note {
            notes.push(note.clone());
        }

        let (pass, gate) = match &assessment {
            Some(a) => (
                Some(a.p_same_author >= self.config.pass_threshold),
                format!("p_same_author >= {:.3}", self.config.pass_threshold),
            ),
            None => {
                match self.reference.calibration() {
                    // The commonest cause is not a missing calibration but a
                    // calibration fitted for a different metric - easy to hit,
                    // because corpus-mimic overrides the reference's default
                    // metric on purpose.
                    Some(existing) => notes.push(format!(
                        "the reference is calibrated for {} but this critique uses {}; \
                         refit with Calibration::fit_with_metric(.., {}) or set \
                         CritiqueConfig::metric to {}",
                        existing.metric().name(),
                        metric.name(),
                        metric.name(),
                        existing.metric().name()
                    )),
                    None => notes.push(
                        "reference carries no calibration, so the holistic pass gate could not \
                         be evaluated; fit one with Calibration::fit against a background corpus"
                            .into(),
                    ),
                }
                (None, "unevaluated (no calibration)".into())
            }
        };

        let mut findings = match self.config.mode {
            Mode::CorpusMimic => self.corpus_findings(&profile, &contributions),
            Mode::Contrast => self.contrast_findings(&profile, metric)?,
        };
        let findings_total = findings.len();
        findings.truncate(self.config.max_findings);

        // Canary: measure the held-out family and the reported ones the same
        // way, then compare against the previous iteration.
        let reported_score = self.family_gap(&contributions, |f| Some(f) != self.canary);
        let canary_score = match self.canary {
            Some(family) => self.family_gap(&contributions, |f| f == family),
            None => 0.0,
        };
        let canary_ok = match (self.canary, self.previous) {
            (Some(_), Some(previous)) => {
                let reported_improved = reported_score < previous.reported * 0.99;
                let canary_worsened = canary_score > previous.canary * 1.01;
                !(reported_improved && canary_worsened)
            }
            _ => true,
        };
        if !canary_ok {
            findings.insert(
                0,
                Finding {
                    id: "guard.feedback_overfitting".into(),
                    family: self.canary.unwrap_or(Family::Surprisal),
                    severity: Severity::High,
                    observed: canary_score,
                    unit: "index".into(),
                    target_band: [0.0, self.previous.map_or(0.0, |p| p.canary)],
                    z: 0.0,
                    direction: Direction::Reduce,
                    spans: Vec::new(),
                    message: format!(
                        "the reported features improved ({:.3} -> {:.3}) while a held-out family \
                         got worse ({:.3} -> {:.3}); the rewrite is tracking the feedback rather \
                         than the style. Rewrite the passage as prose instead of patching the \
                         flagged items.",
                        self.previous.map_or(f64::NAN, |p| p.reported),
                        reported_score,
                        self.previous.map_or(f64::NAN, |p| p.canary),
                        canary_score
                    ),
                    fix: None,
                },
            );
        }
        self.previous = Some(Scores {
            reported: reported_score,
            canary: canary_score,
        });

        // Drift, measured against the first draft this critic ever saw.
        let fingerprint = ContentFingerprint::of(self.reference, text);
        let content_overlap = self.baseline.as_ref().map(|b| b.overlap(&fingerprint));
        if self.baseline.is_none() {
            self.baseline = Some(fingerprint);
        }
        let drift_ok = content_overlap.is_none_or(|o| o >= self.config.min_content_overlap);
        if !drift_ok {
            notes.push(format!(
                "content overlap with the original draft is {:.2}, below the {:.2} floor; \
                 check that the rewrite still says the same thing",
                content_overlap.unwrap_or(0.0),
                self.config.min_content_overlap
            ));
        }

        let artifact_flags: Vec<String> = profile
            .artifacts()
            .kinds()
            .iter()
            .map(|k| k.id().to_owned())
            .collect();

        Ok(CritiqueReport {
            handprint: crate::VERSION.to_owned(),
            contract: CONTRACT_VERSION.to_owned(),
            reference: ReferenceRef {
                name: self.reference.provenance().name.clone(),
                version: self.reference.provenance().version.clone(),
                kind: self.reference.provenance().kind.clone(),
            },
            mode: self.config.mode,
            iteration: self.iteration,
            doc: DocInfo {
                tokens: profile.tokens(),
                length_tier: profile.length_tier(),
                confidence: profile
                    .confidence(self.reference)
                    .into_iter()
                    .map(|(family, confidence)| (family.as_str().to_owned(), confidence))
                    .collect(),
                imputed_dims: profile.missing().len(),
            },
            verdict: VerdictBlock {
                pass,
                gate,
                p_same_author: assessment.map(|a| a.p_same_author),
                p_value_vs_unrelated: assessment.map(|a| a.p_value_vs_unrelated),
                distance: comparison.distance,
                capped_distance: capped,
                metric: metric.name().to_owned(),
            },
            findings,
            findings_total,
            guards: Guards {
                canary_ok,
                drift_ok,
                artifact_flags,
                notes,
                content_overlap,
            },
        })
    }

    /// Mean absolute z-gap between the draft and the target, over the families
    /// matching a predicate.
    fn family_gap(&self, contributions: &[Contribution], keep: impl Fn(Family) -> bool) -> f64 {
        let mut sum = 0.0;
        let mut count = 0usize;
        for c in contributions {
            if keep(c.family) {
                sum += (c.z_a - c.z_b).abs();
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            sum / count as f64
        }
    }

    fn corpus_findings(&self, profile: &Profile, contributions: &[Contribution]) -> Vec<Finding> {
        let mut ranked: Vec<(f64, Finding)> = Vec::new();
        for (index, dim) in self.reference.dims().iter().enumerate() {
            if Some(dim.family) == self.canary || !dim.unit.is_actionable() || dim.aggregate {
                continue;
            }
            if profile.missing().contains(&(index as u32)) {
                continue;
            }
            let stats = &self.reference.stats()[index];
            // A dimension with no corpus variance is not skipped. It is the
            // case where a deviation matters *most*: "this author never uses
            // em dashes and you used eight" is the strongest kind of finding.
            // Its band collapses to the observed constant and its z-score is
            // zero by construction, so severity has to come from the relative
            // deviation instead.
            let (lo, hi) = if stats.is_degenerate() {
                (stats.min, stats.max)
            } else {
                stats.band(self.config.band)
            };
            let observed = profile.raw()[index];
            let direction = if observed > hi {
                Direction::Reduce
            } else if observed < lo {
                Direction::Increase
            } else {
                continue;
            };
            let edge = match direction {
                Direction::Reduce => hi,
                Direction::Increase => lo,
            };
            let deviation = relative_deviation(observed, edge, dim.unit);
            if deviation < MIN_REPORTED_DEVIATION {
                continue;
            }
            let z = stats.z(observed);
            let declared = self.declared_severity(dim.symbol);
            let severity = severity_of(z, deviation, declared, direction);
            if severity < self.config.min_severity {
                continue;
            }
            let name = self.reference.name_of(dim.symbol);
            ranked.push((
                deviation,
                Finding {
                    id: finding_id(name),
                    family: dim.family,
                    severity,
                    observed,
                    unit: dim.unit.as_str().to_owned(),
                    target_band: [lo, hi],
                    z,
                    direction,
                    spans: spans_of(profile, dim.symbol),
                    message: message_for(dim.unit, observed, lo, hi, direction, z),
                    fix: self.fix_for(dim.symbol),
                },
            ));
        }
        sort_findings(&mut ranked, contributions)
    }

    fn contrast_findings(&self, profile: &Profile, metric: Metric) -> Result<Vec<Finding>> {
        let away = self.away.as_ref().ok_or_else(|| Error::InvalidConfig {
            what: "Critic::contrast",
            detail: "contrast mode needs an `away` profile; build the critic with Critic::contrast"
                .into(),
        })?;
        let toward = self.targets.first().ok_or_else(|| Error::InvalidConfig {
            what: "Critic::targets",
            detail: "contrast mode needs a `toward` profile".into(),
        })?;
        let report = ContrastReport::classify(self.reference, profile, away, toward, metric)?;
        let mut ranked: Vec<(f64, Finding)> = Vec::new();
        for (index, dim) in self.reference.dims().iter().enumerate() {
            if Some(dim.family) == self.canary || !dim.unit.is_actionable() || dim.aggregate {
                continue;
            }
            if profile.missing().contains(&(index as u32)) {
                continue;
            }
            let stats = &self.reference.stats()[index];
            let Some(pull) = report.pulls.iter().find(|p| p.symbol == dim.symbol) else {
                continue;
            };
            // Only dimensions pulling toward the side we are moving away from
            // are actionable here.
            if pull.pull <= 0.0 {
                continue;
            }
            let observed = pull.observed;
            let toward = pull.target_b;
            // In contrast mode the reference corpus spans both sides, so its
            // percentile band is not the target. Build the band around the
            // toward-side value instead, half a corpus standard deviation wide.
            let half = (stats.stddev * 0.5).max(dim.unit.noise_floor() * 0.25);
            let (lo, hi) = (toward - half, toward + half);
            let direction = if observed > hi {
                Direction::Reduce
            } else if observed < lo {
                Direction::Increase
            } else {
                continue;
            };
            let edge = match direction {
                Direction::Reduce => hi,
                Direction::Increase => lo,
            };
            let deviation = relative_deviation(observed, edge, dim.unit);
            if deviation < MIN_REPORTED_DEVIATION {
                continue;
            }
            let z = stats.z(observed);
            let declared = self.declared_severity(dim.symbol);
            let severity = severity_of(z, deviation, declared, direction);
            if severity < self.config.min_severity {
                continue;
            }
            let name = self.reference.name_of(dim.symbol);
            ranked.push((
                deviation,
                Finding {
                    id: finding_id(name),
                    family: dim.family,
                    severity,
                    observed,
                    unit: dim.unit.as_str().to_owned(),
                    target_band: [lo, hi],
                    z,
                    direction,
                    spans: spans_of(profile, dim.symbol),
                    message: message_for(dim.unit, observed, lo, hi, direction, z),
                    fix: self.fix_for(dim.symbol),
                },
            ));
        }
        Ok(sort_findings(&mut ranked, &[]))
    }

    fn declared_severity(&self, symbol: Symbol) -> Option<Severity> {
        self.reference.features().iter().find_map(|f| match f {
            Fitted::Lexicon(lexicon) => lexicon.severity_of(symbol),
            _ => None,
        })
    }

    fn fix_for(&self, symbol: Symbol) -> Option<Fix> {
        for feature in self.reference.features() {
            if let Fitted::Lexicon(lexicon) = feature {
                let alternatives = lexicon.alternatives_of(symbol);
                if !alternatives.is_empty() {
                    return Some(Fix {
                        kind: "consider_replace".into(),
                        alternatives: alternatives.to_vec(),
                    });
                }
                if let Some(message) = lexicon.message_of(symbol) {
                    return Some(Fix {
                        kind: "rewrite".into(),
                        alternatives: vec![message.to_owned()],
                    });
                }
            }
        }
        None
    }
}

/// Pick the canary family, falling back when the requested one is absent.
fn resolve_canary(
    reference: &Reference,
    requested: Option<Family>,
) -> (Option<Family>, Option<String>) {
    let Some(requested) = requested else {
        return (
            None,
            Some("canary disabled: nothing is held out, so feedback overfitting will not be detected".into()),
        );
    };
    let families = reference.families();
    if families.contains(&requested) {
        return (Some(requested), None);
    }
    // Any family works as a canary as long as it is not reported; prefer the
    // ones hardest to game by word substitution.
    for fallback in [Family::CharNgram, Family::Richness, Family::Mfw] {
        if families.contains(&fallback) {
            return (
                Some(fallback),
                Some(format!(
                    "canary family {} is not in this reference; holding out {} instead",
                    requested.as_str(),
                    fallback.as_str()
                )),
            );
        }
    }
    (
        None,
        Some(format!(
            "canary family {} is not in this reference and no substitute is available; \
             feedback overfitting will not be detected. Add a SurprisalLm or CharNgrams \
             feature to the pipeline.",
            requested.as_str()
        )),
    )
}

/// Recompute a distance with every dimension's contribution clamped.
///
/// Clamping is symmetric: it stops a dimension from buying a pass *and* from
/// single-handedly failing a draft.
fn cap_distance(contributions: &[Contribution], offset: f64, max_share: f64) -> f64 {
    if contributions.is_empty() {
        return offset;
    }
    let total: f64 = contributions.iter().map(|c| c.value.abs()).sum();
    if total <= f64::MIN_POSITIVE {
        return offset;
    }
    let cap = (max_share.clamp(0.0, 1.0) * total).max(f64::MIN_POSITIVE);
    contributions
        .iter()
        .map(|c| c.value.clamp(-cap, cap))
        .sum::<f64>()
        + offset
}

/// Deviations below this are inside the noise and are not worth an agent's
/// attention, even when they technically fall outside the band.
const MIN_REPORTED_DEVIATION: f64 = 0.05;

/// How far outside its band a value sits, normalized so that dimensions in
/// different units are comparable.
///
/// `|observed − edge| / (|edge| + floor)`. The floor is what keeps a dimension
/// whose band edge is near zero from reporting every rounding difference as a
/// catastrophe, while still letting a genuine "the corpus never does this and
/// you did it eight times per thousand words" score high.
fn relative_deviation(observed: f64, edge: f64, unit: Unit) -> f64 {
    (observed - edge).abs() / (edge.abs() + unit.noise_floor())
}

/// Order findings: most severe, then largest relative deviation, then largest
/// share of the distance, then by id for a stable output.
fn sort_findings(ranked: &mut [(f64, Finding)], contributions: &[Contribution]) -> Vec<Finding> {
    let weight = |f: &Finding| {
        contributions
            .iter()
            .find(|c| finding_id(&c.name) == f.id)
            .map(|c| c.value.abs())
            .unwrap_or(0.0)
    };
    ranked.sort_by(|(da, a), (db, b)| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| db.partial_cmp(da).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| {
                weight(b)
                    .partial_cmp(&weight(a))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    ranked.iter().map(|(_, f)| f.clone()).collect()
}

/// Severity is the stronger of what the z-score says and what the relative
/// deviation says, because either can be uninformative on its own: z is zero
/// for a zero-variance dimension, and a relative deviation is meaningless for a
/// dimension whose corpus spread is huge.
fn severity_of(
    z: f64,
    deviation: f64,
    declared: Option<Severity>,
    direction: Direction,
) -> Severity {
    let by_z = if z.abs() >= 3.0 {
        Severity::High
    } else if z.abs() >= 2.0 {
        Severity::Medium
    } else {
        Severity::Low
    };
    let by_deviation = if deviation >= 1.0 {
        Severity::High
    } else if deviation >= 0.35 {
        Severity::Medium
    } else {
        Severity::Low
    };
    let base = by_z.max(by_deviation);
    // A lexicon term's declared severity only applies when the draft is
    // over-using it. Under-using a slop word is not a high-severity problem.
    match (declared, direction) {
        (Some(declared), Direction::Reduce) => declared.max(base),
        _ => base,
    }
}

fn finding_id(name: &str) -> String {
    name.replace(':', ".")
}

fn spans_of(profile: &Profile, symbol: Symbol) -> Vec<[usize; 2]> {
    profile
        .spans(symbol)
        .iter()
        .map(|s| [s.start, s.end])
        .collect()
}

fn message_for(
    unit: Unit,
    observed: f64,
    lo: f64,
    hi: f64,
    direction: Direction,
    z: f64,
) -> String {
    let target = match direction {
        Direction::Reduce => hi,
        Direction::Increase => lo,
    };
    let verb = match direction {
        Direction::Reduce => "reduce",
        Direction::Increase => "increase",
    };
    let _ = z;
    let change = if observed.abs() > f64::EPSILON {
        let pct = ((observed - target) / observed * 100.0).abs().round();
        format!(" - {verb} ~{pct:.0}%")
    } else {
        format!(" - {verb} from zero")
    };
    match unit {
        Unit::PerThousandTokens => {
            format!("{observed:.2} per 1k tokens vs reference band {lo:.2}-{hi:.2}{change}")
        }
        Unit::PerHundredSentences => {
            format!("{observed:.2} per 100 sentences vs reference band {lo:.2}-{hi:.2}{change}")
        }
        Unit::Fraction => format!(
            "{:.1}% vs reference band {:.1}%-{:.1}%{change}",
            observed * 100.0,
            lo * 100.0,
            hi * 100.0
        ),
        _ => format!("{observed:.3} vs reference band {lo:.3}-{hi:.3}{change}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::{
        CharNgrams, LexiconFeature, MostFrequentWords, PunctTypography, SentenceStats,
    };
    use crate::reference::{CalibrationConfig, Pipeline};
    use crate::Calibration;

    /// Clause fragments a "human" author draws from. Real corpora vary; a
    /// fixture of thirty near-identical documents would make every dimension
    /// degenerate and every band collapse, which tests nothing.
    const HUMAN_BITS: &[&str] = &[
        "I looked at it again and it still comes out the same",
        "not sure whats going on there honestly",
        "maybe the cache is stale, I dont know",
        "I'll poke at it tomorrow when I have a minute",
        "the numbers are in the sheet if you want them",
        "I think the whole thing needs a rewrite but nobody has time",
        "it worked on my machine which is never a good sign",
        "we shipped it anyway and nothing broke, so far",
        "someone should probably write this down somewhere",
        "the old version did this too, for what its worth",
        "I gave up and just hardcoded it",
        "took me an hour to find, it was a typo",
        "no idea why that helps but it does",
        "ask Priya, they wrote most of this",
        "I'd rather not touch that file again",
    ];

    /// Clause fragments a "slopped" author draws from, salted with the markers
    /// the seed lexicon knows about.
    const SLOP_BITS: &[&str] = &[
        "This comprehensive analysis delves into the intricate tapestry of considerations",
        "Additionally, it is worth noting that experts argue this framework is essential",
        "Moreover, the findings underscore a pivotal shift in the ever evolving landscape",
        "The methodology showcases a robust, scalable, and transformative approach",
        "In conclusion, the results are commendable and unlock the potential of the field",
        "Furthermore, this nuanced and multifaceted subject plays a crucial role throughout",
        "Studies show that the paradigm serves as a cornerstone of modern practice",
        "It is important to note that the realm of possibilities remains unparalleled",
        "This groundbreaking work navigates the complexities with meticulous precision",
        "The tapestry of evidence stands as a testament to a truly holistic methodology",
    ];

    fn compose(seed: u64, bits: &[&str], sentences: usize, tail: &str) -> String {
        let mut rng = crate::util::rng(seed);
        let mut text = String::new();
        for _ in 0..sentences {
            let pick = rand::Rng::gen_range(&mut rng, 0..bits.len());
            text.push_str(bits[pick]);
            text.push_str(tail);
            text.push(' ');
        }
        text
    }

    fn human(i: usize) -> String {
        compose(1_000 + i as u64, HUMAN_BITS, 6, ".")
    }

    fn slopped(i: usize) -> String {
        compose(2_000 + i as u64, SLOP_BITS, 6, ".")
    }

    fn corpus(n: usize, make: impl Fn(usize) -> String) -> Corpus {
        let mut corpus = Corpus::new();
        for i in 0..n {
            corpus.add(format!("d{i}"), [Document::new(make(i))]);
        }
        corpus
    }

    fn human_reference(calibrated: bool) -> (Reference, Vec<Profile>) {
        let corpus = corpus(30, human);
        let mut reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(LexiconFeature::default())
            .feature(CharNgrams::new([3]).top(200))
            .name("human-corpus")
            .version("2026.07")
            .kind("corpus")
            .fit(&corpus)
            .unwrap();
        if calibrated {
            let config = CalibrationConfig {
                bins: vec![32, 64],
                different_pairs: 200,
                same_pairs: 100,
                min_per_bin: 10,
                max_docs_per_bin: 60,
                store_samples: 300,
                seed: 3,
            };
            // A background of authors who are each internally consistent and
            // mutually distinct: each draws from its own slice of the fragment
            // pool. Mixing styles *within* an author would inflate the
            // same-author distribution until it said nothing.
            let mut background = Corpus::new();
            let all: Vec<&str> = HUMAN_BITS.iter().chain(SLOP_BITS).copied().collect();
            for a in 0..12 {
                let slice: Vec<&str> = all.iter().cycle().skip(a * 3).take(6).copied().collect();
                let docs: Vec<Document> = (0..5)
                    .map(|d| {
                        Document::new(compose(
                            5_000 + (a * 10 + d) as u64,
                            &slice,
                            8,
                            if a % 3 == 0 { "." } else { "!" },
                        ))
                    })
                    .collect();
                background.add(format!("bg{a}"), docs);
            }
            // Calibrate for the metric corpus-mimic actually uses.
            let cal = Calibration::fit_with_metric(
                &reference,
                &background,
                &config,
                Metric::BurrowsDelta,
            )
            .expect("fixture background should support a calibration");
            reference.set_calibration(cal);
        }
        // Representative documents, not a centroid: see `Critic::corpus_mimic`.
        let targets: Vec<Profile> = (0..12)
            .map(|i| reference.profile(&Document::new(human(i))))
            .collect();
        (reference, targets)
    }

    #[test]
    fn a_slopped_draft_produces_actionable_findings() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let report = critic.review(&slopped(1)).unwrap();

        assert!(report.findings_total > 0);
        assert!(!report.findings.is_empty());
        assert!(report.findings.len() <= report.findings.capacity());
        for finding in &report.findings {
            assert!(
                !finding.id.contains(':'),
                "ids are dot-namespaced: {}",
                finding.id
            );
            assert!(finding.target_band[0] <= finding.target_band[1]);
            assert!(!finding.message.is_empty());
            match finding.direction {
                Direction::Reduce => assert!(finding.observed > finding.target_band[1]),
                Direction::Increase => assert!(finding.observed < finding.target_band[0]),
            }
        }
    }

    #[test]
    fn the_salted_markers_are_surfaced_with_their_spans() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                max_findings: 200,
                ..Default::default()
            },
        );
        let text = slopped(2);
        let report = critic.review(&text).unwrap();

        let word_findings: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.id.starts_with("lex.ai-slop.word."))
            .collect();
        assert!(
            !word_findings.is_empty(),
            "expected per-term lexicon findings, got {:?}",
            report.findings.iter().map(|f| &f.id).collect::<Vec<_>>()
        );

        for finding in &word_findings {
            assert_eq!(finding.direction, Direction::Reduce);
            assert_eq!(finding.severity, Severity::High);
            assert!(!finding.spans.is_empty(), "{} carried no spans", finding.id);
            // Every span must slice the submitted text back to the term itself,
            // which is what lets an agent patch by byte offset.
            let term = finding.id.rsplit('.').next().unwrap();
            for span in &finding.spans {
                assert_eq!(&text[span[0]..span[1]], term, "{}", finding.id);
            }
        }
    }

    #[test]
    fn rollup_dimensions_never_become_findings() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                max_findings: 500,
                ..Default::default()
            },
        );
        let report = critic.review(&slopped(11)).unwrap();
        for finding in &report.findings {
            assert!(
                !finding.id.ends_with(".total") && !finding.id.contains(".cat."),
                "rollup leaked into the findings: {}",
                finding.id
            );
        }
        // The specific hits that a rewrite can act on are still there.
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.id.contains(".word.") || f.id.contains(".phrase.")),
            "{:?}",
            report.findings.iter().map(|f| &f.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn lexicon_findings_carry_alternatives() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                max_findings: 200,
                ..Default::default()
            },
        );
        let report = critic.review(&slopped(3)).unwrap();
        let fix = report
            .findings
            .iter()
            .find(|f| f.id == "lex.ai-slop.word.tapestry")
            .and_then(|f| f.fix.clone())
            .expect("tapestry should suggest alternatives");
        assert_eq!(fix.kind, "consider_replace");
        assert!(fix.alternatives.contains(&"mix".to_string()));
    }

    #[test]
    fn a_matching_draft_produces_far_fewer_findings() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets.clone());
        let slop = critic.review(&slopped(4)).unwrap().findings_total;
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let ok = critic.review(&human(99)).unwrap().findings_total;
        assert!(ok < slop, "matching={ok} slopped={slop}");
    }

    #[test]
    fn only_the_top_k_are_reported_but_everything_is_scored() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                max_findings: 3,
                ..Default::default()
            },
        );
        let report = critic.review(&slopped(5)).unwrap();
        assert_eq!(report.findings.len(), 3);
        assert!(report.findings_total > 3, "{}", report.findings_total);
        // Most severe first.
        for pair in report.findings.windows(2) {
            assert!(pair[0].severity >= pair[1].severity);
        }
    }

    #[test]
    fn the_canary_family_is_never_reported() {
        let corpus = corpus(30, human);
        let reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(CharNgrams::new([3]).top(100))
            .name("c")
            .fit(&corpus)
            .unwrap();
        let targets: Vec<Profile> = (0..12)
            .map(|i| reference.profile(&Document::new(human(i))))
            .collect();
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                max_findings: 500,
                canary: Some(Family::CharNgram),
                ..Default::default()
            },
        );
        let report = critic.review(&slopped(6)).unwrap();
        assert!(critic.canary_family() == Some(Family::CharNgram));
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.family != Family::CharNgram),
            "canary family leaked into the findings"
        );
    }

    #[test]
    fn an_absent_canary_family_falls_back_and_says_so() {
        let corpus = corpus(30, human);
        let reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(CharNgrams::new([3]).top(100))
            .name("c")
            .fit(&corpus)
            .unwrap();
        let targets: Vec<Profile> = (0..12)
            .map(|i| reference.profile(&Document::new(human(i))))
            .collect();
        // The reference has no surprisal family at all.
        let mut critic = Critic::corpus_mimic(&reference, targets);
        assert_eq!(critic.canary_family(), Some(Family::CharNgram));
        let report = critic.review(&human(1)).unwrap();
        assert!(
            report
                .guards
                .notes
                .iter()
                .any(|n| n.contains("holding out")),
            "{:?}",
            report.guards.notes
        );
    }

    #[test]
    fn the_canary_trips_when_reported_features_improve_while_it_worsens() {
        // The reported families are punctuation marks and sentence shape only:
        // letter frequencies and word lengths are switched off so that the two
        // iterations differ on exactly the axis being tested.
        let corpus = corpus(30, human);
        let reference = Pipeline::builder()
            .feature(PunctTypography {
                letters: false,
                digits: false,
                word_lengths: false,
                ..Default::default()
            })
            .feature(SentenceStats {
                openers: false,
                markers: false,
                markdown: false,
                ..Default::default()
            })
            .feature(CharNgrams::new([4]).top(400))
            .name("c")
            .fit(&corpus)
            .unwrap();
        let targets: Vec<Profile> = (0..12)
            .map(|i| reference.profile(&Document::new(human(i))))
            .collect();
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                canary: Some(Family::CharNgram),
                ..Default::default()
            },
        );

        // Iteration 1: corpus-like vocabulary, wildly un-corpus-like
        // punctuation. The reported families are far off; the canary is close.
        let first = critic
            .review(
                "I looked at it again and it still comes out the same!!!                  Not sure whats going on there honestly!!!                  Maybe the cache is stale, I dont know!!!                  I'll poke at it tomorrow when I have a minute!!!                  The numbers are in the sheet if you want them!!!                  I gave up and just hardcoded it!!!",
            )
            .unwrap();
        assert!(first.guards.canary_ok, "no history yet on the first pass");

        // Iteration 2: punctuation now matches the corpus exactly, but the
        // character distribution has been replaced with nonsense. This is the
        // shape of an agent that patched the reported items without writing.
        let second = critic
            .review(
                "Zqx vbn mkj pfw ldh gtr snb qvx zmk lpw.                  Dfj hrn btc gvx qwz mkj pfw ldh gtr snb.                  Qvx zmk lpw dfj hrn btc gvx qwz mkj pfw.                  Ldh gtr snb qvx zmk lpw dfj hrn btc gvx.                  Qwz mkj pfw ldh gtr snb qvx zmk lpw dfj.                  Hrn btc gvx qwz mkj pfw ldh gtr snb qvx.",
            )
            .unwrap();
        assert!(
            !second.guards.canary_ok,
            "canary should have tripped; verdict was {:?}",
            second.verdict
        );
        assert_eq!(
            second.findings[0].id, "guard.feedback_overfitting",
            "the trip must be surfaced as a finding"
        );
        assert_eq!(second.findings[0].severity, Severity::High);
    }

    #[test]
    fn drift_is_measured_against_the_first_draft() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let first = critic.review(&human(1)).unwrap();
        assert_eq!(first.guards.content_overlap, None);
        assert!(first.guards.drift_ok);

        // A light edit keeps the content.
        let edited = human(1).replace("annoying", "irritating");
        let second = critic.review(&edited).unwrap();
        assert!(second.guards.content_overlap.unwrap() > 0.8);
        assert!(second.guards.drift_ok);

        // Replacing the subject entirely does not.
        let third = critic
            .review("Yesterday the weather turned and the garden flooded completely again.")
            .unwrap();
        assert!(third.guards.content_overlap.unwrap() < 0.5, "{third:?}");
        assert!(!third.guards.drift_ok);
        assert!(third
            .guards
            .notes
            .iter()
            .any(|n| n.contains("content overlap")));
    }

    #[test]
    fn without_calibration_the_gate_is_unevaluated_not_failed() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let report = critic.review(&human(2)).unwrap();
        assert_eq!(report.verdict.pass, None);
        assert_eq!(report.exit_code(), 2);
        assert!(report
            .guards
            .notes
            .iter()
            .any(|n| n.contains("no calibration")));
    }

    #[test]
    fn a_calibrated_reference_evaluates_the_holistic_gate() {
        let (reference, targets) = human_reference(true);
        assert!(reference.calibration().is_some());
        let mut critic = Critic::corpus_mimic(&reference, targets.clone());
        let matching = critic.review(&human(3)).unwrap();
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let slop = critic.review(&slopped(3)).unwrap();

        assert!(matching.verdict.pass.is_some());
        assert!(
            matching.verdict.capped_distance < slop.verdict.capped_distance,
            "a corpus-like draft must sit closer than a slopped one: {} vs {}",
            matching.verdict.capped_distance,
            slop.verdict.capped_distance
        );
        assert!(
            matching.verdict.p_same_author.unwrap() >= slop.verdict.p_same_author.unwrap(),
            "matching={:?} slop={:?}",
            matching.verdict,
            slop.verdict
        );
        assert!(matching.verdict.gate.contains("p_same_author"));
        assert!(matching.exit_code() == 0 || matching.exit_code() == 1);
    }

    #[test]
    fn a_calibration_for_the_wrong_metric_says_so() {
        let (reference, targets) = human_reference(true);
        // The reference is calibrated for Burrows; ask for cosine instead.
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                metric: Some(Metric::CosineDelta),
                ..Default::default()
            },
        );
        let report = critic.review(&human(5)).unwrap();
        assert_eq!(report.verdict.pass, None);
        assert!(
            report
                .guards
                .notes
                .iter()
                .any(|n| n.contains("calibrated for burrows_delta")),
            "{:?}",
            report.guards.notes
        );
    }

    #[test]
    fn cosine_is_the_wrong_metric_for_one_class_membership() {
        // A corpus-typical draft sits near the origin in z-space, where the
        // angle to another near-origin point is noise. Burrows measures the
        // magnitude instead and separates the two cleanly. This test exists so
        // that anyone tempted to "just use the default metric everywhere" sees
        // the failure mode written down.
        let (reference, targets) = human_reference(false);
        let distance = |metric: Metric, text: &str| {
            let mut critic = Critic::with_config(
                &reference,
                targets.clone(),
                None,
                CritiqueConfig {
                    metric: Some(metric),
                    ..Default::default()
                },
            );
            critic.review(text).unwrap().verdict.distance
        };

        let burrows_ok = distance(Metric::BurrowsDelta, &human(99));
        let burrows_slop = distance(Metric::BurrowsDelta, &slopped(99));
        assert!(
            burrows_ok < burrows_slop,
            "burrows should separate them: {burrows_ok} vs {burrows_slop}"
        );

        // Cosine puts both at roughly the same distance, near 1.
        let cosine_ok = distance(Metric::CosineDelta, &human(99));
        let cosine_slop = distance(Metric::CosineDelta, &slopped(99));
        assert!(
            (cosine_ok - 1.0).abs() < 0.35 && (cosine_slop - 1.0).abs() < 0.35,
            "cosine collapses toward 1 here: {cosine_ok} vs {cosine_slop}"
        );
    }

    #[test]
    fn capping_stops_one_dimension_from_dominating() {
        let contributions = |values: &[f64]| -> Vec<Contribution> {
            values
                .iter()
                .enumerate()
                .map(|(i, v)| Contribution {
                    symbol: Symbol(i as u32),
                    name: format!("d{i}"),
                    family: Family::Punct,
                    unit: Unit::PerThousandTokens,
                    value: *v,
                    observed_a: 0.0,
                    observed_b: 0.0,
                    z_a: 0.0,
                    z_b: 0.0,
                })
                .collect()
        };
        // One dimension pulling the distance down by 10 while four push it up
        // by 1 each: uncapped that is a distance of -6.
        let c = contributions(&[-10.0, 1.0, 1.0, 1.0, 1.0]);
        assert!((cap_distance(&c, 0.0, 1.0) - -6.0).abs() < 1e-9);
        // Capped at 25% of the total magnitude (14 * 0.25 = 3.5), the outlier
        // can no longer buy the pass.
        let capped = cap_distance(&c, 0.0, 0.25);
        assert!((capped - 0.5).abs() < 1e-9, "{capped}");
    }

    #[test]
    fn contrast_mode_targets_the_toward_side() {
        let mut union = Corpus::new();
        union.add(
            "ai",
            (0..15)
                .map(|i| Document::new(slopped(i)))
                .collect::<Vec<_>>(),
        );
        union.add(
            "human",
            (0..15).map(|i| Document::new(human(i))).collect::<Vec<_>>(),
        );
        let reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(LexiconFeature::default())
            .feature(MostFrequentWords::default().top(60))
            .name("de-ai")
            .fit(&union)
            .unwrap();
        let ai = reference.profile_aggregate(union.author(&"ai".into()).unwrap());
        let human_side = reference.profile_aggregate(union.author(&"human".into()).unwrap());

        let mut critic = Critic::contrast(&reference, ai, human_side);
        let report = critic.review(&slopped(20)).unwrap();
        assert_eq!(report.mode, Mode::Contrast);
        assert!(!report.findings.is_empty());
        let ids: Vec<&str> = report.findings.iter().map(|f| f.id.as_str()).collect();
        assert!(
            ids.iter()
                .any(|i| i.starts_with("lex.") || i.starts_with("sent.opener")),
            "{ids:?}"
        );
    }

    #[test]
    fn contrast_mode_without_an_away_profile_is_an_error() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::with_config(
            &reference,
            targets,
            None,
            CritiqueConfig {
                mode: Mode::Contrast,
                ..Default::default()
            },
        );
        assert!(critic.review(&human(1)).is_err());
    }

    #[test]
    fn the_report_round_trips_as_json_and_matches_the_contract() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let report = critic.review(&slopped(8)).unwrap();
        let json = report.to_json();
        let back: CritiqueReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back, report);

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "handprint",
            "contract",
            "reference",
            "mode",
            "iteration",
            "doc",
            "verdict",
            "findings",
            "findings_total",
            "guards",
        ] {
            assert!(value.get(key).is_some(), "missing top-level key {key}");
        }
        assert_eq!(value["contract"], CONTRACT_VERSION);
        assert_eq!(value["mode"], "corpus-mimic");
        assert!(value["doc"]["confidence"].is_object());
        // Keyed by family name, with one of the three confidence tiers.
        let punct = value["doc"]["confidence"]["punct"].as_str().unwrap();
        assert!(["none", "low", "ok"].contains(&punct), "{punct}");
        assert!(value["guards"]["canary_ok"].is_boolean());
        assert!(
            !report.findings.is_empty(),
            "the fixture must produce at least one finding for the contract check"
        );
        let finding = &value["findings"][0];
        for key in [
            "id",
            "family",
            "severity",
            "observed",
            "unit",
            "target_band",
            "z",
            "direction",
            "message",
        ] {
            assert!(finding.get(key).is_some(), "missing finding key {key}");
        }
    }

    #[test]
    fn confidence_is_reported_per_family() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let long: String = (0..4).map(human).collect::<Vec<_>>().join(" ");
        let report = critic.review(&long).unwrap();
        assert_eq!(report.doc.confidence["punct"], Confidence::Ok);
        assert!(report.doc.confidence.contains_key("lexicon"));
        assert_eq!(report.doc.length_tier, LengthTier::Unreliable);
    }

    #[test]
    fn threshold_profiles_differ() {
        assert!(
            ThresholdProfile::Strict.config().pass_threshold
                > ThresholdProfile::Lenient.config().pass_threshold
        );
        assert_eq!(ThresholdProfile::Strict.config().band, BandWidth::P25P75);
    }

    #[test]
    fn severity_respects_direction() {
        assert_eq!(
            severity_of(1.0, 0.1, Some(Severity::High), Direction::Reduce),
            Severity::High
        );
        // Under-using a slop word is not a high-severity problem.
        assert_eq!(
            severity_of(1.0, 0.1, Some(Severity::High), Direction::Increase),
            Severity::Low
        );
        assert_eq!(
            severity_of(3.5, 0.0, None, Direction::Reduce),
            Severity::High
        );
    }

    #[test]
    fn severity_uses_relative_deviation_when_the_z_score_cannot_help() {
        // A zero-variance dimension gives z = 0 by construction, so severity
        // has to come from how far outside the band the value sits.
        assert_eq!(
            severity_of(0.0, 4.0, None, Direction::Reduce),
            Severity::High
        );
        assert_eq!(
            severity_of(0.0, 0.4, None, Direction::Reduce),
            Severity::Medium
        );
        assert_eq!(
            severity_of(0.0, 0.1, None, Direction::Reduce),
            Severity::Low
        );
    }

    #[test]
    fn relative_deviation_is_floored_by_unit() {
        // Never using em dashes, then using eight per thousand words, is a
        // large deviation even though the band edge is zero.
        let big = relative_deviation(8.2, 0.0, Unit::PerThousandTokens);
        assert!(big > 1.0, "{big}");
        // A rounding-scale difference on a near-zero fraction is not.
        let small = relative_deviation(0.010, 0.001, Unit::Fraction);
        assert!(small < 0.5, "{small}");
    }

    #[test]
    fn artifacts_are_flagged_in_the_guards() {
        let (reference, targets) = human_reference(false);
        let mut critic = Critic::corpus_mimic(&reference, targets);
        let report = critic
            .review("I looked at the p\u{0430}ypal thing again and it still looks wrong to me.")
            .unwrap();
        assert!(
            report
                .guards
                .artifact_flags
                .contains(&"artifact.mixed_script".to_string()),
            "{:?}",
            report.guards.artifact_flags
        );
    }
}
