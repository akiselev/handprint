//! Phase 6 — Fightin' Words: which terms actually distinguish two corpora.
//!
//! Monroe, Colaresi & Quinn (2008), log-odds ratio with an informative
//! Dirichlet prior. Raw frequency differences are dominated by rare words with
//! huge relative swings; the prior shrinks those toward zero and the z-score
//! divides by the right standard error, so what surfaces is what a reader would
//! call characteristic.
//!
//! For term *w* between corpora *i* and *j*, with counts *y*, totals *n*, prior
//! *α_w* summing to *α₀*:
//!
//! ```text
//! δ_w = ln[(y_iw + α_w) / (n_i + α₀ − y_iw − α_w)]
//!     − ln[(y_jw + α_w) / (n_j + α₀ − y_jw − α_w)]
//!
//! σ²_w = 1/(y_iw + α_w) + 1/(n_i + α₀ − y_iw − α_w)        (Eq. 19, the default)
//!      + 1/(y_jw + α_w) + 1/(n_j + α₀ − y_jw − α_w)
//!
//! z_w = δ_w / √(σ²_w)
//! ```
//!
//! # Three things that are easy to get wrong
//!
//! **α₀ is a pseudo-sample size in tokens**, calibrated against the corpora
//! being compared — not against a background corpus. `α₀ ≈ min(n_i, n_j)` is the
//! neutral start ([`Prior::neutral`]); `α₀ → 0` recovers raw log-odds; `α₀ ≫ n_i`
//! shrinks everything to zero. Anchoring it to background size makes
//! regularization depend on an unrelated quantity: a 5M-word background would
//! swamp a 20k-word user corpus.
//!
//! **α₀ must equal Σα_w over the term universe.** With a uniform prior that
//! means `α₀ = alpha · |V|`, which moves by orders of magnitude when the term
//! universe changes. [`ContrastModel::alpha0`] therefore reports the value that
//! was actually used.
//!
//! **One term universe per fit.** If one position emits a unigram *and* a
//! bigram *and* a character 4-gram, `n_i` — the number of draws — is
//! ill-defined and every `n_i + α₀ − y − α` denominator is wrong. Fit each
//! universe separately; [`ContrastSet`] runs several and merges the ranked
//! lists for presentation, but never compares z-scores across them.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::corpus::Corpus;
use crate::error::{Error, Result};
use crate::feature::vocab::{ContrastTerm, ContrastVocab, Universe};
use crate::text::Tokenizer;

/// Which side of a contrast a term favours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// The first corpus. Positive z-scores.
    A,
    /// The second corpus. Negative z-scores.
    B,
}

/// Which variance formula to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Variance {
    /// Equation 19 — the full expression. The default.
    #[default]
    Full,
    /// Equation 20 — the common two-term approximation, which drops the
    /// `1/(n − y + α)` terms.
    ///
    /// It assumes `n ≫ y ≫ α` and inflates `|z|` by `1/√(1−p)`, about 4% at
    /// function-word frequencies. The Python `fightin-words` package implements
    /// this variant, so a cross-check against it must pin `Reduced`.
    Reduced,
}

/// The Dirichlet prior.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "prior", rename_all = "snake_case")]
pub enum Prior {
    /// `α_w = alpha` for every term, so `α₀ = alpha · |V|`.
    ///
    /// Uninformative and simple, but `α₀` moves with the vocabulary size, which
    /// makes it easy to over- or under-shrink by accident. Check
    /// [`ContrastModel::alpha0`] after fitting.
    Uniform {
        /// Prior mass per term.
        alpha: f64,
    },
    /// `α_w` proportional to the term's relative frequency in a background
    /// corpus, scaled so that `Σα_w = α₀` exactly.
    ///
    /// This is the informative prior the paper recommends: common words start
    /// with more prior mass, so a swing in "the" needs more evidence than a
    /// swing in "delve".
    Background {
        /// Total prior mass, in pseudo-tokens.
        alpha0: f64,
    },
}

