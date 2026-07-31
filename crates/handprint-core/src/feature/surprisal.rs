//! Corpus language-model surprisal — the GLTR/Binoculars insight without an LLM.
//!
//! A character n-gram model fitted on the reference corpus at `fit` time gives
//! a per-token surprisal series. What matters is not only its **mean** (how
//! unlike the reference this text is, on average) but its **variance**:
//! machine-generated text is characteristically flat — uniformly low surprisal,
//! few spikes — because sampling from a mode-seeking distribution rarely takes
//! the unlikely branch. That observation is the basis of every zero-shot
//! detector; getting it from an n-gram model fitted on your own corpus costs no
//! dependency and no inference.
//!
//! This family is also the crate's Goodhart canary. A rewrite loop that games
//! the reported features — swapping flagged words, adjusting punctuation rates —
//! does not move a distributional surprisal profile, because it is not a list of
//! things to avoid. Reported features improving while this one worsens is the
//! signature of feedback overfitting, and
//! [`critique`](crate::critique) checks for exactly that.
//!
//! The fitted model is counts only: a `HashMap` of n-gram and context
//! frequencies, serializable with the rest of the reference (invariant #2).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::{Error, Result};
use crate::text::{Analysis, Span};
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Configuration for the surprisal family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurprisalLm {
    /// Character n-gram order. 3 (a two-character context) is the default and
    /// behaves well down to corpora of a few hundred thousand characters.
    #[serde(default = "default_order")]
    pub order: usize,
    /// Add-k smoothing constant.
    #[serde(default = "default_smoothing")]
    pub smoothing: f64,
    /// Also fit a word-bigram model. Costs more memory and needs a larger
    /// corpus, but catches phrase-level flatness the character model misses.
    #[serde(default)]
    pub word_bigrams: bool,
    /// Drop n-grams seen fewer than this many times, to bound the model size.
    /// `0` keeps everything.
    #[serde(default)]
    pub min_count: u32,
    /// Fewest characters the corpus must supply before a fit is meaningful.
    #[serde(default = "default_min_chars")]
    pub min_corpus_chars: usize,
}

fn default_order() -> usize {
    3
}
fn default_smoothing() -> f64 {
    0.5
}
fn default_min_chars() -> usize {
    5_000
}

impl Default for SurprisalLm {
    fn default() -> Self {
        SurprisalLm {
            order: default_order(),
            smoothing: default_smoothing(),
            word_bigrams: false,
            min_count: 0,
            min_corpus_chars: default_min_chars(),
        }
    }
}

/// An add-k smoothed n-gram model over a fixed alphabet, counts only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NgramModel {
    order: usize,
    smoothing: f64,
    /// Counts of complete n-grams, keyed by the joined n-gram.
    ngrams: HashMap<String, u32>,
    /// Counts of `(n-1)`-gram contexts.
    contexts: HashMap<String, u32>,
    /// Vocabulary size, for the smoothing denominator.
    vocab: usize,
}

impl NgramModel {
    /// Surprisal in bits of `unit` following `context`.
    pub fn surprisal(&self, context: &str, unit: &str) -> f64 {
        let key = join(context, unit);
        let joint = self.ngrams.get(&key).copied().unwrap_or(0) as f64;
        let marginal = self.contexts.get(context).copied().unwrap_or(0) as f64;
        let k = self.smoothing;
        let p = (joint + k) / (marginal + k * self.vocab as f64);
        -p.max(f64::MIN_POSITIVE).log2()
    }

    /// The model's order.
    pub fn order(&self) -> usize {
        self.order
    }

    /// Number of distinct n-grams retained.
    pub fn len(&self) -> usize {
        self.ngrams.len()
    }

    /// True when the model saw nothing.
    pub fn is_empty(&self) -> bool {
        self.ngrams.is_empty()
    }
}

/// Unit separator inside model keys.
const SEP: char = '\u{1F}';

