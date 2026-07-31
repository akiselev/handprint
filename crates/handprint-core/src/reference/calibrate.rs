//! Calibration: turning a distance into evidence.
//!
//! A raw distance means nothing on its own. Calibration answers two different
//! questions, and handprint estimates **both** distributions because either one
//! alone is misleading:
//!
//! * **Different-author** distances (`H_d`): how often do *unrelated* authors
//!   land this close? That gives `p_value_vs_unrelated`.
//! * **Same-author** distances (`H_s`): how often does one author's own writing
//!   land this far apart? That gives `p_same_author`, which is the one-class
//!   "does this text belong to this corpus" question — and therefore the critic
//!   loop's stopping criterion.
//!
//! # Length stratification
//!
//! Both distributions are estimated **per length bin**, because the
//! relationship between distance and length is strong and not something a
//! single pooled sample can absorb. Halvani et al. (2019) measured verification
//! accuracy falling to chance between 250 and 500 characters with unadjusted
//! thresholds. At compare time the two bracketing bins are interpolated in
//! log-length.
//!
//! The direction of the length effect is **not** a constant, which is the
//! second reason to stratify rather than to apply a correction factor. For a
//! single-family pipeline, distances shrink as texts lengthen — the usual
//! story. For a mixed-family pipeline the effect can invert at the short end:
//! families that are undefined below their floors get imputed to the corpus
//! mean, which pulls both profiles toward the average and makes two *unrelated*
//! short texts look alike. Either way the practical consequence is the same and
//! worth stating plainly: **a small distance on a short text is weak evidence**,
//! and the calibrated p-value is what says so.
//!
//! # The transposed conditional
//!
//! `p_value_vs_unrelated` is `P(distance ≤ d | different authors)`. It is **not**
//! `P(different authors | distance = d)`, and reading it that way is the
//! prosecutor's fallacy. A p-value of 0.01 does not mean "99% chance of same
//! author"; it means a distance this small occurs in 1% of unrelated pairs, and
//! what that is worth depends on how many unrelated candidates exist. The
//! reported `likelihood_ratio` is closer to what most callers actually want, and
//! is still only a *score-based* likelihood ratio.

use serde::{Deserialize, Serialize};

use crate::compare::Metric;
use crate::corpus::Corpus;
use crate::error::{Error, Result};
use crate::reference::{Profile, Reference};
use crate::text::Document;
use crate::util;
use rand::seq::SliceRandom;
use rand::Rng;

/// How to build a [`Calibration`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationConfig {
    /// Target lexical-token lengths, one bin each. Log-spaced by default.
    pub bins: Vec<usize>,
    /// Cap on different-author pairs sampled per bin.
    pub different_pairs: usize,
    /// Cap on same-author pairs sampled per bin.
    pub same_pairs: usize,
    /// A bin with fewer than this many samples in either distribution is
    /// dropped rather than reported with a distribution nobody should trust.
    pub min_per_bin: usize,
    /// Cap on documents profiled per bin, to bound fitting cost.
    pub max_docs_per_bin: usize,
    /// Order statistics retained per distribution. Sorting then subsampling
    /// preserves the empirical CDF to `1/store_samples` while keeping the
    /// serialized artifact small.
    pub store_samples: usize,
    /// RNG seed; every sampling decision derives from it, so a published number
    /// can be re-derived exactly.
    pub seed: u64,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        CalibrationConfig {
            bins: vec![128, 256, 512, 1024, 2048, 4096, 8192],
            different_pairs: 5_000,
            same_pairs: 2_000,
            min_per_bin: 30,
            max_docs_per_bin: 400,
            store_samples: 2_000,
            seed: DEFAULT_SEED,
        }
    }
}

/// Default sampling seed. Fixed so that two people fitting the same
/// calibration on the same corpus get the same numbers.
pub const DEFAULT_SEED: u64 = 0x_1234_5678_9abc_def0;

/// One length bin's two distributions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBin {
    /// Target lexical-token length of the bin.
    pub tokens: usize,
    /// Sorted different-author distances.
    pub different: Vec<f64>,
    /// Sorted same-author distances.
    pub same: Vec<f64>,
}

