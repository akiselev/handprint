//! Phase 3 — feature families.
//!
//! A [`Feature`] is configuration. Fitting one against a [`Corpus`] yields a
//! [`FittedFeature`], which is *data only* — no closures, no trait objects — so
//! the whole pipeline serializes (design invariant #2). Everything
//! corpus-relative (vocabulary selection, language-model counts, contrast
//! terms) is decided during `fit`; `transform` is pure (invariant #3).
//!
//! Dispatch goes through the [`FeatureSpec`] / [`Fitted`] enums rather than
//! `Box<dyn Feature>` for the same reason.

use serde::{Deserialize, Serialize};

use crate::corpus::Corpus;
use crate::error::Result;
use crate::text::Analysis;
use crate::vector::{Interner, Symbol, VectorBuilder};

pub mod char_ngram;
pub mod lexicon;
pub mod mfw;
pub mod punct;
pub mod richness;
pub mod sentence;
pub mod surprisal;
pub mod vocab;

pub use char_ngram::{CharNgrams, FittedCharNgrams, NgramType, NgramTypes};
pub use lexicon::{FittedLexicon, LexiconFeature, LexiconPack, PackSource, Phrase, Term};
pub use mfw::{FittedMfw, MostFrequentWords, VocabMode};
pub use punct::{FittedPunct, PunctTypography};
pub use richness::{FittedRichness, Richness};
pub use sentence::{FittedSentence, SentenceStats};
pub use surprisal::{FittedSurprisal, SurprisalLm};
pub use vocab::{ContrastVocab, FittedContrastVocab};

/// The family a dimension belongs to.
///
/// Families are the unit of confidence reporting, of top-k finding selection,
/// and of the canary hold-out in the critic loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Family {
    /// Punctuation and typography rates.
    Punct,
    /// Sentence, paragraph and markdown structure.
    Sentence,
    /// Most-frequent-word relative frequencies.
    Mfw,
    /// Typed character n-grams.
    CharNgram,
    /// Length-corrected lexical richness.
    Richness,
    /// Versioned word- and phrase-list hits.
    Lexicon,
    /// Corpus language-model surprisal.
    Surprisal,
    /// Contrastively discovered vocabulary.
    Contrast,
}

impl Family {
    /// Stable lowercase name, used as the namespace of every finding `id`.
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Punct => "punct",
            Family::Sentence => "sentence",
            Family::Mfw => "mfw",
            Family::CharNgram => "char_ngram",
            Family::Richness => "richness",
            Family::Lexicon => "lexicon",
            Family::Surprisal => "surprisal",
            Family::Contrast => "contrast",
        }
    }

    /// Lexical-token counts at which this family becomes trustworthy.
    ///
    /// Returns `(low, ok)`: below `low` the family reports
    /// [`Confidence::None`], between the two [`Confidence::Low`], at or above
    /// `ok` [`Confidence::Ok`]. Punctuation is usable far below the floors that
    /// the frequent-word and richness features need — which is why the critique
    /// contract reports confidence *per family* rather than once per document.
    pub fn token_floors(self) -> (usize, usize) {
        match self {
            Family::Punct => (40, 100),
            Family::Lexicon => (40, 100),
            Family::Sentence => (60, 150),
            Family::Surprisal => (100, 300),
            Family::Richness => (150, 500),
            Family::CharNgram => (300, 1000),
            Family::Contrast => (150, 500),
            Family::Mfw => (500, 2000),
        }
    }

    /// Confidence in this family's output at a given document length.
    pub fn confidence(self, lexical_tokens: usize) -> Confidence {
        let (low, ok) = self.token_floors();
        if lexical_tokens >= ok {
            Confidence::Ok
        } else if lexical_tokens >= low {
            Confidence::Low
        } else {
            Confidence::None
        }
    }
}

/// How much weight a family's output deserves at a document's length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Below the family's floor: ignore.
    None,
    /// Usable only alongside a clear signal.
    Low,
    /// Supported at this length.
    Ok,
}

/// The unit a dimension is measured in.
///
/// The critique layer uses this to decide whether a finding can be phrased as
/// an actionable instruction. "Reduce your em-dash rate from 8.2 to under 2.1
/// per thousand words" is actionable; "reduce dimension `c4:midword:atio` by
/// 0.0003" is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Unit {
    /// Occurrences per 1,000 lexical tokens.
    PerThousandTokens,
    /// Occurrences per 100 sentences.
    PerHundredSentences,
    /// A share of some total, in `[0, 1]`.
    Fraction,
    /// A relative frequency within its own term universe, in `[0, 1]`.
    RelativeFrequency,
    /// A count of tokens.
    Tokens,
    /// Bits of surprisal.
    Bits,
    /// An index with no natural unit (MTLD, Yule's K).
    Index,
}

