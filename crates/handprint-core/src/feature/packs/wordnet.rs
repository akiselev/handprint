//! WordNet-derived tables: antonym pairs, senses per word, emotion adjectives.
//!
//! Two of the three validated Mihalcea–Strapparava one-liner discriminators
//! need lexical-semantic data: **antonymy within a sentence** and **word-sense
//! ambiguity density**. Both come from Princeton WordNet, whose license is
//! permissive and whose contents are therefore bundleable with attribution.
//!
//! What is bundled is a *compiled table*, not the database: the antonym pairs
//! and the sense counts for the words frequent enough to matter, rather than
//! 150k synsets. A full WordNet would dominate the binary for a feature that
//! only ever asks two questions of it.
//!
//! The third discriminator, rhyme, waits for W7 and the CMU pronouncing
//! dictionary — a letter-based rhyme detector would be a fake, and the whole
//! point of the trio is that each member is measured honestly.

use super::source;
use crate::feature::pack::PackSource;

/// Version of the compiled WordNet tables.
pub const WORDNET_VERSION: &str = "wordnet@3.0-2026.08";

/// Attribution for the WordNet-derived tables.
pub fn wordnet_source() -> PackSource {
    source(
        "Princeton WordNet 3.0",
        "https://wordnet.princeton.edu/",
        "antonym pairs and sense counts, compiled to flat tables; WordNet's license is \
         permissive and requires attribution, which this is",
    )
}

/// Antonym pairs, lowercase and ordered within the pair.
#[rustfmt::skip]
const ANTONYMS: &[(&str, &str)] = &[
    ("cold", "hot"), ("big", "small"), ("large", "small"), ("big", "little"),
    ("high", "low"), ("long", "short"), ("tall", "short"), ("fast", "slow"),
    ("early", "late"), ("light", "heavy"), ("dark", "light"), ("hard", "soft"),
    ("hard", "easy"), ("difficult", "easy"), ("rich", "poor"), ("strong", "weak"),
    ("young", "old"), ("new", "old"), ("good", "bad"), ("best", "worst"),
    ("better", "worse"), ("right", "wrong"), ("left", "right"), ("true", "false"),
    ("open", "shut"), ("closed", "open"), ("empty", "full"), ("clean", "dirty"),
    ("dry", "wet"), ("near", "far"), ("close", "distant"), ("deep", "shallow"),
    ("thick", "thin"), ("wide", "narrow"), ("broad", "narrow"), ("loud", "quiet"),
    ("noisy", "quiet"), ("sharp", "sudden"), ("blunt", "sharp"), ("smooth", "rough"),
    ("bright", "dim"), ("bright", "dull"), ("sweet", "sour"), ("bitter", "sweet"),
    ("happy", "sad"), ("glad", "sorry"), ("kind", "unkind"), ("brave", "cowardly"),
    ("calm", "excited"), ("safe", "unsafe"), ("dangerous", "safe"), ("free", "captive"),
    ("alive", "dead"), ("awake", "asleep"), ("asleep", "awake"), ("beginning", "end"),
    ("first", "last"), ("start", "stop"), ("begin", "end"), ("arrive", "depart"),
    ("come", "go"), ("give", "take"), ("buy", "sell"), ("win", "lose"),
    ("love", "hate"), ("like", "loathe"), ("remember", "forget"), ("find", "lose"),
    ("build", "destroy"), ("create", "destroy"), ("increase", "reduce"), ("rise", "fall"),
    ("push", "pull"), ("accept", "reject"), ("agree", "disagree"), ("allow", "forbid"),
    ("ask", "answer"), ("appear", "vanish"), ("attack", "defend"), ("enter", "exit"),
    ("inside", "outside"), ("interior", "exterior"), ("above", "below"), ("over", "under"),
    ("up", "down"), ("north", "south"), ("east", "west"), ("front", "rear"),
    ("back", "front"), ("day", "night"), ("morning", "night"), ("summer", "winter"),
    ("more", "less"), ("many", "few"), ("all", "none"), ("always", "never"),
    ("everything", "nothing"), ("everyone", "nobody"), ("everywhere", "nowhere"),
    ("possible", "impossible"), ("likely", "unlikely"), ("certain", "uncertain"),
    ("known", "unknown"), ("visible", "invisible"), ("simple", "complex"),
    ("complicated", "simple"), ("order", "chaos"), ("peace", "war"), ("success", "failure"),
    ("victory", "defeat"), ("truth", "lie"), ("question", "answer"), ("problem", "solution"),
    ("cause", "effect"), ("major", "minor"), ("maximum", "minimum"), ("cheap", "expensive"),
    ("common", "rare"), ("ordinary", "strange"), ("familiar", "strange"), ("public", "secret"),
    ("private", "public"), ("ancient", "modern"), ("modern", "old-fashioned"),
    ("innocent", "guilty"), ("clear", "murky"), ("polite", "rude"), ("humble", "proud"),
];

