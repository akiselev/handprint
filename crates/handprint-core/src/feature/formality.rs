//! Register clash — formality variance inside a sentence.
//!
//! The device is putting two registers next to each other: bureaucratic
//! language for a cosmic event, slang inside a formal frame, a Latinate
//! adjective in front of an obscenity. Adams, Wodehouse and Thompson are all on
//! this axis, at different poles, and Bukowski is defined by *not* being on it —
//! his prose is uniformly low-register, which only a two-sided band can express
//! as a fingerprint rather than as an absence of data.
//!
//! Nobody has published this as a humor feature. The ingredients are validated
//! separately — Pavlick & Tetreault's per-sentence formality scores, the
//! Heylighen–Dewaele F-score, the register-clash observations in the stylistics
//! literature — and combining them into a within-sentence variance is the part
//! this crate is claiming.
//!
//! # Where the scores come from
//!
//! A [`NormPack`] of per-word formality scores. Two sources, and the choice
//! matters:
//!
//! * **Pavlick & Tetreault** formality annotations, which are the good data and
//!   whose redistribution terms are unverified — so they are **loader-only**.
//!   Supply the pack; handprint will not ship it.
//! * The bundled **Heylighen–Dewaele closed-class proxy**, which scores by
//!   word class rather than by annotation: nouns, prepositions, adjectives and
//!   articles raise formality; pronouns, verbs, adverbs and interjections lower
//!   it. It is coarser, and it works without any external data, which is why it
//!   is the default. A feature that silently does nothing without a download is
//!   worse than a coarse one that always runs.
//!
//! # The Latinate axis
//!
//! `form:latinate_rate` is a suffix heuristic — `-tion`, `-ity`, `-ous`,
//! `-ance`, `-ment` and friends. It over-claims on Latin-derived words that
//! have long since stopped reading as formal (`nation`, `question`) and
//! under-claims on Latinate words with native-looking shapes. Consistent on both
//! sides of a comparison, which is the property a delta needs.
//!
//! `form:latinate_profanity_clash_rate` needs a profanity pack. Without one it
//! is marked missing rather than reported as zero: "this author never mixes
//! Latinate vocabulary with obscenity" and "we did not check" are different
//! claims.

use serde::{Deserialize, Serialize};

use super::{
    per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, PackLicense, Unit,
};
use crate::error::Result;
use crate::feature::lexicon::LexiconPack;
use crate::feature::pack::NormPack;
use crate::text::{Analysis, Span};
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Suffixes that mark a word as Latinate for the purposes of the register axis.
const LATINATE_SUFFIXES: &[&str] = &[
    "tion", "sion", "ment", "ance", "ence", "ity", "ility", "ous", "ious", "eous", "ate", "ify",
    "ise", "ize", "ive", "ative", "itive", "ic", "ical", "ism", "ist", "itude", "escent", "ferous",
];

/// Latinate-suffix words common enough that counting them as formal would
/// measure ordinary English rather than register.
const LATINATE_STOPLIST: &[&str] = &[
    "nation", "station", "question", "mention", "motion", "notion", "option", "action", "section",
    "portion", "city", "pity", "unity", "moment", "comment", "music", "public", "basic", "magic",
    "logic", "topic", "panic", "picnic", "traffic", "classic", "plastic", "atlantic", "pacific",
    "active", "give", "live", "love", "have", "five", "drive", "wise", "rise", "size", "house",
    "mouse", "close", "those", "these", "nice", "price", "twice", "voice", "choice", "office",
    "late", "date", "state", "gate", "rate", "plate", "great", "create", "eight", "weight",
];

/// Configuration for the register-clash family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterClash {
    /// A per-word formality norm pack.
    ///
    /// `None` uses the bundled Heylighen–Dewaele closed-class proxy, which is
    /// coarser and needs no external data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub norms: Option<NormPack>,
    /// A profanity pack, for `form:latinate_profanity_clash_rate`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profanity: Option<LexiconPack>,
    /// How many tokens apart two words may be and still count as clashing.
    #[serde(default = "default_clash_window")]
    pub clash_window: usize,
}

