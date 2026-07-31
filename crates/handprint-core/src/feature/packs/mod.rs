//! Bundled data packs.
//!
//! Every pack here is a [`LexiconPack`] or a [`NormPack`](super::NormPack)
//! constructed from consts in code, which is what "bundled" means: the pack
//! ships inside the binary and can be exported to JSON with
//! `handprint pack export --pack <name>` as the starting point for a
//! project-specific fork. Packs loaded from disk are the other half of the
//! story and take precedence — a pack is *data*, and dated data belongs in a
//! file the project pins.
//!
//! # What is bundled and what is not
//!
//! The licensing rule, verbatim from the plan:
//!
//! * **Re-curate, never vendor** — GPL word lists (biberpy) and book
//!   compilations (Hyland's appendix, Leech's ad-adjective lists). The
//!   individual items are unprotectable words; the compilation is what
//!   copyright attaches to, so the pack is built from the *published feature
//!   definitions* rather than copied from anyone's file.
//! * **Loader-only, never bundled** — resources with no redistribution right
//!   (NRC EmoLex) or a non-commercial clause (probably Warriner VAD, probably
//!   Brysbaert concreteness). handprint ships the feature and the loader; the
//!   user supplies the table.
//! * **Share-alike in its own file** — a CC-BY-SA derivation lives in a
//!   separate pack so the obligation cannot infect a sibling pack that merely
//!   happens to be in the same binary.

use super::lexicon::{LexiconPack, PackSource, Phrase, Severity, Term};

mod cliche_similes;
mod doc_style;
mod hyland;
mod marketing;
mod misc;
mod norms;
mod tech_voice;
mod wordnet;

pub use norms::{concreteness_stub, heylighen_formality, vader_boosters};
pub use wordnet::{
    emotion_adjectives, senses_per_word, wordnet_antonyms, wordnet_source, WORDNET_VERSION,
};

/// Every bundled pack's name, in a stable order.
pub const BUILTIN_PACKS: &[&str] = &[
    "ai-slop",
    "hyland",
    "doc-style",
    "tech-voice",
    "cliche-similes",
    "hyperbole",
    "marketing-eval",
    "marketing-slop",
    "wiki-ai-signs",
    "epistemic-certainty",
    "drug-lexicon",
];

/// Look a bundled lexicon pack up by name.
pub fn builtin(name: &str) -> Option<LexiconPack> {
    Some(match name {
        "ai-slop" => LexiconPack::ai_slop(),
        "hyland" => hyland::pack(),
        "doc-style" => doc_style::pack(),
        "tech-voice" => tech_voice::pack(),
        "cliche-similes" => cliche_similes::pack(),
        "hyperbole" => misc::hyperbole_pack(),
        "marketing-eval" => marketing::eval_pack(),
        "marketing-slop" => marketing::slop_pack(),
        "wiki-ai-signs" => misc::wiki_ai_pack(),
        "epistemic-certainty" => misc::epistemic_pack(),
        "drug-lexicon" => misc::drug_pack(),
        _ => return None,
    })
}

/// Shared helper: build a pack from `(id, word, category, severity, alternatives)`.
type WordRow = (
    &'static str,
    &'static str,
    Severity,
    &'static [&'static str],
);

/// Shared helper: build a pack from `(id, pattern, category, severity, message)`.
type PhraseRow = (
    &'static str,
    &'static str,
    &'static str,
    Severity,
    &'static str,
);

fn terms(rows: &[WordRow]) -> Vec<Term> {
    rows.iter()
        .map(|(word, category, severity, alternatives)| Term {
            id: format!("word.{}", word.replace(' ', "_")),
            word: (*word).to_owned(),
            category: (*category).to_owned(),
            severity: *severity,
            alternatives: alternatives.iter().map(|s| (*s).to_owned()).collect(),
        })
        .collect()
}

fn phrases(rows: &[PhraseRow]) -> Vec<Phrase> {
    rows.iter()
        .map(|(id, pattern, category, severity, message)| Phrase {
            id: (*id).to_owned(),
            pattern: (*pattern).to_owned(),
            category: (*category).to_owned(),
            severity: *severity,
            message: (*message).to_owned(),
        })
        .collect()
}

fn source(name: &str, url: &str, note: &str) -> PackSource {
    PackSource {
        name: name.to_owned(),
        url: url.to_owned(),
        note: note.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_pack_resolves() {
        for name in BUILTIN_PACKS {
            let pack = builtin(name).unwrap_or_else(|| panic!("no builtin pack {name}"));
            assert_eq!(&pack.name, name);
            assert!(!pack.version.is_empty(), "{name} has no version");
            assert!(!pack.license.is_empty(), "{name} declares no license");
            assert!(!pack.sources.is_empty(), "{name} cites no source");
            assert!(
                !pack.terms.is_empty() || !pack.phrases.is_empty(),
                "{name} is empty"
            );
        }
        assert!(builtin("no-such-pack").is_none());
    }

    #[test]
    fn pack_rule_ids_are_unique_and_patterns_parse() {
        for name in BUILTIN_PACKS {
            let pack = builtin(name).unwrap();
            let mut ids: Vec<&str> = pack
                .terms
                .iter()
                .map(|t| t.id.as_str())
                .chain(pack.phrases.iter().map(|p| p.id.as_str()))
                .collect();
            let before = ids.len();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), before, "duplicate rule ids in {name}");
            // A pattern that does not compile matches nothing, and a pack of
            // silent non-matches looks exactly like a pack of true zeros.
            pack.validate_patterns()
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    #[test]
    fn every_pack_round_trips_through_json() {
        for name in BUILTIN_PACKS {
            let pack = builtin(name).unwrap();
            let json = serde_json::to_string(&pack).unwrap();
            let back: LexiconPack = serde_json::from_str(&json).unwrap();
            assert_eq!(back, pack, "{name} did not round-trip");
        }
    }

    #[test]
    fn terms_are_lowercase_so_the_tokenizer_can_match_them() {
        for name in BUILTIN_PACKS {
            let pack = builtin(name).unwrap();
            for term in &pack.terms {
                assert_eq!(
                    term.word,
                    term.word.to_lowercase(),
                    "{name}: {} is not lowercase",
                    term.word
                );
            }
        }
    }

    #[test]
    fn share_alike_obligations_never_reach_a_sibling_pack() {
        // A CC-BY-SA derivation lives in its own pack file. Any pack that is
        // not itself share-alike must stay that way.
        for name in BUILTIN_PACKS {
            let pack = builtin(name).unwrap();
            if pack.license.contains("SA") {
                assert_eq!(
                    *name, "wiki-ai-signs",
                    "{name} is share-alike but is not the isolated derivation"
                );
            }
        }
    }
}
