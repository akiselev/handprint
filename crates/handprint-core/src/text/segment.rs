//! Sentence, paragraph and markdown-block segmentation.
//!
//! The sentence splitter is deliberately cheap and rule-based. Its known
//! limits, in order of how often they bite:
//!
//! 1. Abbreviations outside [`ABBREVIATIONS`] end a sentence early
//!    (`Approx. 5 items` splits).
//! 2. Ellipses used mid-sentence (`well… anyway`) end a sentence.
//! 3. Sentences without terminal punctuation — common in chat registers —
//!    merge with the next one unless a line break separates them.
//!
//! All three inflate sentence-length *variance* rather than biasing the mean,
//! and the same splitter runs over the reference corpus and the query, so the
//! error is shared. Where it matters — the burstiness features — the
//! [`SentenceStats`](crate::feature::SentenceStats) docs say so.

use serde::{Deserialize, Serialize};
use std::ops::Range;

use crate::text::tokenize::TokenStream;
use crate::text::Span;

/// A sentence: its extent in the source text and its token index range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// Byte range in the source text, trimmed of leading/trailing whitespace.
    pub span: Span,
    /// Half-open range into [`TokenStream::tokens`].
    pub tokens: Range<usize>,
}

impl Sentence {
    /// Number of word and number tokens in the sentence.
    pub fn lexical_len(&self, stream: &TokenStream) -> usize {
        stream.tokens()[self.tokens.clone()]
            .iter()
            .filter(|t| t.kind.is_lexical())
            .count()
    }
}

/// What kind of markdown block a paragraph is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BlockKind {
    /// Ordinary prose.
    Paragraph,
    /// An ATX heading; the payload is the level.
    Heading(u8),
    /// A bulleted or numbered list item.
    ListItem,
    /// A fenced code block.
    CodeFence,
    /// A block quote.
    Quote,
    /// A pipe table row.
    Table,
    /// A horizontal rule.
    Rule,
}

/// A block of text delimited by blank lines or markdown structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Byte range in the source text.
    pub span: Span,
    /// What kind of block this is.
    pub kind: BlockKind,
}

/// Counts of markdown structure. LLM post-training leaves a documented
/// affinity for bullets, bold and headers, so the *rates* of these are style
/// features in their own right.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownStats {
    /// Non-empty lines.
    pub lines: usize,
    /// ATX heading lines (`# ...`).
    pub headings: usize,
    /// Bulleted list items (`- `, `* `, `+ `).
    pub bullets: usize,
    /// Numbered list items (`1. `).
    pub numbered: usize,
    /// Block-quote lines.
    pub quotes: usize,
    /// Fenced code blocks (pairs of ```` ``` ````).
    pub code_fences: usize,
    /// Inline code spans.
    pub inline_code: usize,
    /// `**bold**` spans.
    pub bold: usize,
    /// `_italic_` / `*italic*` spans.
    pub italic: usize,
    /// Pipe-table rows.
    pub table_rows: usize,
}

/// Everything the segmenter derives from a document.
#[derive(Debug, Clone, Default)]
pub struct Structure {
    /// Blocks in document order.
    pub blocks: Vec<Block>,
    /// Sentences in document order.
    pub sentences: Vec<Sentence>,
    /// Markdown structure counts.
    pub markdown: MarkdownStats,
}

/// Words that end in `.` without ending a sentence.
pub const ABBREVIATIONS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "prof", "sr", "jr", "st", "mt", "vs", "etc", "eg", "ie", "al", "cf",
    "fig", "no", "vol", "pp", "ed", "eds", "approx", "est", "inc", "ltd", "co", "corp", "dept",
    "univ", "assn", "misc", "min", "max", "avg", "ca", "circa", "e", "i", "u", "a", "p", "n",
];

/// Segment a document.
pub fn segment(source: &str, stream: &TokenStream) -> Structure {
    let (blocks, markdown) = segment_blocks(source);
    let boundaries = sentence_boundaries(source, &blocks);
    let sentences = attach_tokens(source, &boundaries, stream);
    Structure {
        blocks,
        sentences,
        markdown,
    }
}

