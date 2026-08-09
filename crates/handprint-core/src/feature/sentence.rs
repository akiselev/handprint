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
use crate::text::{Analysis, Span};
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
    /// Emit the punch-rhythm and aside dimensions.
    ///
    /// The long-windup-then-short-punch contrast and the parenthetical
    /// digression rate. Off by default and `#[serde(default)]`, so a reference
    /// serialized before these existed loads and profiles unchanged.
    #[serde(default)]
    pub rhythm: bool,
    /// Emit the extended markdown and discourse dimensions: link density,
    /// footnote density, rhetorical-question rate.
    ///
    /// The apparatus contrast — a Luu post with almost no formatting against a
    /// Gwern one dense with links and sidenotes — lives in these three.
    #[serde(default)]
    pub md_extended: bool,
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
            rhythm: false,
            md_extended: false,
            min_sentences: default_min_sentences(),
        }
    }
}

impl SentenceStats {
    /// Turn the punch-rhythm and aside dimensions on.
    pub fn with_rhythm(mut self) -> Self {
        self.rhythm = true;
        self
    }

    /// Turn the extended markdown and discourse dimensions on.
    pub fn with_md_extended(mut self) -> Self {
        self.md_extended = true;
        self
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
    /// `[punch_short_rate, final_clause_len_ratio, aside_rate, aside_len_mean,
    /// aside_share]`, empty unless the spec asked for them.
    #[serde(default)]
    rhythm: Vec<Symbol>,
    /// `[md:link_rate, md:footnote_rate, sent:rhetorical_question_rate]`.
    #[serde(default)]
    md_extended: Vec<Symbol>,
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

        let rhythm = if self.rhythm {
            vec![
                push(interner, "sent:punch_short_rate", Unit::PerHundredSentences),
                push(interner, "sent:final_clause_len_ratio", Unit::Index),
                push(interner, "sent:aside_rate", Unit::PerHundredSentences),
                push(interner, "sent:aside_len_mean", Unit::Tokens),
                push(interner, "sent:aside_share", Unit::Fraction),
            ]
        } else {
            Vec::new()
        };
        let md_extended = if self.md_extended {
            vec![
                push(interner, "md:link_rate", Unit::PerThousandTokens),
                push(interner, "md:footnote_rate", Unit::PerThousandTokens),
                push(
                    interner,
                    "sent:rhetorical_question_rate",
                    Unit::PerHundredSentences,
                ),
            ]
        } else {
            Vec::new()
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
            rhythm,
            md_extended,
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
        self.rhythm_rates(analysis, sentences, tokens, out);
        self.md_extended_rates(analysis, sentences, tokens, out);
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

    /// Punch rhythm and parenthetical asides.
    fn rhythm_rates(
        &self,
        analysis: &Analysis<'_>,
        sentences: usize,
        _tokens: usize,
        out: &mut VectorBuilder,
    ) {
        if self.rhythm.is_empty() {
            return;
        }
        let structure = analysis.structure();
        let stream = analysis.tokens();
        let lengths: Vec<usize> = structure
            .sentences
            .iter()
            .map(|s| s.lexical_len(stream))
            .collect();

        // A short sentence immediately after a long one: the flat punch after
        // the windup. Measured against a *rolling* mean rather than the
        // document mean, so a document whose paragraphs differ in pace is not
        // scored against an average that describes none of them.
        let mut punches = 0usize;
        if lengths.len() >= 3 {
            for i in 1..lengths.len() {
                let window_start = i.saturating_sub(WINDOW);
                let window = &lengths[window_start..i];
                let mean: f64 = window.iter().sum::<usize>() as f64 / window.len() as f64;
                if mean <= 0.0 {
                    continue;
                }
                let previous = lengths[i - 1] as f64;
                let current = lengths[i] as f64;
                if previous >= mean * LONG_FACTOR && current <= mean * SHORT_FACTOR {
                    punches += 1;
                    out.note_span(self.rhythm[0], structure.sentences[i].span);
                }
            }
        }
        out.set(self.rhythm[0], ratio(punches, sentences) * 100.0);

        // Final-clause length as a share of sentence length: an author who
        // saves the turn for the end has a short one.
        let mut final_lengths: Vec<f64> = Vec::new();
        let mut sentence_lengths: Vec<f64> = Vec::new();
        for (sentence, &len) in structure.sentences.iter().zip(&lengths) {
            if len == 0 {
                continue;
            }
            let toks = &stream.tokens()[sentence.tokens.clone()];
            let boundary = toks
                .iter()
                .rposition(|t| CLAUSE_MARKS.iter().any(|c| stream.form(t) == c.to_string()));
            let Some(boundary) = boundary else { continue };
            let after = toks[boundary + 1..]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .count();
            if after == 0 {
                continue;
            }
            final_lengths.push(after as f64);
            sentence_lengths.push(len as f64);
        }
        if final_lengths.is_empty() {
            out.mark_missing(self.rhythm[1]);
        } else {
            out.set(
                self.rhythm[1],
                util::mean(&final_lengths) / util::mean(&sentence_lengths).max(1.0),
            );
        }

        // Asides: paired parentheses, paired em dashes, footnote markers.
        let asides = aside_spans(analysis);
        let aside_tokens: usize = asides
            .iter()
            .map(|span| {
                stream
                    .lexical()
                    .filter(|(t, _)| t.span.start >= span.start && t.span.end <= span.end)
                    .count()
            })
            .sum();
        for span in &asides {
            out.note_span(self.rhythm[2], *span);
        }
        out.set(self.rhythm[2], ratio(asides.len(), sentences) * 100.0);
        out.set(self.rhythm[3], ratio(aside_tokens, asides.len()));
        out.set(self.rhythm[4], ratio(aside_tokens, analysis.lexical_len()));
    }

    /// Link density, footnote density, rhetorical questions.
    fn md_extended_rates(
        &self,
        analysis: &Analysis<'_>,
        sentences: usize,
        tokens: usize,
        out: &mut VectorBuilder,
    ) {
        if self.md_extended.is_empty() {
            return;
        }
        let source = analysis.source();
        let mut links = 0usize;
        for span in link_spans(source) {
            links += 1;
            out.note_span(self.md_extended[0], span);
        }
        out.set(self.md_extended[0], per_1k(links, tokens));

        let mut footnotes = 0usize;
        for span in footnote_spans(source) {
            footnotes += 1;
            out.note_span(self.md_extended[1], span);
        }
        out.set(self.md_extended[1], per_1k(footnotes, tokens));

        let mut rhetorical = 0usize;
        for sentence in &analysis.structure().sentences {
            let text = analysis.text(sentence.span);
            if is_rhetorical_question(text) {
                rhetorical += 1;
                out.note_span(self.md_extended[2], sentence.span);
            }
        }
        out.set(self.md_extended[2], ratio(rhetorical, sentences) * 100.0);
    }
}

/// Sentences looked back over when deciding what counts as "long".
const WINDOW: usize = 4;
/// A sentence is long at this multiple of the rolling mean.
const LONG_FACTOR: f64 = 1.5;
/// A sentence is short at this multiple of the rolling mean.
const SHORT_FACTOR: f64 = 0.5;

/// Byte spans of parenthetical and dash-delimited asides.
///
/// Three shapes: a paired `(…)`, a pair of spaced em dashes ` — … — `, and a
/// footnote marker `[^n]`. Nested parentheses are not tracked — an aside inside
/// an aside is counted once, at the outer pair — because the dimension is a
/// rate and the nesting rate in prose is negligible.
fn aside_spans(analysis: &Analysis<'_>) -> Vec<Span> {
    let source = analysis.source();
    let mut out = Vec::new();

    let mut depth = 0usize;
    let mut open = 0usize;
    for (i, c) in source.char_indices() {
        match c {
            '(' => {
                if depth == 0 {
                    open = i;
                }
                depth += 1;
            }
            // An unmatched `)` closes nothing: a smiley or a list marker must
            // not open an aside that runs to the end of the document.
            ')' if depth == 1 => {
                depth = 0;
                out.push(Span::new(open, i + c.len_utf8()));
            }
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    // Paired em dashes *within one sentence*: an unpaired dash is an
    // interruption, not an aside, and pairing across a sentence boundary would
    // swallow whole paragraphs.
    for sentence in &analysis.structure().sentences {
        let text = analysis.text(sentence.span);
        let dashes: Vec<usize> = text
            .char_indices()
            .filter(|&(_, c)| c == '\u{2014}' || c == '\u{2013}')
            .map(|(i, _)| i)
            .collect();
        for pair in dashes.chunks_exact(2) {
            out.push(Span::new(
                sentence.span.start + pair[0],
                sentence.span.start + pair[1] + '\u{2014}'.len_utf8(),
            ));
        }
    }

    out.extend(footnote_spans(source));
    out.sort_by_key(|s| (s.start, s.end));
    out.dedup();
    out
}

/// Byte spans of markdown links and bare URLs.
fn link_spans(source: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut i = 0usize;
    // The cursor advances by whole characters. Stepping a byte at a time and
    // then slicing `source[i..]` panics on the first multi-byte character, and
    // prose from a real book is full of them: a curly quote, an em dash, a
    // bullet. Every offset pushed below still lands on a boundary, because
    // `find` returns one and `[`, `]`, `(`, `)` are one byte each.
    while let Some(ch) = source[i..].chars().next() {
        if ch == '[' {
            // `[text](target)`, with no nesting inside the label.
            if let Some(close) = source[i..].find("](") {
                let label_end = i + close;
                if let Some(paren) = source[label_end + 2..].find(')') {
                    let end = label_end + 2 + paren + 1;
                    out.push(Span::new(i, end));
                    i = end;
                    continue;
                }
            }
        }
        if source[i..].starts_with("http://") || source[i..].starts_with("https://") {
            let end = source[i..]
                .find(|c: char| c.is_whitespace() || c == ')' || c == '>')
                .map_or(source.len(), |n| i + n);
            // A bare URL already inside a markdown link was counted above; the
            // cursor skipped past it, so reaching here means it stands alone.
            out.push(Span::new(i, end));
            i = end;
            continue;
        }
        i += ch.len_utf8();
    }
    out
}

/// Byte spans of `[^n]`-style footnote markers.
fn footnote_spans(source: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut search = 0usize;
    while let Some(found) = source[search..].find("[^") {
        let start = search + found;
        match source[start..].find(']') {
            Some(close) => {
                let end = start + close + 1;
                out.push(Span::new(start, end));
                search = end;
            }
            None => break,
        }
    }
    out
}

/// Second-person addressee patterns that make a question a real one.
const ADDRESSEE_OPENERS: &[&str] = &[
    "do you",
    "did you",
    "can you",
    "could you",
    "would you",
    "will you",
    "have you",
    "are you",
    "is there any",
    "does anyone",
    "did anyone",
];

/// Whether a question is rhetorical rather than an actual request.
///
/// Documented heuristic: a question is treated as rhetorical unless it opens
/// with one of the second-person addressee patterns above. That over-claims on
/// genuine questions in a conversational register ("Why does this happen?" asked
/// in earnest reads as rhetorical here) and under-claims on the rhetorical
/// second-person ("Do you really think that helps?"). Both errors are the same
/// on the corpus side and the draft side.
fn is_rhetorical_question(sentence: &str) -> bool {
    let trimmed = sentence.trim_end();
    if !trimmed.ends_with('?') {
        return false;
    }
    let lower = trimmed.to_lowercase();
    let head: String = lower
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .chars()
        .take(16)
        .collect();
    !ADDRESSEE_OPENERS.iter().any(|p| head.starts_with(p))
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

    fn transform_with(spec: SentenceStats, text: &str) -> (FeatureVector, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn rhythm_value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform_with(SentenceStats::default().with_rhythm(), text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    fn md_value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform_with(SentenceStats::default().with_md_extended(), text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    #[test]
    fn the_new_dimensions_are_absent_unless_asked_for() {
        // The whole point of the spec flags: an existing reference gains no
        // dimensions and profiles exactly as before.
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        SentenceStats::default().fit(&ctx, &mut interner).unwrap();
        for name in [
            "sent:punch_short_rate",
            "sent:aside_rate",
            "md:link_rate",
            "sent:rhetorical_question_rate",
        ] {
            assert!(interner.get(name).is_none(), "{name} should be absent");
        }
        let spec: SentenceStats = serde_json::from_str("{}").unwrap();
        assert!(!spec.rhythm);
        assert!(!spec.md_extended);
        assert_eq!(spec, SentenceStats::default());
    }

    #[test]
    fn a_short_sentence_after_a_long_one_is_a_punch() {
        // Four ordinary sentences, then a long one, then a two-word punch.
        let text = "aa bb cc dd ee ff. gg hh ii jj kk ll. mm nn oo pp qq rr. \
                    ss tt uu vv ww xx yy zz ab cd ef gh ij kl mn op qq. yes.";
        let rate = rhythm_value(text, "sent:punch_short_rate");
        // One punch across five sentences.
        assert!((rate - 20.0).abs() < 1e-9, "{rate}");
        // Uniform sentences have none.
        let uniform = "aa bb cc dd. ee ff gg hh. ii jj kk ll. mm nn oo pp. qq rr ss tt.";
        assert_eq!(rhythm_value(uniform, "sent:punch_short_rate"), 0.0);
    }

    #[test]
    fn punch_spans_point_at_the_short_sentence() {
        let text = "aa bb cc dd ee ff. gg hh ii jj kk ll. mm nn oo pp qq rr. \
                    ss tt uu vv ww xx yy zz ab cd ef gh ij kl mn op qq. yes.";
        let (v, i) = transform_with(SentenceStats::default().with_rhythm(), text);
        let spans = v.spans(i.get("sent:punch_short_rate").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(&text[spans[0].range()], "yes.");
    }

    #[test]
    fn asides_are_counted_with_their_length_and_share() {
        // Two asides: one parenthetical of two tokens, one em-dashed of three.
        let text = "The build broke (again) this morning. \
                    We fixed it — or thought we had — before lunch. \
                    Nothing else happened. It was fine.";
        assert!((rhythm_value(text, "sent:aside_rate") - 50.0).abs() < 1e-9);
        // Five aside tokens across two asides.
        let mean = rhythm_value(text, "sent:aside_len_mean");
        assert!((mean - 2.5).abs() < 1e-9, "{mean}");
        assert!(rhythm_value(text, "sent:aside_share") > 0.0);
        // Prose with no asides scores zero, not missing.
        let plain = "The build broke this morning. We fixed it. Nothing else. Fine.";
        assert_eq!(rhythm_value(plain, "sent:aside_rate"), 0.0);
    }

    #[test]
    fn an_unpaired_dash_is_an_interruption_not_an_aside() {
        let text = "We fixed it — eventually. Nothing else happened. It was fine. Good.";
        assert_eq!(rhythm_value(text, "sent:aside_rate"), 0.0);
    }

    #[test]
    fn the_final_clause_ratio_falls_when_the_punch_is_short() {
        let short_tail = "the whole thing ran for hours and produced nothing at all, twice. \
                          the second attempt also ran for hours and produced nothing, again. \
                          the third one worked for a while and then stopped entirely, oddly. \
                          the fourth attempt behaved and finished in a couple of minutes, fine.";
        let long_tail = "the whole thing ran for hours, and produced nothing at all across \
                         every single one of the shards we looked at. the second attempt \
                         also ran, and produced nothing at all across every single shard we \
                         checked afterwards. the third one worked, and then stopped entirely \
                         after running for most of the afternoon without warning. the fourth \
                         behaved, and finished in a couple of minutes without any of the \
                         problems the earlier runs had shown.";
        assert!(
            rhythm_value(short_tail, "sent:final_clause_len_ratio")
                < rhythm_value(long_tail, "sent:final_clause_len_ratio")
        );
    }

    #[test]
    fn a_document_with_no_clause_marks_has_no_final_clause_ratio() {
        let (v, i) = transform_with(
            SentenceStats::default().with_rhythm(),
            "One thing. Then another. And a third. Done.",
        );
        assert!(v.is_missing(i.get("sent:final_clause_len_ratio").unwrap()));
    }

    #[test]
    fn link_and_footnote_densities_are_exact_per_thousand() {
        // Ten lexical tokens outside the markup, two links, one footnote.
        let text = "See [the docs](https://example.com/a) and https://example.com/b \
                    for more[^1] on this.";
        let (v, i) = transform_with(SentenceStats::default().with_md_extended(), text);
        let tokens = {
            let doc = Document::new(text);
            doc.analyze(&Tokenizer::default()).lexical_len()
        };
        let links = v.get(i.get("md:link_rate").unwrap());
        assert!(
            (links - 2.0 * 1000.0 / tokens as f64).abs() < 1e-9,
            "{links} over {tokens} tokens"
        );
        let footnotes = v.get(i.get("md:footnote_rate").unwrap());
        assert!(
            (footnotes - 1000.0 / tokens as f64).abs() < 1e-9,
            "{footnotes}"
        );
        assert_eq!(
            md_value("Plain prose with no apparatus at all here.", "md:link_rate"),
            0.0
        );
    }

    #[test]
    fn link_spans_cover_the_whole_markup() {
        let text = "See [the docs](https://example.com/a) now.";
        let (v, i) = transform_with(SentenceStats::default().with_md_extended(), text);
        let spans = v.spans(i.get("md:link_rate").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(&text[spans[0].range()], "[the docs](https://example.com/a)");
    }

    #[test]
    fn link_spans_survive_multi_byte_characters() {
        // The cursor used to step one byte at a time and slice at the new
        // offset, which panics as soon as it lands inside a curly quote. Every
        // book in the corpus opens with one, so this fired on real prose and
        // on nothing in the ASCII fixtures.
        let text = "“Don’t,” he said — a bullet • and an em dash — then \
                    [a link](https://example.com/x) and https://example.com/y.";
        let (v, i) = transform_with(SentenceStats::default().with_md_extended(), text);
        let spans = v.spans(i.get("md:link_rate").unwrap());
        assert_eq!(spans.len(), 2);
        assert_eq!(&text[spans[0].range()], "[a link](https://example.com/x)");
        assert_eq!(&text[spans[1].range()], "https://example.com/y.");
    }

    #[test]
    fn rhetorical_questions_are_separated_from_real_ones() {
        // Two rhetorical, one addressed, one statement.
        let text = "Why does this keep happening? What could possibly go wrong? \
                    Do you have the logs? It broke again.";
        assert!((md_value(text, "sent:rhetorical_question_rate") - 50.0).abs() < 1e-9);
        assert_eq!(
            md_value(
                "It broke. We fixed it. Nothing else. Fine.",
                "sent:rhetorical_question_rate"
            ),
            0.0
        );
    }

    #[test]
    fn the_rhythm_and_aside_dims_stay_in_the_sentence_family() {
        // They must never become canary candidates: the canary has to be a
        // family a word-swapping rewrite cannot move, and these are reported.
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = SentenceStats::default()
            .with_rhythm()
            .with_md_extended()
            .fit(&ctx, &mut interner)
            .unwrap();
        assert!(fitted.dims().iter().all(|d| d.family == Family::Sentence));
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
