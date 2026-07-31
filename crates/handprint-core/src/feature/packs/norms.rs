//! Bundled [`NormPack`]s — weighted lexicons small enough to ship.
//!
//! The good weighted lexicons are mostly not shippable. Brysbaert's
//! concreteness norms and Warriner's VAD ratings are probably CC-BY-NC; NRC
//! EmoLex forbids redistribution outright; Pavlick & Tetreault's formality
//! annotations have terms nobody here has verified. All of those are
//! **loader-only**: handprint ships the feature and the loader, the user
//! supplies the table.
//!
//! What is here is what can be shipped: a closed-class formality proxy that
//! needs no external data at all, and the VADER booster weights, which are MIT.
//!
//! The design rule behind the split is that a feature which silently does
//! nothing without a download is worse than a coarse feature that always runs.
//! Every family with a loader-only dependency has either a bundled fallback or
//! a `mark_missing` path, never a silent zero.

use crate::feature::pack::{NormEntry, NormPack};

use super::source;

/// Version of the bundled norm tables.
pub const VERSION: &str = "2026.08";

/// Word classes and their contribution to the Heylighen–Dewaele F-score.
///
/// The original F-score is computed from *part-of-speech proportions*:
///
/// ```text
/// F = (noun + adjective + preposition + article
///      − pronoun − verb − adverb − interjection + 100) / 2
/// ```
///
/// which needs a tagger. The closed-class approximation here scores the words
/// that *are* their class — articles, prepositions, pronouns, auxiliaries,
/// interjections, discourse particles — and leaves open-class words unscored.
/// That is a real loss: the noun and adjective contributions, the bulk of the
/// formal side, are missing. What remains still separates registers, because
/// the informal side of the formula is almost entirely closed-class, and the
/// formal side is carried by prepositions and articles which are too. Scores
/// are on a −2…+2 scale; only their ordering and spread matter.
type NormRow = (&'static str, f64);

/// Formal-side closed class: prepositions, articles, formal conjunctions.
#[rustfmt::skip]
const FORMAL: &[NormRow] = &[
    ("the", 0.8), ("a", 0.6), ("an", 0.6), ("of", 1.0), ("in", 0.7), ("to", 0.5), ("for", 0.7),
    ("with", 0.8), ("on", 0.6), ("at", 0.6), ("by", 0.8), ("from", 0.7), ("as", 0.7),
    ("into", 0.9), ("during", 1.3), ("including", 1.4), ("until", 1.1), ("against", 1.2),
    ("among", 1.4), ("throughout", 1.5), ("despite", 1.5), ("towards", 1.3), ("toward", 1.2),
    ("upon", 1.6), ("concerning", 1.7), ("regarding", 1.7), ("within", 1.3), ("without", 1.0),
    ("prior", 1.5), ("subsequent", 1.8), ("pursuant", 2.0), ("notwithstanding", 2.0),
    ("hereby", 2.0), ("herein", 2.0), ("thereof", 2.0), ("whereby", 1.9), ("whereas", 1.7),
    ("moreover", 1.6), ("furthermore", 1.7), ("nevertheless", 1.6), ("nonetheless", 1.6),
    ("thus", 1.4), ("hence", 1.5), ("therefore", 1.3), ("accordingly", 1.6),
    ("consequently", 1.5), ("respectively", 1.7), ("approximately", 1.4),
    ("substantially", 1.5), ("primarily", 1.4), ("subsequently", 1.5), ("previously", 1.2),
    ("additionally", 1.3), ("specifically", 1.2), ("particularly", 1.2),
    ("significantly", 1.3), ("relatively", 1.1), ("comparatively", 1.5), ("shall", 1.8),
    ("must", 1.0), ("may", 0.9), ("ought", 1.2), ("whom", 1.7), ("whose", 1.1),
    ("which", 0.9), ("such", 1.0), ("aforementioned", 2.0), ("said", 0.4), ("former", 1.4),
    ("latter", 1.5), ("hereinafter", 2.0), ("thereafter", 1.8), ("henceforth", 1.9),
    ("insofar", 2.0), ("albeit", 1.8), ("whilst", 1.6), ("amongst", 1.6), ("per", 1.3),
    ("via", 1.2), ("versus", 1.3), ("although", 1.1), ("however", 1.2), ("unless", 0.9),
];

/// Informal-side closed class: personal pronouns, contractions, interjections,
/// discourse particles, intensifiers of the spoken register.
#[rustfmt::skip]
const INFORMAL: &[NormRow] = &[
    ("i", -1.2), ("me", -1.2), ("my", -1.0), ("mine", -1.1), ("myself", -0.9),
    ("you", -1.3), ("your", -1.1), ("yours", -1.2), ("yourself", -1.0), ("we", -0.7),
    ("us", -0.7), ("our", -0.6), ("he", -0.5), ("she", -0.5), ("him", -0.5), ("her", -0.5),
    ("they", -0.4), ("them", -0.4), ("it", -0.4), ("that", -0.3), ("this", -0.3),
    ("gonna", -2.0), ("wanna", -2.0), ("gotta", -2.0), ("kinda", -2.0), ("sorta", -2.0),
    ("dunno", -2.0), ("lemme", -2.0), ("gimme", -2.0), ("ain't", -2.0), ("y'all", -2.0),
    ("yeah", -1.9), ("yep", -1.9), ("yup", -1.9), ("nope", -1.9), ("nah", -1.9), ("huh", -1.9),
    ("wow", -1.7), ("ugh", -1.9), ("oh", -1.5), ("ah", -1.5), ("hey", -1.7), ("hi", -1.5),
    ("ok", -1.6), ("okay", -1.5), ("umm", -2.0), ("uh", -2.0), ("er", -1.8), ("hmm", -1.8),
    ("well", -0.8), ("anyway", -1.2), ("anyhow", -1.2), ("basically", -1.0),
    ("actually", -0.7), ("honestly", -1.0), ("frankly", -0.8), ("obviously", -0.6),
    ("literally", -1.0), ("totally", -1.4), ("really", -0.9), ("very", -0.4),
    ("pretty", -1.0), ("super", -1.5), ("stuff", -1.4), ("thing", -1.0), ("things", -1.0),
    ("guy", -1.4), ("guys", -1.5), ("kid", -1.1), ("kids", -1.1), ("lots", -1.2),
    ("bunch", -1.3), ("bit", -0.9), ("just", -0.6), ("maybe", -0.7), ("sure", -0.8),
    ("fine", -0.7), ("cool", -1.5), ("weird", -1.3), ("crazy", -1.4), ("nuts", -1.6),
    ("busted", -1.6), ("broke", -1.0), ("garbage", -1.3), ("mess", -1.2),
    ("whatever", -1.5), ("somehow", -0.8), ("someday", -0.9), ("don't", -1.3),
    ("doesn't", -1.2), ("didn't", -1.2), ("isn't", -1.2), ("aren't", -1.2),
    ("wasn't", -1.2), ("can't", -1.3), ("won't", -1.3), ("i'm", -1.4), ("i'll", -1.4),
    ("i've", -1.4), ("it's", -1.1), ("that's", -1.1), ("there's", -1.0), ("we're", -1.1),
    ("you're", -1.3), ("they're", -1.1), ("let's", -1.2), ("she's", -1.1), ("he's", -1.1),
];

/// The bundled Heylighen–Dewaele-style formality proxy.
///
/// The default for [`RegisterClash`](crate::feature::RegisterClash), and the
/// fallback whenever the Pavlick & Tetreault pack is unavailable.
pub fn heylighen_formality() -> NormPack {
    let entries: Vec<NormEntry> = FORMAL
        .iter()
        .chain(INFORMAL)
        .map(|(term, value)| NormEntry {
            term: (*term).to_owned(),
            value: *value,
        })
        .collect();
    NormPack {
        name: "formality-proxy".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Closed-class formality proxy on a -2..+2 scale, after Heylighen & \
                      Dewaele's F-score. Coarser than annotated per-word formality, and it \
                      needs no external data — which is why it is the bundled default."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "Heylighen & Dewaele, Formality of Language",
                "https://www.researchgate.net/publication/2489015",
                "the POS-proportion F-score this approximates without a tagger",
            ),
            source(
                "Pavlick & Tetreault 2016 (NOT bundled)",
                "https://huggingface.co/datasets/osyvokon/pavlick-formality-scores",
                "the better data, loader-only pending license verification; supply it with \
                 RegisterClash::with_norms",
            ),
        ],
        entries,
    }
}

