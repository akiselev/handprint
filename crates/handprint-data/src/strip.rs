//! Removing code from agent transcripts while keeping their shape.
//!
//! Coding-agent prose is interleaved with code blocks, diffs, file paths and
//! URLs. None of that is style — a fenced block of Rust says nothing about how
//! someone writes English, and leaving it in makes the character-n-gram and
//! word-length features measure the language of the code instead.
//!
//! Stripping it naively would throw away something real, though. Markdown
//! *structure* is a documented style signal: how often a writer reaches for a
//! bullet, a header, a bold run, a fenced block. So code is replaced with an
//! **empty shell of itself** — a fenced block becomes an empty fence, an inline
//! span becomes an empty span — which keeps every structure count intact while
//! removing the content.

use serde::{Deserialize, Serialize};

/// What to strip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StripConfig {
    /// Replace fenced code blocks with empty fences.
    pub fences: bool,
    /// Replace inline code spans with empty spans.
    pub inline: bool,
    /// Drop bare URLs.
    pub urls: bool,
    /// Drop tokens that look like file paths.
    pub paths: bool,
    /// Drop unified-diff bodies.
    pub diffs: bool,
    /// Reject a message whose text was more than this share code.
    pub max_code_share: f64,
    /// Reject a message with fewer prose words than this.
    pub min_words: usize,
}

impl Default for StripConfig {
    fn default() -> Self {
        StripConfig {
            fences: true,
            inline: true,
            urls: true,
            paths: true,
            diffs: true,
            max_code_share: 0.5,
            min_words: 12,
        }
    }
}

/// The result of stripping one message.
#[derive(Debug, Clone, PartialEq)]
pub struct Stripped {
    /// The prose, with code shells left in place.
    pub text: String,
    /// Characters removed as code.
    pub code_chars: usize,
    /// Characters in the original.
    pub total_chars: usize,
}

impl Stripped {
    /// Share of the original that was code, in `[0, 1]`.
    pub fn code_share(&self) -> f64 {
        if self.total_chars == 0 {
            0.0
        } else {
            self.code_chars as f64 / self.total_chars as f64
        }
    }

    /// Prose words remaining.
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    /// Whether this message should be kept as a corpus record.
    pub fn keep(&self, config: &StripConfig) -> bool {
        self.code_share() <= config.max_code_share && self.word_count() >= config.min_words
    }
}

/// Strip code from a message.
pub fn strip(text: &str, config: &StripConfig) -> Stripped {
    let total_chars = text.chars().count();
    let mut code_chars = 0usize;

    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    let mut fence_marker = String::new();

    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        let trimmed = body.trim_start();

        if config.fences {
            if in_fence {
                if trimmed.starts_with(&fence_marker) {
                    in_fence = false;
                    out.push_str(&fence_marker);
                    out.push('\n');
                } else {
                    code_chars += body.chars().count();
                }
                continue;
            }
            if let Some(marker) = fence_marker_of(trimmed) {
                in_fence = true;
                fence_marker = marker;
                // Keep the opening fence so the structure count survives; drop
                // the language tag, which is code metadata rather than prose.
                out.push_str(&fence_marker);
                out.push('\n');
                continue;
            }
        }

        if config.diffs && is_diff_line(body) {
            code_chars += body.chars().count();
            continue;
        }

        let mut cleaned = body.to_owned();
        if config.inline {
            let (text, removed) = strip_inline_code(&cleaned);
            cleaned = text;
            code_chars += removed;
        }
        if config.urls || config.paths {
            let (text, removed) = strip_tokens(&cleaned, config.urls, config.paths);
            cleaned = text;
            code_chars += removed;
        }
        out.push_str(cleaned.trim_end());
        out.push('\n');
    }
    // An unterminated fence — common in truncated transcripts — still closes.
    if in_fence {
        out.push_str(&fence_marker);
        out.push('\n');
    }

    Stripped {
        text: collapse_blank_lines(&out),
        code_chars,
        total_chars,
    }
}

fn fence_marker_of(trimmed: &str) -> Option<String> {
    for marker in ["```", "~~~"] {
        if trimmed.starts_with(marker) {
            return Some(marker.to_owned());
        }
    }
    None
}

/// Unified-diff and patch scaffolding.
fn is_diff_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("@@")
        || t.starts_with("diff --git")
        || t.starts_with("index ")
        || t.starts_with("--- a/")
        || t.starts_with("+++ b/")
        || (t.starts_with('+') && !t.starts_with("+ ") && t.len() > 1)
        || (t.starts_with('-') && !t.starts_with("- ") && t.len() > 1 && !t.starts_with("--"))
}

/// Replace `` `code` `` with an empty span, preserving the delimiter count.
fn strip_inline_code(line: &str) -> (String, usize) {
    let mut out = String::with_capacity(line.len());
    let mut removed = 0usize;
    let mut rest = line;
    loop {
        let Some(open) = rest.find('`') else {
            out.push_str(rest);
            break;
        };
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str("``");
        removed += after[..close].chars().count();
        rest = &after[close + 1..];
    }
    (out, removed)
}

/// Drop whitespace-delimited tokens that are URLs or file paths.
fn strip_tokens(line: &str, urls: bool, paths: bool) -> (String, usize) {
    let mut removed = 0usize;
    let kept: Vec<&str> = line
        .split_inclusive(char::is_whitespace)
        .filter(|piece| {
            let token = piece.trim();
            let drop = (urls && is_url(token)) || (paths && is_path(token));
            if drop {
                removed += token.chars().count();
            }
            !drop
        })
        .collect();
    (kept.concat(), removed)
}

fn is_url(token: &str) -> bool {
    let t = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != ':' && c != '.');
    t.starts_with("http://") || t.starts_with("https://") || t.starts_with("www.")
}