impl Prior {
    /// The neutral starting point: `α₀ = min(n_i, n_j)`, with a background
    /// prior.
    pub fn neutral(n_a: usize, n_b: usize) -> Prior {
        Prior::Background {
            alpha0: n_a.min(n_b) as f64,
        }
    }
}

impl Default for Prior {
    fn default() -> Self {
        Prior::Background { alpha0: 1000.0 }
    }
}

/// Restrictions applied to vocabularies derived from private corpora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyCull {
    /// A term must appear in at least this many distinct documents on its own
    /// side to be retained.
    ///
    /// Rare identifiers — project names, paths, colleagues' names — occur in
    /// one or two sessions, so a document-frequency floor removes them
    /// wholesale. This is what makes a pack fitted on local chat logs safe to
    /// ship.
    pub min_documents: usize,
}

impl Default for PrivacyCull {
    fn default() -> Self {
        PrivacyCull { min_documents: 5 }
    }
}

/// How to fit a contrast.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastConfig {
    /// The single term universe for this fit.
    pub universe: Universe,
    /// The Dirichlet prior.
    pub prior: Prior,
    /// Which variance formula to use.
    pub variance: Variance,
    /// Minimum total count (both sides combined) for a term to enter the
    /// vocabulary.
    pub min_count: usize,
    /// Privacy culling. Applied automatically when either corpus is flagged
    /// private, even if this is `None`.
    pub privacy: Option<PrivacyCull>,
}

impl Default for ContrastConfig {
    fn default() -> Self {
        ContrastConfig {
            universe: Universe::Words,
            prior: Prior::default(),
            variance: Variance::Full,
            min_count: 5,
            privacy: None,
        }
    }
}

impl ContrastConfig {
    /// Contrast a particular term universe.
    pub fn universe(mut self, universe: Universe) -> Self {
        self.universe = universe;
        self
    }

    /// Use a particular prior.
    pub fn prior(mut self, prior: Prior) -> Self {
        self.prior = prior;
        self
    }

    /// Use a particular variance formula.
    pub fn variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    /// Require a term to occur at least this many times overall.
    pub fn min_count(mut self, min_count: usize) -> Self {
        self.min_count = min_count;
        self
    }

    /// Apply privacy culling.
    pub fn privacy(mut self, cull: PrivacyCull) -> Self {
        self.privacy = Some(cull);
        self
    }
}

/// One term's contrast statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TermStats {
    /// The term.
    pub text: String,
    /// Signed z-score. Positive favours side A.
    pub z: f64,
    /// Log-odds difference before dividing by the standard error.
    pub delta: f64,
    /// Prior mass assigned to this term.
    pub alpha: f64,
    /// Count on side A.
    pub count_a: usize,
    /// Count on side B.
    pub count_b: usize,
    /// Documents on side A containing the term.
    pub docs_a: usize,
    /// Documents on side B containing the term.
    pub docs_b: usize,
}

impl TermStats {
    /// Which side the term favours.
    pub fn side(&self) -> Side {
        if self.z >= 0.0 {
            Side::A
        } else {
            Side::B
        }
    }
}

/// A fitted contrast between two corpora over one term universe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastModel {
    name: String,
    universe: Universe,
    variance: Variance,
    terms: Vec<TermStats>,
    n_a: usize,
    n_b: usize,
    alpha0: f64,
    vocab: usize,
    culled: usize,
    privacy: Option<PrivacyCull>,
}

