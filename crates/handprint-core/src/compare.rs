//! Distance metrics, and the decomposition that makes them explainable.
//!
//! # Every score decomposes (invariant #1)
//!
//! A metric here is not a black box that returns a number. It returns a
//! [`Decomposition`]: one signed contribution per dimension plus, at most, one
//! global offset, with the guarantee
//!
//! ```text
//! sum(contributions) + offset == distance
//! ```
//!
//! to floating-point tolerance. That identity is what lets a report say "this
//! distance is 40% em-dash rate and 15% sentence-length variance" instead of
//! "the distance is 0.31". It also rules out non-linear kernels in core: if it
//! cannot decompose, it does not belong here.
//!
//! # Scaling is a per-metric property (invariant #5)
//!
//! The Delta family is defined on z-scores. MinMax/Ruzicka is defined on
//! non-negative relative frequencies, and is simply ill-defined on z-scores —
//! `stylo::dist.minmax` takes no scaling argument for exactly this reason, and
//! Kestemont et al.'s best configurations are `minmax-tf` variants. So each
//! metric declares the [`Space`] it consumes and the
//! [`Reference`](crate::Reference) supplies it. There is no pipeline-global
//! scaling setting to get wrong.

use serde::{Deserialize, Serialize};

use crate::feature::{Family, Unit};
use crate::vector::Symbol;

/// The input space a metric consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Space {
    /// Per-dimension z-scores against the reference corpus.
    ZScore,
    /// Raw values, shifted so that no dimension can be negative.
    ///
    /// For relative-frequency dimensions the shift is zero, so this is the
    /// plain relative frequency the Ruzicka literature assumes. Only dimensions
    /// that can genuinely go negative — a surprisal autocorrelation, say — get
    /// shifted, and only by their corpus minimum.
    NonNegative,
}

/// A distance broken into per-dimension contributions.
#[derive(Debug, Clone, PartialEq)]
pub struct Decomposition {
    /// One signed contribution per dimension, aligned with the reference's
    /// dimension order.
    pub contributions: Vec<f64>,
    /// A constant that does not belong to any dimension.
    ///
    /// Zero for every metric except cosine, where the distance is
    /// `1 − cos(a, b)` and the `1` is the offset.
    pub offset: f64,
}

impl Decomposition {
    /// The distance this decomposition sums to.
    pub fn total(&self) -> f64 {
        self.contributions.iter().sum::<f64>() + self.offset
    }
}

/// A decomposable distance between two scaled profiles.
///
/// Implement this to add a metric for ad-hoc use. A metric that should be
/// *stored in a reference* must also be a [`Metric`] enum variant, because
/// fitted state carries no trait objects (invariant #2).
pub trait DistanceMetric {
    /// Stable name, used in reports.
    fn name(&self) -> &'static str;

    /// Which input space this metric consumes.
    fn space(&self) -> Space;

    /// Decompose the distance between two vectors in this metric's space.
    fn decompose(&self, a: &[f64], b: &[f64]) -> Decomposition;

    /// The distance itself.
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        self.decompose(a, b).total()
    }
}

/// The metrics a reference can be configured with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Metric {
    /// **The default.** The angle between z-score vectors,
    /// `1 − cos(z_a, z_b)`.
    ///
    /// Evert et al. found it consistently better than Classic and Quadratic
    /// Delta and, unlike them, robust as the frequent-word count grows to
    /// 10,000 — "vector normalization appears to be the key to robust
    /// authorship attribution". Length normalization is also why it copes with
    /// mixed feature families: a family with many dimensions cannot dominate by
    /// sheer count.
    #[default]
    CosineDelta,
    /// Burrows's Classic Delta: mean absolute z-score difference. Kept for
    /// comparability with the literature, not because it is better.
    BurrowsDelta,
    /// Eder's rank-weighted Delta: like Burrows, but earlier dimensions count
    /// for more.
    ///
    /// The weighting is over the reference's dimension order, which is
    /// frequency rank only when the pipeline is frequent-word dominated —
    /// with mixed families the weights are arbitrary, so prefer
    /// [`Metric::CosineDelta`] there. stylo's shipped implementation differs
    /// from its own documentation by a constant and an off-by-one weight, so
    /// cross-checks against it can only be expected to agree on *ranking*.
    Eder,
    /// MinMax / Ruzicka distance on non-negative relative frequencies.
    ///
    /// `1 − Σ min(a_i, b_i) / Σ max(a_i, b_i)`. Strong in Kestemont et al.'s
    /// evaluation and, unlike the Delta family, bounded in `[0, 1]`.
    MinMax,
}

