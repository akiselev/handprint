//! Lexical richness — length-corrected indices only.
//!
//! Raw type-token ratio and hapax counts are **not** here, and their absence is
//! deliberate: both fall monotonically with document length, so a TTR
//! difference between a 300-word comment and a 5,000-word essay measures length,
//! not vocabulary. McCarthy & Jarvis (2010) tested the standard indices against
//! length and found MTLD the only one that held; it is the primary dimension
//! here.
//!
//! MATTR is undefined below its window width. When that happens the dimension is
//! marked *missing* rather than silently falling back to whole-document TTR —
//! the fallback would quietly reintroduce the bias the index exists to avoid.
//! Yule's K and Simpson's D are length-invariant in theory and drift in
//! practice (Tweedie & Baayen 1998), so they carry the same length warnings as
//! everything else.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::{Error, Result};
use crate::text::Analysis;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Configuration for the lexical-richness family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Richness {
    /// Emit MTLD.
    #[serde(default = "crate::util::yes")]
    pub mtld: bool,
    /// TTR threshold at which an MTLD factor closes. 0.72 is the value
    /// McCarthy & Jarvis validated; changing it changes the scale of the index.
    #[serde(default = "default_mtld_threshold")]
    pub mtld_threshold: f64,
    /// MATTR window width in tokens. `0` disables MATTR.
    #[serde(default = "default_mattr_window")]
    pub mattr_window: usize,
    /// Emit Yule's K.
    #[serde(default = "crate::util::yes")]
    pub yule: bool,
    /// Emit Simpson's D.
    #[serde(default = "crate::util::yes")]
    pub simpson: bool,
    /// Fewest tokens at which any richness index is emitted at all.
    #[serde(default = "default_min_tokens")]
    pub min_tokens: usize,
}

fn default_mtld_threshold() -> f64 {
    0.72
}
fn default_mattr_window() -> usize {
    100
}
fn default_min_tokens() -> usize {
    50
}

impl Default for Richness {
    fn default() -> Self {
        Richness {
            mtld: true,
            mtld_threshold: default_mtld_threshold(),
            mattr_window: default_mattr_window(),
            yule: true,
            simpson: true,
            min_tokens: default_min_tokens(),
        }
    }
}

/// Fitted [`Richness`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedRichness {
    dims: Vec<DimInfo>,
    mtld: Option<Symbol>,
    mattr: Option<Symbol>,
    yule: Option<Symbol>,
    simpson: Option<Symbol>,
    mtld_threshold: f64,
    mattr_window: usize,
    min_tokens: usize,
}

impl FittedRichness {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Richness;
}

impl Feature for Richness {
    type Fitted = FittedRichness;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedRichness> {
        if !(0.0..1.0).contains(&self.mtld_threshold) {
            return Err(Error::InvalidConfig {
                what: "Richness::mtld_threshold",
                detail: format!("must be in [0, 1), got {}", self.mtld_threshold),
            });
        }
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: String| {
            let sym = interner.intern(&name);
            dims.push(DimInfo::new(sym, Family::Richness, Unit::Index));
            sym
        };
        let mtld = self.mtld.then(|| push(interner, "rich:mtld".into()));
        let mattr = (self.mattr_window > 0)
            .then(|| push(interner, format!("rich:mattr{}", self.mattr_window)));
        let yule = self.yule.then(|| push(interner, "rich:yule_k".into()));
        let simpson = self
            .simpson
            .then(|| push(interner, "rich:simpson_d".into()));

        if dims.is_empty() {
            return Err(Error::InvalidConfig {
                what: "Richness",
                detail: "every index is disabled".into(),
            });
        }
        Ok(FittedRichness {
            dims,
            mtld,
            mattr,
            yule,
            simpson,
            mtld_threshold: self.mtld_threshold,
            mattr_window: self.mattr_window,
            min_tokens: self.min_tokens,
        })
    }
}

impl FittedFeature for FittedRichness {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Richness
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let words: Vec<&str> = analysis.tokens().lexical().map(|(_, f)| f).collect();
        if words.len() < self.min_tokens {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }

        if let Some(sym) = self.mtld {
            match mtld(&words, self.mtld_threshold) {
                Some(value) => out.set(sym, value),
                None => out.mark_missing(sym),
            }
        }
        if let Some(sym) = self.mattr {
            match mattr(&words, self.mattr_window) {
                Some(value) => out.set(sym, value),
                None => out.mark_missing(sym),
            }
        }
        if self.yule.is_some() || self.simpson.is_some() {
            let spectrum = frequency_spectrum(&words);
            let n = words.len();
            if let Some(sym) = self.yule {
                out.set(sym, yules_k(&spectrum, n));
            }
            if let Some(sym) = self.simpson {
                match simpsons_d(&spectrum, n) {
                    Some(value) => out.set(sym, value),
                    None => out.mark_missing(sym),
                }
            }
        }
    }
}

/// Measure of Textual Lexical Diversity, averaged over forward and backward
/// passes (McCarthy & Jarvis 2010).
///
/// Returns `None` when no factor completes in either direction, which means the
/// text never got long enough for the index to mean anything.
pub fn mtld(words: &[&str], threshold: f64) -> Option<f64> {
    let forward = mtld_pass(words.iter().copied(), threshold)?;
    let backward = mtld_pass(words.iter().rev().copied(), threshold)?;
    Some((forward + backward) / 2.0)
}

fn mtld_pass<'a>(words: impl Iterator<Item = &'a str>, threshold: f64) -> Option<f64> {
    let mut factors = 0.0f64;
    let mut types: HashMap<&str, ()> = HashMap::new();
    let mut tokens = 0usize;
    let mut total = 0usize;
    let mut last_ttr = 1.0f64;

    for word in words {
        total += 1;
        tokens += 1;
        types.insert(word, ());
        last_ttr = types.len() as f64 / tokens as f64;
        if last_ttr <= threshold {
            factors += 1.0;
            types.clear();
            tokens = 0;
            last_ttr = 1.0;
        }
    }
    // Partial trailing factor, prorated by how far it got toward the threshold.
    if tokens > 0 {
        let remaining = (1.0 - last_ttr) / (1.0 - threshold);
        factors += remaining;
    }
    if factors <= 0.0 || total == 0 {
        return None;
    }
    Some(total as f64 / factors)
}

