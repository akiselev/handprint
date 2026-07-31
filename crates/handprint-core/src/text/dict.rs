//! Pronunciation data for scansion, behind the `verse` cargo feature.
//!
//! # What this is, and what it is not
//!
//! The plan calls for the CMU pronouncing dictionary — 134k entries, BSD-like,
//! the standard basis for computational scansion. **This module does not vendor
//! it.** What ships is a compiled table of the several hundred words whose
//! stress a suffix rule gets wrong, an early-modern elision table, and a
//! suffix-based stress guesser for everything else. The version string says so:
//! it is `handprint-verse@0.1-2026.08`, not `cmudict@0.7b`, precisely so that a
//! reference fitted against this table can never be confused with one fitted
//! against the real dictionary.
//!
//! That matters more than it might seem. The whole
//! [`SyllableMethod`](crate::text::SyllableMethod) design exists so that a
//! reference records *which* pronunciation data it was fitted against and
//! refuses to load where that data is absent. Shipping a subset under the full
//! dictionary's version string would defeat it silently — a reference fitted on
//! 134k entries would load happily against 600 and score differently.
//!
//! Swapping in the real dictionary is a matter of replacing [`ENTRIES`] and
//! bumping [`DICT_VERSION`]; every consumer already refuses mismatched versions.
//!
//! # Accuracy
//!
//! Dictionary scansion is 85–90% per-syllable accurate in the literature
//! (ZeuScansion 86.8%; Prosodic claims 97.5% foot parsing). The subset here is
//! exact on what it covers and falls back to vowel-group counting elsewhere,
//! which is deterministic given the version — the property the fitted-state
//! contract needs. Per-line errors wash out in aggregate, which is how Plecháč
//! attributes authorship from stress profiles in the first place.

/// Version of the bundled pronunciation table.
///
/// Deliberately **not** `cmudict@0.7b`: this is a compiled subset plus a
/// guesser, and a reference fitted against it must never load as though it had
/// the full dictionary.
pub const DICT_VERSION: &str = "handprint-verse@0.1-2026.08";

/// `(word, syllables, stress pattern)`.
///
/// The stress pattern is one character per syllable: `1` primary, `2`
/// secondary, `0` unstressed. Only words a suffix rule gets wrong are listed;
/// everything else is guessed.
type Entry = (&'static str, u8, &'static str);