impl CalibrationBin {
    /// `P(distance ≤ d | different authors)`.
    pub fn p_value_vs_unrelated(&self, distance: f64) -> f64 {
        util::ecdf_sorted(&self.different, distance)
    }

    /// `P(distance ≥ d | same author)` — the one-class membership p-value.
    pub fn p_same_author(&self, distance: f64) -> f64 {
        1.0 - util::ecdf_sorted(&self.same, distance)
    }

    /// Score-based likelihood ratio `f_same(d) / f_different(d)`.
    ///
    /// Both densities are Gaussian kernel estimates over the stored order
    /// statistics. The result is clamped to `[1e-3, 1e3]`: beyond that the tail
    /// estimate is an artifact of sample size, not evidence.
    pub fn likelihood_ratio(&self, distance: f64) -> f64 {
        let same = kde(&self.same, distance);
        let different = kde(&self.different, distance);
        if different <= f64::MIN_POSITIVE {
            return 1e3;
        }
        (same / different).clamp(1e-3, 1e3)
    }
}

/// Gaussian kernel density estimate with a Silverman bandwidth.
fn kde(sorted: &[f64], at: f64) -> f64 {
    if sorted.len() < 2 {
        return 0.0;
    }
    let n = sorted.len() as f64;
    let sd = util::stddev(sorted);
    let iqr = util::quantile_sorted(sorted, 0.75).unwrap_or(0.0)
        - util::quantile_sorted(sorted, 0.25).unwrap_or(0.0);
    let spread = if iqr > 0.0 { sd.min(iqr / 1.34) } else { sd };
    let h = 0.9 * spread * n.powf(-0.2);
    if h <= f64::MIN_POSITIVE {
        return if (at - sorted[0]).abs() < f64::EPSILON {
            f64::MAX
        } else {
            0.0
        };
    }
    let inv = 1.0 / (n * h * (2.0 * std::f64::consts::PI).sqrt());
    inv * sorted
        .iter()
        .map(|x| (-0.5 * ((at - x) / h).powi(2)).exp())
        .sum::<f64>()
}

/// What a calibrated distance is worth.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Assessment {
    /// `P(distance ≤ d | different authors)`. **Not** the probability that the
    /// authors differ — see the module docs.
    pub p_value_vs_unrelated: f64,
    /// `P(distance ≥ d | same author)`. High means the distance is ordinary for
    /// one author's own writing; this is the critic loop's pass gate.
    pub p_same_author: f64,
    /// Score-based likelihood ratio, `f_same(d) / f_different(d)`, clamped.
    pub likelihood_ratio: f64,
    /// The interpolated bin length the assessment was read at.
    pub at_tokens: f64,
    /// Whether the query length fell inside the calibrated range, or was
    /// clamped to the nearest bin.
    pub extrapolated: bool,
}

/// Two length-stratified distance distributions, fitted on a background corpus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    metric: Metric,
    bins: Vec<CalibrationBin>,
    seed: u64,
    /// Number of authors the background corpus contained.
    authors: usize,
}

impl Calibration {
    /// The metric these distributions were built for. Assessments are only
    /// valid for that metric.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// The fitted bins, shortest first.
    pub fn bins(&self) -> &[CalibrationBin] {
        &self.bins
    }

    /// Number of background authors the distributions were built from.
    pub fn authors(&self) -> usize {
        self.authors
    }

    /// Assess a distance between two documents of the given lengths.
    ///
    /// The two lengths are combined as a geometric mean: a 200-word query
    /// against a 20,000-word reference profile behaves like neither extreme.
    pub fn assess(&self, distance: f64, tokens_a: usize, tokens_b: usize) -> Assessment {
        let tokens = ((tokens_a.max(1) as f64) * (tokens_b.max(1) as f64)).sqrt();
        self.assess_at(distance, tokens)
    }

