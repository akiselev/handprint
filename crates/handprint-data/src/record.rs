//! The one normalized record every extractor emits.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// What kind of writing a record is.
///
/// Register is not a nicety. Agentic-assistant prose, forum comments and
/// published articles differ more from each other than two authors within one
/// register do, so mixing them in a corpus produces a "style" profile that is
/// mostly measuring genre. Every record carries its register so a corpus can be
/// filtered to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Register {
    /// An assistant's prose inside a coding-agent transcript. This is the
    /// deployment register for the critic mode — agents rewriting their own
    /// output — which makes it a feature there and a caveat for any
    /// general-prose claim.
    AgentProse,
    /// An assistant's reasoning trace. A different register from its answers;
    /// keep it separate or drop it.
    Reasoning,
    /// A user's turn in a chat.
    ChatUser,
    /// A forum comment (Hacker News, Reddit).
    ForumComment,
    /// Long-form published prose.
    Article,
    /// Unclassified.
    Unknown,
}

impl Register {
    /// Stable name.
    pub fn as_str(self) -> &'static str {
        match self {
            Register::AgentProse => "agent_prose",
            Register::Reasoning => "reasoning",
            Register::ChatUser => "chat_user",
            Register::ForumComment => "forum_comment",
            Register::Article => "article",
            Register::Unknown => "unknown",
        }
    }
}

/// One unit of text with its provenance.
///
/// Extractors emit these as JSONL; corpora are built from them. Raw corpora are
/// never shipped — only fitted profiles with a provenance manifest — so this
/// type exists to keep the local pipeline honest, not to be redistributed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// The prose.
    pub text: String,
    /// Who wrote it, when there is a human author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Which model produced it, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Where it came from: a dataset name, a file path, an API.
    pub source: String,
    /// ISO-8601 timestamp, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
    /// What kind of writing it is.
    pub register: Register,
    /// License or terms the text is under. `None` means local-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Anything else worth keeping: session id, project, conversation turn.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, String>,
}

impl Record {
    /// A record with only the required fields.
    pub fn new(text: impl Into<String>, source: impl Into<String>, register: Register) -> Record {
        Record {
            text: text.into(),
            author: None,
            model: None,
            source: source.into(),
            ts: None,
            register,
            license: None,
            meta: BTreeMap::new(),
        }
    }

    /// Attach a model label.
    pub fn with_model(mut self, model: impl Into<String>) -> Record {
        self.model = Some(model.into());
        self
    }

    /// Attach an author label.
    pub fn with_author(mut self, author: impl Into<String>) -> Record {
        self.author = Some(author.into());
        self
    }

    /// Attach a timestamp.
    pub fn with_ts(mut self, ts: impl Into<String>) -> Record {
        self.ts = Some(ts.into());
        self
    }

    /// Attach a metadata key.
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Record {
        self.meta.insert(key.into(), value.into());
        self
    }

    /// Rough word count, for filtering short records.
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    /// The label to group by when building a corpus: the model for generated
    /// text, the author for human text.
    pub fn group(&self) -> &str {
        self.model
            .as_deref()
            .or(self.author.as_deref())
            .unwrap_or("unknown")
    }
}

/// Convert records into a [`Corpus`](handprint_core::Corpus), grouped by
/// [`Record::group`].
///
/// One document per record. For registers where individual records are short —
/// forum comments especially — profile the aggregate rather than the documents,
/// via [`Corpus::aggregate`](handprint_core::Corpus::aggregate).
pub fn to_corpus(records: impl IntoIterator<Item = Record>) -> handprint_core::Corpus {
    let mut corpus = handprint_core::Corpus::new();
    for record in records {
        let group = record.group().to_owned();
        corpus.add(group, [handprint_core::Document::new(record.text)]);
    }
    corpus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let record = Record::new("hello there", "test", Register::AgentProse)
            .with_model("claude-opus-4-8")
            .with_ts("2026-07-31T12:00:00Z")
            .with_meta("session", "abc");
        let json = serde_json::to_string(&record).unwrap();
        assert_eq!(serde_json::from_str::<Record>(&json).unwrap(), record);
        // Absent optional fields stay out of the wire format.
        assert!(!json.contains("author"));
    }

    #[test]
    fn grouping_prefers_the_model_label() {
        let record = Record::new("x", "s", Register::AgentProse)
            .with_model("gpt-5.5")
            .with_author("someone");
        assert_eq!(record.group(), "gpt-5.5");
        assert_eq!(
            Record::new("x", "s", Register::ForumComment)
                .with_author("pg")
                .group(),
            "pg"
        );
        assert_eq!(Record::new("x", "s", Register::Unknown).group(), "unknown");
    }

    #[test]
    fn corpus_groups_by_label() {
        let corpus = to_corpus([
            Record::new("a", "s", Register::AgentProse).with_model("m1"),
            Record::new("b", "s", Register::AgentProse).with_model("m1"),
            Record::new("c", "s", Register::AgentProse).with_model("m2"),
        ]);
        assert_eq!(corpus.author_count(), 2);
        assert_eq!(corpus.author(&"m1".into()).unwrap().len(), 2);
    }
}