fn default_clash_window() -> usize {
    12
}

impl Default for RegisterClash {
    fn default() -> Self {
        RegisterClash {
            norms: None,
            profanity: None,
            clash_window: default_clash_window(),
        }
    }
}

impl RegisterClash {
    /// Use a formality norm pack instead of the bundled proxy.
    pub fn with_norms(mut self, norms: NormPack) -> Self {
        self.norms = Some(norms);
        self
    }

    /// Supply the profanity pack the Latinate-clash dimension needs.
    pub fn with_profanity(mut self, pack: LexiconPack) -> Self {
        self.profanity = Some(pack);
        self
    }
}

/// Fitted [`RegisterClash`].
///
/// The norm table is embedded in the fitted state, which is what makes
/// `transform` pure — and what makes a reference fitted from a
/// non-redistributable pack inherit that pack's terms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedFormality {
    dims: Vec<DimInfo>,
    mean: Symbol,
    variance: Symbol,
    bimodality: Symbol,
    clash_rate: Symbol,
    latinate_rate: Symbol,
    latinate_profanity: Symbol,
    /// Sorted `(term, score)` table.
    norms: Vec<(String, f64)>,
    /// Score at or above which a word is top-quartile formal, and at or below
    /// which it is bottom-quartile informal. Taken from the pack's own
    /// distribution rather than from a constant, so a pack on a 1–7 scale and
    /// one on a −3…+3 scale both work.
    high_cut: f64,
    low_cut: f64,
    clash_window: usize,
    profanity: Vec<String>,
    packs: Vec<PackLicense>,
}

impl FittedFormality {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Register;

    /// The formality score of a word, if the table has one.
    pub fn score_of(&self, word: &str) -> Option<f64> {
        self.norms
            .binary_search_by(|(t, _)| t.as_str().cmp(word))
            .ok()
            .map(|i| self.norms[i].1)
    }

    /// How many scored terms the fitted table holds.
    pub fn norms_len(&self) -> usize {
        self.norms.len()
    }
}

impl Feature for RegisterClash {
    type Fitted = FittedFormality;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedFormality> {
        let pack = match &self.norms {
            Some(pack) => pack.clone(),
            None => super::packs::heylighen_formality(),
        };
        pack.validate()?;

        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Register, unit));
            sym
        };
        let mean = push(interner, "form:mean", Unit::Index);
        let variance = push(interner, "form:variance", Unit::Index);
        let bimodality = push(interner, "form:bimodality", Unit::Index);
        let clash_rate = push(interner, "form:clash_rate", Unit::PerHundredSentences);
        let latinate_rate = push(interner, "form:latinate_rate", Unit::PerThousandTokens);
        let latinate_profanity = push(
            interner,
            "form:latinate_profanity_clash_rate",
            Unit::PerHundredSentences,
        );

        let high_cut = pack.quantile(0.75).unwrap_or(0.0);
        let low_cut = pack.quantile(0.25).unwrap_or(0.0);
        let mut packs = vec![PackLicense {
            pack: pack.qualified_name(),
            license: pack.license.clone(),
            redistributable: pack.redistributable,
        }];

        let mut profanity: Vec<String> = Vec::new();
        if let Some(pack) = &self.profanity {
            profanity = pack.terms.iter().map(|t| t.word.to_lowercase()).collect();
            profanity.sort();
            profanity.dedup();
            packs.push(PackLicense {
                pack: pack.qualified_name(),
                license: pack.license.clone(),
                redistributable: pack.redistributable,
            });
        }

        Ok(FittedFormality {
            dims,
            mean,
            variance,
            bimodality,
            clash_rate,
            latinate_rate,
            latinate_profanity,
            norms: pack.table(),
            high_cut,
            low_cut,
            clash_window: self.clash_window.max(2),
            profanity,
            packs,
        })
    }
}