/// Moving-Average Type-Token Ratio over a fixed window.
///
/// Returns `None` when the document is shorter than the window — the index is
/// genuinely undefined there.
pub fn mattr(words: &[&str], window: usize) -> Option<f64> {
    if window == 0 || words.len() < window {
        return None;
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in &words[..window] {
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut sum = counts.len() as f64 / window as f64;
    let mut windows = 1usize;

    for i in window..words.len() {
        let leaving = words[i - window];
        if let Some(count) = counts.get_mut(leaving) {
            *count -= 1;
            if *count == 0 {
                counts.remove(leaving);
            }
        }
        *counts.entry(words[i]).or_insert(0) += 1;
        sum += counts.len() as f64 / window as f64;
        windows += 1;
    }
    Some(sum / windows as f64)
}

/// `V_i`: how many types occur exactly `i` times.
fn frequency_spectrum(words: &[&str]) -> HashMap<usize, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in words {
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut spectrum: HashMap<usize, usize> = HashMap::new();
    for count in counts.values() {
        *spectrum.entry(*count).or_insert(0) += 1;
    }
    spectrum
}

/// Yule's characteristic K: `10^4 * (Σ i² V_i − N) / N²`.
pub fn yules_k(spectrum: &HashMap<usize, usize>, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = spectrum
        .iter()
        .map(|(&i, &v)| (i * i) as f64 * v as f64)
        .sum();
    10_000.0 * (sum - n as f64) / (n as f64 * n as f64)
}

/// Simpson's D: the probability that two tokens drawn without replacement are
/// the same type.
pub fn simpsons_d(spectrum: &HashMap<usize, usize>, n: usize) -> Option<f64> {
    if n < 2 {
        return None;
    }
    let sum: f64 = spectrum
        .iter()
        .map(|(&i, &v)| v as f64 * i as f64 * (i as f64 - 1.0))
        .sum();
    Some(sum / (n as f64 * (n as f64 - 1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::text::{Document, Tokenizer};

    fn fitted(spec: Richness) -> (FittedRichness, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        (fitted, interner)
    }

    fn transform(spec: Richness, text: &str) -> (crate::FeatureVector, Interner) {
        let (fitted, interner) = fitted(spec);
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    #[test]
    fn mattr_is_missing_below_its_window() {
        // 60 tokens, window 100 → missing, and definitively not whole-doc TTR.
        let text = (0..60)
            .map(|i| format!("w{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let (v, i) = transform(Richness::default(), &text);
        let sym = i.get("rich:mattr100").unwrap();
        assert!(v.is_missing(sym));
        assert_eq!(v.get(sym), 0.0);
        // Yule's K is defined at this length and is emitted.
        assert!(!v.is_missing(i.get("rich:yule_k").unwrap()));
    }

    #[test]
    fn everything_is_missing_below_the_floor() {
        let (v, i) = transform(Richness::default(), "only a handful of words here");
        for dim in [
            "rich:mtld",
            "rich:mattr100",
            "rich:yule_k",
            "rich:simpson_d",
        ] {
            assert!(v.is_missing(i.get(dim).unwrap()), "{dim}");
        }
    }

    #[test]
    fn mattr_matches_a_hand_computed_case() {
        // Window 2 over [a, a, b, c]: windows are {a,a}=0.5, {a,b}=1, {b,c}=1.
        let words = ["a", "a", "b", "c"];
        let got = mattr(&words, 2).unwrap();
        assert!((got - (0.5 + 1.0 + 1.0) / 3.0).abs() < 1e-12, "{got}");
        assert_eq!(mattr(&words, 5), None);
    }

    #[test]
    fn yule_and_simpson_match_hand_computed_values() {
        // N = 4, one type appearing twice and two types appearing once:
        // Σ i²V_i = 4·1 + 1·2 = 6; K = 1e4·(6−4)/16 = 1250.
        let words = ["a", "a", "b", "c"];
        let spectrum = frequency_spectrum(&words);
        assert!((yules_k(&spectrum, 4) - 1250.0).abs() < 1e-9);
        // D = Σ V_i·i·(i−1) / (N(N−1)) = 2/12.
        assert!((simpsons_d(&spectrum, 4).unwrap() - 2.0 / 12.0).abs() < 1e-12);
    }

    #[test]
    fn mtld_is_roughly_length_invariant() {
        // The same vocabulary pattern repeated twice should give nearly the
        // same MTLD as once — that is the property the index exists for, and
        // the property raw TTR does not have.
        let cycle: Vec<&str> = "the quick brown fox jumps over a lazy dog and then runs away fast"
            .split(' ')
            .collect();
        let short: Vec<&str> = cycle.iter().cycle().take(200).copied().collect();
        let long: Vec<&str> = cycle.iter().cycle().take(800).copied().collect();
        let (a, b) = (mtld(&short, 0.72).unwrap(), mtld(&long, 0.72).unwrap());
        assert!((a - b).abs() / a < 0.15, "mtld drifted: {a} vs {b}");

        let ttr = |w: &[&str]| {
            w.iter().collect::<std::collections::HashSet<_>>().len() as f64 / w.len() as f64
        };
        assert!(ttr(&short) > ttr(&long) * 1.5, "raw TTR should collapse");
    }

    #[test]
    fn mtld_ranks_repetitive_text_lower() {
        let vocab: Vec<String> = (0..100).map(|i| format!("w{i}")).collect();
        let vocab: Vec<&str> = vocab.iter().map(String::as_str).collect();
        let varied: Vec<&str> = vocab.iter().cycle().take(300).copied().collect();
        let repetitive: Vec<&str> = ["cat", "dog"].iter().cycle().take(300).copied().collect();
        assert!(mtld(&varied, 0.72).unwrap() > mtld(&repetitive, 0.72).unwrap());

        // A document whose every token is unique never closes a factor, so the
        // index is undefined rather than infinite.
        let all_unique: Vec<String> = (0..300).map(|i| format!("u{i}")).collect();
        let all_unique: Vec<&str> = all_unique.iter().map(String::as_str).collect();
        assert_eq!(mtld(&all_unique, 0.72), None);
    }
}
