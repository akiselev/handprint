#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod compare;
pub mod contrast;
pub mod corpus;
pub mod critique;
pub mod error;
pub mod explain;
pub mod feature;
pub mod reference;
pub mod text;
pub(crate) mod util;
pub mod vector;
pub mod verify;

pub use compare::{Contribution, Metric};
pub use contrast::{ContrastConfig, ContrastModel, ContrastSet, Prior, Side, Variance};
pub use corpus::{AuthorDocs, AuthorId, Corpus};
pub use critique::{Critic, CritiqueConfig, CritiqueReport, Finding, Mode, ThresholdProfile};
pub use error::{Error, Result};
pub use explain::{ContrastReport, Highlight, Pull};
pub use feature::{Confidence, Family, Feature, FeatureSpec, FittedFeature, Unit};
pub use reference::{
    Assessment, Calibration, CalibrationConfig, Comparison, LengthTier, Pipeline, Profile,
    Provenance, Reference,
};
pub use text::{Analysis, Document, Span, Tokenizer};
pub use vector::{FeatureVector, Interner, Symbol, VectorBuilder};
pub use verify::{verify, Verdict, VerifyConfig, VerifyScore};

/// The crate version, embedded in every serialized artifact and report so a
/// consumer can tell which code produced a number.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
