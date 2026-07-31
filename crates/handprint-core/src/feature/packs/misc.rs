//! Small single-purpose packs: hyperbole, Wikipedia AI signs, gonzo epistemic
//! certainty, drug lexicon.
//!
//! Each is a few dozen items and would be lost as a section of a larger file.
//! The Wikipedia derivation in particular has to be its own pack: it is
//! CC-BY-SA, and a share-alike obligation must not reach a sibling list that
//! merely happens to be compiled into the same binary.

use super::{phrases, source, terms, PhraseRow, WordRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of these packs' contents.
pub const VERSION: &str = "2026.08";

// ---------------------------------------------------------------------------
// Hyperbole
// ---------------------------------------------------------------------------

/// Extreme-degree words, after the Troiano & Strapparava hyperbole features.
#[rustfmt::skip]
const HYPERBOLE_WORDS: &[WordRow] = &[
    ("infinite", "extreme_degree", Severity::Low, &[]),
    ("infinitely", "extreme_degree", Severity::Low, &[]),
    ("endless", "extreme_degree", Severity::Low, &[]),
    ("endlessly", "extreme_degree", Severity::Low, &[]),
    ("eternal", "extreme_degree", Severity::Low, &[]),
    ("eternally", "extreme_degree", Severity::Low, &[]),
    ("forever", "extreme_degree", Severity::Low, &[]),
    ("unbearable", "extreme_degree", Severity::Low, &[]),
    ("unbearably", "extreme_degree", Severity::Low, &[]),
    ("unthinkable", "extreme_degree", Severity::Low, &[]),
    ("unimaginable", "extreme_degree", Severity::Low, &[]),
    ("indescribable", "extreme_degree", Severity::Low, &[]),
    ("astronomical", "extreme_degree", Severity::Low, &[]),
    ("monumental", "extreme_degree", Severity::Low, &[]),
    ("colossal", "extreme_degree", Severity::Low, &[]),
    ("gargantuan", "extreme_degree", Severity::Low, &[]),
    ("titanic", "extreme_degree", Severity::Low, &[]),
    ("cataclysmic", "extreme_degree", Severity::Low, &[]),
    ("apocalyptic", "extreme_degree", Severity::Low, &[]),
    ("catastrophic", "extreme_degree", Severity::Low, &[]),
    ("devastating", "extreme_degree", Severity::Low, &[]),
    ("annihilated", "extreme_degree", Severity::Low, &[]),
    ("obliterated", "extreme_degree", Severity::Low, &[]),
    ("starving", "extreme_degree", Severity::Low, &[]),
    ("freezing", "extreme_degree", Severity::Low, &[]),
    ("boiling", "extreme_degree", Severity::Low, &[]),
    ("exhausted", "extreme_degree", Severity::Low, &[]),
    ("terrified", "extreme_degree", Severity::Low, &[]),
    ("mortified", "extreme_degree", Severity::Low, &[]),
    ("ecstatic", "extreme_degree", Severity::Low, &[]),
    ("flawless", "extreme_degree", Severity::Low, &[]),
    ("perfect", "extreme_degree", Severity::Low, &[]),
    ("impossible", "extreme_degree", Severity::Low, &[]),
    ("insane", "extreme_degree", Severity::Low, &[]),
    ("ridiculous", "extreme_degree", Severity::Low, &[]),
    ("absurd", "extreme_degree", Severity::Low, &[]),
];

#[rustfmt::skip]
const HYPERBOLE_PHRASES: &[PhraseRow] = &[
    ("hyp.a_million_times", "a {million|billion|thousand} times", "extreme_quantity", Severity::Low, ""),
    ("hyp.the_worst_ever", "the {worst|best} * ever", "extreme_quantity", Severity::Low, ""),
    ("hyp.in_the_world", "in the {world|universe|history}", "extreme_quantity", Severity::Low, ""),
    ("hyp.of_all_time", "of all time", "extreme_quantity", Severity::Low, ""),
    ("hyp.forever_and_ever", "forever and ever", "extreme_quantity", Severity::Low, ""),
    ("hyp.i_could_die", "i could {die|scream}", "extreme_reaction", Severity::Low, ""),
    ("hyp.never_in_my_life", "never in my life", "extreme_reaction", Severity::Low, ""),
    ("hyp.not_in_a_million", "not in a million years", "extreme_reaction", Severity::Low, ""),
    ("hyp.i_nearly_died", "i {nearly|almost} died", "extreme_reaction", Severity::Low, ""),
    ("hyp.blew_my_mind", "blew my mind", "extreme_reaction", Severity::Low, ""),
    ("hyp.took_forever", "took forever", "extreme_reaction", Severity::Low, ""),
    ("hyp.the_end_of_the_world", "the end of the world", "extreme_reaction", Severity::Low, ""),
];

/// Hyperbole markers.
pub fn hyperbole_pack() -> LexiconPack {
    LexiconPack {
        name: "hyperbole".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Hyperbole markers: extreme-degree adjectives, extreme quantities, \
                      extreme reactions. Re-curated from the published feature description."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![source(
            "Troiano & Strapparava, EMNLP 2018",
            "https://aclanthology.org/D18-1367/",
            "hyperbole feature families, re-curated from the published description",
        )],
        terms: terms(HYPERBOLE_WORDS),
        phrases: phrases(HYPERBOLE_PHRASES),
        privacy: None,
    }
}