/// VADER booster weights: words that scale an adjacent sentiment term.
///
/// MIT-licensed and small, so it ships. Consumed by the syntax family's
/// intensifier and booster-chain dimensions — a *run* of these is the Thompson
/// escalation marker, and a run is only definable against a list.
#[rustfmt::skip]
const BOOSTERS: &[NormRow] = &[
    ("absolutely", 0.293), ("amazingly", 0.293), ("awfully", 0.293), ("completely", 0.293),
    ("considerable", 0.293), ("considerably", 0.293), ("decidedly", 0.293), ("deeply", 0.293),
    ("effing", 0.293), ("enormous", 0.293), ("enormously", 0.293), ("entirely", 0.293),
    ("especially", 0.293), ("exceptional", 0.293), ("exceptionally", 0.293),
    ("extreme", 0.293), ("extremely", 0.293), ("fabulously", 0.293), ("flat", 0.293),
    ("fully", 0.293), ("frickin", 0.293), ("frigging", 0.293), ("fucking", 0.293),
    ("greatly", 0.293), ("hella", 0.293), ("highly", 0.293), ("hugely", 0.293),
    ("incredible", 0.293), ("incredibly", 0.293), ("intensely", 0.293), ("major", 0.293),
    ("majorly", 0.293), ("more", 0.293), ("most", 0.293), ("particularly", 0.293),
    ("purely", 0.293), ("quite", 0.293), ("really", 0.293), ("remarkably", 0.293),
    ("so", 0.293), ("substantially", 0.293), ("thoroughly", 0.293), ("total", 0.293),
    ("totally", 0.293), ("tremendous", 0.293), ("tremendously", 0.293), ("uber", 0.293),
    ("unbelievably", 0.293), ("unusually", 0.293), ("utterly", 0.293), ("very", 0.293),
    ("damn", 0.293), ("goddamn", 0.293), ("bloody", 0.293), ("wildly", 0.293),
    ("savagely", 0.293), ("violently", 0.293), ("desperately", 0.293), ("horribly", 0.293),
    ("terribly", 0.293), ("insanely", 0.293), ("ridiculously", 0.293), ("absurdly", 0.293),
    // Dampeners: the same list's negative half.
    ("almost", -0.293), ("barely", -0.293), ("hardly", -0.293), ("just", -0.293),
    ("kind", -0.293), ("kinda", -0.293), ("less", -0.293), ("little", -0.293),
    ("marginal", -0.293), ("marginally", -0.293), ("occasional", -0.293),
    ("occasionally", -0.293), ("partly", -0.293), ("scarce", -0.293), ("scarcely", -0.293),
    ("slight", -0.293), ("slightly", -0.293), ("somewhat", -0.293), ("sort", -0.293),
    ("sorta", -0.293),
];

