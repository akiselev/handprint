//! Extractors for local coding-agent transcripts.
//!
//! These are the freshest per-model corpus available for current models: no
//! public dataset covers them, and the register — an assistant writing prose
//! about code — is *exactly* the deployment register for the critic mode.
//!
//! # Privacy
//!
//! Transcripts are local by default and stay that way. Nothing here uploads
//! anything. A vocabulary derived from them must go through the
//! document-frequency cull in
//! [`handprint_core::contrast`] — which turns on
//! automatically for a corpus flagged
//! [`private`](handprint_core::Corpus::private) — and then a human review,
//! before any artifact leaves the machine. Project names, paths and colleagues'
//! names appear in one or two sessions each, so a document-frequency floor
//! removes them wholesale.
//!
//! # Robustness
//!
//! Rollout files are appended to live and are routinely truncated mid-line.
//! Every parser here skips unparseable lines and reports how many it skipped
//! rather than failing the file.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::record::{Record, Register};
use crate::strip::{strip, StripConfig};
use crate::{Error, Result};

/// What an extraction run produced.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extraction {
    /// The records.
    pub records: Vec<Record>,
    /// Files read.
    pub files: usize,
    /// Lines that would not parse as JSON. Truncated trailing lines are normal;
    /// a large count is not.
    pub bad_lines: usize,
    /// Messages dropped for being mostly code or too short.
    pub dropped: usize,
}

impl Extraction {
    /// Total prose words extracted.
    pub fn word_count(&self) -> usize {
        self.records.iter().map(Record::word_count).sum()
    }

    /// Merge another extraction into this one.
    pub fn merge(&mut self, other: Extraction) {
        self.records.extend(other.records);
        self.files += other.files;
        self.bad_lines += other.bad_lines;
        self.dropped += other.dropped;
    }

    /// A one-line summary for a CLI.
    pub fn summary(&self) -> String {
        format!(
            "{} record(s), {} words, from {} file(s); {} dropped, {} unparseable line(s)",
            self.records.len(),
            self.word_count(),
            self.files,
            self.dropped,
            self.bad_lines
        )
    }
}

/// Options shared by the transcript extractors.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AgentConfig {
    /// How code is removed.
    pub strip: StripConfig,
    /// Also extract reasoning traces, as [`Register::Reasoning`] records.
    ///
    /// Off by default: reasoning is a different register from answers, and
    /// mixing the two produces a profile of neither.
    pub include_reasoning: bool,
}

/// The default location of Claude Code transcripts.
pub fn claude_code_root() -> Option<PathBuf> {
    home().map(|h| h.join(".claude").join("projects"))
}

