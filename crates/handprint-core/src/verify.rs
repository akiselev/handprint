//! Phase 7 — General Imposters authorship verification.
//!
//! Koppel & Winter (2014); winner of PAN 2013 and 2014, and implemented in R
//! `stylo` as `imposters()`. The idea is that a raw distance to a candidate
//! author means little on its own, but *"is the query closer to this author
//! than to any of a crowd of unrelated writers, robustly, under repeated
//! random perturbation of the feature set?"* means a great deal.
//!
//! Each iteration samples a random half of the feature dimensions and a random
//! half of the impostor pool, then asks whether the target still wins. The
//! score is the fraction of iterations it does. Because the feature subset
//! changes every round, a win driven by one or two coincidental dimensions
//! collapses, which is exactly the failure mode a single distance cannot detect.
//!
//! # Abstention is a first-class answer
//!
//! Scores near 0.5 are not weak evidence for one side; they are *no* evidence,
//! and reporting them as a verdict is how verification systems acquire a
//! reputation for confident nonsense. Scores between `p1` and `p2` return
//! [`Verdict::Abstain`]. The defaults are deliberately wide; use
//! [`Thresholds::optimize`] on a labelled split to tune them, which is the
//! analogue of stylo's `imposters.optimize`.
//!
//! # Two uses
//!
//! * **Forensic**: impostors drawn from a background corpus, target set being
//!   the candidate author's known writing.
//! * **Critic loop**: an alternative pass gate to the calibrated
//!   `p_same_author` — "the target corpus wins at score ≥ p2". It is more robust
//!   to topic confound than a global percentile, which matters when a draft is
//!   about a subject the reference corpus never covered.

use serde::{Deserialize, Serialize};

use crate::compare::{DistanceMetric, Metric};
use crate::error::{Error, Result};
use crate::reference::{Profile, Reference};
use crate::util;
use rand::seq::SliceRandom;

/// What a verification concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The target set won often enough to be worth reporting.
    ///
    /// Still not proof: read it as "consistent with common authorship", never
    /// as "same author".
    TargetFavoured,
    /// The impostors won often enough that the target is not indicated.
    TargetDisfavoured,
    /// Inside the grey zone. The honest answer.
    Abstain,
}

/// The grey-zone thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Thresholds {
    /// At or below this the verdict is [`Verdict::TargetDisfavoured`].
    pub p1: f64,
    /// At or above this the verdict is [`Verdict::TargetFavoured`].
    pub p2: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Thresholds { p1: 0.35, p2: 0.65 }
    }
}

impl Thresholds {
    /// Build thresholds, rejecting an inverted pair.
    pub fn new(p1: f64, p2: f64) -> Result<Thresholds> {
        if !(0.0..=1.0).contains(&p1) || !(0.0..=1.0).contains(&p2) || p1 > p2 {
            return Err(Error::InvalidConfig {
                what: "Thresholds",
                detail: format!("need 0 <= p1 <= p2 <= 1, got p1={p1} p2={p2}"),
            });
        }
        Ok(Thresholds { p1, p2 })
    }

    /// Classify a score.
    pub fn verdict(&self, score: f64) -> Verdict {
        if score >= self.p2 {
            Verdict::TargetFavoured
        } else if score <= self.p1 {
            Verdict::TargetDisfavoured
        } else {
            Verdict::Abstain
        }
    }