impl ContrastModel {
    /// Fit a contrast.
    ///
    /// `background` supplies the prior's relative frequencies when
    /// [`Prior::Background`] is used; passing `None` falls back to the pooled
    /// counts of `a` and `b`, which is a reasonable default but a weaker prior
    /// than a genuinely larger corpus.
    pub fn fit(
        name: impl Into<String>,
        a: &Corpus,
        b: &Corpus,
        background: Option<&Corpus>,
        tokenizer: &Tokenizer,
        config: &ContrastConfig,
    ) -> Result<ContrastModel> {
        let (counts_a, docs_a, n_a) = tally(a, tokenizer, config.universe);
        let (counts_b, docs_b, n_b) = tally(b, tokenizer, config.universe);
        if n_a == 0 || n_b == 0 {
            return Err(Error::CorpusTooSmall {
                feature: "ContrastModel",
                detail: format!("side A has {n_a} terms and side B has {n_b}; both must be non-empty"),
            });
        }
        if config.min_count == 0 {
            return Err(Error::InvalidConfig {
                what: "ContrastConfig::min_count",
                detail: "must be at least 1; a term seen zero times has no evidence".into(),
            });
        }

        // Vocabulary: the union of both sides, above the count floor.
        let mut vocabulary: Vec<String> = counts_a
            .keys()
            .chain(counts_b.keys())
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|w| {
                counts_a.get(w).copied().unwrap_or(0) + counts_b.get(w).copied().unwrap_or(0)
                    >= config.min_count
            })
            .collect();

        // Privacy culling runs before the prior is computed, because |V| feeds
        // into alpha0 under a uniform prior.
        let cull = config
            .privacy
            .or_else(|| (a.is_private() || b.is_private()).then(PrivacyCull::default));
        let before = vocabulary.len();
        if let Some(cull) = cull {
            vocabulary.retain(|w| {
                docs_a.get(w).copied().unwrap_or(0).max(docs_b.get(w).copied().unwrap_or(0))
                    >= cull.min_documents
            });
        }
        let culled = before - vocabulary.len();

        if vocabulary.is_empty() {
            return Err(Error::CorpusTooSmall {
                feature: "ContrastModel",
                detail: format!(
                    "no term survived selection (min_count={}, privacy={:?})",
                    config.min_count, cull
                ),
            });
        }

        let v = vocabulary.len();
        let (alphas, alpha0) = match config.prior {
            Prior::Uniform { alpha } => {
                if alpha <= 0.0 {
                    return Err(Error::InvalidConfig {
                        what: "Prior::Uniform::alpha",
                        detail: "must be positive".into(),
                    });
                }
                let alphas: Vec<f64> = vec![alpha; v];
                // alpha0 is alpha times the vocabulary size, and the vocabulary
                // size moves by orders of magnitude with the term universe.
                (alphas, alpha * v as f64)
            }
            Prior::Background { alpha0 } => {
                if alpha0 <= 0.0 {
                    return Err(Error::InvalidConfig {
                        what: "Prior::Background::alpha0",
                        detail: "must be positive".into(),
                    });
                }
                let bg: HashMap<String, usize> = match background {
                    Some(corpus) => tally(corpus, tokenizer, config.universe).0,
                    None => {
                        let mut merged = counts_a.clone();
                        for (w, c) in &counts_b {
                            *merged.entry(w.clone()).or_insert(0) += c;
                        }
                        merged
                    }
                };
                // Add-half smoothing over the retained vocabulary keeps every
                // alpha_w positive and makes the alphas sum to exactly alpha0,
                // which is the invariant the whole derivation assumes.
                let n_bg: f64 = vocabulary
                    .iter()
                    .map(|w| bg.get(w).copied().unwrap_or(0) as f64 + 0.5)
                    .sum();
                let alphas: Vec<f64> = vocabulary
                    .iter()
                    .map(|w| (bg.get(w).copied().unwrap_or(0) as f64 + 0.5) / n_bg * alpha0)
                    .collect();
                (alphas, alpha0)
            }
        };