/// The bundled VADER booster weights.
pub fn vader_boosters() -> NormPack {
    NormPack {
        name: "vader-boosters".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "VADER booster/dampener weights: words that scale an adjacent sentiment \
                      term up or down. Positive weights amplify, negative weights damp."
            .into(),
        license: "MIT".into(),
        redistributable: true,
        sources: vec![
            source(
                "VADER sentiment (cjhutto)",
                "https://github.com/cjhutto/vaderSentiment",
                "BOOSTER_DICT, MIT-licensed",
            ),
            source(
                "Taboada et al., Lexicon-Based Sentiment Analysis",
                "https://aclanthology.org/J11-2001.pdf",
                "the amplifier/downtoner taxonomy behind the split",
            ),
        ],
        entries: BOOSTERS
            .iter()
            .map(|(term, value)| NormEntry {
                term: (*term).to_owned(),
                value: *value,
            })
            .collect(),
    }
}

/// A **demonstration** concreteness table — not Brysbaert's norms.
///
/// Brysbaert, Warriner & Kuperman's 37k-lemma concreteness ratings are the real
/// data and are probably CC-BY-NC, so they are loader-only: supply them with
/// [`DeviceRates::with_concreteness`](crate::feature::DeviceRates::with_concreteness).
///
/// This stub exists so the transferred-epithet and concreteness dimensions are
/// *testable and demonstrable* without that download, and so nobody mistakes an
/// absent table for a working one. It is a hundred hand-scored words on the
/// same 1–5 scale, with `pd_basis` stated in its description, and it is not a
/// substitute for the real norms in any measurement anyone reports.
#[rustfmt::skip]
const CONCRETENESS: &[NormRow] = &[
    // Concrete: things you can point at.
    ("forkful", 4.9), ("sandwich", 5.0), ("teapot", 5.0), ("brick", 5.0), ("chair", 5.0),
    ("table", 4.9), ("window", 5.0), ("door", 5.0), ("hammer", 5.0), ("shoe", 5.0),
    ("dog", 5.0), ("cat", 5.0), ("penguin", 5.0), ("tree", 5.0), ("river", 4.9),
    ("bicycle", 5.0), ("kettle", 5.0), ("spoon", 5.0), ("bottle", 5.0), ("pencil", 5.0),
    ("carpet", 4.9), ("staircase", 4.9), ("umbrella", 5.0), ("newspaper", 4.9),
    ("suitcase", 4.9), ("elbow", 4.9), ("shoulder", 4.9), ("pie", 4.9), ("bread", 5.0),
    ("coffee", 4.9), ("glass", 4.8), ("desk", 5.0), ("lamp", 5.0), ("keyboard", 4.9),
    ("cable", 4.8), ("engine", 4.7), ("wall", 4.9), ("floor", 4.9), ("roof", 4.9),
    ("boot", 4.9), ("hat", 5.0), ("coat", 5.0), ("knife", 5.0), ("plate", 5.0),
    // Middling.
    ("machine", 4.3), ("building", 4.5), ("street", 4.6), ("city", 4.3),
    ("crowd", 4.1), ("meeting", 3.6), ("letter", 4.2), ("report", 3.5), ("message", 3.2),
    // Abstract: things you cannot.
    ("justice", 1.5), ("freedom", 1.7), ("truth", 1.6), ("meaning", 1.6), ("purpose", 1.6),
    ("hope", 1.9), ("despair", 1.8), ("boredom", 1.8), ("happiness", 1.9), ("misery", 2.0),
    ("theory", 1.7), ("concept", 1.6), ("idea", 1.8), ("possibility", 1.5), ("reason", 1.8),
    ("consequence", 1.7), ("existence", 1.6), ("eternity", 1.6), ("infinity", 1.6),
    ("bureaucracy", 2.1), ("policy", 2.0), ("strategy", 1.9), ("assumption", 1.7),
    ("disappointment", 2.0), ("expectation", 1.8), ("responsibility", 1.9),
];