impl Unit {
    /// Whether a finding in this unit can be turned into an instruction.
    pub fn is_actionable(self) -> bool {
        matches!(
            self,
            Unit::PerThousandTokens
                | Unit::PerHundredSentences
                | Unit::Fraction
                | Unit::Tokens
                | Unit::Index
        )
    }

    /// Absolute size below which a difference in this unit is noise.
    ///
    /// Used to make relative deviations comparable across units: a finding's
    /// severity comes from `|observed - band_edge| / (|band_edge| + floor)`,
    /// and without the floor a dimension whose band edge is 0.001 would report
    /// every rounding difference as a catastrophe.
    ///
    /// Also used as the floor under a dimension's z-scoring denominator, so
    /// that a dimension the corpus never varied in reports a large but bounded
    /// z rather than an unbounded one.
    pub fn noise_floor(self) -> f64 {
        match self {
            Unit::PerThousandTokens => 0.5,
            Unit::PerHundredSentences => 1.0,
            Unit::Fraction => 0.02,
            Unit::RelativeFrequency => 0.002,
            Unit::Tokens => 1.0,
            Unit::Bits => 0.1,
            Unit::Index => 1.0,
        }
    }

    /// Machine-readable name used in the critique contract.
    pub fn as_str(self) -> &'static str {
        match self {
            Unit::PerThousandTokens => "per_1k_tokens",
            Unit::PerHundredSentences => "per_100_sentences",
            Unit::Fraction => "fraction",
            Unit::RelativeFrequency => "relative_frequency",
            Unit::Tokens => "tokens",
            Unit::Bits => "bits",
            Unit::Index => "index",
        }
    }
}

/// Everything the rest of the crate needs to know about one dimension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimInfo {
    /// The dimension's symbol in the reference interner.
    pub symbol: Symbol,
    /// Which family produced it.
    pub family: Family,
    /// What it is measured in.
    pub unit: Unit,
    /// Whether this dimension is a rollup of others.
    ///
    /// A category rate or a family total is useful in a *distance*: it stays
    /// informative when the individual terms underneath it are each too sparse
    /// to measure. It is useless in a *finding*, because "reduce your total
    /// AI-lexicon rate by 100%" tells an agent nothing it can act on, and it
    /// crowds out the specific hits that do. Rollups therefore contribute to
    /// scores and never appear in the critique contract.
    #[serde(default)]
    pub aggregate: bool,
}

impl DimInfo {
    /// Build dimension metadata.
    pub fn new(symbol: Symbol, family: Family, unit: Unit) -> Self {
        DimInfo {
            symbol,
            family,
            unit,
            aggregate: false,
        }
    }

    /// Mark this dimension as a rollup of others. See [`DimInfo::aggregate`].
    pub fn rollup(mut self) -> Self {
        self.aggregate = true;
        self
    }
}

/// A pre-analyzed corpus document, handed to every feature's `fit`.
#[derive(Debug)]
pub struct FitDoc<'a> {
    /// Index into [`Corpus::authors`].
    pub author: usize,
    /// The analyzed document.
    pub analysis: Analysis<'a>,
}

/// What a feature sees at fit time.
///
/// Documents are analyzed once for the whole pipeline, so adding a feature
/// costs a pass over the token stream, not a re-tokenization.
#[derive(Debug)]
pub struct FitContext<'a> {
    corpus: &'a Corpus,
    docs: &'a [FitDoc<'a>],
}

impl<'a> FitContext<'a> {
    /// Bundle a corpus with its analyses.
    pub fn new(corpus: &'a Corpus, docs: &'a [FitDoc<'a>]) -> Self {
        FitContext { corpus, docs }
    }

    /// The corpus being fitted.
    #[inline]
    pub fn corpus(&self) -> &'a Corpus {
        self.corpus
    }

    /// The analyzed documents, in corpus order.
    #[inline]
    pub fn docs(&self) -> &'a [FitDoc<'a>] {
        self.docs
    }

    /// Just the analyses.
    pub fn analyses(&self) -> impl Iterator<Item = &'a Analysis<'a>> {
        self.docs.iter().map(|d| &d.analysis)
    }

    /// How many documents the corpus has.
    #[inline]
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    /// True when there is nothing to fit against.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Total lexical tokens across the corpus.
    pub fn total_tokens(&self) -> usize {
        self.docs.iter().map(|d| d.analysis.lexical_len()).sum()
    }
}

