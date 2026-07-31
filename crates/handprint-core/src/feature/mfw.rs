//! Most-frequent words — the classic Delta feature.
//!
//! Vocabulary selection happens at `fit` and only at `fit` (invariant #3): the
//! reference decides which words are dimensions, and every later profile is
//! measured against that fixed list.
//!
//! ## Which mode is topic-robust
//!
//! [`VocabMode::Corpus`] takes the top-*k* words as they fall, which on a
//! topically narrow corpus means content words become dimensions and the
//! "author" signal is partly a topic signal. [`VocabMode::FunctionWords`]
//! restricts the vocabulary to a closed class that carries no topic — this is
//! the setting to use when the query and the reference may differ in subject
//! matter, which is the dominant failure mode in authorship verification. Note
//! that character n-grams transfer *better* still across topic and genre
//! (Stamatatos 2013); frequent words are not the topic-robust feature family,
//! function-word-restricted frequent words merely are the robust *setting of
//! this family*.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::{Error, Result};
use crate::text::{Analysis, TokenKind};
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Closed-class English function words: determiners, pronouns, prepositions,
/// conjunctions, auxiliaries, degree adverbs and the commonest deictics.
///
/// Version-dated because it is data, not code: `2026.07`.
pub const FUNCTION_WORDS: &[&str] = &[
    "a", "about", "above", "across", "after", "again", "against", "all", "almost", "along",
    "already", "also", "although", "always", "am", "among", "an", "and", "another", "any",
    "anybody", "anyone", "anything", "anyway", "anywhere", "are", "around", "as", "at", "away",
    "back", "be", "became", "because", "been", "before", "behind", "being", "below", "beneath",
    "beside", "besides", "between", "beyond", "both", "but", "by", "came", "can", "cannot",
    "could", "did", "do", "does", "doing", "done", "down", "during", "each", "either", "else",
    "enough", "even", "ever", "every", "everybody", "everyone", "everything", "everywhere",
    "except", "few", "for", "former", "from", "further", "get", "got", "had", "has", "have",
    "having", "he", "hence", "her", "here", "hers", "herself", "him", "himself", "his", "how",
    "however", "i", "if", "in", "indeed", "inside", "instead", "into", "is", "it", "its", "itself",
    "just", "keep", "last", "latter", "least", "less", "let", "like", "likely", "made", "make",
    "many", "may", "me", "might", "mine", "more", "moreover", "most", "much", "must", "my",
    "myself", "near", "neither", "never", "nevertheless", "next", "no", "nobody", "none", "nor",
    "not", "nothing", "now", "nowhere", "of", "off", "often", "on", "once", "one", "only", "onto",
    "or", "other", "others", "otherwise", "ought", "our", "ours", "ourselves", "out", "outside",
    "over", "own", "past", "per", "perhaps", "quite", "rather", "really", "same", "seem", "seemed",
    "seems", "several", "shall", "she", "should", "since", "so", "some", "somebody", "somehow",
    "someone", "something", "sometimes", "somewhat", "somewhere", "still", "such", "than", "that",
    "the", "their", "theirs", "them", "themselves", "then", "there", "therefore", "these", "they",
    "this", "those", "though", "through", "throughout", "thus", "to", "together", "too", "toward",
    "towards", "under", "unless", "until", "up", "upon", "us", "used", "very", "via", "was", "we",
    "well", "were", "what", "whatever", "when", "whenever", "where", "whereas", "wherever",
    "whether", "which", "while", "who", "whoever", "whom", "whose", "why", "will", "with",
    "within", "without", "would", "yet", "you", "your", "yours", "yourself", "yourselves",
];

/// Date of the [`FUNCTION_WORDS`] list, carried into provenance manifests.
pub const FUNCTION_WORDS_VERSION: &str = "2026.07";

/// How the vocabulary is chosen at fit time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
#[derive(Default)]
pub enum VocabMode {
    /// The `top` most frequent tokens in the corpus. Maximum power on a
    /// topically homogeneous corpus, maximum topic leakage otherwise.
    #[default]
    Corpus,
    /// Restrict to [`FUNCTION_WORDS`], ordered by corpus frequency. The
    /// topic-robust setting of this family.
    FunctionWords,
    /// An explicit word list, ordered by corpus frequency.
    Explicit {
        /// The words to use as dimensions.
        words: Vec<String>,
    },
}