        let mut terms: Vec<TermStats> = Vec::with_capacity(v);
        for (i, word) in vocabulary.iter().enumerate() {
            let alpha = alphas[i];
            let ya = counts_a.get(word).copied().unwrap_or(0) as f64;
            let yb = counts_b.get(word).copied().unwrap_or(0) as f64;
            let rest_a = (n_a as f64 + alpha0 - ya - alpha).max(f64::MIN_POSITIVE);
            let rest_b = (n_b as f64 + alpha0 - yb - alpha).max(f64::MIN_POSITIVE);
            let delta = ((ya + alpha) / rest_a).ln() - ((yb + alpha) / rest_b).ln();
            let var = match config.variance {
                Variance::Full => {
                    1.0 / (ya + alpha) + 1.0 / rest_a + 1.0 / (yb + alpha) + 1.0 / rest_b
                }
                Variance::Reduced => 1.0 / (ya + alpha) + 1.0 / (yb + alpha),
            };
            terms.push(TermStats {
                text: word.clone(),
                z: delta / var.sqrt(),
                delta,
                alpha,
                count_a: ya as usize,
                count_b: yb as usize,
                docs_a: docs_a.get(word).copied().unwrap_or(0),
                docs_b: docs_b.get(word).copied().unwrap_or(0),
            });
        }
        terms.sort_by(|x, y| {
            y.z.partial_cmp(&x.z)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| x.text.cmp(&y.text))
        });

        Ok(ContrastModel {
            name: name.into(),
            universe: config.universe,
            variance: config.variance,
            terms,
            n_a,
            n_b,
            alpha0,
            vocab: v,
            culled,
            privacy: cull,
        })
    }

    /// The contrast's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The term universe this contrast was fitted over. z-scores from different
    /// universes are not on a common scale and must never be compared.
    pub fn universe(&self) -> Universe {
        self.universe
    }

    /// Every term, sorted by z-score descending.
    pub fn terms(&self) -> &[TermStats] {
        &self.terms
    }

    /// The total prior mass actually used.
    ///
    /// Under [`Prior::Uniform`] this is `alpha · |V|`, which is easy to get
    /// wrong by orders of magnitude — compare it against `min(n_a, n_b)`.
    pub fn alpha0(&self) -> f64 {
        self.alpha0
    }

    /// Term counts on each side, `(n_a, n_b)`.
    pub fn totals(&self) -> (usize, usize) {
        (self.n_a, self.n_b)
    }

    /// Vocabulary size after filtering.
    pub fn vocab_size(&self) -> usize {
        self.vocab
    }

    /// How many terms privacy culling removed.
    pub fn culled(&self) -> usize {
        self.culled
    }

    /// The privacy cull that was applied, if any.
    pub fn privacy(&self) -> Option<PrivacyCull> {
        self.privacy
    }

    /// A one-line summary of the fit's regularization, worth logging: `α₀` is
    /// the setting most likely to be silently wrong.
    pub fn describe(&self) -> String {
        format!(
            "{} [{}]: n_a={} n_b={} |V|={} alpha0={:.1} (neutral would be {}) variance={:?}{}",
            self.name,
            self.universe.as_str(),
            self.n_a,
            self.n_b,
            self.vocab,
            self.alpha0,
            self.n_a.min(self.n_b),
            self.variance,
            if self.culled > 0 {
                format!(", privacy-culled {} term(s)", self.culled)
            } else {
                String::new()
            }
        )
    }

    /// The `n` most characteristic terms for a side.
    pub fn top(&self, n: usize, side: Side) -> Vec<&TermStats> {
        match side {
            Side::A => self.terms.iter().take(n).collect(),
            Side::B => self.terms.iter().rev().take(n).collect(),
        }
    }

    /// Turn discovered terms into a feature.
    ///
    /// Terms with `|z| >= z_threshold` become dimensions, closing the loop from
    /// *discover* to *classify* to *highlight*: the same vocabulary that
    /// explained a corpus difference now measures it on new text.
    pub fn into_feature(&self, z_threshold: f64) -> Result<ContrastVocab> {
        let terms: Vec<ContrastTerm> = self
            .terms
            .iter()
            .filter(|t| t.z.abs() >= z_threshold)
            .map(|t| ContrastTerm {
                text: t.text.clone(),
                z: t.z,
            })
            .collect();
        if terms.is_empty() {
            return Err(Error::InvalidConfig {
                what: "ContrastModel::into_feature",
                detail: format!(
                    "no term reached |z| >= {z_threshold}; the largest was {:.2}",
                    self.terms
                        .iter()
                        .map(|t| t.z.abs())
                        .fold(0.0f64, f64::max)
                ),
            });
        }
        Ok(ContrastVocab::new(self.name.clone(), self.universe, terms))
    }
}