    /// Choose thresholds on a labelled split, maximizing `c@1`.
    ///
    /// `c@1` (Peñas & Rodrigo 2011, the PAN measure) rewards abstaining instead
    /// of guessing: `c@1 = (n_correct + n_abstain · n_correct / n) / n`. A
    /// system that answers everything scores its plain accuracy; one that
    /// abstains on the cases it would have got wrong scores higher. Note the
    /// bonus term divides by the **total**, not by the number answered — with
    /// `n_answered` there, abstaining on all but a handful of certain cases
    /// would score a perfect 1.0. This is the analogue of stylo's
    /// `imposters.optimize`.
    pub fn optimize(labelled: &[(f64, bool)]) -> Result<Thresholds> {
        if labelled.len() < 4 {
            return Err(Error::InvalidConfig {
                what: "Thresholds::optimize",
                detail: format!("need at least 4 labelled scores, got {}", labelled.len()),
            });
        }
        let grid: Vec<f64> = (0..=40).map(|i| i as f64 / 40.0).collect();
        let mut best = (f64::NEG_INFINITY, Thresholds::default());
        for &p1 in &grid {
            for &p2 in grid.iter().filter(|&&p| p >= p1) {
                let t = Thresholds { p1, p2 };
                let score = c_at_1(labelled, t);
                // Prefer the widest grey zone among ties: a tie means the extra
                // answers bought nothing, so decline to give them.
                let width = p2 - p1;
                let current = score * 1000.0 + width;
                if current > best.0 {
                    best = (current, t);
                }
            }
        }
        Ok(best.1)
    }
}

/// The `c@1` measure over a labelled set at given thresholds.
pub fn c_at_1(labelled: &[(f64, bool)], thresholds: Thresholds) -> f64 {
    let n = labelled.len() as f64;
    let mut correct = 0.0;
    let mut abstained = 0.0;
    for &(score, same) in labelled {
        match thresholds.verdict(score) {
            Verdict::Abstain => abstained += 1.0,
            Verdict::TargetFavoured if same => correct += 1.0,
            Verdict::TargetDisfavoured if !same => correct += 1.0,
            _ => {}
        }
    }
    if n <= 0.0 {
        return 0.0;
    }
    (correct + abstained * correct / n) / n
}

/// How to run a verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyConfig {
    /// Bootstrap iterations.
    pub iterations: usize,
    /// Fraction of feature dimensions sampled each iteration.
    pub feature_fraction: f64,
    /// Fraction of the impostor pool sampled each iteration.
    pub impostor_fraction: f64,
    /// Grey-zone thresholds.
    pub thresholds: Thresholds,
    /// Metric override; `None` uses the reference's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<Metric>,
    /// RNG seed.
    pub seed: u64,
}

impl Default for VerifyConfig {
    fn default() -> Self {
        VerifyConfig {
            iterations: 100,
            feature_fraction: 0.5,
            impostor_fraction: 0.5,
            thresholds: Thresholds::default(),
            metric: None,
            seed: 0x_5eed_0000_0000_0001,
        }
    }
}

/// The outcome of a verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyScore {
    /// Fraction of iterations in which the target set beat every sampled
    /// impostor.
    pub score: f64,
    /// The verdict at the configured thresholds.
    pub verdict: Verdict,
    /// Iterations actually run.
    pub iterations: usize,
    /// Size of the impostor pool.
    pub impostors: usize,
    /// Size of the target set.
    pub targets: usize,
    /// Dimensions sampled per iteration.
    pub features_per_iteration: usize,
    /// Thresholds used.
    pub thresholds: Thresholds,
    /// Metric used.
    pub metric: Metric,
}

impl VerifyScore {
    /// A one-line summary that does not overclaim.
    pub fn describe(&self) -> String {
        let verdict = match self.verdict {
            Verdict::TargetFavoured => "target favoured",
            Verdict::TargetDisfavoured => "target disfavoured",
            Verdict::Abstain => "abstain (inside the grey zone)",
        };
        format!(
            "score {:.2} over {} iterations against {} impostors: {verdict} \
             [grey zone {:.2}-{:.2}]",
            self.score, self.iterations, self.impostors, self.thresholds.p1, self.thresholds.p2
        )
    }
}