    /// Assess a distance at an explicit length.
    pub fn assess_at(&self, distance: f64, tokens: f64) -> Assessment {
        debug_assert!(!self.bins.is_empty(), "calibration cannot have zero bins");
        let logt = tokens.max(1.0).ln();
        let centers: Vec<f64> = self.bins.iter().map(|b| (b.tokens as f64).ln()).collect();

        let (lo, hi, frac, extrapolated) = bracket(&centers, logt);
        let blend = |f: &dyn Fn(&CalibrationBin) -> f64| {
            let a = f(&self.bins[lo]);
            let b = f(&self.bins[hi]);
            a * (1.0 - frac) + b * frac
        };

        Assessment {
            p_value_vs_unrelated: blend(&|b| b.p_value_vs_unrelated(distance)),
            p_same_author: blend(&|b| b.p_same_author(distance)),
            likelihood_ratio: blend(&|b| b.likelihood_ratio(distance)),
            at_tokens: tokens,
            extrapolated,
        }
    }

    /// Fit both distributions against a background corpus.
    ///
    /// The corpus should be a *population*, not the author under study: these
    /// distributions describe what unrelated pairs and within-author pairs look
    /// like in general.
    pub fn fit(
        reference: &Reference,
        corpus: &Corpus,
        config: &CalibrationConfig,
    ) -> Result<Calibration> {
        Self::fit_with_metric(reference, corpus, config, reference.metric())
    }

    /// Fit for a specific metric.
    pub fn fit_with_metric(
        reference: &Reference,
        corpus: &Corpus,
        config: &CalibrationConfig,
        metric: Metric,
    ) -> Result<Calibration> {
        if corpus.author_count() < 2 {
            return Err(Error::CorpusTooSmall {
                feature: "Calibration",
                detail: format!(
                    "need at least 2 authors to sample unrelated pairs, got {}",
                    corpus.author_count()
                ),
            });
        }
        if config.bins.is_empty() {
            return Err(Error::InvalidConfig {
                what: "CalibrationConfig::bins",
                detail: "must contain at least one target length".into(),
            });
        }

        // Precompute per-document token boundaries once; slicing them is how
        // short-text bins get populated from long documents, which is the only
        // honest way to calibrate for lengths the corpus does not natively
        // contain.
        let docs: Vec<(usize, &Document, Vec<usize>)> = corpus
            .authors()
            .iter()
            .enumerate()
            .flat_map(|(ai, a)| a.docs.iter().map(move |d| (ai, d)))
            .map(|(ai, doc)| {
                let analysis = doc.analyze(reference.tokenizer());
                let ends: Vec<usize> = analysis
                    .tokens()
                    .lexical()
                    .map(|(t, _)| t.span.end)
                    .collect();
                (ai, doc, ends)
            })
            .collect();

        let mut rng = util::rng(config.seed);
        let mut bins = Vec::new();
        let mut sorted_targets = config.bins.clone();
        sorted_targets.sort_unstable();
        sorted_targets.dedup();

        for &target in &sorted_targets {
            let mut usable: Vec<(usize, Profile)> = Vec::new();
            let mut candidates: Vec<usize> = (0..docs.len())
                .filter(|&i| docs[i].2.len() >= target)
                .collect();
            candidates.shuffle(&mut rng);
            candidates.truncate(config.max_docs_per_bin);

            for i in candidates {
                let (author, doc, ends) = &docs[i];
                // A random window, not always the prefix: document openings are
                // stylistically unrepresentative (greetings, headers).
                let slack = ends.len() - target;
                let start_tok = if slack == 0 {
                    0
                } else {
                    rng.gen_range(0..=slack)
                };
                let start = if start_tok == 0 {
                    0
                } else {
                    ends[start_tok - 1]
                };
                let end = ends[start_tok + target - 1];
                let text = &doc.text()[start..end];
                usable.push((*author, reference.profile(&Document::new(text))));
            }

            if usable.len() < 4 {
                continue;
            }

            let mut different = Vec::new();
            let mut same = Vec::new();

            // Different-author pairs.
            let mut attempts = 0usize;
            let max_attempts = config.different_pairs * 8;
            while different.len() < config.different_pairs && attempts < max_attempts {
                attempts += 1;
                let i = rng.gen_range(0..usable.len());
                let j = rng.gen_range(0..usable.len());
                if usable[i].0 == usable[j].0 {
                    continue;
                }
                if let Ok(cmp) = reference.compare_with(&usable[i].1, &usable[j].1, metric) {
                    different.push(cmp.distance);
                }
            }

            // Same-author pairs.
            let mut by_author: std::collections::HashMap<usize, Vec<usize>> = Default::default();
            for (idx, (author, _)) in usable.iter().enumerate() {
                by_author.entry(*author).or_default().push(idx);
            }
            let mut author_keys: Vec<usize> = by_author.keys().copied().collect();
            author_keys.sort_unstable();
            let mut attempts = 0usize;
            let max_attempts = config.same_pairs * 8;
            while same.len() < config.same_pairs && attempts < max_attempts {
                attempts += 1;
                let Some(&author) = author_keys.choose(&mut rng) else {
                    break;
                };
                let group = &by_author[&author];
                if group.len() < 2 {
                    continue;
                }
                let i = group[rng.gen_range(0..group.len())];
                let j = group[rng.gen_range(0..group.len())];
                if i == j {
                    continue;
                }
                if let Ok(cmp) = reference.compare_with(&usable[i].1, &usable[j].1, metric) {
                    same.push(cmp.distance);
                }
            }

            if different.len() < config.min_per_bin || same.len() < config.min_per_bin {
                continue;
            }
            util::sort_floats(&mut different);
            util::sort_floats(&mut same);
            bins.push(CalibrationBin {
                tokens: target,
                different: subsample(&different, config.store_samples),
                same: subsample(&same, config.store_samples),
            });
        }

        if bins.is_empty() {
            return Err(Error::CorpusTooSmall {
                feature: "Calibration",
                detail: format!(
                    "no length bin reached {} samples in both distributions; the corpus needs \
                     more authors with multiple documents, or shorter target lengths than {:?}",
                    config.min_per_bin, sorted_targets
                ),
            });
        }

        Ok(Calibration {
            metric,
            bins,
            seed: config.seed,
            authors: corpus.author_count(),
        })
    }

