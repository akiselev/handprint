//! Phase 1 — the text layer.
//!
//! [`Document`] owns text. [`Analysis`] is everything derived from it once:
//! the confusable-folded scoring view, the token stream, sentence and block
//! segmentation, and the artifact scan. Every feature family reads an
//! `Analysis`, so a document is tokenized exactly once per profile no matter
//! how many features are configured.

pub mod artifact;
#[cfg(feature = "verse")]
pub mod dict;
pub mod normalize;
pub mod segment;
pub mod syllable;
pub mod tokenize;

use std::ops::Range;

use serde::{Deserialize, Serialize};

pub use artifact::{Artifact, ArtifactKind, ArtifactReport};
pub use normalize::{ScoringText, Script};
pub use segment::{Block, BlockKind, Line, MarkdownStats, Sentence, Structure};
pub use syllable::{count_syllables, SyllableMethod};
pub use tokenize::{ApostrophePolicy, CaseFold, Token, TokenKind, TokenStream, Tokenizer};

/// A half-open byte range in a document's **source** text.
///
/// Spans are the crate's universal coordinate system: everything an agent gets
/// back — findings, contributions, artifact reports — is anchored with spans
/// that index the exact bytes the caller supplied, so a patch can be applied
/// without re-deriving offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Inclusive start offset, in bytes.
    pub start: usize,
    /// Exclusive end offset, in bytes.
    pub end: usize,
}

impl Span {
    /// Build a span, clamping an inverted range to empty.
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Span {
            start,
            end: end.max(start),
        }
    }

    /// The span as a [`Range`], for slicing.
    #[inline]
    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }

    /// Length in bytes.
    #[inline]
    pub fn len(self) -> usize {
        self.end - self.start
    }

    /// True when the span covers no bytes.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Slice a string with this span.
    ///
    /// Returns `None` when the span is out of bounds or lands mid-character,
    /// which can only happen if a span from one document is applied to another.
    #[inline]
    pub fn slice(self, text: &str) -> Option<&str> {
        text.get(self.range())
    }
}

impl From<Range<usize>> for Span {
    fn from(r: Range<usize>) -> Self {
        Span::new(r.start, r.end)
    }
}

/// A unit of text to be profiled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

impl Document {
    /// Wrap a string as a document.
    pub fn new(text: impl Into<String>) -> Self {
        Document {
            text: text.into(),
            id: None,
        }
    }

    /// Attach an identifier, carried through into provenance fields.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// The source text, byte-for-byte as supplied.
    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The document's identifier, if it has one.
    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Derive everything the feature families need, tokenizing once.
    pub fn analyze<'a>(&'a self, tokenizer: &Tokenizer) -> Analysis<'a> {
        Analysis::new(&self.text, tokenizer)
    }
}

impl From<&str> for Document {
    fn from(s: &str) -> Self {
        Document::new(s)
    }
}

impl From<String> for Document {
    fn from(s: String) -> Self {
        Document::new(s)
    }
}

/// Everything derived from a document in one pass.
#[derive(Debug, Clone)]
pub struct Analysis<'a> {
    source: &'a str,
    scoring: ScoringText,
    tokens: TokenStream,
    structure: Structure,
    artifacts: ArtifactReport,
}

impl<'a> Analysis<'a> {
    /// Analyze raw text under a tokenization policy.
    pub fn new(source: &'a str, tokenizer: &Tokenizer) -> Self {
        let scoring = ScoringText::new(source);
        let tokens = tokenizer.tokenize(&scoring);
        let structure = segment::segment(source, &tokens);
        let artifacts = artifact::scan(source);
        Analysis {
            source,
            scoring,
            tokens,
            structure,
            artifacts,
        }
    }

    /// The source text. All spans index this.
    #[inline]
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// The confusable-folded view that scoring runs over.
    #[inline]
    pub fn scoring(&self) -> &ScoringText {
        &self.scoring
    }

    /// The word-level token stream.
    #[inline]
    pub fn tokens(&self) -> &TokenStream {
        &self.tokens
    }

    /// Sentences, blocks and markdown counts.
    #[inline]
    pub fn structure(&self) -> &Structure {
        &self.structure
    }

    /// Tokenizer attacks and generation residue found in the source.
    #[inline]
    pub fn artifacts(&self) -> &ArtifactReport {
        &self.artifacts
    }

    /// Number of word and number tokens — the length used for every rate and
    /// for the length-stratified calibration bins.
    #[inline]
    pub fn lexical_len(&self) -> usize {
        self.tokens.lexical_len()
    }

    /// Slice the source with a span.
    #[inline]
    pub fn text(&self, span: Span) -> &'a str {
        span.slice(self.source).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_is_consistent() {
        let doc = Document::new("First line — with typography. Second one!\n\n- a bullet\n");
        let a = doc.analyze(&Tokenizer::default());
        assert_eq!(a.structure().sentences.len(), 3);
        assert_eq!(a.structure().markdown.bullets, 1);
        assert!(a.artifacts().is_empty());
        assert!(a.lexical_len() > 5);
        for (token, _) in a.tokens().iter() {
            assert!(token.span.slice(a.source()).is_some());
        }
    }

    #[test]
    fn document_round_trips_through_serde() {
        let doc = Document::new("hi").with_id("d1");
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(serde_json::from_str::<Document>(&json).unwrap(), doc);
    }
}