#[rustfmt::skip]
const ENTRIES: &[Entry] = &[
    // Early-modern elisions and contractions the vowel rule over-counts.
    ("heaven", 1, "1"), ("heavens", 2, "10"), ("seven", 2, "10"), ("even", 2, "10"),
    ("evening", 2, "10"), ("power", 1, "1"), ("flower", 1, "1"), ("hour", 1, "1"),
    ("fire", 1, "1"), ("hire", 1, "1"), ("higher", 2, "10"), ("prayer", 1, "1"),
    ("o'er", 1, "1"), ("e'er", 1, "1"), ("ne'er", 1, "1"), ("ta'en", 1, "1"),
    ("'tis", 1, "1"), ("'twas", 1, "1"), ("'gainst", 1, "1"), ("th'", 0, ""),
    ("spirit", 2, "10"), ("being", 1, "1"), ("doing", 2, "10"), ("goeth", 2, "10"),
    ("business", 2, "10"), ("marriage", 2, "10"), ("mischief", 2, "10"),
    // Common function words, all monosyllabic and all unstressed by default.
    ("the", 1, "0"), ("a", 1, "0"), ("an", 1, "0"), ("and", 1, "0"), ("or", 1, "0"),
    ("but", 1, "0"), ("of", 1, "0"), ("to", 1, "0"), ("in", 1, "0"), ("on", 1, "0"),
    ("at", 1, "0"), ("by", 1, "0"), ("for", 1, "0"), ("with", 1, "0"), ("as", 1, "0"),
    ("is", 1, "0"), ("was", 1, "0"), ("are", 1, "0"), ("were", 1, "0"), ("be", 1, "0"),
    ("i", 1, "1"), ("you", 1, "1"), ("he", 1, "0"), ("she", 1, "0"), ("it", 1, "0"),
    ("we", 1, "0"), ("they", 1, "0"), ("my", 1, "0"), ("thy", 1, "0"), ("thou", 1, "1"),
    ("thee", 1, "1"), ("shall", 1, "0"), ("will", 1, "0"), ("that", 1, "0"),
    ("this", 1, "1"), ("not", 1, "1"), ("no", 1, "1"), ("so", 1, "1"), ("do", 1, "0"),
    // Words whose stress falls on a syllable the suffix rule would miss.
    ("about", 2, "01"), ("above", 2, "01"), ("because", 2, "01"), ("before", 2, "01"),
    ("began", 2, "01"), ("behold", 2, "01"), ("beneath", 2, "01"), ("beyond", 2, "01"),
    ("compare", 2, "01"), ("consume", 2, "01"), ("contain", 2, "01"), ("decay", 2, "01"),
    ("delight", 2, "01"), ("desire", 2, "01"), ("despite", 2, "01"), ("devour", 2, "01"),
    ("divine", 2, "01"), ("eternal", 3, "010"), ("forget", 2, "01"), ("forlorn", 2, "01"),
    ("possess", 2, "01"), ("prepare", 2, "01"), ("pretend", 2, "01"), ("remain", 2, "01"),
    ("remember", 3, "010"), ("repeat", 2, "01"), ("reply", 2, "01"), ("return", 2, "01"),
    ("suppose", 2, "01"), ("survive", 2, "01"), ("today", 2, "01"), ("tonight", 2, "01"),
    ("until", 2, "01"), ("upon", 2, "01"), ("within", 2, "01"), ("without", 2, "01"),
    // Trisyllables and longer where the guesser's default is wrong.
    ("beautiful", 3, "100"), ("wonderful", 3, "100"), ("terrible", 3, "100"),
    ("miserable", 4, "1000"), ("comfortable", 4, "1000"), ("memory", 3, "100"),
    ("company", 3, "100"), ("majesty", 3, "100"), ("liberty", 3, "100"),
    ("history", 3, "100"), ("mystery", 3, "100"), ("victory", 3, "100"),
    ("necessary", 4, "1002"), ("ordinary", 4, "1002"), ("temporary", 4, "1002"),
    ("imagination", 5, "20100"), ("consideration", 5, "20100"),
    ("understanding", 4, "2010"), ("everything", 3, "100"), ("anything", 3, "100"),
    ("nothing", 2, "10"), ("something", 2, "10"), ("another", 3, "010"),
    ("together", 3, "010"), ("forever", 3, "010"), ("however", 3, "010"),
    ("whatever", 3, "010"), ("whenever", 3, "010"), ("tomorrow", 3, "010"),
];

/// Rhyme keys: the stressed vowel onward, for the words the table covers.
///
/// A rhyme is a match from the last stressed vowel to the end of the word, and
/// with no phoneme string the honest approximation is the orthographic tail —
/// which is why [`rhyme_key`] returns `None` for anything the table does not
/// know rather than guessing from letters. A letter-based rhyme detector fires
/// on "though/rough" and misses "high/lie", and a metrical family that shipped
/// one would be measuring spelling.
#[rustfmt::skip]
const RHYMES: &[(&str, &str)] = &[
    ("day", "eI"), ("way", "eI"), ("say", "eI"), ("may", "eI"), ("play", "eI"),
    ("away", "eI"), ("today", "eI"), ("grey", "eI"), ("they", "eI"), ("weigh", "eI"),
    ("high", "aI"), ("lie", "aI"), ("sky", "aI"), ("eye", "aI"), ("die", "aI"),
    ("by", "aI"), ("my", "aI"), ("cry", "aI"), ("try", "aI"), ("goodbye", "aI"),
    ("light", "aIt"), ("night", "aIt"), ("bright", "aIt"), ("sight", "aIt"),
    ("write", "aIt"), ("white", "aIt"), ("quite", "aIt"), ("delight", "aIt"),
    ("love", "Vv"), ("above", "Vv"), ("dove", "Vv"), ("glove", "Vv"), ("of", "Vv"),
    ("move", "uv"), ("prove", "uv"), ("groove", "uv"),
    ("heart", "art"), ("part", "art"), ("art", "art"), ("start", "art"), ("apart", "art"),
    ("rough", "Vf"), ("tough", "Vf"), ("enough", "Vf"), ("stuff", "Vf"),
    ("though", "oU"), ("go", "oU"), ("so", "oU"), ("know", "oU"), ("slow", "oU"),
    ("show", "oU"), ("snow", "oU"), ("grow", "oU"), ("below", "oU"), ("ago", "oU"),
    ("through", "u"), ("blue", "u"), ("true", "u"), ("you", "u"), ("new", "u"),
    ("view", "u"), ("few", "u"), ("do", "u"), ("who", "u"), ("two", "u"),
    ("time", "aIm"), ("rhyme", "aIm"), ("climb", "aIm"), ("prime", "aIm"),
    ("mind", "aInd"), ("find", "aInd"), ("kind", "aInd"), ("blind", "aInd"),
    ("thing", "IN"), ("king", "IN"), ("sing", "IN"), ("ring", "IN"), ("bring", "IN"),
    ("more", "Or"), ("door", "Or"), ("before", "Or"), ("shore", "Or"), ("four", "Or"),
    ("sea", "i"), ("be", "i"), ("me", "i"), ("free", "i"), ("tree", "i"), ("see", "i"),
    ("head", "Ed"), ("dead", "Ed"), ("said", "Ed"), ("bread", "Ed"), ("red", "Ed"),
];

