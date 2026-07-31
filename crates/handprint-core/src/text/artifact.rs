//! Artifact scanning: tokenizer attacks and generation residue.
//!
//! None of what this module finds is *style*. Homoglyph substitution is an
//! attack on the feature extractor, invisible characters are an attack on the
//! tokenizer, and chatbot residue is a copy-paste accident. They are reported
//! as findings so a human sees them, and the scoring text folds them away so
//! they cannot change a score.

use serde::{Deserialize, Serialize};

use crate::text::normalize::{self, Script};
use crate::text::Span;

/// What kind of artifact was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ArtifactKind {
    /// A word mixes scripts (e.g. Latin `p` with Cyrillic `а`). The classic
    /// homoglyph substitution attack.
    MixedScript,
    /// A character was folded to its Latin skeleton before scoring.
    Confusable,
    /// An invisible formatting character was removed before scoring.
    ZeroWidth,
    /// A zero-width joiner outside an emoji sequence.
    StrayJoiner,
    /// Residue from a chatbot UI (citation markers, tool-call scaffolding).
    ChatbotResidue,
}

impl ArtifactKind {
    /// Stable identifier used in [`CritiqueReport`](crate::critique::CritiqueReport)
    /// findings.
    pub fn id(self) -> &'static str {
        match self {
            ArtifactKind::MixedScript => "artifact.mixed_script",
            ArtifactKind::Confusable => "artifact.confusable",
            ArtifactKind::ZeroWidth => "artifact.zero_width",
            ArtifactKind::StrayJoiner => "artifact.stray_joiner",
            ArtifactKind::ChatbotResidue => "artifact.chatbot_residue",
        }
    }
}

/// One artifact occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    /// What was found.
    pub kind: ArtifactKind,
    /// Where, in source-text byte offsets.
    pub span: Span,
    /// Short human-readable detail (the offending substring or a pattern name).
    pub detail: String,
}

/// The result of scanning a document for artifacts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReport {
    /// Every occurrence found, in document order.
    pub artifacts: Vec<Artifact>,
}

impl ArtifactReport {
    /// True when the document is clean.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    /// The distinct kinds present, in a stable order.
    pub fn kinds(&self) -> Vec<ArtifactKind> {
        let mut kinds: Vec<ArtifactKind> = Vec::new();
        for a in &self.artifacts {
            if !kinds.contains(&a.kind) {
                kinds.push(a.kind);
            }
        }
        kinds
    }

    /// Occurrences of one kind.
    pub fn of_kind(&self, kind: ArtifactKind) -> impl Iterator<Item = &Artifact> {
        self.artifacts.iter().filter(move |a| a.kind == kind)
    }
}

/// Substrings that only appear in text pasted out of a chatbot UI.
const RESIDUE_PATTERNS: &[&str] = &[
    "oaicite",
    "contentReference",
    "turn0search",
    "turn0news",
    "navlist",
    "citeturn",
    "\u{E200}", // private-use markers used by some web UIs
    "\u{E201}",
    "\u{3010}", // 【 ... †source】 citation brackets
];

/// Scan a document for tokenizer attacks and generation residue.
pub fn scan(source: &str) -> ArtifactReport {
    let mut artifacts = Vec::new();
    scan_characters(source, &mut artifacts);
    scan_words(source, &mut artifacts);
    scan_residue(source, &mut artifacts);
    artifacts.sort_by_key(|a| (a.span.start, a.span.end));
    ArtifactReport { artifacts }
}

fn scan_characters(source: &str, out: &mut Vec<Artifact>) {
    // ASCII cannot be invisible, a joiner, or a confusable, and prose is
    // overwhelmingly ASCII - so these fast paths decide the throughput of every
    // profile. Only the zero-width joiner needs neighbour context, and it is
    // resolved with one pending slot rather than a materialized char vector.
    if source.is_ascii() {
        return;
    }
    let mut prev: Option<char> = None;
    let mut pending_joiner: Option<(usize, Option<char>)> = None;

    for (offset, c) in source.char_indices() {
        if let Some((joiner_offset, before)) = pending_joiner.take() {
            if !joins_emoji(before, Some(c)) {
                out.push(stray_joiner(joiner_offset));
            }
        }
        if c.is_ascii() {
            prev = Some(c);
            continue;
        }
        if normalize::is_invisible(c) {
            out.push(Artifact {
                kind: ArtifactKind::ZeroWidth,
                span: Span::new(offset, offset + c.len_utf8()),
                detail: format!("U+{:04X}", c as u32),
            });
        } else if normalize::is_zero_width_joiner(c) {
            pending_joiner = Some((offset, prev));
        } else if let Some(folded) = normalize::fold_confusable(c) {
            out.push(Artifact {
                kind: ArtifactKind::Confusable,
                span: Span::new(offset, offset + c.len_utf8()),
                detail: format!("U+{:04X} '{c}' folds to '{folded}'", c as u32),
            });
        }
        prev = Some(c);
    }
    // A joiner at the very end of the text joins nothing.
    if let Some((joiner_offset, _)) = pending_joiner {
        out.push(stray_joiner(joiner_offset));
    }
}