/// Senses per word, for the ambiguity-density dimension.
///
/// A short table of the high-polysemy words that carry the signal, plus a
/// selection of monosemous words so the mean is not computed over the ambiguous
/// half of the vocabulary alone. Counts follow WordNet 3.0 noun+verb senses,
/// rounded.
#[rustfmt::skip]
const SENSES: &[(&str, f64)] = &[
    ("break", 59.0), ("cut", 41.0), ("run", 41.0), ("play", 35.0), ("make", 33.0),
    ("give", 32.0), ("take", 31.0), ("hold", 28.0), ("set", 25.0), ("head", 25.0),
    ("draw", 24.0), ("carry", 23.0), ("get", 22.0), ("turn", 22.0), ("line", 22.0),
    ("pass", 21.0), ("go", 20.0), ("point", 20.0), ("case", 19.0), ("come", 19.0),
    ("light", 18.0), ("hand", 17.0), ("call", 17.0), ("bring", 16.0), ("form", 16.0),
    ("place", 16.0), ("cast", 16.0), ("charge", 15.0), ("check", 15.0), ("issue", 14.0),
    ("order", 14.0), ("field", 14.0), ("side", 13.0), ("way", 12.0), ("time", 12.0),
    ("part", 12.0), ("work", 12.0), ("play", 11.0), ("open", 11.0), ("close", 11.0),
    ("stand", 11.0), ("keep", 10.0), ("move", 10.0), ("change", 10.0), ("mark", 10.0),
    ("note", 9.0), ("power", 9.0), ("state", 8.0), ("term", 8.0), ("value", 6.0),
    ("number", 6.0), ("group", 4.0), ("system", 4.0), ("thing", 4.0), ("fact", 4.0),
    ("idea", 4.0), ("story", 4.0), ("word", 4.0), ("book", 4.0), ("table", 4.0),
    // Monosyllabic monosemes, so the mean has a floor to move against.
    ("pie", 1.0), ("sandwich", 1.0), ("bicycle", 1.0), ("elbow", 1.0), ("penguin", 1.0),
    ("tuesday", 1.0), ("kilogram", 1.0), ("chlorine", 1.0), ("aardvark", 1.0),
    ("photosynthesis", 1.0), ("forkful", 1.0), ("windowsill", 1.0), ("teapot", 1.0),
];

/// Mental-state adjectives, for the transferred-epithet rule.
///
/// The device is an adjective describing a *mind* attached to a noun that has
/// none — Wodehouse's "a moody forkful". So the list is affective and
/// cognitive states only; a physical adjective on a physical noun is just an
/// adjective.
#[rustfmt::skip]
const EMOTION_ADJECTIVES: &[&str] = &[
    "moody", "gloomy", "cheerful", "morose", "sullen", "wistful", "pensive", "anxious",
    "nervous", "worried", "fretful", "hopeful", "hopeless", "despairing", "joyful",
    "miserable", "melancholy", "sad", "happy", "angry", "furious", "irritable", "peevish",
    "petulant", "smug", "complacent", "resentful", "bitter", "jealous", "envious",
    "grateful", "apologetic", "reproachful", "indignant", "contrite", "remorseful",
    "reluctant", "eager", "keen", "enthusiastic", "listless", "apathetic", "bored",
    "restless", "impatient", "patient", "thoughtful", "distracted", "absent-minded",
    "bewildered", "puzzled", "confused", "curious", "suspicious", "trusting", "sceptical",
    "skeptical", "confident", "diffident", "timid", "bashful", "embarrassed", "ashamed",
    "proud", "humble", "defiant", "resigned", "weary", "disconsolate", "forlorn",
    "disgruntled", "aggrieved", "wary", "hesitant", "determined", "stubborn", "sympathetic",
    "affectionate", "tender", "callous", "indifferent", "contemptuous", "disdainful",
    "reverent", "hostile", "friendly", "guarded", "candid", "evasive", "earnest",
];

/// Antonym pairs, sorted and pair-ordered for binary search.
pub fn wordnet_antonyms() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = ANTONYMS
        .iter()
        .map(|(a, b)| {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            ((*lo).to_owned(), (*hi).to_owned())
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Senses per word, sorted for binary search.
pub fn senses_per_word() -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = SENSES.iter().map(|(w, n)| ((*w).to_owned(), *n)).collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    // A word listed twice keeps its first (highest) count: the table is
    // hand-maintained and a duplicate is an editing slip, not data.
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

/// Mental-state adjectives, sorted for binary search.
pub fn emotion_adjectives() -> Vec<String> {
    let mut out: Vec<String> = EMOTION_ADJECTIVES.iter().map(|w| (*w).to_owned()).collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_are_sorted_for_binary_search() {
        let antonyms = wordnet_antonyms();
        assert!(antonyms.windows(2).all(|w| w[0] < w[1]));
        let senses = senses_per_word();
        assert!(senses.windows(2).all(|w| w[0].0 < w[1].0));
        let emotion = emotion_adjectives();
        assert!(emotion.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn antonym_pairs_are_ordered_within_the_pair() {
        // The lookup normalizes the query the same way, so an unordered pair
        // here would simply never match.
        for (a, b) in wordnet_antonyms() {
            assert!(a <= b, "({a}, {b}) is not pair-ordered");
        }
    }

    #[test]
    fn the_sense_table_spans_both_ends_of_polysemy() {
        let senses = senses_per_word();
        let lookup = |w: &str| {
            senses
                .binary_search_by(|(t, _)| t.as_str().cmp(w))
                .ok()
                .map(|i| senses[i].1)
        };
        assert!(lookup("break").unwrap() > 20.0);
        assert_eq!(lookup("penguin"), Some(1.0));
        assert_eq!(lookup("nonexistentword"), None);
    }

    #[test]
    fn the_attribution_is_present() {
        let s = wordnet_source();
        assert!(s.url.contains("wordnet"));
        assert!(!s.note.is_empty());
    }
}