fn join(context: &str, unit: &str) -> String {
    if context.is_empty() {
        unit.to_owned()
    } else {
        format!("{context}{SEP}{unit}")
    }
}

/// The statistics computed from one surprisal series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurprisalStats {
    /// Mean surprisal in bits.
    pub mean: f64,
    /// Standard deviation.
    pub stddev: f64,
    /// Variance.
    pub variance: f64,
    /// Variance-to-mean ratio.
    pub fano: f64,
    /// 10th percentile.
    pub p10: f64,
    /// 90th percentile.
    pub p90: f64,
    /// Lag-1 autocorrelation.
    pub autocorr: f64,
    /// Fraction of units below the reference corpus's own low-surprisal
    /// threshold.
    pub low_share: f64,
}

/// Statistic names, in emission order.
const STATS: &[&str] = &[
    "mean",
    "stddev",
    "variance",
    "fano",
    "p10",
    "p90",
    "autocorr",
    "low_share",
];

/// Fitted [`SurprisalLm`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedSurprisal {
    dims: Vec<DimInfo>,
    chars: NgramModel,
    words: Option<NgramModel>,
    /// Surprisal below which a unit counts as "low", taken as the 25th
    /// percentile over the corpus the model was fitted on.
    char_low_threshold: f64,
    word_low_threshold: f64,
    char_dims: Vec<Symbol>,
    word_dims: Vec<Symbol>,
}

impl FittedSurprisal {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Surprisal;

    /// The fitted character model.
    pub fn char_model(&self) -> &NgramModel {
        &self.chars
    }

    /// The fitted word-bigram model, if one was requested.
    pub fn word_model(&self) -> Option<&NgramModel> {
        self.words.as_ref()
    }