fn segment_blocks(source: &str) -> (Vec<Block>, MarkdownStats) {
    let mut blocks = Vec::new();
    let mut md = MarkdownStats::default();
    let mut in_fence = false;

    let mut current: Option<(usize, usize, BlockKind)> = None;
    let flush = |current: &mut Option<(usize, usize, BlockKind)>, blocks: &mut Vec<Block>| {
        if let Some((start, end, kind)) = current.take() {
            blocks.push(Block {
                span: Span::new(start, end),
                kind,
            });
        }
    };

    for (start, line) in line_offsets(source) {
        let end = start + line.len();
        let trimmed = line.trim();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if !in_fence {
                md.code_fences += 1;
            }
            in_fence = !in_fence;
            flush(&mut current, &mut blocks);
            blocks.push(Block {
                span: Span::new(start, end),
                kind: BlockKind::CodeFence,
            });
            continue;
        }
        if in_fence {
            // Code inside a fence is not prose; it becomes its own block and
            // contributes no sentences.
            match &mut current {
                Some((_, e, BlockKind::CodeFence)) => *e = end,
                _ => {
                    flush(&mut current, &mut blocks);
                    current = Some((start, end, BlockKind::CodeFence));
                }
            }
            continue;
        }
        if trimmed.is_empty() {
            flush(&mut current, &mut blocks);
            continue;
        }

        md.lines += 1;
        let kind = classify_line(trimmed, &mut md);

        match (&mut current, kind) {
            // Prose lines accrete into one block; structural lines stand alone.
            (Some((_, e, BlockKind::Paragraph)), BlockKind::Paragraph) => *e = end,
            (Some((_, e, BlockKind::Quote)), BlockKind::Quote) => *e = end,
            (Some((_, e, BlockKind::Table)), BlockKind::Table) => *e = end,
            _ => {
                flush(&mut current, &mut blocks);
                current = Some((start, end, kind));
            }
        }
    }
    flush(&mut current, &mut blocks);
    count_inline_markup(source, &mut md);
    (blocks, md)
}

fn classify_line(trimmed: &str, md: &mut MarkdownStats) -> BlockKind {
    let bytes = trimmed.as_bytes();
    if trimmed.starts_with('#') {
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level <= 6 && trimmed[level..].starts_with(' ') {
            md.headings += 1;
            return BlockKind::Heading(level as u8);
        }
    }
    if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-' || c == '*' || c == '_') {
        return BlockKind::Rule;
    }
    if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) && matches!(bytes.get(1), Some(b' ')) {
        md.bullets += 1;
        return BlockKind::ListItem;
    }
    if let Some(dot) = trimmed.find(['.', ')']) {
        if dot > 0
            && dot <= 3
            && trimmed[..dot].bytes().all(|b| b.is_ascii_digit())
            && matches!(bytes.get(dot + 1), Some(b' '))
        {
            md.numbered += 1;
            return BlockKind::ListItem;
        }
    }
    if trimmed.starts_with('>') {
        md.quotes += 1;
        return BlockKind::Quote;
    }
    if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2 {
        md.table_rows += 1;
        return BlockKind::Table;
    }
    BlockKind::Paragraph
}

fn count_inline_markup(source: &str, md: &mut MarkdownStats) {
    md.bold = count_delimited(source, "**");
    md.inline_code = count_delimited(source, "`");
    // Single-asterisk italics, excluding the ones already consumed by `**`.
    let single = count_delimited(source, "*");
    md.italic = single.saturating_sub(md.bold * 2);
}

/// Count *pairs* of a delimiter, which is what a markup span costs.
fn count_delimited(source: &str, delim: &str) -> usize {
    source.matches(delim).count() / 2
}

fn line_offsets(source: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    source.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line.trim_end_matches('\n').trim_end_matches('\r'))
    })
}

/// Terminal punctuation that can end a sentence.
fn is_terminator(c: char) -> bool {
    matches!(
        c,
        '.' | '!' | '?' | '\u{2026}' | '\u{FF01}' | '\u{FF1F}' | '\u{3002}'
    )
}

/// Compute sentence spans, restricted to blocks that hold prose.
fn sentence_boundaries(source: &str, blocks: &[Block]) -> Vec<Span> {
    let mut spans = Vec::new();
    for block in blocks {
        if matches!(block.kind, BlockKind::CodeFence | BlockKind::Rule) {
            continue;
        }
        split_block(source, block.span, &mut spans);
    }
    spans
}

fn split_block(source: &str, block: Span, out: &mut Vec<Span>) {
    let text = &source[block.range()];
    let mut start = 0usize;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (offset, c) = chars[i];
        if !is_terminator(c) {
            i += 1;
            continue;
        }
        // Consume a run of terminators (`...`, `?!`).
        let run_start = i;
        while i < chars.len() && is_terminator(chars[i].1) {
            i += 1;
        }
        let mut run_end = chars.get(i).map(|&(o, _)| o).unwrap_or(text.len());

        // Allow closing quotes and brackets to belong to the sentence.
        while i < chars.len()
            && matches!(
                chars[i].1,
                '"' | '\'' | ')' | ']' | '\u{201D}' | '\u{2019}' | '\u{00BB}'
            )
        {
            i += 1;
            run_end = chars.get(i).map(|&(o, _)| o).unwrap_or(text.len());
        }

        let single_period = run_end - offset == 1 && c == '.';
        let followed_by_break = i >= chars.len() || chars[i].1.is_whitespace();
        if !followed_by_break {
            continue;
        }
        if single_period && is_abbreviation_before(text, run_start, &chars) {
            continue;
        }
        // `1.` at the start of a line is a list marker, not a sentence end.
        if single_period && is_list_marker(text, offset) {
            continue;
        }

        push_trimmed(text, block.start, start, run_end, out);
        start = run_end;
    }
    push_trimmed(text, block.start, start, text.len(), out);
}

