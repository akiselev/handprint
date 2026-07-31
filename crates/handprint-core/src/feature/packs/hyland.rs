//! Hyland's metadiscourse taxonomy, re-curated.
//!
//! Hyland (2005) splits metadiscourse — the parts of a text that are about the
//! text and its readers rather than about the subject — into two halves:
//!
//! * **Interactive**, which organizes the argument for the reader: transitions,
//!   frame markers, endophorics (pointers to other parts of the text),
//!   evidentials (pointers outside it), code glosses ("i.e.", "namely").
//! * **Interactional**, which positions the writer relative to the claim:
//!   hedges, boosters, attitude markers, engagement markers, self-mentions.
//!
//! The categories are the point. Individual item rates are too sparse to mean
//! anything at draft length, which is why this pack is fitted with
//! [`LexiconFeature::category_findings`](crate::feature::LexiconFeature::category_findings)
//! and reports `lex:hyland:cat:hedges` rather than `lex:hyland:word.perhaps`.
//!
//! **Double duty as a de-AI family.** Humans hedge roughly 40% more than LLMs
//! in advice text, and the booster:hedge ratio separates them further. So the
//! same dimension that describes an academic register also flags machine prose
//! — which is the argument for building this family first.
//!
//! **Known noise, and why it is fine.** `may`, `about`, `just` and `like` are
//! polysemous, and no lookup can tell the epistemic `may` from the deontic one.
//! The mistake rate is a property of English, not of a corpus, so it is the same
//! on the reference side and the draft side and cancels in the comparison. An
//! absolute hedge count from this pack would be wrong by a wide margin; a delta
//! against a corpus fitted the same way is not.
//!
//! Re-curated from the published category tables, not copied from the book's
//! appendix: the items are unprotectable words, the compilation is not.