/// Run General Imposters verification.
///
/// `target` is the candidate author's known writing (one profile per document,
/// or one aggregate); `impostors` is a crowd of unrelated writers, which should
/// come from the same register and ideally the same topics as the query — an
/// impostor pool that is easy to beat makes every query look like the target.
pub fn verify(
    reference: &Reference,
    query: &Profile,
    target: &[Profile],
    impostors: &[Profile],
    config: &VerifyConfig,
) -> Result<VerifyScore> {
    if target.is_empty() {
        return Err(Error::InvalidConfig {
            what: "verify::target",
            detail: "target set is empty".into(),
        });
    }
    if impostors.len() < 2 {
        return Err(Error::InvalidConfig {
            what: "verify::impostors",
            detail: format!(
                "need at least 2 impostors for the comparison to mean anything, got {}",
                impostors.len()
            ),
        });
    }
    if config.iterations == 0 {
        return Err(Error::InvalidConfig {
            what: "VerifyConfig::iterations",
            detail: "must be at least 1".into(),
        });
    }
    if !(0.0..=1.0).contains(&config.feature_fraction)
        || !(0.0..=1.0).contains(&config.impostor_fraction)
    {
        return Err(Error::InvalidConfig {
            what: "VerifyConfig",
            detail: "feature_fraction and impostor_fraction must be in [0, 1]".into(),
        });
    }

    let metric = config.metric.unwrap_or_else(|| reference.metric());
    let space = metric.space();
    let n_dims = reference.dims().len();

    let query_v = reference.vector_in(query, space);
    let target_v: Vec<Vec<f64>> = target
        .iter()
        .map(|p| reference.vector_in(p, space))
        .collect();
    let impostor_v: Vec<Vec<f64>> = impostors
        .iter()
        .map(|p| reference.vector_in(p, space))
        .collect();

    let k = ((n_dims as f64 * config.feature_fraction).round() as usize).clamp(1, n_dims);
    let m = ((impostors.len() as f64 * config.impostor_fraction).round() as usize)
        .clamp(1, impostors.len());

    let mut rng = util::rng(config.seed);
    let mut dim_indices: Vec<usize> = (0..n_dims).collect();
    let mut impostor_indices: Vec<usize> = (0..impostors.len()).collect();
    let mut wins = 0usize;

    let mut qs = vec![0.0; k];
    let mut cs = vec![0.0; k];

    for _ in 0..config.iterations {
        dim_indices.shuffle(&mut rng);
        impostor_indices.shuffle(&mut rng);
        let dims = &dim_indices[..k];

        gather(&query_v, dims, &mut qs);

        let mut best_target = f64::INFINITY;
        for v in &target_v {
            gather(v, dims, &mut cs);
            best_target = best_target.min(metric.distance(&qs, &cs));
        }
        let mut best_impostor = f64::INFINITY;
        for &i in &impostor_indices[..m] {
            gather(&impostor_v[i], dims, &mut cs);
            best_impostor = best_impostor.min(metric.distance(&qs, &cs));
        }
        if best_target < best_impostor {
            wins += 1;
        }
    }

    let score = wins as f64 / config.iterations as f64;
    Ok(VerifyScore {
        score,
        verdict: config.thresholds.verdict(score),
        iterations: config.iterations,
        impostors: impostors.len(),
        targets: target.len(),
        features_per_iteration: k,
        thresholds: config.thresholds,
        metric,
    })
}

/// Draw an impostor pool from a background corpus, excluding named authors.
///
/// Impostors are aggregated per author, since a per-document impostor pool
/// mixes "unrelated author" with "short document" and the query would beat it
/// for the wrong reason.
pub fn impostor_pool(
    reference: &Reference,
    background: &crate::corpus::Corpus,
    exclude: &[&str],
) -> Vec<Profile> {
    background
        .authors()
        .iter()
        .filter(|a| !exclude.contains(&a.author.as_str()))
        .map(|a| reference.profile_aggregate(&a.docs))
        .collect()
}