/// The bundled concreteness demonstration table. See [`CONCRETENESS`].
pub fn concreteness_stub() -> NormPack {
    NormPack {
        name: "concreteness-stub".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "DEMONSTRATION concreteness table on the 1-5 Brysbaert scale — about a \
                      hundred hand-scored words, NOT the 37k-lemma norms. Supply the real \
                      ones with with_concreteness(); they are loader-only because their \
                      likely CC-BY-NC terms conflict with commercial redistribution."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "Original hand-scored stub",
                "https://github.com/akiselev/handprint",
                "a demonstration table so the dimension is testable without a download",
            ),
            source(
                "Brysbaert, Warriner & Kuperman (NOT bundled)",
                "https://link.springer.com/article/10.3758/s13428-013-0403-5",
                "the real 37k-lemma norms — loader-only, likely CC-BY-NC",
            ),
        ],
        entries: CONCRETENESS
            .iter()
            .map(|(term, value)| NormEntry {
                term: (*term).to_owned(),
                value: *value,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_booster_pack_splits_amplifiers_from_dampeners() {
        let pack = vader_boosters();
        pack.validate().unwrap();
        assert!(pack.get("extremely").unwrap() > 0.0);
        assert!(pack.get("barely").unwrap() < 0.0);
        assert_eq!(pack.get("table"), None);
        assert_eq!(pack.license, "MIT");
    }

    #[test]
    fn the_concreteness_stub_is_labelled_as_a_stub() {
        let pack = concreteness_stub();
        pack.validate().unwrap();
        // Nobody should be able to mistake this for the real norms.
        assert!(pack.name.contains("stub"));
        assert!(pack.description.contains("DEMONSTRATION"));
        assert!(pack.sources.iter().any(|s| s.name.contains("NOT bundled")));
        // It still has to span the scale, or the quartile cut is meaningless.
        assert!(pack.get("brick").unwrap() > 4.0);
        assert!(pack.get("justice").unwrap() < 2.0);
    }

    #[test]
    fn the_proxy_validates_and_round_trips() {
        let pack = heylighen_formality();
        pack.validate().unwrap();
        let json = serde_json::to_string(&pack).unwrap();
        assert_eq!(serde_json::from_str::<NormPack>(&json).unwrap(), pack);
    }

    #[test]
    fn no_term_is_scored_twice() {
        // A duplicate would make `table()` silently keep whichever sorted
        // first, so the two halves of the scale must not overlap.
        let pack = heylighen_formality();
        let mut terms: Vec<&str> = pack.entries.iter().map(|e| e.term.as_str()).collect();
        let before = terms.len();
        terms.sort_unstable();
        terms.dedup();
        assert_eq!(terms.len(), before, "a term is in both halves of the scale");
    }

    #[test]
    fn the_scale_spans_both_registers() {
        let pack = heylighen_formality();
        assert!(pack.get("notwithstanding").unwrap() > 1.5);
        assert!(pack.get("gonna").unwrap() < -1.5);
        // The quartile cuts the clash detector uses must actually separate.
        let high = pack.quantile(0.75).unwrap();
        let low = pack.quantile(0.25).unwrap();
        assert!(high > low, "high={high} low={low}");
        assert!(low < 0.0 && high > 0.0, "high={high} low={low}");
    }

    #[test]
    fn every_entry_is_lowercase_so_the_tokenizer_can_match_it() {
        for entry in heylighen_formality().entries {
            assert_eq!(entry.term, entry.term.to_lowercase());
        }
    }
}