fn push_trimmed(text: &str, base: usize, start: usize, end: usize, out: &mut Vec<Span>) {
    let slice = &text[start..end];
    let lead = slice.len() - slice.trim_start().len();
    let trail = slice.len() - slice.trim_end().len();
    if lead + trail >= slice.len() {
        return;
    }
    out.push(Span::new(base + start + lead, base + end - trail));
}

fn is_abbreviation_before(text: &str, term_idx: usize, chars: &[(usize, char)]) -> bool {
    let mut j = term_idx;
    let mut word = String::new();
    while j > 0 {
        let (_, c) = chars[j - 1];
        if c.is_alphanumeric() {
            word.insert(0, c.to_ascii_lowercase());
            j -= 1;
        } else {
            break;
        }
    }
    if word.is_empty() {
        return false;
    }
    // A trailing digit run before a period is usually a version or list number.
    if word.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let _ = text;
    ABBREVIATIONS.contains(&word.as_str())
}

fn is_list_marker(text: &str, period_offset: usize) -> bool {
    let head = &text[..period_offset];
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let prefix = head[line_start..].trim_start();
    !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit())
}

fn attach_tokens(source: &str, spans: &[Span], stream: &TokenStream) -> Vec<Sentence> {
    let _ = source;
    let mut sentences: Vec<Sentence> = spans
        .iter()
        .map(|&span| Sentence { span, tokens: 0..0 })
        .collect();
    if sentences.is_empty() {
        return sentences;
    }
    let mut si = 0usize;
    for (ti, token) in stream.tokens().iter().enumerate() {
        while si < sentences.len() && token.span.start >= sentences[si].span.end {
            si += 1;
        }
        if si >= sentences.len() {
            break;
        }
        if token.span.start < sentences[si].span.start {
            continue;
        }
        let range = &mut sentences[si].tokens;
        if range.start == range.end {
            *range = ti..ti + 1;
        } else {
            range.end = ti + 1;
        }
    }
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::normalize::ScoringText;
    use crate::text::tokenize::Tokenizer;

    fn segment_str(text: &str) -> (Structure, TokenStream) {
        let scoring = ScoringText::new(text);
        let stream = Tokenizer::default().tokenize(&scoring);
        (segment(text, &stream), stream)
    }

    #[test]
    fn splits_simple_sentences() {
        let text = "One thing. Then another! And a third?";
        let (s, _) = segment_str(text);
        let got: Vec<&str> = s.sentences.iter().map(|x| &text[x.span.range()]).collect();
        assert_eq!(got, vec!["One thing.", "Then another!", "And a third?"]);
    }

    #[test]
    fn respects_abbreviations() {
        let text = "Ask Dr. Smith, i.e. the one from Fig. 2, about it. Then go.";
        let (s, _) = segment_str(text);
        let got: Vec<&str> = s.sentences.iter().map(|x| &text[x.span.range()]).collect();
        assert_eq!(
            got,
            vec![
                "Ask Dr. Smith, i.e. the one from Fig. 2, about it.",
                "Then go."
            ]
        );
    }

    #[test]
    fn token_ranges_cover_sentences() {
        let text = "Alpha beta. Gamma delta epsilon.";
        let (s, stream) = segment_str(text);
        assert_eq!(s.sentences.len(), 2);
        assert_eq!(s.sentences[0].lexical_len(&stream), 2);
        assert_eq!(s.sentences[1].lexical_len(&stream), 3);
    }

    #[test]
    fn counts_markdown_structure() {
        let text = "# Title\n\nSome **bold** prose.\n\n- one\n- two\n\n1. first\n\n> quoted\n\n```\ncode();\n```\n";
        let (s, _) = segment_str(text);
        assert_eq!(s.markdown.headings, 1);
        assert_eq!(s.markdown.bullets, 2);
        assert_eq!(s.markdown.numbered, 1);
        assert_eq!(s.markdown.quotes, 1);
        assert_eq!(s.markdown.bold, 1);
        assert_eq!(s.markdown.code_fences, 1);
        // Code inside the fence must not become a sentence.
        let joined: String = s
            .sentences
            .iter()
            .map(|x| &text[x.span.range()])
            .collect::<Vec<_>>()
            .join("|");
        assert!(!joined.contains("code()"), "{joined}");
    }

    #[test]
    fn numbered_list_markers_do_not_split() {
        let text = "1. First item runs on\n2. Second item";
        let (s, _) = segment_str(text);
        let got: Vec<&str> = s.sentences.iter().map(|x| &text[x.span.range()]).collect();
        assert_eq!(got, vec!["1. First item runs on", "2. Second item"]);
    }

    #[test]
    fn handles_quotes_after_terminator() {
        let text = "He said \"go home.\" She left.";
        let (s, _) = segment_str(text);
        assert_eq!(s.sentences.len(), 2);
        assert_eq!(&text[s.sentences[0].span.range()], "He said \"go home.\"");
    }
}