fn gather(source: &[f64], indices: &[usize], out: &mut [f64]) {
    for (slot, &i) in out.iter_mut().zip(indices) {
        *slot = source[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::{MostFrequentWords, PunctTypography, SentenceStats};
    use crate::reference::Pipeline;
    use crate::text::Document;
    use rand::Rng;

    const POOL: &[&str] = &[
        "the", "a", "of", "and", "to", "in", "that", "it", "is", "was", "for", "on", "with", "as",
        "at", "by", "from", "they", "we", "you", "this", "but", "not", "have", "had", "would",
        "could", "there", "when", "which", "about", "into", "than", "then", "some", "other",
    ];

    /// Authors with distinct word-preference and punctuation profiles.
    fn synth(authors: usize, docs: usize, sentences: usize) -> Corpus {
        let mut corpus = Corpus::new();
        for a in 0..authors {
            let mut style = util::rng(4_000 + a as u64);
            let weights: Vec<f64> = POOL.iter().map(|_| style.gen_range(0.2..4.0)).collect();
            let total: f64 = weights.iter().sum();
            let comma: f64 = style.gen_range(0.0..0.5);
            let mean_len: f64 = style.gen_range(8.0..18.0);
            let made: Vec<Document> = (0..docs)
                .map(|d| {
                    let mut rng = util::rng(a as u64 * 7_777 + d as u64 + 1);
                    let mut text = String::new();
                    for _ in 0..sentences {
                        let len = (mean_len + rng.gen_range(-3.0..3.0)).max(4.0) as usize;
                        for w in 0..len {
                            if w > 0 {
                                if rng.gen_bool((comma * 0.2).min(1.0)) {
                                    text.push(',');
                                }
                                text.push(' ');
                            }
                            let mut pick = rng.gen_range(0.0..total);
                            let mut chosen = POOL[0];
                            for (i, word) in POOL.iter().enumerate() {
                                pick -= weights[i];
                                if pick <= 0.0 {
                                    chosen = word;
                                    break;
                                }
                            }
                            text.push_str(chosen);
                        }
                        text.push_str(". ");
                    }
                    Document::new(text)
                })
                .collect();
            corpus.add(format!("a{a}"), made);
        }
        corpus
    }

    fn setup() -> (Reference, Corpus) {
        let corpus = synth(14, 6, 25);
        let reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(MostFrequentWords::default().top(40))
            .name("gi")
            .fit(&corpus)
            .unwrap();
        (reference, corpus)
    }

    #[test]
    fn the_true_author_wins_and_a_stranger_does_not() {
        let (reference, corpus) = setup();
        let docs = corpus.author(&"a0".into()).unwrap();
        let query = reference.profile(&docs[0]);
        let target = vec![reference.profile_aggregate(&docs[1..])];
        let impostors = impostor_pool(&reference, &corpus, &["a0"]);

        let hit = verify(
            &reference,
            &query,
            &target,
            &impostors,
            &VerifyConfig::default(),
        )
        .unwrap();

        let other = corpus.author(&"a7".into()).unwrap();
        let wrong_target = vec![reference.profile_aggregate(other)];
        let miss = verify(
            &reference,
            &query,
            &wrong_target,
            &impostor_pool(&reference, &corpus, &["a7"]),
            &VerifyConfig::default(),
        )
        .unwrap();

        assert!(
            hit.score > miss.score,
            "true author {:.2} should beat stranger {:.2}",
            hit.score,
            miss.score
        );
        assert_eq!(hit.verdict, Verdict::TargetFavoured, "{}", hit.describe());
        assert_ne!(miss.verdict, Verdict::TargetFavoured, "{}", miss.describe());
    }

    #[test]
    fn scores_are_stable_across_seeds() {
        let (reference, corpus) = setup();
        let docs = corpus.author(&"a3".into()).unwrap();
        let query = reference.profile(&docs[0]);
        let target = vec![reference.profile_aggregate(&docs[1..])];
        let impostors = impostor_pool(&reference, &corpus, &["a3"]);

        let scores: Vec<f64> = [1u64, 2, 3, 4, 5]
            .iter()
            .map(|&seed| {
                verify(
                    &reference,
                    &query,
                    &target,
                    &impostors,
                    &VerifyConfig {
                        seed,
                        iterations: 200,
                        ..Default::default()
                    },
                )
                .unwrap()
                .score
            })
            .collect();
        let spread = scores.iter().cloned().fold(f64::MIN, f64::max)
            - scores.iter().cloned().fold(f64::MAX, f64::min);
        assert!(
            spread < 0.15,
            "scores varied too much across seeds: {scores:?}"
        );
    }

    #[test]
    fn the_same_seed_reproduces_the_same_score() {
        let (reference, corpus) = setup();
        let docs = corpus.author(&"a1".into()).unwrap();
        let query = reference.profile(&docs[0]);
        let target = vec![reference.profile_aggregate(&docs[1..])];
        let impostors = impostor_pool(&reference, &corpus, &["a1"]);
        let run = || {
            verify(
                &reference,
                &query,
                &target,
                &impostors,
                &VerifyConfig::default(),
            )
            .unwrap()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn abstention_covers_the_middle() {
        let t = Thresholds::default();
        assert_eq!(t.verdict(0.9), Verdict::TargetFavoured);
        assert_eq!(t.verdict(0.5), Verdict::Abstain);
        assert_eq!(t.verdict(0.1), Verdict::TargetDisfavoured);
        assert!(Thresholds::new(0.8, 0.2).is_err());
    }

    #[test]
    fn threshold_optimization_prefers_abstaining_over_guessing() {
        // Cleanly separated positives and negatives, plus an ambiguous middle
        // that is labelled inconsistently: the optimizer should carve the
        // middle out rather than answer it.
        let mut labelled: Vec<(f64, bool)> = Vec::new();
        for i in 0..20 {
            labelled.push((0.90 + (i % 5) as f64 * 0.01, true));
            labelled.push((0.05 + (i % 5) as f64 * 0.01, false));
            labelled.push((0.50, i % 2 == 0));
        }
        let t = Thresholds::optimize(&labelled).unwrap();
        assert_eq!(t.verdict(0.5), Verdict::Abstain, "{t:?}");
        assert_eq!(t.verdict(0.92), Verdict::TargetFavoured, "{t:?}");
        assert_eq!(t.verdict(0.06), Verdict::TargetDisfavoured, "{t:?}");
        assert!(c_at_1(&labelled, t) >= c_at_1(&labelled, Thresholds { p1: 0.5, p2: 0.5 }));
    }

    #[test]
    fn c_at_1_rewards_a_well_placed_grey_zone() {
        let labelled = [(0.9, true), (0.8, true), (0.5, false), (0.1, false)];
        let answer_everything = c_at_1(&labelled, Thresholds { p1: 0.5, p2: 0.5 });
        let abstain_on_the_hard_one = c_at_1(&labelled, Thresholds { p1: 0.3, p2: 0.7 });
        assert!(abstain_on_the_hard_one > answer_everything);
    }

    #[test]
    fn feature_subsampling_actually_happens() {
        let (reference, corpus) = setup();
        let docs = corpus.author(&"a2".into()).unwrap();
        let query = reference.profile(&docs[0]);
        let target = vec![reference.profile_aggregate(&docs[1..])];
        let impostors = impostor_pool(&reference, &corpus, &["a2"]);
        let score = verify(
            &reference,
            &query,
            &target,
            &impostors,
            &VerifyConfig {
                feature_fraction: 0.25,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            score.features_per_iteration,
            (reference.dims().len() as f64 * 0.25).round() as usize
        );
    }

    #[test]
    fn degenerate_inputs_are_rejected() {
        let (reference, corpus) = setup();
        let docs = corpus.author(&"a0".into()).unwrap();
        let query = reference.profile(&docs[0]);
        let target = vec![reference.profile(&docs[1])];
        let impostors = impostor_pool(&reference, &corpus, &["a0"]);
        assert!(verify(
            &reference,
            &query,
            &[],
            &impostors,
            &VerifyConfig::default()
        )
        .is_err());
        assert!(verify(
            &reference,
            &query,
            &target,
            &impostors[..1],
            &VerifyConfig::default()
        )
        .is_err());
        assert!(verify(
            &reference,
            &query,
            &target,
            &impostors,
            &VerifyConfig {
                iterations: 0,
                ..Default::default()
            }
        )
        .is_err());
    }

    #[test]
    fn impostor_pool_excludes_named_authors() {
        let (reference, corpus) = setup();
        let pool = impostor_pool(&reference, &corpus, &["a0", "a1"]);
        assert_eq!(pool.len(), corpus.author_count() - 2);
    }
}