// ---------------------------------------------------------------------------
// Wikipedia signs of AI writing — CC-BY-SA, isolated
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const WIKI_PHRASES: &[PhraseRow] = &[
    ("wiki.stands_as_a", "stands as a {testament|symbol|reminder}", "inflation", Severity::High, ""),
    ("wiki.plays_a_role", "plays a {vital|crucial|significant|key} role", "inflation", Severity::High, ""),
    ("wiki.continues_to_captivate", "continues to {captivate|inspire|shape}", "inflation", Severity::High, ""),
    ("wiki.rich_cultural", "rich {cultural|historical} {heritage|tapestry}", "inflation", Severity::High, ""),
    ("wiki.enduring_legacy", "enduring {legacy|appeal|impact}", "inflation", Severity::High, ""),
    ("wiki.left_an_indelible", "left an indelible mark", "inflation", Severity::High, ""),
    ("wiki.serves_as_a", "serves as a {reminder|testament|symbol}", "copula_avoidance", Severity::High, ""),
    ("wiki.is_known_for_its", "is known for its", "copula_avoidance", Severity::Medium, ""),
    ("wiki.boasts_a", "boasts {a|an}", "copula_avoidance", Severity::High, "use \"has\""),
    ("wiki.it_is_important_to", "it is {important|worth} {to|noting}", "editorializing", Severity::Medium, ""),
    ("wiki.notably_the", "notably the", "editorializing", Severity::Low, ""),
    ("wiki.in_the_realm_of", "in the {realm|world} of", "editorializing", Severity::High, ""),
    ("wiki.a_wide_array", "a {wide|broad} {array|range|variety} of", "editorializing", Severity::Low, ""),
    ("wiki.experts_believe", "{experts|scholars|historians} {believe|argue|note}", "vague_attribution", Severity::High, "name them"),
    ("wiki.some_have_argued", "some have {argued|suggested|claimed}", "vague_attribution", Severity::High, "name them"),
    ("wiki.it_is_widely", "it is widely {believed|regarded|considered}", "vague_attribution", Severity::High, ""),
    ("wiki.in_summary", "in {summary|conclusion}", "formulaic_close", Severity::High, ""),
    ("wiki.overall_the", "overall the", "formulaic_close", Severity::Medium, ""),
    ("wiki.ultimately_the", "ultimately the", "formulaic_close", Severity::Medium, ""),
];

/// The Wikipedia signs-of-AI derivation.
///
/// CC-BY-SA, which is why it is its own pack file: the share-alike obligation
/// attaches to this list and must not reach a sibling.
pub fn wiki_ai_pack() -> LexiconPack {
    LexiconPack {
        name: "wiki-ai-signs".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Derived from Wikipedia's Signs of AI writing guide. CC-BY-SA: this pack \
                      is share-alike and is kept in its own file so the obligation cannot \
                      reach a sibling pack."
            .into(),
        license: "CC-BY-SA-4.0".into(),
        redistributable: true,
        sources: vec![source(
            "Wikipedia: Signs of AI writing",
            "https://en.wikipedia.org/wiki/Wikipedia:WikiProject_AI_Cleanup/Signs_of_AI_writing",
            "derived work under CC-BY-SA-4.0, with attribution and share-alike",
        )],
        terms: Vec::new(),
        phrases: phrases(WIKI_PHRASES),
        privacy: None,
    }
}