    /// Per-word mean character surprisal, for span attribution.
    ///
    /// This is what makes the family explainable despite being distributional:
    /// the flattest or spikiest region of a draft can be pointed at.
    pub fn word_surprisal(&self, analysis: &Analysis<'_>) -> Vec<(Span, f64)> {
        let source = analysis.scoring().as_str();
        let series = char_series(&self.chars, source);
        let chars: Vec<usize> = source.char_indices().map(|(i, _)| i).collect();
        let mut out = Vec::new();
        for (token, _) in analysis.tokens().lexical() {
            // Map the token's source span onto the character series by scanning
            // for the covered character indices.
            let mut sum = 0.0;
            let mut count = 0usize;
            for (ci, &start) in chars.iter().enumerate() {
                let mapped = analysis.scoring().to_source_offset(start);
                if mapped >= token.span.end {
                    break;
                }
                if mapped >= token.span.start {
                    if let Some(&s) = series.get(ci) {
                        sum += s;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                out.push((token.span, sum / count as f64));
            }
        }
        out
    }
}

impl Feature for SurprisalLm {
    type Fitted = FittedSurprisal;

    fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedSurprisal> {
        if self.order < 2 {
            return Err(Error::InvalidConfig {
                what: "SurprisalLm::order",
                detail: "must be at least 2".into(),
            });
        }
        if self.smoothing <= 0.0 {
            return Err(Error::InvalidConfig {
                what: "SurprisalLm::smoothing",
                detail: "must be positive; add-0 smoothing gives infinite surprisal".into(),
            });
        }

        let total_chars: usize = ctx
            .analyses()
            .map(|a| a.scoring().as_str().chars().count())
            .sum();
        if total_chars < self.min_corpus_chars {
            return Err(Error::CorpusTooSmall {
                feature: "SurprisalLm",
                detail: format!(
                    "corpus has {total_chars} characters, need at least {}",
                    self.min_corpus_chars
                ),
            });
        }

        let mut chars = fit_char_model(ctx, self.order, self.smoothing, self.min_count);
        prune(&mut chars, self.min_count);
        let char_low_threshold = low_threshold(ctx, &chars, true);

        let (words, word_low_threshold) = if self.word_bigrams {
            let mut m = fit_word_model(ctx, self.smoothing, self.min_count);
            prune(&mut m, self.min_count);
            let t = low_threshold(ctx, &m, false);
            (Some(m), t)
        } else {
            (None, 0.0)
        };

        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: String, unit: Unit| {
            let sym = interner.intern(&name);
            dims.push(DimInfo::new(sym, Family::Surprisal, unit));
            sym
        };
        let char_dims = STATS
            .iter()
            .map(|s| {
                let unit = stat_unit(s);
                push(interner, format!("surp:char_{s}"), unit)
            })
            .collect();
        let word_dims = if words.is_some() {
            STATS
                .iter()
                .map(|s| {
                    let unit = stat_unit(s);
                    push(interner, format!("surp:word_{s}"), unit)
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(FittedSurprisal {
            dims,
            chars,
            words,
            char_low_threshold,
            word_low_threshold,
            char_dims,
            word_dims,
        })
    }
}

fn stat_unit(stat: &str) -> Unit {
    match stat {
        "mean" | "stddev" | "p10" | "p90" => Unit::Bits,
        "low_share" => Unit::Fraction,
        _ => Unit::Index,
    }
}

fn fit_char_model(ctx: &FitContext<'_>, order: usize, smoothing: f64, _min: u32) -> NgramModel {
    let mut ngrams: HashMap<String, u32> = HashMap::new();
    let mut contexts: HashMap<String, u32> = HashMap::new();
    let mut alphabet: HashMap<char, ()> = HashMap::new();

    for analysis in ctx.analyses() {
        let chars: Vec<char> = analysis.scoring().as_str().chars().collect();
        for c in &chars {
            alphabet.insert(*c, ());
        }
        for i in 0..chars.len() {
            let ctx_start = i.saturating_sub(order - 1);
            let context: String = chars[ctx_start..i].iter().collect();
            let unit = chars[i].to_string();
            *ngrams.entry(join(&context, &unit)).or_insert(0) += 1;
            *contexts.entry(context).or_insert(0) += 1;
        }
    }
    NgramModel {
        order,
        smoothing,
        ngrams,
        contexts,
        // +1 leaves smoothing mass for characters the corpus never contained.
        vocab: alphabet.len() + 1,
    }
}

fn fit_word_model(ctx: &FitContext<'_>, smoothing: f64, _min: u32) -> NgramModel {
    let mut ngrams: HashMap<String, u32> = HashMap::new();
    let mut contexts: HashMap<String, u32> = HashMap::new();
    let mut vocab: HashMap<String, ()> = HashMap::new();

    for analysis in ctx.analyses() {
        let words: Vec<&str> = analysis.tokens().lexical().map(|(_, f)| f).collect();
        for (i, word) in words.iter().enumerate() {
            vocab.insert((*word).to_owned(), ());
            let context = if i == 0 { "" } else { words[i - 1] };
            *ngrams.entry(join(context, word)).or_insert(0) += 1;
            *contexts.entry(context.to_owned()).or_insert(0) += 1;
        }
    }
    NgramModel {
        order: 2,
        smoothing,
        ngrams,
        contexts,
        vocab: vocab.len() + 1,
    }
}

fn prune(model: &mut NgramModel, min_count: u32) {
    if min_count <= 1 {
        return;
    }
    model.ngrams.retain(|_, &mut c| c >= min_count);
}

/// The 25th percentile of per-unit surprisal over the fitting corpus.
///
/// `low_share` counts units below this, so the dimension asks "is this text
/// flatter than the reference is with itself?" rather than comparing to an
/// arbitrary constant.
fn low_threshold(ctx: &FitContext<'_>, model: &NgramModel, chars: bool) -> f64 {
    let mut all: Vec<f64> = Vec::new();
    for analysis in ctx.analyses() {
        if chars {
            all.extend(char_series(model, analysis.scoring().as_str()));
        } else {
            all.extend(word_series(model, analysis));
        }
        if all.len() > 200_000 {
            break;
        }
    }
    util::sort_floats(&mut all);
    util::quantile_sorted(&all, 0.25).unwrap_or(0.0)
}

fn char_series(model: &NgramModel, text: &str) -> Vec<f64> {
    let chars: Vec<char> = text.chars().collect();
    (0..chars.len())
        .map(|i| {
            let start = i.saturating_sub(model.order - 1);
            let context: String = chars[start..i].iter().collect();
            model.surprisal(&context, &chars[i].to_string())
        })
        .collect()
}

fn word_series(model: &NgramModel, analysis: &Analysis<'_>) -> Vec<f64> {
    let words: Vec<&str> = analysis.tokens().lexical().map(|(_, f)| f).collect();
    words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let context = if i == 0 { "" } else { words[i - 1] };
            model.surprisal(context, w)
        })
        .collect()
}

/// Summarize a surprisal series.
pub fn summarize(series: &[f64], low_threshold: f64) -> Option<SurprisalStats> {
    if series.len() < 4 {
        return None;
    }
    let mean = util::mean(series);
    let variance = util::variance(series);
    let mut sorted = series.to_vec();
    util::sort_floats(&mut sorted);
    let low = series.iter().filter(|&&s| s < low_threshold).count() as f64 / series.len() as f64;
    Some(SurprisalStats {
        mean,
        stddev: variance.sqrt(),
        variance,
        fano: if mean > 0.0 { variance / mean } else { 0.0 },
        p10: util::quantile_sorted(&sorted, 0.10).unwrap_or(0.0),
        p90: util::quantile_sorted(&sorted, 0.90).unwrap_or(0.0),
        autocorr: autocorrelation(series, 1),
        low_share: low,
    })
}

/// Lag-`k` Pearson autocorrelation.
fn autocorrelation(series: &[f64], lag: usize) -> f64 {
    if series.len() <= lag + 1 {
        return 0.0;
    }
    let mean = util::mean(series);
    let denom: f64 = series.iter().map(|x| (x - mean).powi(2)).sum();
    if denom <= f64::EPSILON {
        return 0.0;
    }
    let numer: f64 = series[..series.len() - lag]
        .iter()
        .zip(&series[lag..])
        .map(|(a, b)| (a - mean) * (b - mean))
        .sum();
    numer / denom
}

impl FittedFeature for FittedSurprisal {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Surprisal
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let series = char_series(&self.chars, analysis.scoring().as_str());
        match summarize(&series, self.char_low_threshold) {
            Some(stats) => write_stats(&self.char_dims, &stats, out),
            None => {
                for &sym in &self.char_dims {
                    out.mark_missing(sym);
                }
            }
        }
        if let Some(model) = &self.words {
            let series = word_series(model, analysis);
            match summarize(&series, self.word_low_threshold) {
                Some(stats) => write_stats(&self.word_dims, &stats, out),
                None => {
                    for &sym in &self.word_dims {
                        out.mark_missing(sym);
                    }
                }
            }
        }
    }
}

fn write_stats(dims: &[Symbol], stats: &SurprisalStats, out: &mut VectorBuilder) {
    let values = [
        stats.mean,
        stats.stddev,
        stats.variance,
        stats.fano,
        stats.p10,
        stats.p90,
        stats.autocorr,
        stats.low_share,
    ];
    debug_assert_eq!(values.len(), STATS.len());
    for (&sym, &value) in dims.iter().zip(values.iter()) {
        out.set(sym, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::FitDoc;
    use crate::text::{Document, Tokenizer};

    fn fit_on(texts: &[String], spec: SurprisalLm) -> (FittedSurprisal, Interner) {
        let corpus = Corpus::new();
        let tok = Tokenizer::default();
        let docs: Vec<Document> = texts.iter().map(|t| Document::new(t.clone())).collect();
        let fit_docs: Vec<FitDoc<'_>> = docs
            .iter()
            .map(|d| FitDoc {
                author: 0,
                analysis: d.analyze(&tok),
            })
            .collect();
        let ctx = FitContext::new(&corpus, &fit_docs);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        (fitted, interner)
    }

    fn corpus_text() -> Vec<String> {
        let base = "the quick brown fox jumps over the lazy dog while the cat watches quietly \
                    from the windowsill and thinks about dinner plans for later tonight. ";
        vec![base.repeat(60)]
    }

    fn value(fitted: &FittedSurprisal, interner: &Interner, text: &str, dim: &str) -> f64 {
        let doc = Document::new(text);
        let a = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&a, &mut b);
        b.build().get(interner.get(dim).unwrap())
    }

    #[test]
    fn in_corpus_text_is_less_surprising_than_out_of_corpus_text() {
        let (fitted, interner) = fit_on(&corpus_text(), SurprisalLm::default());
        let familiar = value(
            &fitted,
            &interner,
            "the quick brown fox jumps over the lazy dog",
            "surp:char_mean",
        );
        let alien = value(
            &fitted,
            &interner,
            "zyxwvu qponml kjihgf edcbaz yxwvut",
            "surp:char_mean",
        );
        assert!(alien > familiar, "{alien} should exceed {familiar}");
    }

    #[test]
    fn repetitive_text_has_flatter_surprisal() {
        let (fitted, interner) = fit_on(&corpus_text(), SurprisalLm::default());
        let flat = "the the the the the the the the the the the the the the the";
        let varied = "the quick brown fox jumps over a lazy dog near zebras in Qatar";
        let flat_var = value(&fitted, &interner, flat, "surp:char_variance");
        let varied_var = value(&fitted, &interner, varied, "surp:char_variance");
        assert!(
            flat_var < varied_var,
            "flat={flat_var} should be below varied={varied_var}"
        );
    }

    #[test]
    fn model_serializes_and_reproduces_scores() {
        let (fitted, interner) = fit_on(&corpus_text(), SurprisalLm::default());
        let json = serde_json::to_string(&fitted).unwrap();
        let back: FittedSurprisal = serde_json::from_str(&json).unwrap();
        let text = "the quick brown fox";
        assert_eq!(
            value(&fitted, &interner, text, "surp:char_mean"),
            value(&back, &interner, text, "surp:char_mean")
        );
    }

    #[test]
    fn word_bigram_model_is_optional() {
        let spec = SurprisalLm {
            word_bigrams: true,
            ..Default::default()
        };
        let (fitted, interner) = fit_on(&corpus_text(), spec);
        assert!(fitted.word_model().is_some());
        assert!(interner.get("surp:word_mean").is_some());
        let v = value(
            &fitted,
            &interner,
            "the quick brown fox jumps over the lazy dog again",
            "surp:word_mean",
        );
        assert!(v > 0.0);
    }

    #[test]
    fn short_corpus_is_rejected() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let err = SurprisalLm::default().fit(&ctx, &mut interner).unwrap_err();
        assert!(matches!(err, Error::CorpusTooSmall { .. }));
    }

    #[test]
    fn word_surprisal_attributes_spans() {
        let (fitted, _) = fit_on(&corpus_text(), SurprisalLm::default());
        let text = "the quick zzzqqq fox";
        let doc = Document::new(text);
        let a = doc.analyze(&Tokenizer::default());
        let per_word = fitted.word_surprisal(&a);
        assert_eq!(per_word.len(), 4);
        let odd = per_word
            .iter()
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
            .unwrap();
        assert_eq!(&text[odd.0.range()], "zzzqqq");
    }

    #[test]
    fn autocorrelation_of_a_constant_series_is_zero() {
        assert_eq!(autocorrelation(&[1.0, 1.0, 1.0, 1.0], 1), 0.0);
        // A perfectly alternating series is strongly negatively correlated.
        assert!(autocorrelation(&[0.0, 1.0, 0.0, 1.0, 0.0, 1.0], 1) < -0.5);
    }
}