fn joins_emoji(before: Option<char>, after: Option<char>) -> bool {
    before.is_some_and(crate::text::tokenize::is_emoji)
        && after.is_some_and(crate::text::tokenize::is_emoji)
}

fn stray_joiner(offset: usize) -> Artifact {
    Artifact {
        kind: ArtifactKind::StrayJoiner,
        span: Span::new(offset, offset + '\u{200D}'.len_utf8()),
        detail: "U+200D outside an emoji sequence".into(),
    }
}

/// A word whose letters come from two different scripts is a substitution
/// attack essentially every time; the false positives (transliteration,
/// deliberate mixed-script names) are rare and worth surfacing anyway.
fn scan_words(source: &str, out: &mut Vec<Artifact>) {
    use unicode_segmentation::UnicodeSegmentation;
    // No all-ASCII text can mix scripts, so an ASCII document skips a whole
    // second pass of word segmentation.
    if source.is_ascii() {
        return;
    }
    for (offset, word) in source.split_word_bound_indices() {
        // An all-ASCII word is entirely Latin and Common, so it can never mix
        // scripts. Skipping it early keeps this scan off the critical path for
        // ordinary prose.
        if word.is_ascii() || !word.chars().any(char::is_alphabetic) {
            continue;
        }
        let mut scripts: Vec<Script> = Vec::new();
        for c in word.chars() {
            let s = normalize::script_of(c);
            if s != Script::Common && !scripts.contains(&s) {
                scripts.push(s);
            }
        }
        if scripts.len() > 1 {
            out.push(Artifact {
                kind: ArtifactKind::MixedScript,
                span: Span::new(offset, offset + word.len()),
                detail: format!("{word:?} mixes {scripts:?}"),
            });
        }
    }
}

fn scan_residue(source: &str, out: &mut Vec<Artifact>) {
    for pattern in RESIDUE_PATTERNS {
        let mut from = 0;
        while let Some(rel) = source[from..].find(pattern) {
            let start = from + rel;
            out.push(Artifact {
                kind: ArtifactKind::ChatbotResidue,
                span: Span::new(start, start + pattern.len()),
                detail: (*pattern).to_owned(),
            });
            from = start + pattern.len();
        }
    }
    // `[cite: 12]`, `[cite_start]`, `[citation:3]` - bracketed citation stubs.
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = source[i..].find('[') {
        let start = i + rel;
        let head = &bytes[start..bytes.len().min(start + 5)];
        if head.len() == 5 && head[1..].eq_ignore_ascii_case(b"cite") {
            if let Some(close) = source[start..].find(']') {
                if close < 24 {
                    out.push(Artifact {
                        kind: ArtifactKind::ChatbotResidue,
                        span: Span::new(start, start + close + 1),
                        detail: source[start..start + close + 1].to_owned(),
                    });
                }
            }
        }
        i = start + 1;
        if i >= bytes.len() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_has_no_artifacts() {
        assert!(scan("Perfectly ordinary prose — with an em dash.").is_empty());
    }

    #[test]
    fn detects_homoglyph_word() {
        let text = "log in to p\u{0430}ypal now";
        let report = scan(text);
        assert!(report.kinds().contains(&ArtifactKind::MixedScript));
        assert!(report.kinds().contains(&ArtifactKind::Confusable));
        let mixed = report.of_kind(ArtifactKind::MixedScript).next().unwrap();
        assert_eq!(&text[mixed.span.range()], "p\u{0430}ypal");
    }

    #[test]
    fn detects_zero_width_characters() {
        let report = scan("de\u{200B}lve");
        assert_eq!(report.artifacts.len(), 1);
        assert_eq!(report.artifacts[0].kind, ArtifactKind::ZeroWidth);
    }

    #[test]
    fn emoji_zwj_sequences_are_not_flagged() {
        assert!(scan("family: \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}").is_empty());
    }

    #[test]
    fn detects_chatbot_residue() {
        let report = scan("as shown :contentReference[oaicite:0]{index=0} and [cite: 12] here");
        let kinds = report.kinds();
        assert_eq!(kinds, vec![ArtifactKind::ChatbotResidue]);
        assert!(report.artifacts.len() >= 3);
    }
}