impl Metric {
    /// Every metric, for iteration in tests and CLI help.
    pub const ALL: &'static [Metric] = &[
        Metric::CosineDelta,
        Metric::BurrowsDelta,
        Metric::Eder,
        Metric::MinMax,
    ];

    /// Parse a metric from its serialized name.
    pub fn parse(s: &str) -> Option<Metric> {
        match s {
            "cosine_delta" | "cosine" => Some(Metric::CosineDelta),
            "burrows_delta" | "burrows" | "delta" => Some(Metric::BurrowsDelta),
            "eder" => Some(Metric::Eder),
            "min_max" | "minmax" | "ruzicka" => Some(Metric::MinMax),
            _ => None,
        }
    }
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl DistanceMetric for Metric {
    fn name(&self) -> &'static str {
        match self {
            Metric::CosineDelta => "cosine_delta",
            Metric::BurrowsDelta => "burrows_delta",
            Metric::Eder => "eder",
            Metric::MinMax => "min_max",
        }
    }

    fn space(&self) -> Space {
        match self {
            Metric::CosineDelta | Metric::BurrowsDelta | Metric::Eder => Space::ZScore,
            Metric::MinMax => Space::NonNegative,
        }
    }

    fn decompose(&self, a: &[f64], b: &[f64]) -> Decomposition {
        debug_assert_eq!(a.len(), b.len(), "profiles must share a dimension set");
        let n = a.len();
        if n == 0 {
            return Decomposition {
                contributions: Vec::new(),
                offset: 0.0,
            };
        }
        match self {
            // 1 − Σ a_i b_i / (‖a‖‖b‖). The `1` is the offset; each dimension
            // contributes the negative of its share of the cosine, so a
            // dimension where the two profiles agree *reduces* the distance.
            Metric::CosineDelta => {
                let na = norm(a);
                let nb = norm(b);
                let denom = na * nb;
                if denom <= f64::EPSILON {
                    return Decomposition {
                        contributions: vec![0.0; n],
                        offset: 1.0,
                    };
                }
                Decomposition {
                    contributions: (0..n).map(|i| -(a[i] * b[i]) / denom).collect(),
                    offset: 1.0,
                }
            }
            Metric::BurrowsDelta => Decomposition {
                contributions: (0..n).map(|i| (a[i] - b[i]).abs() / n as f64).collect(),
                offset: 0.0,
            },
            Metric::Eder => {
                let weights = eder_weights(n);
                Decomposition {
                    contributions: (0..n)
                        .map(|i| weights[i] * (a[i] - b[i]).abs() / n as f64)
                        .collect(),
                    offset: 0.0,
                }
            }
            // 1 − Σmin/Σmax rearranges exactly to Σ|a−b| / Σmax.
            Metric::MinMax => {
                let max_sum: f64 = (0..n).map(|i| a[i].max(b[i])).sum();
                if max_sum <= f64::EPSILON {
                    return Decomposition {
                        contributions: vec![0.0; n],
                        offset: 0.0,
                    };
                }
                Decomposition {
                    contributions: (0..n).map(|i| (a[i] - b[i]).abs() / max_sum).collect(),
                    offset: 0.0,
                }
            }
        }
    }
}

/// Eder's rank weights: dimension `i` of `n` is weighted `(n − i) / n`, so the
/// first dimension counts fully and the last counts for almost nothing.
fn eder_weights(n: usize) -> Vec<f64> {
    (0..n).map(|i| (n - i) as f64 / n as f64).collect()
}

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// One dimension's share of a distance, with everything needed to explain it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    /// The dimension.
    pub symbol: Symbol,
    /// Its human-readable name.
    pub name: String,
    /// Which family it came from.
    pub family: Family,
    /// What it is measured in.
    pub unit: Unit,
    /// Signed share of the distance. Positive pushes the two profiles apart;
    /// for cosine, negative means the dimension pulls them together.
    pub value: f64,
    /// The left profile's raw (unscaled) value.
    pub observed_a: f64,
    /// The right profile's raw (unscaled) value.
    pub observed_b: f64,
    /// The left profile's z-score against the reference corpus.
    pub z_a: f64,
    /// The right profile's z-score against the reference corpus.
    pub z_b: f64,
}

