//! Phase 9 — corpus extractors for handprint.
//!
//! Every extractor emits the same [`Record`]: text plus provenance. Records
//! become JSONL on disk and a [`Corpus`](handprint_core::Corpus) in memory.
//!
//! # What ships and what does not
//!
//! **Raw corpora never ship. Only fitted profiles with a provenance manifest
//! do.** The licensing landmines are all about redistributing text — LMSYS and
//! PAN forbid it outright, HC3 is share-alike, arena outputs are under provider
//! terms, HN and Reddit bulk text is grey — while fitted numeric profiles are
//! low-risk everywhere. This crate is built around that split: it helps you get
//! text onto your own disk and turn it into a profile, and it has no upload
//! path.
//!
//! For corpora derived from private data — local agent transcripts especially —
//! the profile is not automatically safe either. Run the document-frequency cull
//! in [`handprint_core::contrast`] and review the surviving vocabulary before
//! publishing anything.

#![forbid(unsafe_code)]

pub mod agent;
pub mod gutenberg;
pub mod hn;
pub mod jsonl;
pub mod record;
pub mod strip;

pub use agent::{AgentConfig, Extraction};
pub use gutenberg::{Book, Chapter, GutenbergConfig, PdBasis};
pub use record::{to_corpus, Record, Register};
pub use strip::{strip, StripConfig, Stripped};

/// Errors from extraction.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A file could not be read or written.
    #[error("{path}: {source}")]
    Io {
        /// The path involved.
        path: std::path::PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },

    /// A response or record was not valid JSON.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Data parsed but was not shaped as expected.
    #[error("unexpected format: {0}")]
    Format(String),

    /// A network request failed.
    #[error("network: {0}")]
    Network(String),
}

/// Convenience alias.
pub type Result<T, E = Error> = core::result::Result<T, E>;
