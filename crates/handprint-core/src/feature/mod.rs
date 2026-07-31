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

pub mod biber;
pub mod char_ngram;
pub mod device;
pub mod formality;
pub mod frames;
pub mod lexicon;
pub mod mfw;
pub mod norms;
pub mod pack;
pub mod packs;
pub mod punct;
pub mod readability;
pub mod richness;
pub mod sentence;
pub mod surprisal;
pub mod vocab;

pub use biber::{BiberTier1, FittedBiber};
pub use char_ngram::{CharNgrams, FittedCharNgrams, NgramType, NgramTypes};
pub use device::{DeviceRates, FittedDevices};
pub use formality::{FittedFormality, RegisterClash};
pub use frames::{ComparisonFrames, FittedFrames};
pub use lexicon::{FittedLexicon, LexiconFeature, LexiconPack, PackSource, Phrase, Term};
pub use mfw::{FittedMfw, MostFrequentWords, VocabMode};
pub use norms::{FittedNorms, NormDensity};
pub use pack::{CountPack, NormEntry, NormPack};
pub use punct::{FittedPunct, PunctTypography};
pub use readability::{FittedReadability, Readability};
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
    /// Register: Biber lexico-grammatical rates, readability grades, weighted
    /// norm densities, formality variance.
    Register,
    /// Rhetorical device rates: comparison frames, litotes, absurd precision,
    /// transferred epithets, marketing structure.
    Device,
    /// Punch rhythm: where surprisal spikes land inside a sentence.
    ///
    /// Separate from [`Family::Surprisal`] on purpose. The surprisal family is
    /// the critic loop's default canary — scored, never reported — and a
    /// punchline dimension emitted by the same fitted feature would be
    /// unreportable by inheritance, or would contaminate the canary if it were
    /// reported. Splitting the family is what lets both work.
    Rhythm,
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
            Family::Register => "register",
            Family::Device => "device",
            Family::Rhythm => "rhythm",
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
            // Closed-class rates stabilize faster than frequent-word
            // distributions and slower than punctuation: a hedge or a modal is
            // common enough to count in a paragraph, rare enough that a
            // two-sentence draft says nothing about the rate.
            Family::Register => (150, 600),
            Family::Surprisal => (100, 300),
            // Pattern rates need length: a simile or a litotes is rare enough
            // that a short draft's rate is dominated by whether it happens to
            // contain one at all.
            Family::Device => (300, 1200),
            // A punch ratio needs several clause-split sentences before its
            // mean says anything, and those are a minority of sentences.
            Family::Rhythm => (200, 800),
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

/// A data pack embedded in a fitted feature, for the license rollup.
///
/// A fitted feature that consumed a pack carries that pack's contents inside
/// its own state — which is the point (transform must be pure), and which means
/// the reference inherits the pack's redistribution terms. Recording the terms
/// on the artifact is the only way a later reader can tell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackLicense {
    /// `name@version` of the pack.
    pub pack: String,
    /// The pack's declared license.
    pub license: String,
    /// Whether the pack may be shipped inside a published artifact.
    pub redistributable: bool,
}

impl PackLicense {
    /// A one-line rendering for [`Provenance::licenses`](crate::Provenance).
    pub fn describe(&self) -> String {
        format!(
            "{} ({}{})",
            self.pack,
            if self.license.is_empty() {
                "license unstated"
            } else {
                &self.license
            },
            if self.redistributable {
                ""
            } else {
                ", NOT redistributable"
            }
        )
    }
}

/// A fitted feature: pure, deterministic, serializable.
pub trait FittedFeature {
    /// Write this feature's dimensions for one document.
    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder);

    /// The dimensions this feature can emit, in a stable order.
    fn dims(&self) -> &[DimInfo];

    /// Which family this feature belongs to.
    fn family(&self) -> Family;

    /// Data packs whose contents this fitted state embeds.
    ///
    /// Defaults to none, which is right for every feature whose state comes
    /// from the corpus rather than from a shipped table.
    fn pack_licenses(&self) -> Vec<PackLicense> {
        Vec::new()
    }
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
            fn pack_licenses(&self) -> Vec<PackLicense> {
                match self { $( Fitted::$variant(f) => f.pack_licenses(), )* }
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
    /// Biber Tier-1 lexico-grammatical rates.
    Biber => BiberTier1, FittedBiber;
    /// Readability grades, passive proxy, acronym density.
    Readability => Readability, FittedReadability;
    /// Comparison frames: similes, negated vehicles, ironic hedges.
    Frames => ComparisonFrames, FittedFrames;
    /// Within-sentence formality variance and register clash.
    Formality => RegisterClash, FittedFormality;
    /// Rhetorical device rates and the marketing-structural family.
    Device => DeviceRates, FittedDevices;
    /// Weighted-lexicon density: concreteness, arousal, sensory, boosters.
    NormDensity => NormDensity, FittedNorms;
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