    /// The seed the sampling used.
    pub fn seed(&self) -> u64 {
        self.seed
    }
}

/// Locate `x` between sorted `centers`, returning `(lo, hi, fraction, clamped)`.
fn bracket(centers: &[f64], x: f64) -> (usize, usize, f64, bool) {
    if centers.len() == 1 {
        return (0, 0, 0.0, x < centers[0] * 0.9 || x > centers[0] * 1.1);
    }
    if x <= centers[0] {
        return (0, 0, 0.0, x < centers[0]);
    }
    if x >= centers[centers.len() - 1] {
        let last = centers.len() - 1;
        return (last, last, 0.0, x > centers[last]);
    }
    let hi = centers.partition_point(|&c| c < x);
    let lo = hi - 1;
    let frac = (x - centers[lo]) / (centers[hi] - centers[lo]);
    (lo, hi, frac, false)
}

/// Keep `keep` evenly spaced order statistics from a sorted slice.
fn subsample(sorted: &[f64], keep: usize) -> Vec<f64> {
    if keep == 0 || sorted.len() <= keep {
        return sorted.to_vec();
    }
    (0..keep)
        .map(|i| {
            let pos = i as f64 / (keep - 1) as f64 * (sorted.len() - 1) as f64;
            sorted[pos.round() as usize]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{MostFrequentWords, PunctTypography, SentenceStats};
    use crate::reference::Pipeline;

    /// Word pool the synthetic authors draw from.
    const POOL: &[&str] = &[
        "the", "a", "of", "and", "to", "in", "that", "it", "is", "was", "for", "on", "with", "as",
        "at", "by", "from", "they", "we", "you", "this", "but", "not", "have", "had", "would",
        "could", "there", "when", "which", "about", "into", "than", "then", "some", "other",
        "thing", "people", "time", "way", "work", "part", "case", "point", "fact", "reason",
    ];

    /// Generate authors whose *style parameters* differ — word preferences,
    /// punctuation habits, sentence lengths — rather than authors who merely
    /// mention their own index. Only the former is a real test of whether
    /// same-author pairs land closer than unrelated ones.
    fn background(authors: usize, per_author: usize, sentences: usize) -> Corpus {
        let mut corpus = Corpus::new();
        for a in 0..authors {
            let mut style = util::rng(9_000 + a as u64);
            let weights: Vec<f64> = POOL.iter().map(|_| style.gen_range(0.2..4.0)).collect();
            let total: f64 = weights.iter().sum();
            let comma_rate: f64 = style.gen_range(0.0..0.6);
            let dash_rate: f64 = style.gen_range(0.0..0.35);
            let mean_len: f64 = style.gen_range(7.0..18.0);

            let docs: Vec<Document> = (0..per_author)
                .map(|d| {
                    let mut rng = util::rng(a as u64 * 1_000_003 + d as u64 + 1);
                    let mut text = String::new();
                    for _ in 0..sentences {
                        let len = (mean_len + rng.gen_range(-3.0..3.0)).max(4.0) as usize;
                        for w in 0..len {
                            if w > 0 {
                                if rng.gen_bool(comma_rate.min(1.0) * 0.2) {
                                    text.push(',');
                                } else if rng.gen_bool(dash_rate.min(1.0) * 0.15) {
                                    text.push('\u{2014}');
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
            corpus.add(format!("a{a}"), docs);
        }
        corpus
    }

    fn reference(corpus: &Corpus) -> Reference {
        Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(MostFrequentWords::default().top(60))
            .name("bg")
            .fit(corpus)
            .unwrap()
    }

    fn small_config() -> CalibrationConfig {
        CalibrationConfig {
            bins: vec![64, 128, 256],
            different_pairs: 300,
            same_pairs: 150,
            min_per_bin: 20,
            max_docs_per_bin: 60,
            store_samples: 500,
            seed: 7,
        }
    }

    #[test]
    fn fits_both_distributions_per_bin() {
        let corpus = background(12, 4, 8);
        let reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        assert!(!cal.bins().is_empty());
        for bin in cal.bins() {
            assert!(bin.different.len() >= 20, "bin {}", bin.tokens);
            assert!(bin.same.len() >= 20, "bin {}", bin.tokens);
            assert!(bin.different.windows(2).all(|w| w[0] <= w[1]));
        }
    }

    #[test]
    fn same_author_distances_sit_below_different_author_ones() {
        let corpus = background(12, 4, 8);
        let reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        for bin in cal.bins() {
            let same_median = util::quantile_sorted(&bin.same, 0.5).unwrap();
            let diff_median = util::quantile_sorted(&bin.different, 0.5).unwrap();
            assert!(
                same_median < diff_median,
                "bin {}: same={same_median} different={diff_median}",
                bin.tokens
            );
        }
    }

    #[test]
    fn the_same_distance_reads_differently_at_different_lengths() {
        let corpus = background(14, 5, 40);
        let reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        assert!(cal.bins().len() >= 2, "need at least two bins to compare");

        let short_bin = &cal.bins()[0];
        let long_bin = cal.bins().last().unwrap();
        // Read one distance at both ends of the calibrated range.
        let d = util::quantile_sorted(&long_bin.different, 0.5).unwrap();
        let short = cal.assess_at(d, short_bin.tokens as f64);
        let long = cal.assess_at(d, long_bin.tokens as f64);

        // Each end must reproduce its own bin's empirical CDF exactly: that is
        // what stratification means.
        assert!(
            (short.p_value_vs_unrelated - short_bin.p_value_vs_unrelated(d)).abs() < 1e-12,
            "short bin not read directly"
        );
        assert!((long.p_value_vs_unrelated - 0.5).abs() < 0.02, "{long:?}");

        // And the two must actually disagree - an unstratified calibration
        // would return the same number at both lengths, which is the
        // over-confidence this whole mechanism exists to avoid.
        assert!(
            (short.p_value_vs_unrelated - long.p_value_vs_unrelated).abs() > 0.1,
            "short={:.3} long={:.3}: length stratification made no difference",
            short.p_value_vs_unrelated,
            long.p_value_vs_unrelated
        );

        // The direction follows the fitted bins, whichever way they run.
        let short_median = util::quantile_sorted(&short_bin.different, 0.5).unwrap();
        let long_median = util::quantile_sorted(&long_bin.different, 0.5).unwrap();
        if short_median > long_median {
            assert!(short.p_value_vs_unrelated < long.p_value_vs_unrelated);
        } else {
            assert!(short.p_value_vs_unrelated > long.p_value_vs_unrelated);
        }
    }

    #[test]
    fn interpolation_lands_between_the_bracketing_bins() {
        let corpus = background(14, 5, 40);
        let reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        assert!(cal.bins().len() >= 2);
        let (lo, hi) = (&cal.bins()[0], &cal.bins()[1]);
        let d = util::quantile_sorted(&lo.different, 0.5).unwrap();
        let mid = ((lo.tokens as f64) * (hi.tokens as f64)).sqrt();
        let got = cal.assess_at(d, mid).p_value_vs_unrelated;
        let (a, b) = (lo.p_value_vs_unrelated(d), hi.p_value_vs_unrelated(d));
        assert!(
            got >= a.min(b) - 1e-9 && got <= a.max(b) + 1e-9,
            "{got} not between {a} and {b}"
        );
    }

    #[test]
    fn p_values_are_probabilities_and_monotone() {
        let corpus = background(10, 4, 8);
        let reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        let mut previous = -1.0;
        for step in 0..20 {
            let d = step as f64 * 0.05;
            let a = cal.assess_at(d, 128.0);
            assert!((0.0..=1.0).contains(&a.p_value_vs_unrelated));
            assert!((0.0..=1.0).contains(&a.p_same_author));
            assert!(a.p_value_vs_unrelated >= previous - 1e-12);
            previous = a.p_value_vs_unrelated;
        }
    }

    #[test]
    fn assessment_flows_through_compare() {
        let corpus = background(10, 4, 8);
        let mut reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        reference.set_calibration(cal);

        let a = reference.profile(&corpus.author(&"a0".into()).unwrap()[0]);
        let b = reference.profile(&corpus.author(&"a0".into()).unwrap()[1]);
        let c = reference.profile(&corpus.author(&"a5".into()).unwrap()[0]);

        let within = reference.compare(&a, &b).unwrap().assessment.unwrap();
        let across = reference.compare(&a, &c).unwrap().assessment.unwrap();
        assert!(
            within.p_same_author > across.p_same_author,
            "within={within:?} across={across:?}"
        );
        assert!(within.likelihood_ratio > across.likelihood_ratio);
    }

    #[test]
    fn calibration_round_trips_and_is_reproducible() {
        let corpus = background(10, 4, 8);
        let reference = reference(&corpus);
        let a = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        let b = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        assert_eq!(a, b, "same seed must give the same calibration");

        let json = serde_json::to_string(&a).unwrap();
        let back: Calibration = serde_json::from_str(&json).unwrap();
        assert_eq!(back, a);
    }

    #[test]
    fn single_author_corpus_is_rejected() {
        let corpus = background(1, 4, 8);
        let reference = reference(&background(6, 4, 8));
        let err = Calibration::fit(&reference, &corpus, &small_config()).unwrap_err();
        assert!(matches!(err, Error::CorpusTooSmall { .. }));
    }

    #[test]
    fn extrapolation_is_flagged() {
        let corpus = background(10, 4, 8);
        let reference = reference(&corpus);
        let cal = Calibration::fit(&reference, &corpus, &small_config()).unwrap();
        assert!(cal.assess_at(0.5, 5.0).extrapolated);
        assert!(cal.assess_at(0.5, 1_000_000.0).extrapolated);
        assert!(
            !cal.assess_at(0.5, cal.bins()[0].tokens as f64 + 5.0)
                .extrapolated
        );
    }

    #[test]
    fn subsampling_preserves_the_ecdf() {
        let full: Vec<f64> = (0..10_000).map(|i| i as f64 / 10_000.0).collect();
        let kept = subsample(&full, 200);
        assert_eq!(kept.len(), 200);
        for probe in [0.0, 0.1, 0.5, 0.9, 1.0] {
            let a = util::ecdf_sorted(&full, probe);
            let b = util::ecdf_sorted(&kept, probe);
            assert!((a - b).abs() < 0.02, "{probe}: {a} vs {b}");
        }
    }

    #[test]
    fn bracket_interpolates_in_log_space() {
        let centers = [1.0, 2.0, 3.0];
        let (lo, hi, frac, ex) = bracket(&centers, 2.5);
        assert_eq!((lo, hi), (1, 2));
        assert!((frac - 0.5).abs() < 1e-12);
        assert!(!ex);
        assert!(bracket(&centers, 0.5).3);
        assert!(bracket(&centers, 9.0).3);
    }
}