/// Feature configuration: what to measure.
pub trait Feature {
    /// The data-only fitted form.
    type Fitted: FittedFeature;

    /// Decide every corpus-relative parameter and intern the dimension names.
    fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<Self::Fitted>;
}

/// A fitted feature: pure, deterministic, serializable.
pub trait FittedFeature {
    /// Write this feature's dimensions for one document.
    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder);

    /// The dimensions this feature can emit, in a stable order.
    fn dims(&self) -> &[DimInfo];

    /// Which family this feature belongs to.
    fn family(&self) -> Family;
}

macro_rules! feature_kinds {
    ($( $(#[$meta:meta])* $variant:ident => $spec:ty, $fitted:ty );* $(;)?) => {
        /// A feature configuration, in serializable form.
        ///
        /// This enum exists instead of `Box<dyn Feature>` so that a pipeline
        /// round-trips through `serde` (invariant #2).
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(tag = "feature", rename_all = "snake_case")]
        #[non_exhaustive]
        pub enum FeatureSpec {
            $( $(#[$meta])* $variant($spec), )*
        }

        /// A fitted feature, in serializable form.
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(tag = "feature", rename_all = "snake_case")]
        #[non_exhaustive]
        pub enum Fitted {
            $( $(#[$meta])* $variant($fitted), )*
        }

        impl FeatureSpec {
            /// Fit this feature against a corpus.
            pub fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<Fitted> {
                Ok(match self {
                    $( FeatureSpec::$variant(f) => Fitted::$variant(Feature::fit(f, ctx, interner)?), )*
                })
            }

            /// The family this configuration produces.
            pub fn family(&self) -> Family {
                match self {
                    $( FeatureSpec::$variant(_) => <$fitted>::FAMILY, )*
                }
            }
        }

        impl FittedFeature for Fitted {
            fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
                match self { $( Fitted::$variant(f) => f.transform(analysis, out), )* }
            }
            fn dims(&self) -> &[DimInfo] {
                match self { $( Fitted::$variant(f) => f.dims(), )* }
            }
            fn family(&self) -> Family {
                match self { $( Fitted::$variant(f) => f.family(), )* }
            }
        }

        $(
            impl From<$spec> for FeatureSpec {
                fn from(f: $spec) -> Self { FeatureSpec::$variant(f) }
            }
        )*
    };
}

feature_kinds! {
    /// Punctuation and typography rates.
    Punct => PunctTypography, FittedPunct;
    /// Sentence, paragraph and markdown structure.
    Sentence => SentenceStats, FittedSentence;
    /// Most-frequent-word relative frequencies.
    Mfw => MostFrequentWords, FittedMfw;
    /// Typed character n-grams.
    CharNgram => CharNgrams, FittedCharNgrams;
    /// Length-corrected lexical richness.
    Richness => Richness, FittedRichness;
    /// Versioned word- and phrase-list hits.
    Lexicon => LexiconFeature, FittedLexicon;
    /// Corpus language-model surprisal.
    Surprisal => SurprisalLm, FittedSurprisal;
    /// Contrastively discovered vocabulary, from `contrast::ContrastModel`.
    Contrast => ContrastVocab, FittedContrastVocab;
}

/// Rate per 1,000 tokens, guarding the zero-length case.
pub(crate) fn per_1k(count: usize, tokens: usize) -> f64 {
    if tokens == 0 {
        0.0
    } else {
        count as f64 * 1000.0 / tokens as f64
    }
}

/// A ratio, guarding the zero-denominator case.
pub(crate) fn ratio(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_tiers_are_ordered() {
        assert_eq!(Family::Punct.confidence(10), Confidence::None);
        assert_eq!(Family::Punct.confidence(50), Confidence::Low);
        assert_eq!(Family::Punct.confidence(500), Confidence::Ok);
        // At 400 tokens punctuation is trustworthy and frequent words are not:
        // this is the whole reason confidence is per family.
        assert_eq!(Family::Punct.confidence(400), Confidence::Ok);
        assert_eq!(Family::Mfw.confidence(400), Confidence::None);
    }

    #[test]
    fn feature_spec_round_trips_through_serde() {
        let spec = FeatureSpec::from(PunctTypography::default());
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"feature\":\"punct\""), "{json}");
        assert_eq!(serde_json::from_str::<FeatureSpec>(&json).unwrap(), spec);
    }
}
