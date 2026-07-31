//! Sentence, paragraph and markdown structure — including burstiness.
//!
//! The headline dimension here is the *dispersion* of sentence length, not its
//! mean. Low sentence-length variance ("burstiness") is the most consistently
//! documented structural marker of LLM prose, and the markdown-affinity
//! dimensions capture a post-training artifact of the same kind: models trained
//! on markdown-heavy data reach for bullets, bold and headers at rates humans
//! writing the same register do not.
//!
//! Caveat inherited from [`segment`](crate::text::segment): the sentence
//! splitter is rule-based, and its errors show up as inflated variance. The
//! same splitter runs over the reference corpus and the query, so a *relative*
//! comparison stays meaningful; an absolute burstiness number does not.

use serde::{Deserialize, Serialize};

use super::{per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::Result;
use crate::text::Analysis;
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Sentence openers, matched as a token sequence at the start of a sentence.
///
/// The first block is the formulaic-transition set that LLM prose over-uses;
/// the second is ordinary English openers, present so the formulaic rates have
/// something to be high *relative to*.
const OPENERS: &[(&str, &[&str])] = &[
    ("additionally", &["additionally"]),
    ("moreover", &["moreover"]),
    ("furthermore", &["furthermore"]),
    ("however", &["however"]),
    ("therefore", &["therefore"]),
    ("thus", &["thus"]),
    ("consequently", &["consequently"]),
    ("notably", &["notably"]),
    ("importantly", &["importantly"]),
    ("ultimately", &["ultimately"]),
    ("overall", &["overall"]),
    ("indeed", &["indeed"]),
    ("crucially", &["crucially"]),
    ("essentially", &["essentially"]),
    ("interestingly", &["interestingly"]),
    ("in_conclusion", &["in", "conclusion"]),
    ("in_summary", &["in", "summary"]),
    ("in_short", &["in", "short"]),
    ("in_essence", &["in", "essence"]),
    ("in_other_words", &["in", "other", "words"]),
    ("at_its_core", &["at", "its", "core"]),
    ("that_said", &["that", "said"]),
    ("on_the_other_hand", &["on", "the", "other", "hand"]),
    ("as_a_result", &["as", "a", "result"]),
    ("for_example", &["for", "example"]),
    ("for_instance", &["for", "instance"]),
    ("firstly", &["firstly"]),
    ("secondly", &["secondly"]),
    ("finally", &["finally"]),
    // Ordinary openers.
    ("i", &["i"]),
    ("it", &["it"]),
    ("this", &["this"]),
    ("there", &["there"]),
    ("the", &["the"]),
    ("we", &["we"]),
    ("you", &["you"]),
    ("but", &["but"]),
    ("and", &["and"]),
    ("so", &["so"]),
    ("if", &["if"]),
    ("when", &["when"]),
    ("while", &["while"]),
    ("what", &["what"]),
    ("why", &["why"]),
];

/// Discourse markers counted anywhere in the text.
///
/// Deliberately excludes the plain conjunctions (`and`, `but`, `or`) that the
/// frequent-word family already covers, so the two families stay close to
/// independent.
const MARKERS: &[&str] = &[
    "however",
    "therefore",
    "moreover",
    "furthermore",
    "additionally",
    "nevertheless",
    "nonetheless",
    "thus",
    "hence",
    "consequently",
    "meanwhile",
    "whereas",
    "arguably",
    "presumably",
    "apparently",
    "indeed",
    "essentially",
    "basically",
    "honestly",
    "obviously",
    "clearly",
    "actually",
    "frankly",
    "notably",
    "importantly",
    "ultimately",
    "crucially",
    "specifically",
    "particularly",
    "fundamentally",
];

/// Clause-separating punctuation, counted per sentence.
const CLAUSE_MARKS: &[char] = &[',', ';', ':', '\u{2014}', '\u{2013}'];

/// Configuration for the sentence and structure family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceStats {
    /// Emit per-opener rates.
    #[serde(default = "crate::util::yes")]
    pub openers: bool,
    /// Emit discourse-marker rates.
    #[serde(default = "crate::util::yes")]
    pub markers: bool,
    /// Emit markdown-structure rates.
    #[serde(default = "crate::util::yes")]
    pub markdown: bool,
    /// Fewest sentences at which dispersion statistics are meaningful. Below
    /// this the dispersion dimensions are marked missing rather than computed
    /// from two data points.
    #[serde(default = "default_min_sentences")]
    pub min_sentences: usize,
}