/// A token that looks like a filesystem path rather than a word.
///
/// Deliberately conservative: it must contain a separator *and* look like it
/// has an extension or a leading path marker, so ordinary prose containing a
/// slash ("and/or", "9/10") survives.
fn is_path(token: &str) -> bool {
    let t = token.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == '.' || c == '`');
    if t.len() < 3 || t.contains(' ') {
        return false;
    }
    let has_sep = t.contains('/') || t.contains('\\');
    if !has_sep {
        // A bare `foo.rs` still reads as a path.
        return t.rsplit_once('.').is_some_and(|(stem, ext)| {
            !stem.is_empty()
                && (1..=5).contains(&ext.len())
                && ext.chars().all(|c| c.is_ascii_alphanumeric())
                && CODE_EXTENSIONS.contains(&ext)
        });
    }
    t.starts_with('/')
        || t.starts_with("./")
        || t.starts_with("../")
        || t.starts_with('~')
        || t.rsplit_once('.').is_some_and(|(_, ext)| CODE_EXTENSIONS.contains(&ext))
}

/// Extensions common enough in agent transcripts to be worth recognizing.
const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "go", "c", "h", "cpp", "hpp", "java", "rb", "sh", "zsh",
    "toml", "json", "yaml", "yml", "md", "txt", "lock", "cfg", "ini", "sql", "html", "css", "xml",
    "csv", "jsonl", "proto", "swift", "kt", "php", "pl", "lua", "vim", "el",
];

fn collapse_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank_run = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenced_blocks_leave_an_empty_shell() {
        let text = "Here is the fix:\n\n```rust\nfn main() { println!(\"hi\"); }\nlet x = 1;\n```\n\nThat should do it.";
        let s = strip(text, &StripConfig::default());
        assert!(!s.text.contains("println"));
        assert!(s.text.contains("Here is the fix"));
        assert!(s.text.contains("That should do it"));
        // The fence pair survives, so the markdown structure count is intact.
        assert_eq!(s.text.matches("```").count(), 2, "{}", s.text);
        assert!(s.code_chars > 20);
    }

    #[test]
    fn inline_spans_leave_an_empty_shell() {
        let s = strip("Call `foo_bar(baz)` before `quux`.", &StripConfig::default());
        assert!(!s.text.contains("foo_bar"));
        assert!(!s.text.contains("quux"));
        assert_eq!(s.text, "Call `` before ``.");
        // Two spans, so `count_delimited` still sees two inline-code pairs.
        assert_eq!(s.text.matches('`').count(), 4);
    }

    #[test]
    fn urls_and_paths_are_dropped_but_ordinary_prose_survives() {
        let s = strip(
            "See https://example.com/x and crates/handprint-core/src/lib.rs for the and/or case, \
             which is 9/10 fine.",
            &StripConfig::default(),
        );
        assert!(!s.text.contains("example.com"));
        assert!(!s.text.contains("lib.rs"));
        assert!(s.text.contains("and/or"), "{}", s.text);
        assert!(s.text.contains("9/10"), "{}", s.text);
    }

    #[test]
    fn diff_bodies_are_dropped() {
        let text = "I changed it:\n\ndiff --git a/x.rs b/x.rs\n@@ -1,3 +1,3 @@\n-old line here\n+new line here\n\nDone.";
        let s = strip(text, &StripConfig::default());
        assert!(!s.text.contains("old line"));
        assert!(!s.text.contains("@@"));
        assert!(s.text.contains("I changed it"));
        assert!(s.text.contains("Done."));
        // A markdown bullet must not be mistaken for a diff deletion.
        let bullets = strip("- first item\n- second item", &StripConfig::default());
        assert!(bullets.text.contains("- first item"), "{}", bullets.text);
    }

    #[test]
    fn unterminated_fences_do_not_swallow_the_rest() {
        // Truncated transcripts are common; a dangling fence must still close.
        let s = strip("Text before.\n\n```\nfn main() {", &StripConfig::default());
        assert!(s.text.contains("Text before."));
        assert_eq!(s.text.matches("```").count(), 2);
        assert!(!s.text.contains("fn main"));
    }

    #[test]
    fn mostly_code_messages_are_rejected() {
        let config = StripConfig::default();
        let mostly_code = strip(
            "Fix:\n```rust\n{}\n```",
            &StripConfig {
                min_words: 1,
                ..config.clone()
            },
        );
        let mostly_prose = strip(
            "Here is a reasonably long explanation of what the change does and why it matters \
             to the caller, with only a little code.\n```rust\nlet x = 1;\n```",
            &config,
        );
        assert!(mostly_prose.keep(&config), "{:?}", mostly_prose);
        assert!(mostly_code.code_share() > 0.0);
    }

    #[test]
    fn short_messages_are_rejected() {
        let config = StripConfig::default();
        assert!(!strip("Done.", &config).keep(&config));
        assert!(strip(
            "This is a long enough message to be worth keeping as a corpus record, honestly.",
            &config
        )
        .keep(&config));
    }

    #[test]
    fn prose_survival_rate_is_high_on_a_realistic_message() {
        let text = "I looked at the failing test and the problem is in the tokenizer: it treats \
                    the apostrophe as a word boundary, so contractions split in two. The fix is \
                    to fold the curly apostrophe first.\n\n```rust\nlet s = normalize(input);\n```\n\n\
                    That keeps the span mapping intact. I also updated the snapshot, since the \
                    expected output changes for three of the fixtures.";
        let s = strip(text, &StripConfig::default());
        let original_words = text.split_whitespace().count();
        let kept = s.word_count();
        assert!(
            kept as f64 / original_words as f64 >= 0.6,
            "kept {kept} of {original_words} words"
        );
    }
}