use super::{phrases, source, terms, PhraseRow, WordRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of this pack's contents.
pub const VERSION: &str = "2026.08";

/// Hedges: markers that withhold full commitment to a claim.
const HEDGES: &[WordRow] = &[
    ("about", "hedges", Severity::Low, &[]),
    ("almost", "hedges", Severity::Low, &[]),
    ("apparent", "hedges", Severity::Low, &[]),
    ("apparently", "hedges", Severity::Low, &[]),
    ("approximately", "hedges", Severity::Low, &[]),
    ("argue", "hedges", Severity::Low, &[]),
    ("argued", "hedges", Severity::Low, &[]),
    ("argues", "hedges", Severity::Low, &[]),
    ("assume", "hedges", Severity::Low, &[]),
    ("assumed", "hedges", Severity::Low, &[]),
    ("broadly", "hedges", Severity::Low, &[]),
    ("claim", "hedges", Severity::Low, &[]),
    ("claimed", "hedges", Severity::Low, &[]),
    ("could", "hedges", Severity::Low, &[]),
    ("doubt", "hedges", Severity::Low, &[]),
    ("essentially", "hedges", Severity::Low, &[]),
    ("estimate", "hedges", Severity::Low, &[]),
    ("estimated", "hedges", Severity::Low, &[]),
    ("fairly", "hedges", Severity::Low, &[]),
    ("feel", "hedges", Severity::Low, &[]),
    ("feels", "hedges", Severity::Low, &[]),
    ("frequently", "hedges", Severity::Low, &[]),
    ("generally", "hedges", Severity::Low, &[]),
    ("guess", "hedges", Severity::Low, &[]),
    ("indicate", "hedges", Severity::Low, &[]),
    ("indicated", "hedges", Severity::Low, &[]),
    ("largely", "hedges", Severity::Low, &[]),
    ("likely", "hedges", Severity::Low, &[]),
    ("mainly", "hedges", Severity::Low, &[]),
    ("may", "hedges", Severity::Low, &[]),
    ("maybe", "hedges", Severity::Low, &[]),
    ("might", "hedges", Severity::Low, &[]),
    ("mostly", "hedges", Severity::Low, &[]),
    ("often", "hedges", Severity::Low, &[]),
    ("ostensibly", "hedges", Severity::Low, &[]),
    ("perhaps", "hedges", Severity::Low, &[]),
    ("plausible", "hedges", Severity::Low, &[]),
    ("plausibly", "hedges", Severity::Low, &[]),
    ("possible", "hedges", Severity::Low, &[]),
    ("possibly", "hedges", Severity::Low, &[]),
    ("postulate", "hedges", Severity::Low, &[]),
    ("presumably", "hedges", Severity::Low, &[]),
    ("probable", "hedges", Severity::Low, &[]),
    ("probably", "hedges", Severity::Low, &[]),
    ("relatively", "hedges", Severity::Low, &[]),
    ("roughly", "hedges", Severity::Low, &[]),
    ("seem", "hedges", Severity::Low, &[]),
    ("seemed", "hedges", Severity::Low, &[]),
    ("seems", "hedges", Severity::Low, &[]),
    ("sometimes", "hedges", Severity::Low, &[]),
    ("somewhat", "hedges", Severity::Low, &[]),
    ("suggest", "hedges", Severity::Low, &[]),
    ("suggested", "hedges", Severity::Low, &[]),
    ("suggests", "hedges", Severity::Low, &[]),
    ("suppose", "hedges", Severity::Low, &[]),
    ("supposed", "hedges", Severity::Low, &[]),
    ("suspect", "hedges", Severity::Low, &[]),
    ("tend", "hedges", Severity::Low, &[]),
    ("tended", "hedges", Severity::Low, &[]),
    ("tends", "hedges", Severity::Low, &[]),
    ("typically", "hedges", Severity::Low, &[]),
    ("unclear", "hedges", Severity::Low, &[]),
    ("unlikely", "hedges", Severity::Low, &[]),
    ("usually", "hedges", Severity::Low, &[]),
    ("would", "hedges", Severity::Low, &[]),
];

/// Boosters: markers that close down alternatives and assert certainty.
const BOOSTERS: &[WordRow] = &[
    ("actually", "boosters", Severity::Low, &[]),
    ("always", "boosters", Severity::Low, &[]),
    ("certain", "boosters", Severity::Low, &[]),
    ("certainly", "boosters", Severity::Low, &[]),
    ("clear", "boosters", Severity::Low, &[]),
    ("clearly", "boosters", Severity::Low, &[]),
    ("conclusively", "boosters", Severity::Low, &[]),
    ("decidedly", "boosters", Severity::Low, &[]),
    ("definite", "boosters", Severity::Low, &[]),
    ("definitely", "boosters", Severity::Low, &[]),
    ("demonstrate", "boosters", Severity::Low, &[]),
    ("demonstrates", "boosters", Severity::Low, &[]),
    ("doubtless", "boosters", Severity::Low, &[]),
    ("essential", "boosters", Severity::Low, &[]),
    ("establish", "boosters", Severity::Low, &[]),
    ("established", "boosters", Severity::Low, &[]),
    ("evident", "boosters", Severity::Low, &[]),
    ("evidently", "boosters", Severity::Low, &[]),
    ("indeed", "boosters", Severity::Low, &[]),
    ("indisputable", "boosters", Severity::Low, &[]),
    ("inevitably", "boosters", Severity::Low, &[]),
    ("must", "boosters", Severity::Low, &[]),
    ("never", "boosters", Severity::Low, &[]),
    ("obvious", "boosters", Severity::Low, &[]),
    ("obviously", "boosters", Severity::Low, &[]),
    ("proved", "boosters", Severity::Low, &[]),
    ("proves", "boosters", Severity::Low, &[]),
    ("shows", "boosters", Severity::Low, &[]),
    ("substantially", "boosters", Severity::Low, &[]),
    ("surely", "boosters", Severity::Low, &[]),
    ("truly", "boosters", Severity::Low, &[]),
    ("undeniably", "boosters", Severity::Low, &[]),
    ("undoubtedly", "boosters", Severity::Low, &[]),
    ("well-known", "boosters", Severity::Low, &[]),
];

/// Attitude markers: the writer's affective stance toward a proposition.
const ATTITUDE: &[WordRow] = &[
    ("agree", "attitude", Severity::Low, &[]),
    ("amazing", "attitude", Severity::Low, &[]),
    ("appropriate", "attitude", Severity::Low, &[]),
    ("appropriately", "attitude", Severity::Low, &[]),
    ("astonishing", "attitude", Severity::Low, &[]),
    ("correctly", "attitude", Severity::Low, &[]),
    ("curious", "attitude", Severity::Low, &[]),
    ("curiously", "attitude", Severity::Low, &[]),
    ("desirable", "attitude", Severity::Low, &[]),
    ("disappointing", "attitude", Severity::Low, &[]),
    ("dramatically", "attitude", Severity::Low, &[]),
    ("expected", "attitude", Severity::Low, &[]),
    ("fortunately", "attitude", Severity::Low, &[]),
    ("hopefully", "attitude", Severity::Low, &[]),
    ("important", "attitude", Severity::Low, &[]),
    ("importantly", "attitude", Severity::Low, &[]),
    ("inappropriate", "attitude", Severity::Low, &[]),
    ("interesting", "attitude", Severity::Low, &[]),
    ("interestingly", "attitude", Severity::Low, &[]),
    ("prefer", "attitude", Severity::Low, &[]),
    ("preferred", "attitude", Severity::Low, &[]),
    ("remarkable", "attitude", Severity::Low, &[]),
    ("remarkably", "attitude", Severity::Low, &[]),
    ("shocking", "attitude", Severity::Low, &[]),
    ("striking", "attitude", Severity::Low, &[]),
    ("strikingly", "attitude", Severity::Low, &[]),
    ("surprising", "attitude", Severity::Low, &[]),
    ("surprisingly", "attitude", Severity::Low, &[]),
    ("unbelievable", "attitude", Severity::Low, &[]),
    ("unexpected", "attitude", Severity::Low, &[]),
    ("unfortunate", "attitude", Severity::Low, &[]),
    ("unfortunately", "attitude", Severity::Low, &[]),
    ("unusual", "attitude", Severity::Low, &[]),
    ("unusually", "attitude", Severity::Low, &[]),
];

/// Engagement markers: direct address, directives, shared-knowledge appeals.
///
/// `suppose` is a Hyland engagement marker *and* a Hyland hedge. A word can
/// only carry one dimension here — interning it twice would silently overwrite
/// the first rule and make the category rates stop adding up — so it is filed
/// under hedges, its commoner reading in prose.
const ENGAGEMENT: &[WordRow] = &[
    ("consider", "engagement", Severity::Low, &[]),
    ("imagine", "engagement", Severity::Low, &[]),
    ("notice", "engagement", Severity::Low, &[]),
    ("recall", "engagement", Severity::Low, &[]),
    ("remember", "engagement", Severity::Low, &[]),
    ("note", "engagement", Severity::Low, &[]),
    ("observe", "engagement", Severity::Low, &[]),
    ("determine", "engagement", Severity::Low, &[]),
    ("your", "engagement", Severity::Low, &[]),
    ("yours", "engagement", Severity::Low, &[]),
    ("yourself", "engagement", Severity::Low, &[]),
    ("yourselves", "engagement", Severity::Low, &[]),
];

/// Self-mentions: explicit authorial presence.
const SELF_MENTION: &[WordRow] = &[
    ("i", "self_mention", Severity::Low, &[]),
    ("me", "self_mention", Severity::Low, &[]),
    ("mine", "self_mention", Severity::Low, &[]),
    ("my", "self_mention", Severity::Low, &[]),
    ("myself", "self_mention", Severity::Low, &[]),
    ("our", "self_mention", Severity::Low, &[]),
    ("ours", "self_mention", Severity::Low, &[]),
    ("ourselves", "self_mention", Severity::Low, &[]),
    ("us", "self_mention", Severity::Low, &[]),
    ("we", "self_mention", Severity::Low, &[]),
];

/// Transitions: the interactive markers that mark logical relations.
const TRANSITIONS: &[WordRow] = &[
    ("accordingly", "transitions", Severity::Low, &[]),
    ("additionally", "transitions", Severity::Low, &[]),
    ("also", "transitions", Severity::Low, &[]),
    ("alternatively", "transitions", Severity::Low, &[]),
    ("although", "transitions", Severity::Low, &[]),
    ("besides", "transitions", Severity::Low, &[]),
    ("consequently", "transitions", Severity::Low, &[]),
    ("conversely", "transitions", Severity::Low, &[]),
    ("equally", "transitions", Severity::Low, &[]),
    ("furthermore", "transitions", Severity::Low, &[]),
    ("hence", "transitions", Severity::Low, &[]),
    ("however", "transitions", Severity::Low, &[]),
    ("likewise", "transitions", Severity::Low, &[]),
    ("moreover", "transitions", Severity::Low, &[]),
    ("nevertheless", "transitions", Severity::Low, &[]),
    ("nonetheless", "transitions", Severity::Low, &[]),
    ("similarly", "transitions", Severity::Low, &[]),
    ("therefore", "transitions", Severity::Low, &[]),
    ("thereby", "transitions", Severity::Low, &[]),
    ("though", "transitions", Severity::Low, &[]),
    ("thus", "transitions", Severity::Low, &[]),
    ("whereas", "transitions", Severity::Low, &[]),
    ("while", "transitions", Severity::Low, &[]),
];

/// Frame markers: sequencing, staging, topic shifts, goal announcements.
const FRAME_MARKERS: &[WordRow] = &[
    ("finally", "frame_markers", Severity::Low, &[]),
    ("first", "frame_markers", Severity::Low, &[]),
    ("firstly", "frame_markers", Severity::Low, &[]),
    ("lastly", "frame_markers", Severity::Low, &[]),
    ("next", "frame_markers", Severity::Low, &[]),
    ("second", "frame_markers", Severity::Low, &[]),
    ("secondly", "frame_markers", Severity::Low, &[]),
    ("subsequently", "frame_markers", Severity::Low, &[]),
    ("third", "frame_markers", Severity::Low, &[]),
    ("thirdly", "frame_markers", Severity::Low, &[]),
];

/// Endophorics: pointers to other parts of the same text.
const ENDOPHORICS: &[WordRow] = &[
    ("above", "endophorics", Severity::Low, &[]),
    ("below", "endophorics", Severity::Low, &[]),
    ("earlier", "endophorics", Severity::Low, &[]),
    ("later", "endophorics", Severity::Low, &[]),
    ("aforementioned", "endophorics", Severity::Low, &[]),
    ("appendix", "endophorics", Severity::Low, &[]),
    ("figure", "endophorics", Severity::Low, &[]),
    ("table", "endophorics", Severity::Low, &[]),
    ("section", "endophorics", Severity::Low, &[]),
];

/// Evidentials: attributions of a claim to a source outside the text.
const EVIDENTIALS: &[WordRow] = &[
    ("according", "evidentials", Severity::Low, &[]),
    ("cited", "evidentials", Severity::Low, &[]),
    ("citing", "evidentials", Severity::Low, &[]),
    ("quoted", "evidentials", Severity::Low, &[]),
    ("reported", "evidentials", Severity::Low, &[]),
    ("reportedly", "evidentials", Severity::Low, &[]),
    ("allegedly", "evidentials", Severity::Low, &[]),
];

/// Code glosses: reformulations and exemplifications.
const CODE_GLOSSES: &[WordRow] = &[
    ("eg", "code_glosses", Severity::Low, &[]),
    ("ie", "code_glosses", Severity::Low, &[]),
    ("namely", "code_glosses", Severity::Low, &[]),
    ("specifically", "code_glosses", Severity::Low, &[]),
    ("viz", "code_glosses", Severity::Low, &[]),
];

/// The multi-token members of the same categories.
const HYLAND_PHRASES: &[PhraseRow] = &[
    // Hedges.
    (
        "hyland.in_most_cases",
        "in most cases",
        "hedges",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_general",
        "in general",
        "hedges",
        Severity::Low,
        "",
    ),
    (
        "hyland.to_some_extent",
        "to {some|a} {extent|degree}",
        "hedges",
        Severity::Low,
        "",
    ),
    (
        "hyland.more_or_less",
        "more or less",
        "hedges",
        Severity::Low,
        "",
    ),
    (
        "hyland.it_appears",
        "it {appears|seems} that",
        "hedges",
        Severity::Low,
        "",
    ),
    (
        "hyland.tend_to",
        "{tend|tends|tended} to",
        "hedges",
        Severity::Low,
        "",
    ),
    ("hyland.as_far_as", "as far as", "hedges", Severity::Low, ""),
    // Boosters.
    (
        "hyland.of_course",
        "of course",
        "boosters",
        Severity::Low,
        "",
    ),
    ("hyland.in_fact", "in fact", "boosters", Severity::Low, ""),
    (
        "hyland.without_doubt",
        "without {doubt|question}",
        "boosters",
        Severity::Low,
        "",
    ),
    (
        "hyland.beyond_doubt",
        "beyond {doubt|question}",
        "boosters",
        Severity::Low,
        "",
    ),
    (
        "hyland.it_is_clear",
        "it is {clear|evident|obvious} that",
        "boosters",
        Severity::Low,
        "",
    ),
    (
        "hyland.no_doubt",
        "there is no doubt",
        "boosters",
        Severity::Low,
        "",
    ),
    // Attitude.
    (
        "hyland.it_is_important",
        "it is {important|essential|crucial} to",
        "attitude",
        Severity::Low,
        "",
    ),
    (
        "hyland.i_agree",
        "{i|we} agree",
        "attitude",
        Severity::Low,
        "",
    ),
    // Engagement.
    (
        "hyland.note_that",
        "note that",
        "engagement",
        Severity::Low,
        "",
    ),
    (
        "hyland.consider_the",
        "consider the",
        "engagement",
        Severity::Low,
        "",
    ),
    (
        "hyland.you_can_see",
        "{you|we} can see",
        "engagement",
        Severity::Low,
        "",
    ),
    (
        "hyland.lets",
        "{let|lets|let's} us",
        "engagement",
        Severity::Low,
        "",
    ),
    (
        "hyland.as_you_know",
        "as you {know|see}",
        "engagement",
        Severity::Low,
        "",
    ),
    // Self-mention.
    (
        "hyland.in_my_view",
        "in {my|our} {view|opinion|experience}",
        "self_mention",
        Severity::Low,
        "",
    ),
    (
        "hyland.the_author",
        "the {author|authors}",
        "self_mention",
        Severity::Low,
        "",
    ),
    // Transitions.
    (
        "hyland.on_the_other_hand",
        "on the other hand",
        "transitions",
        Severity::Low,
        "",
    ),
    (
        "hyland.as_a_result",
        "as a result",
        "transitions",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_addition",
        "in addition",
        "transitions",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_contrast",
        "in contrast",
        "transitions",
        Severity::Low,
        "",
    ),
    (
        "hyland.by_contrast",
        "by contrast",
        "transitions",
        Severity::Low,
        "",
    ),
    (
        "hyland.even_though",
        "even though",
        "transitions",
        Severity::Low,
        "",
    ),
    // Frame markers.
    (
        "hyland.to_begin",
        "to {begin|start} with",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_conclusion",
        "in conclusion",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.to_conclude",
        "to conclude",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.my_purpose",
        "{my|our} {purpose|aim|goal} is",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.turn_to",
        "{turn|turning} to",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.so_far",
        "so far",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_short",
        "in short",
        "frame_markers",
        Severity::Low,
        "",
    ),
    (
        "hyland.to_summarize",
        "to {summarize|summarise|sum} up",
        "frame_markers",
        Severity::Low,
        "",
    ),
    // Endophorics.
    (
        "hyland.see_figure",
        "see {figure|table|section|appendix} *",
        "endophorics",
        Severity::Low,
        "",
    ),
    (
        "hyland.as_noted",
        "as {noted|shown|discussed} {above|below|earlier}",
        "endophorics",
        Severity::Low,
        "",
    ),
    (
        "hyland.the_next_section",
        "the {next|previous|following} section",
        "endophorics",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_section",
        "in {section|chapter} *",
        "endophorics",
        Severity::Low,
        "",
    ),
    // Evidentials.
    (
        "hyland.according_to",
        "according to",
        "evidentials",
        Severity::Low,
        "",
    ),
    (
        "hyland.cited_in",
        "{cited|quoted} in",
        "evidentials",
        Severity::Low,
        "",
    ),
    ("hyland.et_al", "et al", "evidentials", Severity::Low, ""),
    // Code glosses.
    (
        "hyland.that_is",
        "that is to say",
        "code_glosses",
        Severity::Low,
        "",
    ),
    (
        "hyland.in_other_words",
        "in other words",
        "code_glosses",
        Severity::Low,
        "",
    ),
    (
        "hyland.for_example",
        "for {example|instance}",
        "code_glosses",
        Severity::Low,
        "",
    ),
    (
        "hyland.such_as",
        "such as",
        "code_glosses",
        Severity::Low,
        "",
    ),
    (
        "hyland.which_means",
        "which means that",
        "code_glosses",
        Severity::Low,
        "",
    ),
    (
        "hyland.known_as",
        "known as",
        "code_glosses",
        Severity::Low,
        "",
    ),
];

/// Build the pack.
pub fn pack() -> LexiconPack {
    let rows: Vec<&[WordRow]> = vec![
        HEDGES,
        BOOSTERS,
        ATTITUDE,
        ENGAGEMENT,
        SELF_MENTION,
        TRANSITIONS,
        FRAME_MARKERS,
        ENDOPHORICS,
        EVIDENTIALS,
        CODE_GLOSSES,
    ];
    let all: Vec<WordRow> = rows.into_iter().flatten().copied().collect();
    LexiconPack {
        name: "hyland".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Hyland (2005) metadiscourse taxonomy: interactive (transitions, frame \
                      markers, endophorics, evidentials, code glosses) and interactional \
                      (hedges, boosters, attitude, engagement, self-mention) markers. Fit with \
                      category_findings; the category rates are the usable dimensions."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            super::source(
                "Hyland 2005, Metadiscourse",
                "https://www.researchgate.net/figure/Hylands-2005-taxonomy-of-metadiscourse_tbl1_322164943",
                "category definitions re-curated from the published taxonomy tables; the \
                 appendix compilation itself is not reproduced",
            ),
            source(
                "Hedging gap in LLM advice text",
                "https://arxiv.org/pdf/2604.22143",
                "humans hedge roughly 40% more than LLMs, which makes the hedge and \
                 booster:hedge dimensions de-AI features as well as register features",
            ),
        ],
        terms: terms(&all),
        phrases: phrases(HYLAND_PHRASES),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ten_hyland_categories_are_present() {
        let pack = pack();
        let categories = pack.categories();
        for expected in [
            "attitude",
            "boosters",
            "code_glosses",
            "endophorics",
            "engagement",
            "evidentials",
            "frame_markers",
            "hedges",
            "self_mention",
            "transitions",
        ] {
            assert!(
                categories.contains(&expected.to_string()),
                "missing category {expected}"
            );
        }
        assert_eq!(categories.len(), 10);
    }

    #[test]
    fn the_pack_is_the_size_the_taxonomy_implies() {
        let pack = pack();
        let items = pack.terms.len() + pack.phrases.len();
        assert!(
            (250..=600).contains(&items),
            "expected the taxonomy's few hundred items, got {items}"
        );
    }

    #[test]
    fn no_word_appears_in_two_categories() {
        // A term interned twice would silently overwrite the first rule's
        // dimension, so the category rates would stop adding up.
        let pack = pack();
        let mut words: Vec<&str> = pack.terms.iter().map(|t| t.word.as_str()).collect();
        let before = words.len();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), before, "a word is in two Hyland categories");
    }
}