// ---------------------------------------------------------------------------
// Gonzo epistemic certainty
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const EPISTEMIC_PHRASES: &[PhraseRow] = &[
    ("ep.of_course", "of course", "paranoid_certainty", Severity::Low, ""),
    ("ep.no_doubt", "{no|without} doubt", "paranoid_certainty", Severity::Low, ""),
    ("ep.sure_as_hell", "sure as hell", "paranoid_certainty", Severity::Low, ""),
    ("ep.you_can_bet", "you can bet", "paranoid_certainty", Severity::Low, ""),
    ("ep.make_no_mistake", "make no mistake", "paranoid_certainty", Severity::Low, ""),
    ("ep.mark_my_words", "mark my words", "paranoid_certainty", Severity::Low, ""),
    ("ep.the_truth_is", "the truth is", "paranoid_certainty", Severity::Low, ""),
    ("ep.let_me_tell_you", "let me tell you", "paranoid_certainty", Severity::Low, ""),
    ("ep.i_knew_it", "i knew it", "paranoid_certainty", Severity::Low, ""),
    ("ep.obviously", "obviously the", "paranoid_certainty", Severity::Low, ""),
    ("ep.clearly_the", "clearly the", "paranoid_certainty", Severity::Low, ""),
    ("ep.beyond_question", "beyond {question|dispute}", "paranoid_certainty", Severity::Low, ""),
    ("ep.anyone_could_see", "anyone could see", "paranoid_certainty", Severity::Low, ""),
    ("ep.every_last_one", "every last one", "paranoid_certainty", Severity::Low, ""),
    ("ep.nothing_less_than", "nothing less than", "paranoid_certainty", Severity::Low, ""),
    ("ep.the_whole_thing_was", "the whole thing was", "paranoid_certainty", Severity::Low, ""),
    ("ep.they_knew", "they {knew|know}", "conspiracy", Severity::Low, ""),
    ("ep.somebody_somewhere", "somebody somewhere", "conspiracy", Severity::Low, ""),
    ("ep.the_bastards", "the {bastards|swine|fixers}", "conspiracy", Severity::Low, ""),
    ("ep.what_they_want", "what they want", "conspiracy", Severity::Low, ""),
    ("ep.it_was_all", "it was all a", "conspiracy", Severity::Low, ""),
    ("ep.only_a_matter_of_time", "only a matter of time", "conspiracy", Severity::Low, ""),
    ("ep.god_only_knows", "god only knows", "conspiracy", Severity::Low, ""),
    ("ep.jesus_christ", "jesus christ", "conspiracy", Severity::Low, ""),
    ("ep.no_way_out", "no way out", "conspiracy", Severity::Low, ""),
    ("ep.doomed_from_the_start", "doomed from the start", "conspiracy", Severity::Low, ""),
    ("ep.a_bad_craziness", "bad craziness", "conspiracy", Severity::Low, ""),
    ("ep.too_weird_to_live", "too weird to live", "conspiracy", Severity::Low, ""),
    ("ep.we_were_somewhere", "we were somewhere around", "conspiracy", Severity::Low, ""),
    ("ep.there_was_madness", "there was madness", "conspiracy", Severity::Low, ""),
];

/// Gonzo paranoid-certainty markers.
pub fn epistemic_pack() -> LexiconPack {
    LexiconPack {
        name: "epistemic-certainty".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Paranoid-certainty and conspiracy-frame phrases, the gonzo epistemic \
                      register. The inverse of a hedge: assertion without evidence, offered \
                      as though evidence were beneath discussion."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![source(
            "Original curation",
            "https://github.com/akiselev/handprint",
            "compiled against the gonzo-journalism stylistics literature (Mosser; Wills)",
        )],
        terms: Vec::new(),
        phrases: phrases(EPISTEMIC_PHRASES),
        privacy: None,
    }
}

