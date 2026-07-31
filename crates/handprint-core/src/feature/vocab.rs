//! Term universes and contrastively discovered vocabulary.
//!
//! A **term universe** is one consistent way of cutting text into countable
//! units: words, word bigrams, character 4-grams. It matters because the
//! Fightin' Words model in [`contrast`](crate::contrast) does multinomial
//! bookkeeping — `n_i` is the number of draws — and mixing unigrams with
//! bigrams makes that number meaningless. One universe per contrast fit; this
//! type is what enforces it.
//!
//! [`ContrastVocab`] closes the loop from discovery to classification: terms a
//! contrast model found discriminative become ordinary feature dimensions, so
//! the same vocabulary that *explained* a difference can be used to *measure*
//! it on new text.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::{Error, Result};
use crate::text::{Analysis, ScoringText, Span};
use crate::vector::{Interner, Symbol, VectorBuilder};

/// One consistent way of cutting text into countable units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "universe", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Universe {
    /// Word and number tokens.
    Words,
    /// Adjacent word pairs.
    WordBigrams,
    /// Adjacent word triples.
    WordTrigrams,
    /// Character n-grams over the scoring text, spaces included.
    CharNgrams {
        /// Window width in characters.
        n: usize,
    },
}

impl Universe {
    /// Short name used in dimension names and log lines.
    pub fn as_str(self) -> String {
        match self {
            Universe::Words => "w1".into(),
            Universe::WordBigrams => "w2".into(),
            Universe::WordTrigrams => "w3".into(),
            Universe::CharNgrams { n } => format!("c{n}"),
        }
    }

    /// Extract every term in this universe from a document, with its span.
    ///
    /// The returned length is `n_i` for the contrast model: the number of draws
    /// from this universe's multinomial.
    pub fn extract(self, analysis: &Analysis<'_>) -> Vec<(String, Span)> {
        match self {
            Universe::Words => analysis
                .tokens()
                .lexical()
                .map(|(t, f)| (f.to_owned(), t.span))
                .collect(),
            Universe::WordBigrams => word_ngrams(analysis, 2),
            Universe::WordTrigrams => word_ngrams(analysis, 3),
            Universe::CharNgrams { n } => char_ngrams(analysis.scoring(), n),
        }
    }
}

fn word_ngrams(analysis: &Analysis<'_>, n: usize) -> Vec<(String, Span)> {
    let items: Vec<(&str, Span)> = analysis
        .tokens()
        .lexical()
        .map(|(t, f)| (f, t.span))
        .collect();
    if items.len() < n {
        return Vec::new();
    }
    items
        .windows(n)
        .map(|w| {
            let text = w.iter().map(|(f, _)| *f).collect::<Vec<_>>().join(" ");
            (text, Span::new(w[0].1.start, w[n - 1].1.end))
        })
        .collect()
}

fn char_ngrams(scoring: &ScoringText, n: usize) -> Vec<(String, Span)> {
    let text = scoring.as_str();
    let starts: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    if starts.len() < n || n == 0 {
        return Vec::new();
    }
    (0..=starts.len() - n)
        .map(|i| {
            let start = starts[i];
            let end = starts.get(i + n).copied().unwrap_or(text.len());
            (
                text[start..end].to_lowercase(),
                scoring.to_source_span(Span::new(start, end)),
            )
        })
        .collect()
}

/// One term a contrast model kept, with the z-score that earned it a place.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastTerm {
    /// The term itself, in the universe's normalized form.
    pub text: String,
    /// Signed z-score: positive means the term favours side A.
    pub z: f64,
}

/// A vocabulary discovered by [`ContrastModel`](crate::contrast::ContrastModel),
/// used as a feature.
///
/// This is the only feature whose "fit" reads nothing from the corpus: the
/// vocabulary was already decided by the contrast that produced it, and re-deriving
/// it here would be a second, inconsistent selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastVocab {
    /// Name of the contrast this came from, used as the dimension namespace.
    pub name: String,
    /// Which universe the terms are counted in.
    pub universe: Universe,
    /// The terms.
    pub terms: Vec<ContrastTerm>,
}

impl ContrastVocab {
    /// Build a vocabulary feature directly.
    pub fn new(name: impl Into<String>, universe: Universe, terms: Vec<ContrastTerm>) -> Self {
        ContrastVocab {
            name: name.into(),
            universe,
            terms,
        }
    }
}

/// Fitted [`ContrastVocab`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedContrastVocab {
    dims: Vec<DimInfo>,
    name: String,
    universe: Universe,
    lookup: HashMap<String, Symbol>,
    z: HashMap<String, f64>,
}

