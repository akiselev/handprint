//! Small shared helpers.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// serde default for boolean fields that default to `true`.
pub(crate) const fn yes() -> bool {
    true
}

/// The crate's RNG. Every sampling step — calibration, General Imposters,
/// privacy culling — takes an explicit seed so results are reproducible and a
/// published number can be re-derived.
pub type Rng = ChaCha8Rng;

/// Build the crate RNG from a seed.
pub fn rng(seed: u64) -> Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// Mean of a slice, or zero when empty.
pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

/// Sample standard deviation (n-1 denominator), or zero for fewer than two
/// values.
pub fn stddev(xs: &[f64]) -> f64 {
    variance(xs).sqrt()
}

/// Sample variance (n-1 denominator), or zero for fewer than two values.
pub fn variance(xs: &[f64]) -> f64 {
    if xs.len() < 2 {
        return 0.0;
    }
    let m = mean(xs);
    xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (xs.len() - 1) as f64
}

/// Linear-interpolated quantile of a **sorted** slice.
///
/// `q` is clamped to `[0, 1]`. Returns `None` for an empty slice.
pub fn quantile_sorted(sorted: &[f64], q: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let q = q.clamp(0.0, 1.0);
    let pos = q * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return Some(sorted[lo]);
    }
    let frac = pos - lo as f64;
    Some(sorted[lo] * (1.0 - frac) + sorted[hi] * frac)
}

/// Fraction of a **sorted** slice that is `<= value`, in `[0, 1]`.
///
/// This is the empirical CDF, which is exactly the p-value the calibration
/// layer reports.
pub fn ecdf_sorted(sorted: &[f64], value: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = sorted.partition_point(|&x| x <= value);
    idx as f64 / sorted.len() as f64
}

/// Sort a vector of floats, putting NaN last.
pub fn sort_floats(xs: &mut [f64]) {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantiles_interpolate() {
        let xs = [0.0, 1.0, 2.0, 3.0];
        assert_eq!(quantile_sorted(&xs, 0.0), Some(0.0));
        assert_eq!(quantile_sorted(&xs, 1.0), Some(3.0));
        assert_eq!(quantile_sorted(&xs, 0.5), Some(1.5));
        assert_eq!(quantile_sorted(&[], 0.5), None);
    }

    #[test]
    fn ecdf_counts_at_or_below() {
        let xs = [1.0, 2.0, 2.0, 5.0];
        assert_eq!(ecdf_sorted(&xs, 0.0), 0.0);
        assert_eq!(ecdf_sorted(&xs, 2.0), 0.75);
        assert_eq!(ecdf_sorted(&xs, 9.0), 1.0);
    }

    #[test]
    fn moments() {
        let xs = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        assert!((mean(&xs) - 5.0).abs() < 1e-12);
        // Sample stddev of this textbook set is sqrt(32/7).
        assert!((stddev(&xs) - (32.0f64 / 7.0).sqrt()).abs() < 1e-12);
        assert_eq!(stddev(&[1.0]), 0.0);
    }
}