/// Several contrasts over different term universes, merged for presentation.
///
/// The merge is *presentational only*. Each model keeps its own `n_i` and `α₀`,
/// and [`ContrastSet::ranked`] interleaves the per-universe rankings rather than
/// sorting a pooled list — pooling would compare z-scores that are not on a
/// common scale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastSet {
    models: Vec<ContrastModel>,
}

impl ContrastSet {
    /// Fit one contrast per universe.
    pub fn fit(
        name: &str,
        a: &Corpus,
        b: &Corpus,
        background: Option<&Corpus>,
        tokenizer: &Tokenizer,
        universes: &[Universe],
        config: &ContrastConfig,
    ) -> Result<ContrastSet> {
        let mut models = Vec::with_capacity(universes.len());
        for &universe in universes {
            let mut per = config.clone();
            per.universe = universe;
            models.push(ContrastModel::fit(
                format!("{name}:{}", universe.as_str()),
                a,
                b,
                background,
                tokenizer,
                &per,
            )?);
        }
        Ok(ContrastSet { models })
    }

    /// The individual models.
    pub fn models(&self) -> &[ContrastModel] {
        &self.models
    }

    /// Interleave each universe's top terms round-robin, up to `n` per
    /// universe.
    pub fn ranked(&self, n: usize, side: Side) -> Vec<(Universe, &TermStats)> {
        let per_universe: Vec<Vec<&TermStats>> =
            self.models.iter().map(|m| m.top(n, side)).collect();
        let mut out = Vec::new();
        for rank in 0..n {
            for (m, terms) in self.models.iter().zip(&per_universe) {
                if let Some(term) = terms.get(rank) {
                    out.push((m.universe(), *term));
                }
            }
        }
        out
    }
}