/// The default locations of Codex CLI rollouts.
pub fn codex_roots() -> Vec<PathBuf> {
    home()
        .map(|h| {
            vec![
                h.join(".codex").join("sessions"),
                h.join(".codex").join("archived_sessions"),
            ]
        })
        .unwrap_or_default()
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Extract assistant prose from Claude Code transcripts.
///
/// Reads `type == "assistant"` lines, takes the `message.content[]` entries
/// whose `type` is `"text"`, and labels each with the per-message
/// `message.model`. Per-message model labels matter: a single session can span
/// several models.
pub fn extract_claude_code(root: &Path, config: &AgentConfig) -> Result<Extraction> {
    let mut out = Extraction::default();
    for path in jsonl_files(root)? {
        out.merge(extract_claude_code_file(&path, config)?);
    }
    Ok(out)
}

/// Extract from a single Claude Code transcript file.
pub fn extract_claude_code_file(path: &Path, config: &AgentConfig) -> Result<Extraction> {
    let text = std::fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut out = parse_claude_code(&text, &path.display().to_string(), config);
    out.files = 1;
    Ok(out)
}

/// Parse Claude Code JSONL from a string. Exposed for testing against fixtures.
pub fn parse_claude_code(jsonl: &str, source: &str, config: &AgentConfig) -> Extraction {
    let mut out = Extraction::default();
    let session = session_id_of(source);
    for line in jsonl.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            out.bad_lines += 1;
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let message = value.get("message");
        let model = message
            .and_then(|m| m.get("model"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let ts = value.get("timestamp").and_then(Value::as_str);
        let Some(content) = message
            .and_then(|m| m.get("content"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for block in content {
            let kind = block.get("type").and_then(Value::as_str);
            let register = match kind {
                Some("text") => Register::AgentProse,
                Some("thinking") if config.include_reasoning => Register::Reasoning,
                _ => continue,
            };
            let field = if register == Register::Reasoning {
                "thinking"
            } else {
                "text"
            };
            let Some(raw) = block.get(field).and_then(Value::as_str) else {
                continue;
            };
            push_record(&mut out, raw, source, model, ts, register, &session, config);
        }
    }
    out
}

/// Extract assistant prose from Codex CLI rollouts.
///
/// Reads `type == "response_item"` lines whose `payload.type` is `"message"`
/// and `payload.role` is `"assistant"`, taking `payload.content[].text`. The
/// model comes from the most recent `turn_context` line, because the payload
/// itself does not carry one.
pub fn extract_codex(root: &Path, config: &AgentConfig) -> Result<Extraction> {
    let mut out = Extraction::default();
    for path in jsonl_files(root)? {
        out.merge(extract_codex_file(&path, config)?);
    }
    Ok(out)
}

/// Extract from a single Codex rollout file.
pub fn extract_codex_file(path: &Path, config: &AgentConfig) -> Result<Extraction> {
    let text = std::fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut out = parse_codex(&text, &path.display().to_string(), config);
    out.files = 1;
    Ok(out)
}

/// Parse Codex JSONL from a string. Exposed for testing against fixtures.
pub fn parse_codex(jsonl: &str, source: &str, config: &AgentConfig) -> Extraction {
    let mut out = Extraction::default();
    let session = session_id_of(source);
    // The model is carried by `turn_context` lines and applies to every message
    // until the next one.
    let mut model = "unknown".to_owned();

    for line in jsonl.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            out.bad_lines += 1;
            continue;
        };
        let ts = value.get("timestamp").and_then(Value::as_str);
        let payload = value.get("payload");

        if value.get("type").and_then(Value::as_str) == Some("turn_context") {
            if let Some(m) = payload.and_then(|p| p.get("model")).and_then(Value::as_str) {
                model = m.to_owned();
            }
            continue;
        }
        if value.get("type").and_then(Value::as_str) != Some("response_item") {
            continue;
        }
        let Some(payload) = payload else { continue };
        let payload_type = payload.get("type").and_then(Value::as_str);

        let register = match payload_type {
            Some("message") if payload.get("role").and_then(Value::as_str) == Some("assistant") => {
                Register::AgentProse
            }
            Some("reasoning") if config.include_reasoning => Register::Reasoning,
            _ => continue,
        };

        let blocks = payload
            .get("content")
            .or_else(|| payload.get("summary"))
            .and_then(Value::as_array);
        let Some(blocks) = blocks else { continue };
        for block in blocks {
            let Some(raw) = block
                .get("text")
                .and_then(Value::as_str)
                .or_else(|| block.as_str())
            else {
                continue;
            };
            push_record(
                &mut out, raw, source, &model, ts, register, &session, config,
            );
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn push_record(
    out: &mut Extraction,
    raw: &str,
    source: &str,
    model: &str,
    ts: Option<&str>,
    register: Register,
    session: &str,
    config: &AgentConfig,
) {
    let stripped = strip(raw, &config.strip);
    if !stripped.keep(&config.strip) {
        out.dropped += 1;
        return;
    }
    let code_share = format!("{:.3}", stripped.code_share());
    let mut record = Record::new(stripped.text, source, register)
        .with_model(model)
        .with_meta("session", session)
        .with_meta("code_share", code_share);
    if let Some(ts) = ts {
        record = record.with_ts(ts);
    }
    out.records.push(record);
}

/// The session identifier is the file stem, which is what the privacy cull
/// counts distinct occurrences across.
fn session_id_of(source: &str) -> String {
    Path::new(source)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.to_owned())
}

/// Every `.jsonl` file under a directory, recursively, in sorted order.
pub fn jsonl_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_jsonl(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_jsonl(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| Error::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| Error::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAUDE_FIXTURE: &str = r#"
{"type":"user","message":{"role":"user","content":"fix the tokenizer please"}}
{"type":"assistant","timestamp":"2026-07-30T10:00:00Z","message":{"model":"claude-opus-4-8","content":[{"type":"text","text":"I looked at the failing test and the problem is in the tokenizer: it treats the apostrophe as a word boundary, so contractions split in two.\n\n```rust\nlet s = normalize(input);\n```\n\nThat keeps the span mapping intact for every fixture we have."}]}}
{"type":"assistant","timestamp":"2026-07-30T10:01:00Z","message":{"model":"claude-fable-5","content":[{"type":"thinking","thinking":"The user wants me to consider whether the apostrophe handling is correct here in this particular case."},{"type":"text","text":"Done. I updated the snapshot as well, since the expected output changes for three of the fixtures and the old ones would fail."}]}}
{"type":"assistant","message":{"model":"claude-opus-4-8","content":[{"type":"text","text":"ok"}]}}
{"type":"assistant","message":{"model":"claude-opus-4-8","content":[{"type":"tool_use","name":"Edit","input":{}}]}}
{"type":"assistant","message":{"model":"claude-opus-4-8","content":[{"type":"text","text":"truncated
"#;

    const CODEX_FIXTURE: &str = r#"
{"type":"turn_context","payload":{"model":"gpt-5.6-sol"}}
{"type":"response_item","timestamp":"2026-07-30T11:00:00Z","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"what changed"}]}}
{"type":"response_item","timestamp":"2026-07-30T11:00:05Z","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"I rewrote the calibration so both distributions are estimated per length bin, which is what the short-text case needed all along."}]}}
{"type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"I should check whether the bins are wide enough for the corpus we have here."}]}}
{"type":"turn_context","payload":{"model":"gpt-5.3-codex"}}
{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"The stratification now interpolates between bins in log space, so a query that falls between two bins gets a blended answer rather than the nearest one."}]}}
{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"typ
"#;

    #[test]
    fn claude_code_extracts_assistant_prose_with_per_message_models() {
        let e = parse_claude_code(CLAUDE_FIXTURE, "/x/sess-abc.jsonl", &AgentConfig::default());
        assert_eq!(e.records.len(), 2, "{:?}", e.records);
        assert_eq!(e.records[0].model.as_deref(), Some("claude-opus-4-8"));
        assert_eq!(e.records[1].model.as_deref(), Some("claude-fable-5"));
        assert_eq!(e.records[0].register, Register::AgentProse);
        assert_eq!(e.records[0].ts.as_deref(), Some("2026-07-30T10:00:00Z"));
        assert_eq!(
            e.records[0].meta.get("session").map(String::as_str),
            Some("sess-abc")
        );
        // Code is gone; prose is not.
        assert!(!e.records[0].text.contains("normalize(input)"));
        assert!(e.records[0].text.contains("apostrophe"));
    }

    #[test]
    fn claude_code_skips_short_messages_tool_calls_and_truncated_lines() {
        let e = parse_claude_code(CLAUDE_FIXTURE, "/x/s.jsonl", &AgentConfig::default());
        // "ok" is too short; the tool_use block has no text.
        assert!(e.dropped >= 1);
        // The final line is truncated mid-JSON.
        assert_eq!(e.bad_lines, 1);
        assert!(e.records.iter().all(|r| !r.text.contains("tool_use")));
    }

    #[test]
    fn reasoning_is_opt_in_and_labelled_separately() {
        let default = parse_claude_code(CLAUDE_FIXTURE, "/x/s.jsonl", &AgentConfig::default());
        assert!(default
            .records
            .iter()
            .all(|r| r.register == Register::AgentProse));

        let with_reasoning = parse_claude_code(
            CLAUDE_FIXTURE,
            "/x/s.jsonl",
            &AgentConfig {
                include_reasoning: true,
                strip: StripConfig {
                    min_words: 5,
                    ..Default::default()
                },
            },
        );
        assert!(with_reasoning
            .records
            .iter()
            .any(|r| r.register == Register::Reasoning));
    }

    #[test]
    fn codex_tracks_the_model_across_turn_contexts() {
        let e = parse_codex(CODEX_FIXTURE, "/y/rollout-1.jsonl", &AgentConfig::default());
        assert_eq!(e.records.len(), 2, "{:?}", e.records);
        assert_eq!(e.records[0].model.as_deref(), Some("gpt-5.6-sol"));
        // The second message follows a new turn_context.
        assert_eq!(e.records[1].model.as_deref(), Some("gpt-5.3-codex"));
        assert!(e.records[0].text.contains("calibration"));
    }

    #[test]
    fn codex_ignores_user_turns_and_survives_truncation() {
        let e = parse_codex(CODEX_FIXTURE, "/y/r.jsonl", &AgentConfig::default());
        assert!(e.records.iter().all(|r| !r.text.contains("what changed")));
        assert_eq!(e.bad_lines, 1);
    }

    #[test]
    fn codex_reasoning_is_opt_in() {
        let e = parse_codex(
            CODEX_FIXTURE,
            "/y/r.jsonl",
            &AgentConfig {
                include_reasoning: true,
                strip: StripConfig {
                    min_words: 5,
                    ..Default::default()
                },
            },
        );
        assert!(e.records.iter().any(|r| r.register == Register::Reasoning));
    }

    #[test]
    fn an_empty_transcript_is_not_an_error() {
        let e = parse_codex("", "/y/r.jsonl", &AgentConfig::default());
        assert!(e.records.is_empty());
        assert_eq!(e.bad_lines, 0);
    }

    #[test]
    fn extraction_summarizes_itself() {
        let e = parse_claude_code(CLAUDE_FIXTURE, "/x/s.jsonl", &AgentConfig::default());
        let summary = e.summary();
        assert!(summary.contains("record(s)"));
        assert!(e.word_count() > 20);
    }

    #[test]
    fn files_are_discovered_recursively_and_sorted() {
        let dir = std::env::temp_dir().join(format!("handprint-agent-{}", std::process::id()));
        let nested = dir.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(dir.join("z.jsonl"), CLAUDE_FIXTURE).unwrap();
        std::fs::write(nested.join("a.jsonl"), CLAUDE_FIXTURE).unwrap();
        std::fs::write(dir.join("ignore.txt"), "not jsonl").unwrap();

        let files = jsonl_files(&dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "jsonl"));

        let e = extract_claude_code(&dir, &AgentConfig::default()).unwrap();
        assert_eq!(e.files, 2);
        assert_eq!(e.records.len(), 4);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_directory_is_an_error_not_a_panic() {
        assert!(
            extract_claude_code(Path::new("/nonexistent/handprint"), &AgentConfig::default())
                .is_err()
        );
    }
}