/// Configuration for the frequent-word family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MostFrequentWords {
    /// How many words to keep.
    #[serde(default = "default_top")]
    pub top: usize,
    /// Minimum fraction of corpus documents a word must appear in.
    ///
    /// Culling removes words that are frequent because one document is obsessed
    /// with them. `0.0` disables it; `0.5` is aggressive.
    #[serde(default)]
    pub culling: f64,
    /// How the vocabulary is chosen.
    #[serde(default)]
    pub mode: VocabMode,
    /// Treat punctuation tokens as vocabulary items too.
    #[serde(default)]
    pub include_punct: bool,
}

fn default_top() -> usize {
    500
}

impl Default for MostFrequentWords {
    fn default() -> Self {
        MostFrequentWords {
            top: default_top(),
            culling: 0.0,
            mode: VocabMode::default(),
            include_punct: false,
        }
    }
}

impl MostFrequentWords {
    /// Keep the `top` most frequent words.
    pub fn top(mut self, top: usize) -> Self {
        self.top = top;
        self
    }

    /// Require a word to appear in at least this fraction of corpus documents.
    pub fn culling(mut self, fraction: f64) -> Self {
        self.culling = fraction;
        self
    }

    /// Use a particular vocabulary mode.
    pub fn mode(mut self, mode: VocabMode) -> Self {
        self.mode = mode;
        self
    }

    /// Shorthand for the topic-robust setting.
    pub fn function_words(mut self) -> Self {
        self.mode = VocabMode::FunctionWords;
        self
    }
}

/// Fitted [`MostFrequentWords`]: the chosen vocabulary and its symbols.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedMfw {
    dims: Vec<DimInfo>,
    /// Vocabulary in rank order (most frequent first), aligned with `dims`.
    vocab: Vec<String>,
    lookup: HashMap<String, Symbol>,
    include_punct: bool,
}

impl FittedMfw {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Mfw;

    /// The fitted vocabulary, most frequent first.
    ///
    /// Rank order matters: [`Eder`](crate::compare::Eder) weights dimensions by
    /// it.
    pub fn vocab(&self) -> &[String] {
        &self.vocab
    }
}

impl Feature for MostFrequentWords {
    type Fitted = FittedMfw;

    fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedMfw> {
        if self.top == 0 {
            return Err(Error::InvalidConfig {
                what: "MostFrequentWords::top",
                detail: "must be at least 1".into(),
            });
        }
        if !(0.0..=1.0).contains(&self.culling) {
            return Err(Error::InvalidConfig {
                what: "MostFrequentWords::culling",
                detail: format!("must be in [0, 1], got {}", self.culling),
            });
        }

        let mut total: HashMap<&str, usize> = HashMap::new();
        let mut doc_freq: HashMap<&str, usize> = HashMap::new();
        let n_docs = ctx.len();
        for analysis in ctx.analyses() {
            let mut seen: HashMap<&str, ()> = HashMap::new();
            for (token, form) in analysis.tokens().iter() {
                if !self.accepts(token.kind) {
                    continue;
                }
                *total.entry(form).or_insert(0) += 1;
                seen.entry(form).or_insert(());
            }
            for word in seen.keys() {
                *doc_freq.entry(word).or_insert(0) += 1;
            }
        }

        let allowed: Option<Vec<&str>> = match &self.mode {
            VocabMode::Corpus => None,
            VocabMode::FunctionWords => Some(FUNCTION_WORDS.to_vec()),
            VocabMode::Explicit { words } => Some(words.iter().map(String::as_str).collect()),
        };

        let min_docs = if n_docs == 0 {
            0
        } else {
            (self.culling * n_docs as f64).ceil() as usize
        };

        let mut ranked: Vec<(&str, usize)> = total
            .iter()
            .filter(|(word, _)| allowed.as_ref().is_none_or(|a| a.contains(word)))
            .filter(|(word, _)| doc_freq.get(*word).copied().unwrap_or(0) >= min_docs)
            .map(|(w, c)| (*w, *c))
            .collect();
        // Frequency descending, then alphabetical, so the fit is deterministic
        // regardless of hash iteration order.
        ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        ranked.truncate(self.top);

        // An explicit or function-word list may name words the corpus never
        // used; keeping them as always-zero dimensions is worse than useless,
        // so they are dropped and the shortfall is only an error when nothing
        // at all survives.
        if ranked.is_empty() {
            return Err(Error::CorpusTooSmall {
                feature: "MostFrequentWords",
                detail: format!(
                    "no vocabulary survived selection over {n_docs} document(s) \
                     (culling={}, mode={:?})",
                    self.culling, self.mode
                ),
            });
        }

        let mut dims = Vec::with_capacity(ranked.len());
        let mut vocab = Vec::with_capacity(ranked.len());
        let mut lookup = HashMap::with_capacity(ranked.len());
        for (word, _) in ranked {
            let sym = interner.intern(&format!("mfw:{word}"));
            dims.push(DimInfo::new(sym, Family::Mfw, Unit::RelativeFrequency));
            vocab.push(word.to_owned());
            lookup.insert(word.to_owned(), sym);
        }

        Ok(FittedMfw {
            dims,
            vocab,
            lookup,
            include_punct: self.include_punct,
        })
    }
}

