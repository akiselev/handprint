//! Tech-blogger voice markers — the phrase-shaped half.
//!
//! Commentary on Levine, Graham, Luu, Evans, Gwern and patio11 keeps naming the
//! same habits, and most of them reduce to counts. This pack holds the ones that
//! are *phrases*; the ones that are structure (link density, footnote markers,
//! rhetorical questions, parenthetical asides) are code-side dimensions in the
//! sentence family, because they need token and block context that a lexicon
//! lookup does not have.
//!
//! Categories:
//!
//! * `pivot` — discourse-pivot openers ("Anyway,", "OK, so", "Look,", "Here's
//!   the thing"). The move from one thread of an argument to another, out loud.
//! * `hypothetical` — hypothetical-dialogue frames ("you might say", "you might
//!   be wondering"). The writer arguing with an imagined reader.
//! * `analogy` — analogy frames ("it's like", "think of X as", "imagine").
//! * `self_deprecation` — the hedged-competence move ("I'm probably wrong here",
//!   "caveat: I am not a lawyer").
//! * `concession` — the explicit-uncertainty move ("to be fair", "in fairness").
//!
//! Original curation. The items are ordinary English phrases; what is new here
//! is the claim that their *rates* characterize a register, which is the same
//! claim the rest of this crate makes about punctuation.

use super::{phrases, source, PhraseRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of this pack's contents.
pub const VERSION: &str = "2026.08";

#[rustfmt::skip]
const TECH_VOICE_PHRASES: &[PhraseRow] = &[
    // Discourse pivots.
    ("tech.anyway", "anyway", "pivot", Severity::Low, ""),
    ("tech.anyhow", "anyhow", "pivot", Severity::Low, ""),
    ("tech.ok_so", "{ok|okay} so", "pivot", Severity::Low, ""),
    (
        "tech.alright_so",
        "{alright|all} right so",
        "pivot",
        Severity::Low,
        "",
    ),
    (
        "tech.heres_the_thing",
        "{here's|heres} the thing",
        "pivot",
        Severity::Low,
        "",
    ),
    (
        "tech.here_is_the_thing",
        "here is the thing",
        "pivot",
        Severity::Low,
        "",
    ),
    (
        "tech.the_thing_is",
        "the thing is",
        "pivot",
        Severity::Low,
        "",
    ),
    ("tech.but_wait", "but wait", "pivot", Severity::Low, ""),
    ("tech.back_up", "let me back up", "pivot", Severity::Low, ""),
    ("tech.side_note", "side note", "pivot", Severity::Low, ""),
    (
        "tech.as_an_aside",
        "as an aside",
        "pivot",
        Severity::Low,
        "",
    ),
    (
        "tech.digression",
        "brief digression",
        "pivot",
        Severity::Low,
        "",
    ),
    ("tech.moving_on", "moving on", "pivot", Severity::Low, ""),
    (
        "tech.that_aside",
        "{that|this} aside",
        "pivot",
        Severity::Low,
        "",
    ),
    ("tech.so_anyway", "so anyway", "pivot", Severity::Low, ""),
    (
        "tech.long_story_short",
        "long story short",
        "pivot",
        Severity::Low,
        "",
    ),
    ("tech.tldr", "{tldr|tl}", "pivot", Severity::Low, ""),
    // Hypothetical dialogue.
    (
        "tech.you_might_say",
        "you {might|may|could} say",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.you_might_be_wondering",
        "you {might|may} be {wondering|thinking|asking}",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.you_might_think",
        "you {might|may|would} think",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.someone_will_say",
        "{someone|somebody} will {say|object|ask}",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.the_obvious_objection",
        "the obvious {objection|response|question}",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.but_surely",
        "but surely",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.i_hear_you",
        "i {hear|see} you",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.wait_you_say",
        "wait you say",
        "hypothetical",
        Severity::Low,
        "",
    ),
    (
        "tech.why_not_just",
        "why not just",
        "hypothetical",
        Severity::Low,
        "",
    ),
    // Analogy frames.
    (
        "tech.its_like",
        "{it's|its} {like|basically}",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.sort_of_like",
        "{sort|kind} of like",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.think_of_as",
        "think of * as",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.imagine_that",
        "imagine {that|a|an|the}",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.the_analogy",
        "the analogy {is|here}",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.picture_a",
        "picture {a|an|the}",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.in_the_same_way",
        "in the same way",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.the_equivalent_of",
        "the equivalent of",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.roughly_analogous",
        "roughly analogous to",
        "analogy",
        Severity::Low,
        "",
    ),
    (
        "tech.is_basically",
        "is basically",
        "analogy",
        Severity::Low,
        "",
    ),
    // Self-deprecation and competence hedging.
    (
        "tech.im_probably_wrong",
        "{i'm|im} probably wrong",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.i_am_probably_wrong",
        "i am probably wrong",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.i_am_not_a",
        "{i'm|im} not {a|an}",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.i_could_be_wrong",
        "i {could|might|may} be wrong",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.no_expert",
        "{i'm|im} no expert",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.take_this_with",
        "take this with {a|the}",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.your_mileage",
        "your mileage may vary",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    (
        "tech.caveat",
        "the usual {caveat|caveats}",
        "self_deprecation",
        Severity::Low,
        "",
    ),
    // Concession.
    (
        "tech.to_be_fair",
        "to be fair",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.in_fairness",
        "in {fairness|truth}",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.to_be_clear",
        "to be clear",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.that_said",
        "that {said|admitted}",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.that_being_said",
        "that being said",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.granted",
        "granted that",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.admittedly",
        "admittedly",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.to_be_honest",
        "to be honest",
        "concession",
        Severity::Low,
        "",
    ),
    (
        "tech.on_reflection",
        "on {reflection|balance}",
        "concession",
        Severity::Low,
        "",
    ),
];

/// Build the pack.
pub fn pack() -> LexiconPack {
    LexiconPack {
        name: "tech-voice".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Tech-blogger voice markers: discourse-pivot openers, hypothetical-dialogue \
                      frames, analogy frames, competence hedging, concession. Fit with \
                      category_findings; the structural half (links, footnotes, rhetorical \
                      questions, asides) lives in the sentence family."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "Original curation",
                "https://github.com/akiselev/handprint",
                "phrase-shaped habits named in commentary on Levine, Graham, Luu, Evans, \
                 Gwern and patio11; the rate claim is this project's",
            ),
            source(
                "Writing Like the Best",
                "https://arxiv.org/pdf/2505.18859",
                "structural/rhetorical similarity beats topical similarity when selecting \
                 exemplars — the reason move-level markers are worth counting separately",
            ),
        ],
        terms: Vec::new(),
        phrases: phrases(TECH_VOICE_PHRASES),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_five_categories_are_present() {
        let mut categories = pack().categories();
        categories.sort();
        assert_eq!(
            categories,
            vec![
                "analogy",
                "concession",
                "hypothetical",
                "pivot",
                "self_deprecation"
            ]
        );
    }

    #[test]
    fn the_pack_is_phrases_only() {
        // Single-word tech-voice markers would collide with the Hyland and
        // doc-style packs, which already own the word-level layer.
        assert!(pack().terms.is_empty());
        assert!(pack().phrases.len() > 30);
    }
}