impl Contribution {
    /// Magnitude of the contribution, which is what reports sort by.
    pub fn magnitude(&self) -> f64 {
        self.value.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn decomposition_sums_to_distance_for_every_metric() {
        let a = [1.0, -2.0, 0.5, 3.0, 0.0];
        let b = [0.5, 1.0, 0.5, -1.0, 2.0];
        for metric in Metric::ALL {
            // MinMax needs non-negative input, which is what the reference
            // guarantees it; use shifted vectors here.
            let (x, y): (Vec<f64>, Vec<f64>) = if metric.space() == Space::NonNegative {
                (
                    a.iter().map(|v| v + 2.0).collect(),
                    b.iter().map(|v| v + 2.0).collect(),
                )
            } else {
                (a.to_vec(), b.to_vec())
            };
            let d = metric.decompose(&x, &y);
            approx(d.total(), metric.distance(&x, &y));
            assert_eq!(d.contributions.len(), a.len());
        }
    }

    #[test]
    fn identical_profiles_are_at_distance_zero() {
        let a = [1.0, 2.0, 3.0, 4.0];
        for metric in Metric::ALL {
            approx(metric.distance(&a, &a), 0.0);
        }
    }

    #[test]
    fn burrows_delta_is_mean_absolute_difference() {
        let a = [1.0, 2.0];
        let b = [3.0, 2.0];
        approx(Metric::BurrowsDelta.distance(&a, &b), 1.0);
    }

    #[test]
    fn cosine_delta_matches_the_textbook_formula() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        approx(Metric::CosineDelta.distance(&a, &b), 1.0);
        let c = [1.0, 1.0];
        approx(
            Metric::CosineDelta.distance(&a, &c),
            1.0 - 1.0 / 2.0f64.sqrt(),
        );
    }

    #[test]
    fn cosine_offset_is_documented_and_exact() {
        let a = [3.0, 1.0, -2.0];
        let b = [1.0, 4.0, 0.5];
        let d = Metric::CosineDelta.decompose(&a, &b);
        assert_eq!(d.offset, 1.0);
        let cosine: f64 = -d.contributions.iter().sum::<f64>();
        let expect = a.iter().zip(&b).map(|(x, y)| x * y).sum::<f64>() / (norm(&a) * norm(&b));
        approx(cosine, expect);
    }

    #[test]
    fn minmax_equals_one_minus_ruzicka_similarity() {
        let a: [f64; 3] = [0.5, 0.3, 0.2];
        let b: [f64; 3] = [0.1, 0.6, 0.3];
        let min_sum: f64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y): (&f64, &f64)| x.min(*y))
            .sum();
        let max_sum: f64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y): (&f64, &f64)| x.max(*y))
            .sum();
        approx(Metric::MinMax.distance(&a, &b), 1.0 - min_sum / max_sum);
    }

    #[test]
    fn minmax_is_bounded_in_unit_interval() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 1.0];
        let d = Metric::MinMax.distance(&a, &b);
        assert!((0.0..=1.0).contains(&d), "{d}");
        approx(d, 1.0);
    }

    #[test]
    fn eder_weights_earlier_dimensions_more() {
        // Same absolute difference in dimension 0 vs dimension 3.
        let base = [0.0, 0.0, 0.0, 0.0];
        let mut first = base;
        first[0] = 1.0;
        let mut last = base;
        last[3] = 1.0;
        assert!(Metric::Eder.distance(&base, &first) > Metric::Eder.distance(&base, &last));
        // Burrows treats them identically.
        approx(
            Metric::BurrowsDelta.distance(&base, &first),
            Metric::BurrowsDelta.distance(&base, &last),
        );
    }

    #[test]
    fn metric_names_round_trip() {
        for metric in Metric::ALL {
            assert_eq!(Metric::parse(metric.name()), Some(*metric));
            let json = serde_json::to_string(metric).unwrap();
            assert_eq!(serde_json::from_str::<Metric>(&json).unwrap(), *metric);
        }
    }

    #[test]
    fn scaling_space_is_a_metric_property() {
        assert_eq!(Metric::CosineDelta.space(), Space::ZScore);
        assert_eq!(Metric::MinMax.space(), Space::NonNegative);
    }

    #[test]
    fn zero_vectors_do_not_produce_nan() {
        let zero = [0.0, 0.0, 0.0];
        let a = [1.0, 2.0, 3.0];
        for metric in Metric::ALL {
            let d = metric.distance(&zero, &zero);
            assert!(d.is_finite(), "{metric}: {d}");
            assert!(metric.distance(&zero, &a).is_finite());
        }
    }
}