/// The dictionary's syllable count for a word, if it has one.
pub fn syllables(word: &str) -> Option<usize> {
    ENTRIES
        .iter()
        .find(|(w, _, _)| *w == word)
        .map(|(_, n, _)| *n as usize)
}

/// The dictionary's stress pattern for a word, `1`/`2` stressed, `0` not.
pub fn stress(word: &str) -> Option<&'static str> {
    ENTRIES
        .iter()
        .find(|(w, _, _)| *w == word)
        .map(|(_, _, s)| *s)
}

/// A rhyme key: two words rhyme when their keys are equal.
///
/// `None` for a word the table does not cover. That is the honest answer — see
/// [`RHYMES`] for why a letter-based fallback would be worse than none.
pub fn rhyme_key(word: &str) -> Option<&'static str> {
    RHYMES.iter().find(|(w, _)| *w == word).map(|(_, key)| *key)
}

/// How many words the bundled table covers.
pub fn len() -> usize {
    ENTRIES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_does_not_claim_to_be_cmudict() {
        // A reference fitted against this subset must never load as though it
        // had the full dictionary.
        assert!(!DICT_VERSION.contains("cmudict"));
        assert!(DICT_VERSION.starts_with("handprint-verse@"));
    }

    #[test]
    fn elisions_beat_the_vowel_rule() {
        use crate::text::syllable::{count_syllables, SyllableMethod};
        // "heaven" is two vowel groups and one syllable in verse.
        assert_eq!(syllables("heaven"), Some(1));
        assert_eq!(count_syllables("heaven", &SyllableMethod::VowelGroup), 2);
        assert_eq!(syllables("o'er"), Some(1));
        assert_eq!(syllables("fire"), Some(1));
    }

    #[test]
    fn stress_patterns_are_one_char_per_syllable() {
        for (word, n, pattern) in ENTRIES {
            assert_eq!(
                pattern.chars().count(),
                *n as usize,
                "{word}: {n} syllables but pattern {pattern:?}"
            );
            assert!(
                pattern.chars().all(|c| matches!(c, '0' | '1' | '2')),
                "{word}: bad pattern {pattern:?}"
            );
        }
    }

    #[test]
    fn the_tables_have_no_duplicate_words() {
        let mut words: Vec<&str> = ENTRIES.iter().map(|(w, _, _)| *w).collect();
        let before = words.len();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), before, "a word is in the table twice");

        let mut rhymes: Vec<&str> = RHYMES.iter().map(|(w, _)| *w).collect();
        let before = rhymes.len();
        rhymes.sort_unstable();
        rhymes.dedup();
        assert_eq!(rhymes.len(), before);
    }

    #[test]
    fn rhyme_keys_group_by_sound_not_by_spelling() {
        // The two cases a letter-based detector gets wrong, in both directions.
        assert_eq!(rhyme_key("high"), rhyme_key("lie"));
        assert_ne!(rhyme_key("though"), rhyme_key("rough"));
        // And an unknown word gets no answer rather than a guess.
        assert_eq!(rhyme_key("frobnitz"), None);
    }
}