fn default_min_sentences() -> usize {
    4
}

impl Default for SentenceStats {
    fn default() -> Self {
        SentenceStats {
            openers: true,
            markers: true,
            markdown: true,
            min_sentences: default_min_sentences(),
        }
    }
}

/// Fitted [`SentenceStats`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedSentence {
    dims: Vec<DimInfo>,
    len_mean: Symbol,
    len_stddev: Symbol,
    len_variance: Symbol,
    len_fano: Symbol,
    len_cv: Symbol,
    len_p10: Symbol,
    len_p50: Symbol,
    len_p90: Symbol,
    count_rate: Symbol,
    clause_per_sentence: Symbol,
    comma_per_sentence: Symbol,
    para_sentences: Symbol,
    para_tokens: Symbol,
    para_rate: Symbol,
    openers: Vec<Symbol>,
    markers: Vec<Symbol>,
    markdown: Vec<Symbol>,
    md_structured_share: Option<Symbol>,
    min_sentences: usize,
}

/// Markdown rate dimensions, in emission order.
const MARKDOWN_DIMS: &[&str] = &[
    "heading",
    "bullet",
    "numbered",
    "quote",
    "code_fence",
    "inline_code",
    "bold",
    "italic",
    "table_row",
];

impl FittedSentence {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Sentence;

    /// Dimensions whose value depends on having enough sentences to estimate
    /// dispersion.
    fn dispersion_dims(&self) -> [Symbol; 8] {
        [
            self.len_mean,
            self.len_stddev,
            self.len_variance,
            self.len_fano,
            self.len_cv,
            self.len_p10,
            self.len_p50,
            self.len_p90,
        ]
    }
}

