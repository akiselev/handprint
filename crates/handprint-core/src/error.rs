//! Error type for the core crate.

/// Errors produced while fitting or applying a [`Reference`](crate::Reference).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A fit was attempted against a corpus that cannot support it.
    #[error("corpus is too small to fit {feature}: {detail}")]
    CorpusTooSmall {
        /// Feature family that rejected the corpus.
        feature: &'static str,
        /// Human-readable explanation of the shortfall.
        detail: String,
    },

    /// A pipeline was built without any features.
    #[error("pipeline has no features")]
    EmptyPipeline,

    /// Two profiles came from different references and cannot be compared.
    #[error("profiles come from different references ({left} vs {right})")]
    ReferenceMismatch {
        /// Fingerprint of the left profile's reference.
        left: String,
        /// Fingerprint of the right profile's reference.
        right: String,
    },

    /// A metric was asked for input it cannot consume (invariant #5).
    #[error("metric {metric} requires {required} input but the reference cannot provide it: {detail}")]
    ScalingUnavailable {
        /// Metric name.
        metric: &'static str,
        /// Required input space.
        required: &'static str,
        /// Explanation.
        detail: String,
    },

    /// A configuration value was outside its valid range.
    #[error("invalid configuration for {what}: {detail}")]
    InvalidConfig {
        /// The setting at fault.
        what: &'static str,
        /// Explanation.
        detail: String,
    },

    /// Calibration was requested but the reference carries none.
    #[error("reference has no calibration; call Pipeline::calibrate(..) before fit")]
    NotCalibrated,
}

/// Convenience alias.
pub type Result<T, E = Error> = core::result::Result<T, E>;