// ---------------------------------------------------------------------------
// Drug lexicon
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const DRUG_WORDS: &[WordRow] = &[
    ("mescaline", "substance", Severity::Low, &[]),
    ("ether", "substance", Severity::Low, &[]),
    ("adrenochrome", "substance", Severity::Low, &[]),
    ("amyl", "substance", Severity::Low, &[]),
    ("amphetamine", "substance", Severity::Low, &[]),
    ("amphetamines", "substance", Severity::Low, &[]),
    ("barbiturate", "substance", Severity::Low, &[]),
    ("barbiturates", "substance", Severity::Low, &[]),
    ("benzedrine", "substance", Severity::Low, &[]),
    ("cocaine", "substance", Severity::Low, &[]),
    ("codeine", "substance", Severity::Low, &[]),
    ("dexedrine", "substance", Severity::Low, &[]),
    ("hashish", "substance", Severity::Low, &[]),
    ("heroin", "substance", Severity::Low, &[]),
    ("laudanum", "substance", Severity::Low, &[]),
    ("marijuana", "substance", Severity::Low, &[]),
    ("mezcal", "substance", Severity::Low, &[]),
    ("morphine", "substance", Severity::Low, &[]),
    ("opium", "substance", Severity::Low, &[]),
    ("peyote", "substance", Severity::Low, &[]),
    ("quaalude", "substance", Severity::Low, &[]),
    ("quaaludes", "substance", Severity::Low, &[]),
    ("tequila", "substance", Severity::Low, &[]),
    ("whiskey", "substance", Severity::Low, &[]),
    ("whisky", "substance", Severity::Low, &[]),
    ("bourbon", "substance", Severity::Low, &[]),
    ("gin", "substance", Severity::Low, &[]),
    ("rum", "substance", Severity::Low, &[]),
    ("booze", "substance", Severity::Low, &[]),
    ("acid", "substance", Severity::Low, &[]),
    ("hallucination", "effect", Severity::Low, &[]),
    ("hallucinations", "effect", Severity::Low, &[]),
    ("hallucinating", "effect", Severity::Low, &[]),
    ("stoned", "effect", Severity::Low, &[]),
    ("wasted", "effect", Severity::Low, &[]),
    ("hammered", "effect", Severity::Low, &[]),
    ("blitzed", "effect", Severity::Low, &[]),
    ("delirium", "effect", Severity::Low, &[]),
    ("delirious", "effect", Severity::Low, &[]),
    ("paranoia", "effect", Severity::Low, &[]),
    ("paranoid", "effect", Severity::Low, &[]),
    ("trembling", "effect", Severity::Low, &[]),
    ("sweating", "effect", Severity::Low, &[]),
    ("hangover", "effect", Severity::Low, &[]),
    ("withdrawal", "effect", Severity::Low, &[]),
    ("overdose", "effect", Severity::Low, &[]),
    ("binge", "effect", Severity::Low, &[]),
    ("bender", "effect", Severity::Low, &[]),
];

/// A small drug-and-intoxication lexicon.
///
/// A register marker, not a content warning: the rate separates gonzo and
/// dirty-realism prose from every other register in the battery, which is the
/// only reason it is here.
pub fn drug_pack() -> LexiconPack {
    LexiconPack {
        name: "drug-lexicon".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Intoxicants and intoxication effects. A register marker: the rate \
                      separates gonzo and dirty-realism prose from the rest of the battery."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![source(
            "Original curation",
            "https://github.com/akiselev/handprint",
            "compiled for the gonzo register axis",
        )],
        terms: terms(DRUG_WORDS),
        phrases: Vec::new(),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pack_here_validates_its_patterns() {
        for pack in [
            hyperbole_pack(),
            wiki_ai_pack(),
            epistemic_pack(),
            drug_pack(),
        ] {
            pack.validate_patterns()
                .unwrap_or_else(|e| panic!("{}: {e}", pack.name));
        }
    }

    #[test]
    fn the_wikipedia_derivation_declares_share_alike() {
        let pack = wiki_ai_pack();
        assert!(pack.license.contains("CC-BY-SA"));
        assert!(pack.sources[0].url.contains("wikipedia.org"));
    }

    #[test]
    fn the_other_packs_do_not_inherit_share_alike() {
        for pack in [hyperbole_pack(), epistemic_pack(), drug_pack()] {
            assert!(
                !pack.license.contains("SA"),
                "{} inherited share-alike",
                pack.name
            );
        }
    }

    #[test]
    fn the_epistemic_pack_is_about_the_size_the_plan_called_for() {
        let pack = epistemic_pack();
        assert!(
            (25..=60).contains(&pack.phrases.len()),
            "{}",
            pack.phrases.len()
        );
    }
}