impl FittedFeature for FittedFormality {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Register
    }

    fn pack_licenses(&self) -> Vec<PackLicense> {
        self.packs.clone()
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        let stream = analysis.tokens();
        let sentences = analysis.structure().sentences.len().max(1);

        // Latinate rate, over every lexical token.
        let mut latinate = 0usize;
        for (token, form) in stream.lexical() {
            if is_latinate(form) {
                latinate += 1;
                out.note_span(self.latinate_rate, token.span);
            }
        }
        out.set(self.latinate_rate, per_1k(latinate, tokens));

        // Formality distribution over scored tokens only. Unscored tokens are
        // omitted rather than imputed to the mean: a proper noun the table has
        // never seen carries no information about register, and imputing one
        // would drag the variance toward zero exactly where an author's rare
        // vocabulary lives.
        let scores: Vec<f64> = stream
            .lexical()
            .filter_map(|(_, f)| self.score_of(f))
            .collect();
        if scores.len() < MIN_SCORED_TOKENS {
            for sym in [self.mean, self.variance, self.bimodality] {
                out.mark_missing(sym);
            }
        } else {
            out.set(self.mean, util::mean(&scores));
            out.set(self.variance, util::variance(&scores));
            out.set(self.bimodality, bimodality(&scores));
        }

        // Clash: a top-quartile and a bottom-quartile word inside one sentence
        // and within `clash_window` tokens of each other.
        let mut clashes = 0usize;
        let mut profane_clashes = 0usize;
        for sentence in &analysis.structure().sentences {
            let words: Vec<(&str, Span)> = stream.tokens()[sentence.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| (stream.form(t), t.span))
                .collect();
            if let Some((a, b)) = self.find_clash(&words) {
                clashes += 1;
                out.note_span(
                    self.clash_rate,
                    Span::new(a.start.min(b.start), a.end.max(b.end)),
                );
            }
            if !self.profanity.is_empty() {
                if let Some((a, b)) = self.find_latinate_profanity_clash(&words) {
                    profane_clashes += 1;
                    out.note_span(
                        self.latinate_profanity,
                        Span::new(a.start.min(b.start), a.end.max(b.end)),
                    );
                }
            }
        }
        out.set(self.clash_rate, ratio(clashes, sentences) * 100.0);
        if self.profanity.is_empty() {
            out.mark_missing(self.latinate_profanity);
        } else {
            out.set(
                self.latinate_profanity,
                ratio(profane_clashes, sentences) * 100.0,
            );
        }
    }
}

/// Fewest scored tokens before the distribution statistics mean anything.
const MIN_SCORED_TOKENS: usize = 20;

impl FittedFormality {
    /// The first high/low pair within the window, if the sentence has one.
    fn find_clash(&self, words: &[(&str, Span)]) -> Option<(Span, Span)> {
        for (i, (form, span)) in words.iter().enumerate() {
            let Some(score) = self.score_of(form) else {
                continue;
            };
            let high = score >= self.high_cut;
            let low = score <= self.low_cut;
            if !(high || low) {
                continue;
            }
            let end = (i + self.clash_window + 1).min(words.len());
            for (other_form, other_span) in &words[i + 1..end] {
                let Some(other) = self.score_of(other_form) else {
                    continue;
                };
                if (high && other <= self.low_cut) || (low && other >= self.high_cut) {
                    return Some((*span, *other_span));
                }
            }
        }
        None
    }

    /// The Thompson pole: a Latinate word next to an obscenity.
    fn find_latinate_profanity_clash(&self, words: &[(&str, Span)]) -> Option<(Span, Span)> {
        for (i, (form, span)) in words.iter().enumerate() {
            if !is_latinate(form) {
                continue;
            }
            let lo = i.saturating_sub(self.clash_window);
            let hi = (i + self.clash_window + 1).min(words.len());
            for (other_form, other_span) in words[lo..hi].iter() {
                if self
                    .profanity
                    .binary_search(&other_form.to_string())
                    .is_ok()
                {
                    return Some((*span, *other_span));
                }
            }
        }
        None
    }
}