/// Count terms, document frequencies and total draws for one corpus.
fn tally(
    corpus: &Corpus,
    tokenizer: &Tokenizer,
    universe: Universe,
) -> (HashMap<String, usize>, HashMap<String, usize>, usize) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut docs: HashMap<String, usize> = HashMap::new();
    let mut total = 0usize;
    for doc in corpus.documents() {
        let analysis = doc.analyze(tokenizer);
        let terms = universe.extract(&analysis);
        total += terms.len();
        let mut seen: std::collections::HashSet<&str> = Default::default();
        for (text, _) in &terms {
            *counts.entry(text.clone()).or_insert(0) += 1;
            seen.insert(text.as_str());
        }
        for term in seen {
            *docs.entry(term.to_owned()).or_insert(0) += 1;
        }
    }
    (counts, docs, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Document;

    fn corpus_of(texts: &[&str]) -> Corpus {
        let mut c = Corpus::new();
        for (i, t) in texts.iter().enumerate() {
            c.add(format!("d{i}"), [Document::new(*t)]);
        }
        c
    }

    /// Hand-computed reference implementation of the paper's equations.
    fn expected(
        ya: f64,
        yb: f64,
        na: f64,
        nb: f64,
        alpha: f64,
        alpha0: f64,
        variance: Variance,
    ) -> (f64, f64) {
        let rest_a = na + alpha0 - ya - alpha;
        let rest_b = nb + alpha0 - yb - alpha;
        let delta = ((ya + alpha) / rest_a).ln() - ((yb + alpha) / rest_b).ln();
        let var = match variance {
            Variance::Full => 1.0 / (ya + alpha) + 1.0 / rest_a + 1.0 / (yb + alpha) + 1.0 / rest_b,
            Variance::Reduced => 1.0 / (ya + alpha) + 1.0 / (yb + alpha),
        };
        (delta, delta / var.sqrt())
    }

    #[test]
    fn matches_a_hand_computed_three_term_example() {
        // Side A: "x x y", side B: "y z z". Uniform prior alpha = 1, so
        // alpha0 = 1 * 3 = 3 over a three-term vocabulary.
        let a = corpus_of(&["x x y"]);
        let b = corpus_of(&["y z z"]);
        let config = ContrastConfig::default()
            .prior(Prior::Uniform { alpha: 1.0 })
            .min_count(1);
        let model = ContrastModel::fit("t", &a, &b, None, &Tokenizer::default(), &config).unwrap();

        assert_eq!(model.totals(), (3, 3));
        assert_eq!(model.vocab_size(), 3);
        assert!((model.alpha0() - 3.0).abs() < 1e-12);

        for term in model.terms() {
            let (ya, yb) = match term.text.as_str() {
                "x" => (2.0, 0.0),
                "y" => (1.0, 1.0),
                "z" => (0.0, 2.0),
                other => panic!("unexpected term {other}"),
            };
            let (delta, z) = expected(ya, yb, 3.0, 3.0, 1.0, 3.0, Variance::Full);
            assert!((term.delta - delta).abs() < 1e-12, "{}", term.text);
            assert!((term.z - z).abs() < 1e-12, "{}", term.text);
        }
        // Symmetric setup: "y" must be exactly neutral.
        let y = model.terms().iter().find(|t| t.text == "y").unwrap();
        assert!(y.z.abs() < 1e-12);
    }

    #[test]
    fn reduced_variance_inflates_the_z_score() {
        let a = corpus_of(&["x x y"]);
        let b = corpus_of(&["y z z"]);
        let base = ContrastConfig::default()
            .prior(Prior::Uniform { alpha: 1.0 })
            .min_count(1);
        let full = ContrastModel::fit("t", &a, &b, None, &Tokenizer::default(), &base).unwrap();
        let reduced = ContrastModel::fit(
            "t",
            &a,
            &b,
            None,
            &Tokenizer::default(),
            &base.clone().variance(Variance::Reduced),
        )
        .unwrap();
        let zf = full.terms().iter().find(|t| t.text == "x").unwrap().z;
        let zr = reduced.terms().iter().find(|t| t.text == "x").unwrap().z;
        assert!(zr.abs() > zf.abs(), "reduced={zr} full={zf}");

        // And it must match the reduced formula exactly, so a cross-check
        // against the Python `fightin-words` package has something to pin to.
        let (_, z) = expected(2.0, 0.0, 3.0, 3.0, 1.0, 3.0, Variance::Reduced);
        assert!((zr - z).abs() < 1e-12);
    }

    #[test]
    fn alpha0_is_reported_because_it_moves_with_the_vocabulary() {
        let a = corpus_of(&["x x y"]);
        let b = corpus_of(&["y z z"]);
        let uniform = ContrastModel::fit(
            "t",
            &a,
            &b,
            None,
            &Tokenizer::default(),
            &ContrastConfig::default()
                .prior(Prior::Uniform { alpha: 0.01 })
                .min_count(1),
        )
        .unwrap();
        assert!((uniform.alpha0() - 0.03).abs() < 1e-12);
        assert!(uniform.describe().contains("alpha0=0.0"));

        let neutral = Prior::neutral(3, 3);
        assert_eq!(neutral, Prior::Background { alpha0: 3.0 });
    }

    #[test]
    fn background_prior_alphas_sum_to_alpha0() {
        let a = corpus_of(&["the cat sat on the mat with the hat"]);
        let b = corpus_of(&["a dog ran past a log in a bog"]);
        let model = ContrastModel::fit(
            "t",
            &a,
            &b,
            None,
            &Tokenizer::default(),
            &ContrastConfig::default()
                .prior(Prior::Background { alpha0: 50.0 })
                .min_count(1),
        )
        .unwrap();
        let sum: f64 = model.terms().iter().map(|t| t.alpha).sum();
        assert!((sum - 50.0).abs() < 1e-9, "{sum}");
        // Common words get more prior mass than rare ones.
        let the = model.terms().iter().find(|t| t.text == "the").unwrap().alpha;
        let mat = model.terms().iter().find(|t| t.text == "mat").unwrap().alpha;
        assert!(the > mat);
    }

    #[test]
    fn larger_alpha0_shrinks_every_z_toward_zero() {
        let a = corpus_of(&["delve delve delve tapestry realm intricate nuanced meticulous"]);
        let b = corpus_of(&["dig into the thing i wrote about it yesterday honestly"]);
        let z_at = |alpha0: f64| {
            ContrastModel::fit(
                "t",
                &a,
                &b,
                None,
                &Tokenizer::default(),
                &ContrastConfig::default()
                    .prior(Prior::Background { alpha0 })
                    .min_count(1),
            )
            .unwrap()
            .terms()
            .iter()
            .map(|t| t.z.abs())
            .fold(0.0f64, f64::max)
        };
        assert!(z_at(1.0) > z_at(100.0), "{} vs {}", z_at(1.0), z_at(100.0));
        assert!(z_at(100.0) > z_at(100_000.0));
    }

    #[test]
    fn salted_markers_surface_and_z_is_monotone_in_salting_rate() {
        let human: Vec<String> = (0..40)
            .map(|i| {
                format!(
                    "I ran the numbers again on {i} and they still look wrong to me honestly. \
                     Not sure what to make of it yet."
                )
            })
            .collect();
        let salted = |rate: usize| -> Corpus {
            let mut c = Corpus::new();
            for i in 0..40 {
                let mut text = format!(
                    "This analysis examines the data from run {i} and reports the outcome clearly."
                );
                for _ in 0..rate {
                    text.push_str(" We delve into the tapestry of results.");
                }
                c.add(format!("m{i}"), [Document::new(text)]);
            }
            c
        };
        let mut human_corpus = Corpus::new();
        for (i, t) in human.iter().enumerate() {
            human_corpus.add(format!("h{i}"), [Document::new(t.clone())]);
        }

        let z_of = |rate: usize, word: &str| {
            let ai = salted(rate);
            let model = ContrastModel::fit(
                "ai-vs-human",
                &ai,
                &human_corpus,
                None,
                &Tokenizer::default(),
                &ContrastConfig::default()
                    .prior(Prior::neutral(1000, 1000))
                    .min_count(2),
            )
            .unwrap();
            model
                .terms()
                .iter()
                .find(|t| t.text == word)
                .map(|t| t.z)
                .unwrap_or(0.0)
        };

        let low = z_of(1, "delve");
        let high = z_of(4, "delve");
        assert!(low > 0.0, "delve should favour side A: {low}");
        assert!(high > low, "z should grow with salting rate: {low} -> {high}");

        // The top terms for side A are the salted markers.
        let model = ContrastModel::fit(
            "ai-vs-human",
            &salted(3),
            &human_corpus,
            None,
            &Tokenizer::default(),
            &ContrastConfig::default()
                .prior(Prior::neutral(1000, 1000))
                .min_count(2),
        )
        .unwrap();
        let top: Vec<&str> = model.top(6, Side::A).iter().map(|t| t.text.as_str()).collect();
        assert!(top.contains(&"delve"), "{top:?}");
        assert!(top.contains(&"tapestry"), "{top:?}");

        // And the other side surfaces the human markers.
        let bottom: Vec<&str> = model.top(6, Side::B).iter().map(|t| t.text.as_str()).collect();
        assert!(
            bottom.contains(&"honestly") || bottom.contains(&"wrong") || bottom.contains(&"i"),
            "{bottom:?}"
        );
    }

    #[test]
    fn privacy_culling_removes_rare_identifiers() {
        let mut private = Corpus::new().private(true);
        for i in 0..20 {
            // "widget" is everywhere; each project name appears once.
            private.add(
                format!("s{i}"),
                [Document::new(format!(
                    "the widget was fine but projectcodename{i} broke again today"
                ))],
            );
        }
        let other = corpus_of(&["nothing here at all"; 8]);
        let model = ContrastModel::fit(
            "private",
            &private,
            &other,
            None,
            &Tokenizer::default(),
            &ContrastConfig::default().min_count(1),
        )
        .unwrap();
        let terms: Vec<&str> = model.terms().iter().map(|t| t.text.as_str()).collect();
        assert!(terms.contains(&"widget"), "{terms:?}");
        assert!(
            !terms.iter().any(|t| t.starts_with("projectcodename")),
            "private identifiers leaked: {terms:?}"
        );
        assert!(model.culled() > 0);
        assert!(model.privacy().is_some(), "private corpus must cull by default");
    }

    #[test]
    fn discovered_terms_become_a_feature() {
        let a = corpus_of(&["delve delve delve tapestry tapestry realm realm intricate"; 4]);
        let b = corpus_of(&["i wrote this myself and it reads fine to me honestly"; 4]);
        let model = ContrastModel::fit(
            "ai",
            &a,
            &b,
            None,
            &Tokenizer::default(),
            &ContrastConfig::default().prior(Prior::neutral(30, 30)).min_count(1),
        )
        .unwrap();
        let vocab = model.into_feature(1.0).unwrap();
        assert_eq!(vocab.universe, Universe::Words);
        assert!(vocab.terms.iter().any(|t| t.text == "delve"));
        assert!(model.into_feature(1e9).is_err());
    }

    #[test]
    fn universes_are_fitted_separately_and_never_pooled() {
        let a = corpus_of(&["we delve into the tapestry of results here today"; 6]);
        let b = corpus_of(&["i looked at the numbers and they seemed off"; 6]);
        let set = ContrastSet::fit(
            "ai",
            &a,
            &b,
            None,
            &Tokenizer::default(),
            &[Universe::Words, Universe::WordBigrams],
            &ContrastConfig::default().prior(Prior::neutral(60, 60)).min_count(1),
        )
        .unwrap();
        assert_eq!(set.models().len(), 2);
        // Each universe has its own n_i, so the totals differ.
        let (a1, _) = set.models()[0].totals();
        let (a2, _) = set.models()[1].totals();
        assert_ne!(a1, a2);
        let ranked = set.ranked(3, Side::A);
        assert!(ranked.iter().any(|(u, _)| *u == Universe::Words));
        assert!(ranked.iter().any(|(u, _)| *u == Universe::WordBigrams));
    }

    #[test]
    fn model_round_trips_through_serde() {
        let a = corpus_of(&["x x y"]);
        let b = corpus_of(&["y z z"]);
        let model = ContrastModel::fit(
            "t",
            &a,
            &b,
            None,
            &Tokenizer::default(),
            &ContrastConfig::default().min_count(1),
        )
        .unwrap();
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(serde_json::from_str::<ContrastModel>(&json).unwrap(), model);
    }

    #[test]
    fn empty_side_is_rejected() {
        let err = ContrastModel::fit(
            "t",
            &Corpus::new(),
            &corpus_of(&["a b c"]),
            None,
            &Tokenizer::default(),
            &ContrastConfig::default().min_count(1),
        )
        .unwrap_err();
        assert!(matches!(err, Error::CorpusTooSmall { .. }));
    }
}