impl Feature for SentenceStats {
    type Fitted = FittedSentence;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedSentence> {
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Sentence, unit));
            sym
        };

        let len_mean = push(interner, "sent:len_mean", Unit::Tokens);
        let len_stddev = push(interner, "sent:len_stddev", Unit::Tokens);
        let len_variance = push(interner, "sent:len_variance", Unit::Index);
        let len_fano = push(interner, "sent:len_fano", Unit::Index);
        let len_cv = push(interner, "sent:len_cv", Unit::Index);
        let len_p10 = push(interner, "sent:len_p10", Unit::Tokens);
        let len_p50 = push(interner, "sent:len_p50", Unit::Tokens);
        let len_p90 = push(interner, "sent:len_p90", Unit::Tokens);
        let count_rate = push(interner, "sent:count_rate", Unit::PerThousandTokens);
        let clause_per_sentence = push(interner, "sent:clause_per_sentence", Unit::Index);
        let comma_per_sentence = push(interner, "sent:comma_per_sentence", Unit::Index);
        let para_sentences = push(interner, "para:sentences_mean", Unit::Index);
        let para_tokens = push(interner, "para:tokens_mean", Unit::Tokens);
        let para_rate = push(interner, "para:count_rate", Unit::PerThousandTokens);

        let openers = if self.openers {
            OPENERS
                .iter()
                .map(|(id, _)| {
                    push(
                        interner,
                        &format!("sent:opener:{id}"),
                        Unit::PerHundredSentences,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let markers = if self.markers {
            MARKERS
                .iter()
                .map(|m| {
                    push(
                        interner,
                        &format!("sent:marker:{m}"),
                        Unit::PerThousandTokens,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let (markdown, md_structured_share) = if self.markdown {
            let rates = MARKDOWN_DIMS
                .iter()
                .map(|m| push(interner, &format!("md:{m}_rate"), Unit::PerThousandTokens))
                .collect();
            (
                rates,
                Some(push(interner, "md:structured_line_share", Unit::Fraction)),
            )
        } else {
            (Vec::new(), None)
        };

        Ok(FittedSentence {
            dims,
            len_mean,
            len_stddev,
            len_variance,
            len_fano,
            len_cv,
            len_p10,
            len_p50,
            len_p90,
            count_rate,
            clause_per_sentence,
            comma_per_sentence,
            para_sentences,
            para_tokens,
            para_rate,
            openers,
            markers,
            markdown,
            md_structured_share,
            min_sentences: self.min_sentences.max(2),
        })
    }
}

impl FittedFeature for FittedSentence {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Sentence
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        let structure = analysis.structure();
        let stream = analysis.tokens();

        let mut lengths: Vec<f64> = structure
            .sentences
            .iter()
            .map(|s| s.lexical_len(stream) as f64)
            .filter(|&n| n > 0.0)
            .collect();

        if lengths.len() >= self.min_sentences {
            let mean = util::mean(&lengths);
            let variance = util::variance(&lengths);
            util::sort_floats(&mut lengths);
            out.set(self.len_mean, mean);
            out.set(self.len_stddev, variance.sqrt());
            out.set(self.len_variance, variance);
            // Fano factor: variance-to-mean ratio. A Poisson process gives 1;
            // uniform sentence lengths drive it toward 0.
            out.set(
                self.len_fano,
                if mean > 0.0 { variance / mean } else { 0.0 },
            );
            out.set(
                self.len_cv,
                if mean > 0.0 {
                    variance.sqrt() / mean
                } else {
                    0.0
                },
            );
            out.set(
                self.len_p10,
                util::quantile_sorted(&lengths, 0.10).unwrap_or(0.0),
            );
            out.set(
                self.len_p50,
                util::quantile_sorted(&lengths, 0.50).unwrap_or(0.0),
            );
            out.set(
                self.len_p90,
                util::quantile_sorted(&lengths, 0.90).unwrap_or(0.0),
            );
        } else {
            for sym in self.dispersion_dims() {
                out.mark_missing(sym);
            }
        }

        let sentences = structure.sentences.len();
        out.set(self.count_rate, per_1k(sentences, tokens));

        let mut clause_marks = 0usize;
        let mut commas = 0usize;
        for sentence in &structure.sentences {
            for c in analysis.text(sentence.span).chars() {
                if CLAUSE_MARKS.contains(&c) {
                    clause_marks += 1;
                }
                if c == ',' {
                    commas += 1;
                }
            }
        }
        out.set(self.clause_per_sentence, ratio(clause_marks, sentences));
        out.set(self.comma_per_sentence, ratio(commas, sentences));

        let prose_blocks: Vec<_> = structure
            .blocks
            .iter()
            .filter(|b| !matches!(b.kind, crate::text::BlockKind::CodeFence))
            .collect();
        let block_count = prose_blocks.len();
        out.set(self.para_sentences, ratio(sentences, block_count));
        out.set(self.para_tokens, ratio(tokens, block_count));
        out.set(self.para_rate, per_1k(block_count, tokens));

        self.opener_rates(analysis, sentences, out);
        self.marker_rates(analysis, tokens, out);
        self.markdown_rates(analysis, tokens, out);
    }
}

impl FittedSentence {
    fn opener_rates(&self, analysis: &Analysis<'_>, sentences: usize, out: &mut VectorBuilder) {
        if self.openers.is_empty() {
            return;
        }
        let mut counts = vec![0usize; OPENERS.len()];
        let stream = analysis.tokens();
        for sentence in &analysis.structure().sentences {
            let forms: Vec<&str> = stream.tokens()[sentence.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .take(4)
                .map(|t| stream.form(t))
                .collect();
            for (i, (_, pattern)) in OPENERS.iter().enumerate() {
                if forms.len() >= pattern.len() && forms[..pattern.len()] == **pattern {
                    counts[i] += 1;
                    out.note_span(self.openers[i], sentence.span);
                }
            }
        }
        for (i, &sym) in self.openers.iter().enumerate() {
            out.set(sym, ratio(counts[i], sentences) * 100.0);
        }
    }

    fn marker_rates(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        if self.markers.is_empty() {
            return;
        }
        let mut counts = vec![0usize; MARKERS.len()];
        for (token, form) in analysis.tokens().lexical() {
            if let Some(i) = MARKERS.iter().position(|m| *m == form) {
                counts[i] += 1;
                out.note_span(self.markers[i], token.span);
            }
        }
        for (i, &sym) in self.markers.iter().enumerate() {
            out.set(sym, per_1k(counts[i], tokens));
        }
    }

    fn markdown_rates(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        if self.markdown.is_empty() {
            return;
        }
        let md = &analysis.structure().markdown;
        let counts = [
            md.headings,
            md.bullets,
            md.numbered,
            md.quotes,
            md.code_fences,
            md.inline_code,
            md.bold,
            md.italic,
            md.table_rows,
        ];
        debug_assert_eq!(counts.len(), MARKDOWN_DIMS.len());
        for (i, &sym) in self.markdown.iter().enumerate() {
            out.set(sym, per_1k(counts[i], tokens));
        }
        if let Some(sym) = self.md_structured_share {
            let structured = md.headings + md.bullets + md.numbered + md.table_rows;
            out.set(sym, ratio(structured, md.lines));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn transform(text: &str) -> (FeatureVector, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = SentenceStats::default().fit(&ctx, &mut interner).unwrap();
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform(text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    #[test]
    fn uniform_sentence_lengths_have_low_dispersion() {
        let uniform = "aa bb cc dd. ee ff gg hh. ii jj kk ll. mm nn oo qq. rr ss tt uu.";
        let bursty = "Yes. aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr. No. \
                      tt uu vv ww xx yy zz ab cd ef gh ij kl mn op qr st uv wx. Sure.";
        assert!(value(uniform, "sent:len_variance") < value(bursty, "sent:len_variance"));
        assert!(value(uniform, "sent:len_fano") < value(bursty, "sent:len_fano"));
        assert!((value(uniform, "sent:len_mean") - 4.0).abs() < 1e-9);
        assert_eq!(value(uniform, "sent:len_stddev"), 0.0);
    }

    #[test]
    fn dispersion_is_missing_below_the_floor() {
        let (v, i) = transform("Only one sentence here.");
        assert!(v.is_missing(i.get("sent:len_variance").unwrap()));
        assert!(v.is_missing(i.get("sent:len_mean").unwrap()));
        // Rates that do not need dispersion are still emitted.
        assert!(v.get(i.get("sent:count_rate").unwrap()) > 0.0);
    }

    #[test]
    fn formulaic_openers_are_counted_per_hundred_sentences() {
        let text = "Additionally, we note this. Moreover, that. In conclusion, done. Fine.";
        assert!((value(text, "sent:opener:additionally") - 25.0).abs() < 1e-9);
        assert!((value(text, "sent:opener:moreover") - 25.0).abs() < 1e-9);
        assert!((value(text, "sent:opener:in_conclusion") - 25.0).abs() < 1e-9);
        assert_eq!(value(text, "sent:opener:however"), 0.0);
    }

    #[test]
    fn opener_spans_point_at_the_sentence() {
        let text = "Additionally, we note this. Fine.";
        let (v, i) = transform(text);
        let spans = v.spans(i.get("sent:opener:additionally").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(&text[spans[0].range()], "Additionally, we note this.");
    }

    #[test]
    fn discourse_markers_are_rates_per_thousand() {
        // Nine lexical tokens, one marker → 111.11 per 1k.
        let text = "one two however three four five six seven eight";
        assert!((value(text, "sent:marker:however") - 1000.0 / 9.0).abs() < 1e-6);
    }

    #[test]
    fn markdown_affinity_shows_up() {
        let plain = "Just prose here. Nothing structural at all in this text.";
        let structured = "# Head\n\n- one\n- two\n- three\n\n**bold** text here now.";
        assert_eq!(value(plain, "md:structured_line_share"), 0.0);
        assert!(value(structured, "md:structured_line_share") > 0.5);
        assert!(value(structured, "md:bullet_rate") > 0.0);
    }
}
