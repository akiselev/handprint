//! Data-pack formats beyond the term/phrase [`LexiconPack`].
//!
//! Three pack formats exist, all data-only JSON with the same provenance
//! envelope, because three genuinely different shapes of external data feed the
//! feature families:
//!
//! 1. [`LexiconPack`] — terms and phrase patterns. Membership is the datum.
//! 2. [`NormPack`] — a *weighted* lexicon: formality scores, concreteness
//!    ratings, arousal, VADER booster weights. The number attached to the term
//!    is the datum.
//! 3. [`CountPack`] — a background frequency table. The [`Feature`] trait sees
//!    only the reference corpus at fit time (by design: nothing corpus-relative
//!    may enter `transform`), so a background distribution has to arrive as a
//!    versioned *data input on the spec*, produced offline, never magically at
//!    fit time.
//!
//! Grammatical closed-class lists that do not decay — Biber's verb classes,
//! subordinators, interjections — are consts in code with a `*_VERSION` string
//! instead, following the [`FUNCTION_WORDS`](crate::feature::mfw::FUNCTION_WORDS)
//! precedent. Those lists are properties of the language, not dated
//! observations about a model.
//!
//! # Licensing
//!
//! Several of the best weighted lexicons are research-only or non-commercial
//! (NRC EmoLex, probably Warriner VAD, probably Brysbaert concreteness). Those
//! are **loader-only**: handprint ships the loader and the feature, never the
//! table. A pack whose [`NormPack::redistributable`] is false is embedded in
//! any reference fitted from it, which makes that reference non-redistributable
//! too — see [`Provenance::licenses`](crate::reference::Provenance::licenses).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub use crate::feature::lexicon::PackSource;

/// One weighted lexicon entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormEntry {
    /// The term, in tokenizer-normalized (lowercase) form.
    pub term: String,
    /// The score attached to it. Scale is pack-defined and documented in
    /// [`NormPack::description`].
    pub value: f64,
}

/// A dated, sourced table of per-term scores.
///
/// Mirrors [`LexiconPack`](crate::feature::LexiconPack)'s provenance envelope
/// exactly, so the two load, pin and attribute the same way.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormPack {
    /// Short name, used as the dimension namespace.
    pub name: String,
    /// Version string; bump when contents change.
    pub version: String,
    /// ISO date the contents were compiled.
    pub date: String,
    /// One-line description, including the value scale.
    #[serde(default)]
    pub description: String,
    /// License of the pack contents.
    #[serde(default)]
    pub license: String,
    /// Whether this pack may be shipped inside a published artifact.
    ///
    /// `false` for research-only and non-commercial tables. It is not a legal
    /// opinion, it is a flag the CLI warns on: a reference fitted with a
    /// non-redistributable pack embeds that pack's table and inherits the
    /// obligation.
    #[serde(default = "crate::util::yes")]
    pub redistributable: bool,
    /// Where the contents came from.
    #[serde(default)]
    pub sources: Vec<PackSource>,
    /// The scored terms.
    #[serde(default)]
    pub entries: Vec<NormEntry>,
}

impl NormPack {
    /// `name@version`, as it appears in provenance blocks.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Look a term's score up.
    pub fn get(&self, term: &str) -> Option<f64> {
        self.entries
            .iter()
            .find(|e| e.term == term)
            .map(|e| e.value)
    }

    /// The entries as a lookup table, lowercased.
    pub fn table(&self) -> Vec<(String, f64)> {
        let mut out: Vec<(String, f64)> = self
            .entries
            .iter()
            .map(|e| (e.term.to_lowercase(), e.value))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.dedup_by(|a, b| a.0 == b.0);
        out
    }

    /// Quantile of the pack's own value distribution.
    ///
    /// Used to place the "top quartile" / "bottom quartile" cut points that the
    /// register-clash detector needs, relative to the pack rather than to a
    /// magic constant.
    pub fn quantile(&self, q: f64) -> Option<f64> {
        let mut values: Vec<f64> = self.entries.iter().map(|e| e.value).collect();
        crate::util::sort_floats(&mut values);
        crate::util::quantile_sorted(&values, q)
    }

    /// Reject an empty or malformed pack at load time rather than at transform.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() || self.version.is_empty() {
            return Err(Error::InvalidConfig {
                what: "NormPack",
                detail: "name and version are required".into(),
            });
        }
        if self.entries.is_empty() {
            return Err(Error::InvalidConfig {
                what: "NormPack::entries",
                detail: format!("{} carries no entries", self.qualified_name()),
            });
        }
        if self.entries.iter().any(|e| !e.value.is_finite()) {
            return Err(Error::InvalidConfig {
                what: "NormPack::entries",
                detail: format!("{} has a non-finite value", self.qualified_name()),
            });
        }
        Ok(())
    }
}

