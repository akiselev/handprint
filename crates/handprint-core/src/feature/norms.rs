//! Generic weighted-lexicon density.
//!
//! Concreteness density, arousal density, sensory-modality density and the
//! booster-weight profile are the same measurement over different tables: take
//! a per-word score, look every token up, and report the distribution. One
//! feature covers all of them, which is why there is no `Concreteness` feature
//! and no `Arousal` feature.
//!
//! Three dimensions per pack:
//!
//! * `norm:{pack}:mean` — the central tendency. Bukowski's concreteness mean is
//!   high, Poe's arousal mean is high, and the marketing register's arousal mean
//!   is the thing the affect family is actually about.
//! * `norm:{pack}:p90` — the tail. An author who reaches for one very concrete
//!   image per paragraph and is otherwise abstract has a middling mean and a
//!   high p90; the mean alone cannot tell them apart.
//! * `norm:{pack}:high_share` — the share of scored tokens in the pack's own
//!   top quartile, which is the cheapest form of "how often does this author go
//!   to that end of the scale".
//!
//! **Unscored tokens are excluded, never imputed.** A word the table has never
//! seen carries no information about the dimension, and imputing the mean would
//! drag the variance toward zero exactly where an author's unusual vocabulary
//! lives. Below [`MIN_SCORED`] scored tokens every dimension is marked missing.

use serde::{Deserialize, Serialize};

use super::{ratio, DimInfo, Family, Feature, FitContext, FittedFeature, PackLicense, Unit};
use crate::error::Result;
use crate::feature::pack::NormPack;
use crate::text::Analysis;
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Fewest scored tokens before a density means anything.
pub const MIN_SCORED: usize = 20;

/// Configuration for the norm-density family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormDensity {
    /// The weighted lexicon to measure against.
    pub pack: NormPack,
}

impl NormDensity {
    /// Measure against a norm pack.
    pub fn new(pack: NormPack) -> Self {
        NormDensity { pack }
    }
}

/// Fitted [`NormDensity`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedNorms {
    dims: Vec<DimInfo>,
    mean: Symbol,
    p90: Symbol,
    high_share: Symbol,
    table: Vec<(String, f64)>,
    high_cut: f64,
    license: PackLicense,
}

impl FittedNorms {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Register;

    /// The score of a word, if the table has one.
    pub fn score_of(&self, word: &str) -> Option<f64> {
        self.table
            .binary_search_by(|(t, _)| t.as_str().cmp(word))
            .ok()
            .map(|i| self.table[i].1)
    }
}

impl Feature for NormDensity {
    type Fitted = FittedNorms;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedNorms> {
        self.pack.validate()?;
        let ns = &self.pack.name;
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: String| {
            let sym = interner.intern(&name);
            dims.push(DimInfo::new(sym, Family::Register, Unit::Index));
            sym
        };
        let mean = push(interner, format!("norm:{ns}:mean"));
        let p90 = push(interner, format!("norm:{ns}:p90"));
        let high_share = {
            let sym = interner.intern(&format!("norm:{ns}:high_share"));
            dims.push(DimInfo::new(sym, Family::Register, Unit::Fraction));
            sym
        };

        Ok(FittedNorms {
            dims,
            mean,
            p90,
            high_share,
            table: self.pack.table(),
            high_cut: self.pack.quantile(0.75).unwrap_or(0.0),
            license: PackLicense {
                pack: self.pack.qualified_name(),
                license: self.pack.license.clone(),
                redistributable: self.pack.redistributable,
            },
        })
    }
}