impl MostFrequentWords {
    fn accepts(&self, kind: TokenKind) -> bool {
        kind.is_lexical() || (self.include_punct && kind == TokenKind::Punct)
    }
}

impl FittedFeature for FittedMfw {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Mfw
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let mut counts: HashMap<Symbol, usize> = HashMap::new();
        let mut total = 0usize;
        for (token, form) in analysis.tokens().iter() {
            let accepted =
                token.kind.is_lexical() || (self.include_punct && token.kind == TokenKind::Punct);
            if !accepted {
                continue;
            }
            total += 1;
            if let Some(&sym) = self.lookup.get(form) {
                *counts.entry(sym).or_insert(0) += 1;
                out.note_span(sym, token.span);
            }
        }
        if total == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
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
    use crate::feature::FitDoc;
    use crate::text::{Document, Tokenizer};

    fn fit(spec: MostFrequentWords, texts: &[&str]) -> (FittedMfw, Interner) {
        let mut corpus = Corpus::new();
        for (i, t) in texts.iter().enumerate() {
            corpus.add(format!("a{i}"), [Document::new(*t)]);
        }
        let tok = Tokenizer::default();
        let docs: Vec<Document> = texts.iter().map(|t| Document::new(*t)).collect();
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

    #[test]
    fn selects_top_k_by_frequency() {
        let (fitted, _) = fit(
            MostFrequentWords::default().top(3),
            &["the cat the dog the bird", "the cat a cat"],
        );
        assert_eq!(fitted.vocab(), &["the", "cat", "a"]);
    }

    #[test]
    fn culling_drops_document_specific_words() {
        // "zebra" is frequent but confined to one of three documents.
        let texts = &["zebra zebra zebra zebra the", "the cat", "the dog"];
        let (loose, _) = fit(MostFrequentWords::default().top(10), texts);
        assert!(loose.vocab().contains(&"zebra".to_string()));
        let (culled, _) = fit(MostFrequentWords::default().top(10).culling(0.5), texts);
        assert!(!culled.vocab().contains(&"zebra".to_string()));
        assert!(culled.vocab().contains(&"the".to_string()));
    }

    #[test]
    fn function_word_mode_excludes_content_words() {
        let (fitted, _) = fit(
            MostFrequentWords::default().top(50).function_words(),
            &["the quantum chromodynamics of the vacuum is not the same as that of a solid"],
        );
        assert!(fitted.vocab().contains(&"the".to_string()));
        assert!(!fitted.vocab().contains(&"quantum".to_string()));
    }

    #[test]
    fn transform_emits_relative_frequencies() {
        let (fitted, interner) = fit(MostFrequentWords::default().top(2), &["the cat the dog"]);
        let doc = Document::new("the the the cat");
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&analysis, &mut b);
        let v = b.build();
        assert!((v.get(interner.get("mfw:the").unwrap()) - 0.75).abs() < 1e-12);
        assert!((v.get(interner.get("mfw:cat").unwrap()) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn hand_computed_fixture_matches() {
        // Golden: vocabulary {the, of, a}; document has 10 lexical tokens with
        // counts the=3, of=2, a=1.
        let (fitted, interner) = fit(
            MostFrequentWords::default().top(3),
            &["the the the of of a x y z w"],
        );
        assert_eq!(fitted.vocab(), &["the", "of", "a"]);
        let doc = Document::new("the the the of of a x y z w");
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&analysis, &mut b);
        let v = b.build();
        for (word, expect) in [("the", 0.3), ("of", 0.2), ("a", 0.1)] {
            let got = v.get(interner.get(&format!("mfw:{word}")).unwrap());
            assert!((got - expect).abs() < 1e-12, "{word}: {got} != {expect}");
        }
    }

    #[test]
    fn empty_vocabulary_is_an_error() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let err = MostFrequentWords::default().fit(&ctx, &mut interner).unwrap_err();
        assert!(matches!(err, Error::CorpusTooSmall { .. }));
    }
}