/// A background frequency table.
///
/// Novelty and rarity dimensions ("is this adjective–noun pair unusual?", "how
/// often does this draft reach below the common-word tail?") are only meaningful
/// against a *background* distribution, and the [`Feature`](crate::Feature)
/// trait deliberately never sees one. A `CountPack` is how a background enters:
/// built offline, versioned, pinned on the feature spec, and embedded in the
/// fitted state so `transform` stays pure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountPack {
    /// Short name, used in provenance.
    pub name: String,
    /// Version string.
    pub version: String,
    /// ISO date the counts were taken.
    pub date: String,
    /// One-line description, including what corpus was counted.
    #[serde(default)]
    pub description: String,
    /// License of the counts.
    #[serde(default)]
    pub license: String,
    /// Whether this pack may be shipped inside a published artifact.
    #[serde(default = "crate::util::yes")]
    pub redistributable: bool,
    /// Where the counts came from.
    #[serde(default)]
    pub sources: Vec<PackSource>,
    /// `(term, count)` pairs.
    #[serde(default)]
    pub entries: Vec<(String, u64)>,
    /// Total draws the counts were taken from, so a rate can be derived.
    #[serde(default)]
    pub total: u64,
}

impl CountPack {
    /// `name@version`.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// The count for a term, zero when unseen.
    pub fn count(&self, term: &str) -> u64 {
        self.entries
            .iter()
            .find(|(t, _)| t == term)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    }

    /// Relative frequency of a term against [`CountPack::total`].
    pub fn frequency(&self, term: &str) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.count(term) as f64 / self.total as f64
    }

    /// The entries as a sorted lookup table.
    pub fn table(&self) -> Vec<(String, u64)> {
        let mut out: Vec<(String, u64)> = self
            .entries
            .iter()
            .map(|(t, c)| (t.to_lowercase(), *c))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.dedup_by(|a, b| a.0 == b.0);
        out
    }

    /// The `total` field if set, else the sum of the entries.
    pub fn effective_total(&self) -> u64 {
        if self.total > 0 {
            self.total
        } else {
            self.entries.iter().map(|(_, c)| *c).sum()
        }
    }

    /// Reject an empty pack at load time.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() || self.version.is_empty() {
            return Err(Error::InvalidConfig {
                what: "CountPack",
                detail: "name and version are required".into(),
            });
        }
        if self.entries.is_empty() {
            return Err(Error::InvalidConfig {
                what: "CountPack::entries",
                detail: format!("{} carries no entries", self.qualified_name()),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norms() -> NormPack {
        NormPack {
            name: "formality".into(),
            version: "1".into(),
            date: "2026-08-01".into(),
            description: "test".into(),
            license: "CC0-1.0".into(),
            redistributable: true,
            sources: vec![],
            entries: vec![
                NormEntry {
                    term: "notwithstanding".into(),
                    value: 2.0,
                },
                NormEntry {
                    term: "gonna".into(),
                    value: -2.0,
                },
                NormEntry {
                    term: "table".into(),
                    value: 0.0,
                },
            ],
        }
    }

    #[test]
    fn norm_pack_round_trips_through_json() {
        let pack = norms();
        let json = serde_json::to_string(&pack).unwrap();
        assert_eq!(serde_json::from_str::<NormPack>(&json).unwrap(), pack);
        assert_eq!(pack.qualified_name(), "formality@1");
    }

    #[test]
    fn norm_lookups_and_quantiles() {
        let pack = norms();
        assert_eq!(pack.get("gonna"), Some(-2.0));
        assert_eq!(pack.get("absent"), None);
        assert_eq!(pack.quantile(0.5), Some(0.0));
        let table = pack.table();
        assert_eq!(table.len(), 3);
        // Sorted, so a binary search works at transform time.
        assert!(table.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn empty_packs_are_rejected() {
        let mut pack = norms();
        pack.entries.clear();
        assert!(pack.validate().is_err());
        assert!(norms().validate().is_ok());
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let mut pack = norms();
        pack.entries[0].value = f64::NAN;
        assert!(pack.validate().is_err());
    }

    #[test]
    fn count_pack_round_trips_and_computes_frequencies() {
        let pack = CountPack {
            name: "bg".into(),
            version: "1".into(),
            date: "2026-08-01".into(),
            description: String::new(),
            license: "CC0-1.0".into(),
            redistributable: true,
            sources: vec![],
            entries: vec![("the".into(), 900), ("atavistic".into(), 1)],
            total: 1_000,
        };
        let json = serde_json::to_string(&pack).unwrap();
        assert_eq!(serde_json::from_str::<CountPack>(&json).unwrap(), pack);
        assert_eq!(pack.count("the"), 900);
        assert_eq!(pack.count("nope"), 0);
        assert!((pack.frequency("atavistic") - 0.001).abs() < 1e-12);
        assert_eq!(pack.effective_total(), 1_000);
    }

    #[test]
    fn count_pack_total_falls_back_to_the_entry_sum() {
        let pack = CountPack {
            name: "bg".into(),
            version: "1".into(),
            date: "2026-08-01".into(),
            description: String::new(),
            license: String::new(),
            redistributable: true,
            sources: vec![],
            entries: vec![("a".into(), 3), ("b".into(), 7)],
            total: 0,
        };
        assert_eq!(pack.effective_total(), 10);
    }
}