impl FittedFeature for FittedNorms {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Register
    }

    fn pack_licenses(&self) -> Vec<PackLicense> {
        vec![self.license.clone()]
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let mut scores: Vec<f64> = Vec::new();
        let mut high = 0usize;
        for (token, form) in analysis.tokens().lexical() {
            let Some(score) = self.score_of(form) else {
                continue;
            };
            scores.push(score);
            if score >= self.high_cut {
                high += 1;
                out.note_span(self.high_share, token.span);
            }
        }
        if scores.len() < MIN_SCORED {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        out.set(self.mean, util::mean(&scores));
        util::sort_floats(&mut scores);
        out.set(
            self.p90,
            util::quantile_sorted(&scores, 0.90).unwrap_or(0.0),
        );
        out.set(self.high_share, ratio(high, scores.len()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::packs;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn transform(pack: NormPack, text: &str) -> (FeatureVector, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = NormDensity::new(pack).fit(&ctx, &mut interner).unwrap();
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    /// Twenty-plus concrete nouns, so the floor is cleared.
    const CONCRETE: &str = "the brick the chair the table the window the door the hammer the \
        shoe the dog the cat the tree the river the bicycle the kettle the spoon the bottle \
        the pencil the carpet the umbrella the suitcase the elbow the pie the bread";

    /// The same shape with abstractions.
    const ABSTRACT: &str = "the justice the freedom the truth the meaning the purpose the hope \
        the despair the boredom the happiness the misery the theory the concept the idea the \
        possibility the reason the consequence the existence the eternity the infinity the \
        bureaucracy the policy the strategy the assumption";

    #[test]
    fn a_concrete_document_scores_above_an_abstract_one() {
        let name = "norm:concreteness-stub:mean";
        let (concrete, i) = transform(packs::concreteness_stub(), CONCRETE);
        let (abstract_doc, _) = transform(packs::concreteness_stub(), ABSTRACT);
        let sym = i.get(name).unwrap();
        assert!(
            concrete.get(sym) > abstract_doc.get(sym),
            "{} vs {}",
            concrete.get(sym),
            abstract_doc.get(sym)
        );
    }

    #[test]
    fn the_high_share_and_p90_separate_the_tail_from_the_mean() {
        let (v, i) = transform(packs::concreteness_stub(), CONCRETE);
        assert!(v.get(i.get("norm:concreteness-stub:high_share").unwrap()) > 0.6);
        assert!(v.get(i.get("norm:concreteness-stub:p90").unwrap()) > 4.0);
        let (v, i) = transform(packs::concreteness_stub(), ABSTRACT);
        assert!(v.get(i.get("norm:concreteness-stub:high_share").unwrap()) < 0.2);
    }

    #[test]
    fn a_document_with_too_few_scored_tokens_is_missing_not_zero() {
        // Unscored tokens are excluded rather than imputed, so a document the
        // table barely covers has no density at all — which is the honest
        // answer, not a mean dragged toward the middle by imputation.
        let (v, i) = transform(packs::concreteness_stub(), "xyzzy plugh frobnitz quux");
        assert!(v.is_missing(i.get("norm:concreteness-stub:mean").unwrap()));
        assert!(v.is_missing(i.get("norm:concreteness-stub:p90").unwrap()));
    }

    #[test]
    fn one_feature_covers_every_norm_pack() {
        // The reason this is generic: booster weights are a different question
        // with the same shape, and they get the same three dimensions.
        let (_, i) = transform(packs::vader_boosters(), CONCRETE);
        assert!(i.get("norm:vader-boosters:mean").is_some());
        assert!(i.get("norm:vader-boosters:high_share").is_some());
    }

    #[test]
    fn the_pack_license_rolls_up() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = NormDensity::new(packs::vader_boosters())
            .fit(&ctx, &mut interner)
            .unwrap();
        let licenses = fitted.pack_licenses();
        assert_eq!(licenses[0].license, "MIT");
        assert!(licenses[0].pack.starts_with("vader-boosters@"));
    }

    #[test]
    fn an_empty_pack_is_rejected_at_fit() {
        let mut pack = packs::vader_boosters();
        pack.entries.clear();
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        assert!(NormDensity::new(pack).fit(&ctx, &mut interner).is_err());
    }
}