/// Whether a word carries a Latinate suffix.
fn is_latinate(form: &str) -> bool {
    if form.chars().count() < 6 || LATINATE_STOPLIST.contains(&form) {
        return false;
    }
    LATINATE_SUFFIXES.iter().any(|s| form.ends_with(s))
}

/// Sarle's bimodality coefficient: `(skew² + 1) / kurtosis`.
///
/// Above roughly 5/9 the distribution is bimodal rather than unimodal, which is
/// exactly the shape a clashing sentence produces: two clumps of register with
/// a gap between them, rather than one spread. Reported as a raw index — the
/// threshold is the reader's business, and the band is what the critic uses.
fn bimodality(scores: &[f64]) -> f64 {
    let n = scores.len() as f64;
    if n < 4.0 {
        return 0.0;
    }
    let mean = util::mean(scores);
    let m2 = scores.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    if m2 <= f64::EPSILON {
        return 0.0;
    }
    let m3 = scores.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / n;
    let m4 = scores.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n;
    let skew = m3 / m2.powf(1.5);
    let kurtosis = m4 / (m2 * m2);
    if kurtosis <= f64::EPSILON {
        return 0.0;
    }
    (skew * skew + 1.0) / kurtosis
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::packs;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn fit(spec: RegisterClash) -> (FittedFormality, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        (fitted, interner)
    }

    fn transform(spec: RegisterClash, text: &str) -> (FeatureVector, Interner) {
        let (fitted, interner) = fit(spec);
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform(RegisterClash::default(), text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    /// Uniformly formal prose: every scored token is on the formal pole.
    const FORMAL: &str = "Notwithstanding the aforementioned findings, the committee shall \
        therefore proceed pursuant to the regulation. Furthermore, the evaluation was \
        conducted throughout the institutions concerned, whereby moreover the documentation \
        was distributed among the departments. Consequently, subsequent consultation is \
        recommended, whereas the previously approximated figures shall be substantially \
        revised. Hence the report, prior to its submission, shall be reviewed accordingly.";

    /// Uniformly informal prose: every scored token is on the informal pole.
    const INFORMAL: &str = "Yeah I dunno, it kinda broke and nobody noticed. We poked at it a \
        bit, honestly, and then we gave up. Nah, my guess is it's still busted, basically. \
        Whatever, somebody will totally get to it, maybe. Ugh, you know how this stuff goes \
        around here, right? Hey, it's fine, we're cool, anyway.";

    /// The two registers interleaved inside single sentences.
    const CLASHING: &str = "Notwithstanding the aforementioned findings, yeah, the whole \
        thing is totally busted. Furthermore the evaluation, honestly, was garbage and \
        nobody had bothered, whatever. Consequently my guess is we're gonna wing it, \
        pursuant to nothing. Hence the report, ugh, is basically stuff nobody reads, \
        moreover.";

    #[test]
    fn interleaved_registers_score_higher_variance_than_either_pure_one() {
        // The Bukowski ○-cell test in miniature: uniform prose sits near zero,
        // and only a two-sided band can catch a pastiche drifting *up*.
        let clashing = value(CLASHING, "form:variance");
        let formal = value(FORMAL, "form:variance");
        let informal = value(INFORMAL, "form:variance");
        assert!(
            clashing > formal && clashing > informal,
            "clashing={clashing} formal={formal} informal={informal}"
        );
    }

    #[test]
    fn the_clash_rate_separates_interleaved_from_uniform_prose() {
        let clashing = value(CLASHING, "form:clash_rate");
        let formal = value(FORMAL, "form:clash_rate");
        assert!(clashing > formal, "clashing={clashing} formal={formal}");
        assert!(clashing > 0.0);
    }

    #[test]
    fn formal_prose_scores_a_higher_mean_than_informal_prose() {
        assert!(value(FORMAL, "form:mean") > value(INFORMAL, "form:mean"));
    }

    #[test]
    fn clash_spans_cover_the_clashing_pair() {
        let text = "Notwithstanding the aforementioned findings, yeah, it is busted.";
        let (v, i) = transform(RegisterClash::default(), text);
        let spans = v.spans(i.get("form:clash_rate").unwrap());
        assert_eq!(spans.len(), 1, "one clash in one sentence");
        let covered = &text[spans[0].range()];
        assert!(covered.starts_with("Notwithstanding"), "{covered:?}");
        assert!(covered.ends_with("yeah"), "{covered:?}");
    }

    #[test]
    fn the_latinate_suffix_heuristic_has_a_documented_stoplist() {
        assert!(is_latinate("consideration"));
        assert!(is_latinate("atavistic"));
        assert!(is_latinate("magnanimous"));
        // Latin-derived but no longer reading as formal.
        assert!(!is_latinate("nation"));
        assert!(!is_latinate("question"));
        assert!(!is_latinate("music"));
        assert!(!is_latinate("great"));
        // Too short for a derived form.
        assert!(!is_latinate("city"));
    }

    #[test]
    fn the_latinate_profanity_clash_needs_a_pack_and_says_so_without_one() {
        let (v, i) = transform(RegisterClash::default(), CLASHING);
        assert!(
            v.is_missing(i.get("form:latinate_profanity_clash_rate").unwrap()),
            "without a profanity pack this is unknown, not zero"
        );
    }

    #[test]
    fn distribution_stats_are_missing_on_a_document_with_too_few_scored_words() {
        let (v, i) = transform(RegisterClash::default(), "Xyzzy plugh frobnitz.");
        assert!(v.is_missing(i.get("form:variance").unwrap()));
        // The Latinate rate needs no norm table and is still emitted.
        assert!(!v.is_missing(i.get("form:latinate_rate").unwrap()));
    }

    #[test]
    fn the_bundled_proxy_needs_no_external_data() {
        let (fitted, _) = fit(RegisterClash::default());
        assert!(fitted.norms_len() > 100);
        let licenses = fitted.pack_licenses();
        assert_eq!(licenses.len(), 1);
        assert!(
            licenses[0].redistributable,
            "the fallback must be bundleable"
        );
    }

    #[test]
    fn a_loader_only_norm_pack_makes_the_reference_non_redistributable() {
        let mut pack = packs::heylighen_formality();
        pack.name = "pavlick-formality".into();
        pack.redistributable = false;
        pack.license = "unverified".into();
        let (fitted, _) = fit(RegisterClash::default().with_norms(pack));
        let licenses = fitted.pack_licenses();
        assert!(!licenses[0].redistributable);
        assert!(licenses[0].describe().contains("NOT redistributable"));
    }

    #[test]
    fn bimodality_separates_two_clumps_from_one_spread() {
        let bimodal: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { -1.0 } else { 1.0 })
            .collect();
        let unimodal: Vec<f64> = (0..20).map(|i| (i as f64 - 10.0) / 10.0).collect();
        assert!(
            bimodality(&bimodal) > bimodality(&unimodal),
            "{} vs {}",
            bimodality(&bimodal),
            bimodality(&unimodal)
        );
        assert_eq!(bimodality(&[1.0, 1.0, 1.0, 1.0]), 0.0);
    }

    #[test]
    fn the_spec_round_trips_through_serde() {
        let spec = RegisterClash::default();
        assert_eq!(serde_json::from_str::<RegisterClash>("{}").unwrap(), spec);
        assert_eq!(spec.clash_window, 12);
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<RegisterClash>(&json).unwrap(), spec);
    }
}