impl FittedContrastVocab {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Contrast;

    /// The contrast this vocabulary came from.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The universe terms are counted in.
    pub fn universe(&self) -> Universe {
        self.universe
    }

    /// The signed z-score a term carried out of the contrast, if it is in the
    /// vocabulary. Positive favours side A.
    pub fn z_of(&self, term: &str) -> Option<f64> {
        self.z.get(term).copied()
    }
}

impl Feature for ContrastVocab {
    type Fitted = FittedContrastVocab;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedContrastVocab> {
        if self.terms.is_empty() {
            return Err(Error::InvalidConfig {
                what: "ContrastVocab::terms",
                detail: "vocabulary is empty; lower the z threshold on the contrast model".into(),
            });
        }
        let ns = &self.name;
        let u = self.universe.as_str();
        let mut dims = Vec::with_capacity(self.terms.len());
        let mut lookup = HashMap::with_capacity(self.terms.len());
        let mut z = HashMap::with_capacity(self.terms.len());
        for term in &self.terms {
            let sym = interner.intern(&format!("contrast:{ns}:{u}:{}", term.text));
            dims.push(DimInfo::new(sym, Family::Contrast, Unit::RelativeFrequency));
            lookup.insert(term.text.clone(), sym);
            z.insert(term.text.clone(), term.z);
        }
        Ok(FittedContrastVocab {
            dims,
            name: self.name.clone(),
            universe: self.universe,
            lookup,
            z,
        })
    }
}

impl FittedFeature for FittedContrastVocab {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Contrast
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let terms = self.universe.extract(analysis);
        if terms.is_empty() {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        let total = terms.len();
        let mut counts: HashMap<Symbol, usize> = HashMap::new();
        for (text, span) in &terms {
            if let Some(&sym) = self.lookup.get(text) {
                *counts.entry(sym).or_insert(0) += 1;
                out.note_span(sym, *span);
            }
        }
        for dim in &self.dims {
            let count = counts.get(&dim.symbol).copied().unwrap_or(0);
            out.set(dim.symbol, ratio(count, total));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::text::{Document, Tokenizer};

    fn analyze(text: &str) -> (Document, Tokenizer) {
        (Document::new(text), Tokenizer::default())
    }

    #[test]
    fn word_universe_extracts_lexical_tokens_only() {
        let (doc, tok) = analyze("hello, world 42!");
        let a = doc.analyze(&tok);
        let terms: Vec<String> = Universe::Words
            .extract(&a)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert_eq!(terms, vec!["hello", "world", "42"]);
    }

    #[test]
    fn bigram_universe_spans_both_words() {
        let text = "alpha beta gamma";
        let (doc, tok) = analyze(text);
        let a = doc.analyze(&tok);
        let terms = Universe::WordBigrams.extract(&a);
        assert_eq!(terms[0].0, "alpha beta");
        assert_eq!(&text[terms[0].1.range()], "alpha beta");
        assert_eq!(terms.len(), 2);
    }

    #[test]
    fn char_universe_keeps_spaces() {
        let (doc, tok) = analyze("ab cd");
        let a = doc.analyze(&tok);
        let terms: Vec<String> = Universe::CharNgrams { n: 3 }
            .extract(&a)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert_eq!(terms, vec!["ab ", "b c", " cd"]);
    }

    #[test]
    fn contrast_vocab_measures_relative_frequency() {
        let vocab = ContrastVocab::new(
            "ai-vs-human",
            Universe::Words,
            vec![
                ContrastTerm {
                    text: "delve".into(),
                    z: 4.2,
                },
                ContrastTerm {
                    text: "tapestry".into(),
                    z: 3.1,
                },
            ],
        );
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = vocab.fit(&ctx, &mut interner).unwrap();

        let doc = Document::new("delve delve into things");
        let a = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&a, &mut b);
        let v = b.build();
        let sym = interner.get("contrast:ai-vs-human:w1:delve").unwrap();
        assert!((v.get(sym) - 0.5).abs() < 1e-12);
        assert_eq!(
            v.get(interner.get("contrast:ai-vs-human:w1:tapestry").unwrap()),
            0.0
        );
        assert_eq!(fitted.z_of("delve"), Some(4.2));
    }

    #[test]
    fn empty_vocabulary_is_rejected() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let err = ContrastVocab::new("x", Universe::Words, vec![])
            .fit(&ctx, &mut interner)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidConfig { .. }));
    }
}
