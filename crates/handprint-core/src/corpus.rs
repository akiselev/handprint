//! Corpora: documents grouped by author.

use serde::{Deserialize, Serialize};

use crate::text::Document;

/// An author identifier. Opaque to the crate — it can be a username, a model
/// name, or `"human"` vs `"ai"` for contrastive work.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthorId(pub String);

impl AuthorId {
    /// The identifier as a string.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AuthorId {
    fn from(s: &str) -> Self {
        AuthorId(s.to_owned())
    }
}

impl From<String> for AuthorId {
    fn from(s: String) -> Self {
        AuthorId(s)
    }
}

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One author's documents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorDocs {
    /// Who wrote them.
    pub author: AuthorId,
    /// What they wrote.
    pub docs: Vec<Document>,
}

/// A collection of documents grouped by author.
///
/// Marking a corpus `private` turns on privacy culling wherever a vocabulary
/// would otherwise be derived from it — see
/// [`ContrastModel`](crate::contrast::ContrastModel).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Corpus {
    authors: Vec<AuthorDocs>,
    #[serde(default)]
    private: bool,
}

impl Corpus {
    /// An empty corpus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the corpus as derived from private data.
    pub fn private(mut self, private: bool) -> Self {
        self.private = private;
        self
    }

    /// Whether the corpus is flagged private.
    #[inline]
    pub fn is_private(&self) -> bool {
        self.private
    }

    /// Add documents for an author, merging into an existing entry.
    pub fn add(&mut self, author: impl Into<AuthorId>, docs: impl IntoIterator<Item = Document>) {
        let author = author.into();
        match self.authors.iter_mut().find(|a| a.author == author) {
            Some(entry) => entry.docs.extend(docs),
            None => self.authors.push(AuthorDocs {
                author,
                docs: docs.into_iter().collect(),
            }),
        }
    }

    /// Builder-style [`Corpus::add`].
    pub fn with(
        mut self,
        author: impl Into<AuthorId>,
        docs: impl IntoIterator<Item = Document>,
    ) -> Self {
        self.add(author, docs);
        self
    }

    /// Per-author document groups.
    #[inline]
    pub fn authors(&self) -> &[AuthorDocs] {
        &self.authors
    }

    /// Number of distinct authors.
    #[inline]
    pub fn author_count(&self) -> usize {
        self.authors.len()
    }

    /// Every document, in author order.
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.authors.iter().flat_map(|a| a.docs.iter())
    }

    /// Every `(author, document)` pair.
    pub fn labelled(&self) -> impl Iterator<Item = (&AuthorId, &Document)> {
        self.authors
            .iter()
            .flat_map(|a| a.docs.iter().map(move |d| (&a.author, d)))
    }

    /// Total document count.
    pub fn len(&self) -> usize {
        self.authors.iter().map(|a| a.docs.len()).sum()
    }

    /// True when the corpus has no documents.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// One author's documents.
    pub fn author(&self, id: &AuthorId) -> Option<&[Document]> {
        self.authors
            .iter()
            .find(|a| &a.author == id)
            .map(|a| a.docs.as_slice())
    }

    /// Concatenate an author's documents into one pseudo-document.
    ///
    /// Individual HN comments are far below any usable length floor; the
    /// standard remedy is to profile the aggregate. Documents are joined with a
    /// blank line so sentence and block segmentation still see boundaries.
    pub fn aggregate(&self, id: &AuthorId) -> Option<Document> {
        let docs = self.author(id)?;
        Some(aggregate_docs(docs).with_id(id.as_str().to_owned()))
    }

    /// Aggregate every author into one pseudo-document each.
    pub fn aggregated(&self) -> Vec<(AuthorId, Document)> {
        self.authors
            .iter()
            .map(|a| {
                (
                    a.author.clone(),
                    aggregate_docs(&a.docs).with_id(a.author.as_str().to_owned()),
                )
            })
            .collect()
    }

    /// Split each author's documents into two halves, for the same-author leg
    /// of calibration and for General Imposters target sets.
    ///
    /// Authors with fewer than two documents are skipped: a single document
    /// cannot yield an honest within-author pair.
    pub fn split_halves(&self) -> Vec<(AuthorId, Document, Document)> {
        self.authors
            .iter()
            .filter(|a| a.docs.len() >= 2)
            .map(|a| {
                let mid = a.docs.len() / 2;
                (
                    a.author.clone(),
                    aggregate_docs(&a.docs[..mid]),
                    aggregate_docs(&a.docs[mid..]),
                )
            })
            .collect()
    }
}

impl FromIterator<(AuthorId, Vec<Document>)> for Corpus {
    fn from_iter<T: IntoIterator<Item = (AuthorId, Vec<Document>)>>(iter: T) -> Self {
        let mut c = Corpus::new();
        for (author, docs) in iter {
            c.add(author, docs);
        }
        c
    }
}

fn aggregate_docs(docs: &[Document]) -> Document {
    let mut text = String::new();
    for doc in docs {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(doc.text());
    }
    Document::new(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus() -> Corpus {
        Corpus::new()
            .with("alice", [Document::new("one"), Document::new("two")])
            .with("bob", [Document::new("three")])
    }

    #[test]
    fn grouping_and_counts() {
        let c = corpus();
        assert_eq!(c.author_count(), 2);
        assert_eq!(c.len(), 3);
        assert_eq!(c.author(&"alice".into()).unwrap().len(), 2);
    }

    #[test]
    fn aggregation_joins_with_blank_lines() {
        let c = corpus();
        let agg = c.aggregate(&"alice".into()).unwrap();
        assert_eq!(agg.text(), "one\n\ntwo");
        assert_eq!(agg.id(), Some("alice"));
    }

    #[test]
    fn split_halves_skips_single_document_authors() {
        let c = corpus();
        let halves = c.split_halves();
        assert_eq!(halves.len(), 1);
        assert_eq!(halves[0].0, "alice".into());
        assert_eq!(halves[0].1.text(), "one");
        assert_eq!(halves[0].2.text(), "two");
    }

    #[test]
    fn add_merges_repeated_authors() {
        let mut c = Corpus::new();
        c.add("alice", [Document::new("one")]);
        c.add("alice", [Document::new("two")]);
        assert_eq!(c.author_count(), 1);
        assert_eq!(c.len(), 2);
    }
}
