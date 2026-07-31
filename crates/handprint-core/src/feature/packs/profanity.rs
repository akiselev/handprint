//! Profanity, with severity tags.
//!
//! A register marker. Bukowski and Thompson are both high on it and differ on
//! nearly everything else, which is the point: profanity rate alone is a poor
//! discriminator, and profanity rate *next to* Latinate vocabulary is the
//! Thompson pole of `form:latinate_profanity_clash_rate`.
//!
//! Source: the LDNOOBW list, whose terms are permissively licensed. The
//! severity tags come from this project's own reading — dsojevic's tagged list
//! was the intended source and its terms were not verified, so the fallback the
//! plan names (LDNOOBW-only, tag locally) is what shipped.
//!
//! [`Severity`] here means "how strong is the word", not "how much should this
//! be changed". Nothing in this pack suggests an alternative, because the
//! critique layer would render it as advice, and telling an author to swear
//! less is not what a stylometer is for. The two-sided band is: an author whose
//! profanity rate falls *below* their own corpus band is drifting out of voice
//! just as surely as one who goes above it.

use super::{source, terms, WordRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of this pack's contents.
pub const VERSION: &str = "2026.08";

/// Mild: the words that pass in most published prose.
#[rustfmt::skip]
const MILD: &[WordRow] = &[
    ("damn", "mild", Severity::Low, &[]),
    ("damned", "mild", Severity::Low, &[]),
    ("dammit", "mild", Severity::Low, &[]),
    ("goddamn", "mild", Severity::Low, &[]),
    ("goddamned", "mild", Severity::Low, &[]),
    ("hell", "mild", Severity::Low, &[]),
    ("crap", "mild", Severity::Low, &[]),
    ("crappy", "mild", Severity::Low, &[]),
    ("bloody", "mild", Severity::Low, &[]),
    ("bugger", "mild", Severity::Low, &[]),
    ("bollocks", "mild", Severity::Low, &[]),
    ("arse", "mild", Severity::Low, &[]),
    ("ass", "mild", Severity::Low, &[]),
    ("arsehole", "mild", Severity::Low, &[]),
    ("bastard", "mild", Severity::Low, &[]),
    ("bastards", "mild", Severity::Low, &[]),
    ("git", "mild", Severity::Low, &[]),
    ("sod", "mild", Severity::Low, &[]),
    ("piss", "mild", Severity::Low, &[]),
    ("pissed", "mild", Severity::Low, &[]),
    ("bleeding", "mild", Severity::Low, &[]),
    ("blimey", "mild", Severity::Low, &[]),
];

/// Strong: the words that mark a register on their own.
#[rustfmt::skip]
const STRONG: &[WordRow] = &[
    ("shit", "strong", Severity::Medium, &[]),
    ("shits", "strong", Severity::Medium, &[]),
    ("shitty", "strong", Severity::Medium, &[]),
    ("bullshit", "strong", Severity::Medium, &[]),
    ("horseshit", "strong", Severity::Medium, &[]),
    ("shithead", "strong", Severity::Medium, &[]),
    ("asshole", "strong", Severity::Medium, &[]),
    ("assholes", "strong", Severity::Medium, &[]),
    ("dickhead", "strong", Severity::Medium, &[]),
    ("prick", "strong", Severity::Medium, &[]),
    ("wanker", "strong", Severity::Medium, &[]),
    ("tosser", "strong", Severity::Medium, &[]),
    ("bitch", "strong", Severity::Medium, &[]),
    ("bitches", "strong", Severity::Medium, &[]),
    ("whore", "strong", Severity::Medium, &[]),
    ("slut", "strong", Severity::Medium, &[]),
    ("scumbag", "strong", Severity::Medium, &[]),
    ("douchebag", "strong", Severity::Medium, &[]),
    ("jackass", "strong", Severity::Medium, &[]),
    ("dumbass", "strong", Severity::Medium, &[]),
];

/// Strongest: the words a publisher asks about.
#[rustfmt::skip]
const SEVERE: &[WordRow] = &[
    ("fuck", "severe", Severity::High, &[]),
    ("fucks", "severe", Severity::High, &[]),
    ("fucked", "severe", Severity::High, &[]),
    ("fucking", "severe", Severity::High, &[]),
    ("fucker", "severe", Severity::High, &[]),
    ("fuckers", "severe", Severity::High, &[]),
    ("motherfucker", "severe", Severity::High, &[]),
    ("motherfuckers", "severe", Severity::High, &[]),
    ("motherfucking", "severe", Severity::High, &[]),
    ("clusterfuck", "severe", Severity::High, &[]),
    ("cunt", "severe", Severity::High, &[]),
    ("cunts", "severe", Severity::High, &[]),
    ("cock", "severe", Severity::High, &[]),
    ("dick", "severe", Severity::High, &[]),
    ("twat", "severe", Severity::High, &[]),
];

/// Build the pack.
pub fn pack() -> LexiconPack {
    let all: Vec<WordRow> = MILD.iter().chain(STRONG).chain(SEVERE).copied().collect();
    LexiconPack {
        name: "profanity".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Profanity by strength (mild / strong / severe). A register marker, not \
                      advice: no entry suggests an alternative, and the two-sided band means \
                      falling below an author's own rate is as much a drift as going above it."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "LDNOOBW",
                "https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words",
                "term list, permissively licensed",
            ),
            source(
                "Severity tags: original",
                "https://github.com/akiselev/handprint",
                "tagged here rather than taken from dsojevic's severity list, whose terms were \
                 not verified — the LDNOOBW-only fallback",
            ),
        ],
        terms: terms(&all),
        phrases: Vec::new(),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_strength_bands_are_present_and_ordered() {
        let pack = pack();
        let mut categories = pack.categories();
        categories.sort();
        assert_eq!(categories, vec!["mild", "severe", "strong"]);

        let severity_of = |category: &str| {
            pack.terms
                .iter()
                .find(|t| t.category == category)
                .map(|t| t.severity)
                .unwrap()
        };
        assert!(severity_of("mild") < severity_of("strong"));
        assert!(severity_of("strong") < severity_of("severe"));
    }

    #[test]
    fn no_entry_suggests_a_replacement() {
        // Telling an author to swear less is not what a stylometer is for, and
        // an `alternatives` list would render as exactly that advice.
        for term in pack().terms {
            assert!(term.alternatives.is_empty(), "{} suggests a fix", term.word);
        }
    }
}
