//! Documentation style-guide rules, mined from Vale's MIT-licensed packs.
//!
//! Vale's style packs (Microsoft Writing Style Guide, Google developer
//! documentation style guide, `write-good`, `proselint`) are MIT-licensed YAML:
//! regex substitutions and word lists, no code to speak of. The substitution
//! rules map exactly onto [`Term::alternatives`](crate::feature::Term), which
//! the critique layer already renders as a `consider_replace` fix — so the
//! translation costs nothing and the fix comes out the far end for free.
//!
//! Note what the *packs themselves* say about register: Microsoft's guide
//! actively prefers contractions. A style guide is a register description as
//! much as a rulebook, which is why these dimensions belong in a stylometer and
//! not only in a linter.
//!
//! Categories:
//!
//! * `weasel` — vague intensifiers with no referent ("very", "several").
//! * `wordy` — long forms with a short equivalent ("utilize" → "use").
//! * `cliche` — dead metaphors.
//! * `opener` — sentence-initial "So", the documented conversational tic.
//! * `jargon` — corporate abstractions.

use super::{phrases, source, terms, PhraseRow, WordRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of this pack's contents.
pub const VERSION: &str = "2026.08";

const WEASEL: &[WordRow] = &[
    ("very", "weasel", Severity::Low, &["(cut it)"]),
    ("really", "weasel", Severity::Low, &["(cut it)"]),
    ("quite", "weasel", Severity::Low, &["(cut it)"]),
    ("rather", "weasel", Severity::Low, &["(cut it)"]),
    ("somewhat", "weasel", Severity::Low, &["(cut it)"]),
    ("fairly", "weasel", Severity::Low, &["(cut it)"]),
    ("extremely", "weasel", Severity::Low, &["(cut it)"]),
    ("exceedingly", "weasel", Severity::Low, &["(cut it)"]),
    ("vastly", "weasel", Severity::Low, &["(cut it)"]),
    ("relatively", "weasel", Severity::Low, &["(cut it)"]),
    ("remarkably", "weasel", Severity::Low, &["(cut it)"]),
    ("surprisingly", "weasel", Severity::Low, &["(cut it)"]),
    ("several", "weasel", Severity::Low, &["say how many"]),
    ("various", "weasel", Severity::Low, &["say which"]),
    ("numerous", "weasel", Severity::Low, &["say how many"]),
    ("many", "weasel", Severity::Low, &["say how many"]),
    ("most", "weasel", Severity::Low, &["say what share"]),
    ("simply", "weasel", Severity::Medium, &["(cut it)"]),
    ("easily", "weasel", Severity::Medium, &["(cut it)"]),
    ("obviously", "weasel", Severity::Medium, &["(cut it)"]),
    ("clearly", "weasel", Severity::Medium, &["(cut it)"]),
    ("basically", "weasel", Severity::Low, &["(cut it)"]),
    ("literally", "weasel", Severity::Medium, &["(cut it)"]),
    ("interestingly", "weasel", Severity::Low, &["(cut it)"]),
    ("importantly", "weasel", Severity::Low, &["(cut it)"]),
];

const WORDY: &[WordRow] = &[
    ("utilize", "wordy", Severity::Medium, &["use"]),
    ("utilizes", "wordy", Severity::Medium, &["uses"]),
    ("utilizing", "wordy", Severity::Medium, &["using"]),
    ("utilization", "wordy", Severity::Medium, &["use"]),
    ("commence", "wordy", Severity::Medium, &["start", "begin"]),
    ("commencing", "wordy", Severity::Medium, &["starting"]),
    ("terminate", "wordy", Severity::Medium, &["end", "stop"]),
    ("endeavor", "wordy", Severity::Medium, &["try"]),
    ("endeavour", "wordy", Severity::Medium, &["try"]),
    (
        "ascertain",
        "wordy",
        Severity::Medium,
        &["find out", "check"],
    ),
    (
        "facilitate",
        "wordy",
        Severity::Medium,
        &["help", "make easier"],
    ),
    ("initiate", "wordy", Severity::Medium, &["start"]),
    ("prioritize", "wordy", Severity::Low, &["rank", "put first"]),
    ("methodology", "wordy", Severity::Medium, &["method"]),
    (
        "functionality",
        "wordy",
        Severity::Medium,
        &["features", "what it does"],
    ),
    (
        "aforementioned",
        "wordy",
        Severity::Medium,
        &["this", "that"],
    ),
    ("subsequently", "wordy", Severity::Low, &["then", "later"]),
    ("additionally", "wordy", Severity::Low, &["also"]),
    ("approximately", "wordy", Severity::Low, &["about"]),
    ("sufficient", "wordy", Severity::Low, &["enough"]),
    ("obtain", "wordy", Severity::Low, &["get"]),
    ("purchase", "wordy", Severity::Low, &["buy"]),
    ("require", "wordy", Severity::Low, &["need"]),
    ("demonstrate", "wordy", Severity::Low, &["show"]),
    ("indicate", "wordy", Severity::Low, &["show", "say"]),
    ("modify", "wordy", Severity::Low, &["change"]),
    ("implement", "wordy", Severity::Low, &["build", "do"]),
    ("leverage", "wordy", Severity::Medium, &["use"]),
    ("regarding", "wordy", Severity::Low, &["about"]),
    ("concerning", "wordy", Severity::Low, &["about"]),
    ("whilst", "wordy", Severity::Low, &["while"]),
    ("amongst", "wordy", Severity::Low, &["among"]),
    ("furthermore", "wordy", Severity::Low, &["also"]),
];

const CLICHE: &[WordRow] = &[
    (
        "paradigm",
        "cliche",
        Severity::Medium,
        &["model", "approach"],
    ),
    ("synergy", "cliche", Severity::High, &["overlap", "fit"]),
    ("synergies", "cliche", Severity::High, &["overlaps"]),
    (
        "bandwidth",
        "cliche",
        Severity::Medium,
        &["time", "capacity"],
    ),
    (
        "holistic",
        "cliche",
        Severity::Medium,
        &["whole", "overall"],
    ),
    ("ideate", "cliche", Severity::High, &["think", "plan"]),
    (
        "operationalize",
        "cliche",
        Severity::High,
        &["put into practice"],
    ),
    ("incentivize", "cliche", Severity::Medium, &["encourage"]),
    ("actionize", "cliche", Severity::High, &["act on"]),
    (
        "disruptive",
        "cliche",
        Severity::Medium,
        &["new", "different"],
    ),
];

const DOC_STYLE_PHRASES: &[PhraseRow] = &[
    // Wordy phrases with a documented short form.
    (
        "style.in_order_to",
        "in order to",
        "wordy",
        Severity::Medium,
        "use \"to\"",
    ),
    (
        "style.due_to_the_fact",
        "due to the fact that",
        "wordy",
        Severity::High,
        "use \"because\"",
    ),
    (
        "style.in_the_event_that",
        "in the event that",
        "wordy",
        Severity::Medium,
        "use \"if\"",
    ),
    (
        "style.for_the_purpose_of",
        "for the purpose of",
        "wordy",
        Severity::Medium,
        "use \"to\"",
    ),
    (
        "style.at_this_point_in_time",
        "at this point in time",
        "wordy",
        Severity::High,
        "use \"now\"",
    ),
    (
        "style.in_the_process_of",
        "in the process of",
        "wordy",
        Severity::Medium,
        "cut the frame",
    ),
    (
        "style.a_number_of",
        "a number of",
        "wordy",
        Severity::Low,
        "use \"some\" or say how many",
    ),
    (
        "style.the_majority_of",
        "the majority of",
        "wordy",
        Severity::Low,
        "use \"most\"",
    ),
    (
        "style.is_able_to",
        "is able to",
        "wordy",
        Severity::Low,
        "use \"can\"",
    ),
    (
        "style.has_the_ability_to",
        "has the ability to",
        "wordy",
        Severity::Medium,
        "use \"can\"",
    ),
    (
        "style.in_spite_of_the_fact",
        "in spite of the fact that",
        "wordy",
        Severity::High,
        "use \"although\"",
    ),
    (
        "style.with_regard_to",
        "with {regard|respect} to",
        "wordy",
        Severity::Medium,
        "use \"about\"",
    ),
    (
        "style.in_terms_of",
        "in terms of",
        "wordy",
        Severity::Low,
        "usually cuttable",
    ),
    (
        "style.on_a_regular_basis",
        "on a regular basis",
        "wordy",
        Severity::Medium,
        "use \"regularly\"",
    ),
    (
        "style.prior_to",
        "prior to",
        "wordy",
        Severity::Low,
        "use \"before\"",
    ),
    (
        "style.subsequent_to",
        "subsequent to",
        "wordy",
        Severity::Medium,
        "use \"after\"",
    ),
    (
        "style.it_should_be_noted",
        "it should be noted that",
        "wordy",
        Severity::High,
        "delete the frame",
    ),
    (
        "style.needless_to_say",
        "needless to say",
        "wordy",
        Severity::Medium,
        "then do not say it",
    ),
    (
        "style.please_note_that",
        "please note that",
        "wordy",
        Severity::Low,
        "delete the frame",
    ),
    (
        "style.each_and_every",
        "each and every",
        "wordy",
        Severity::Medium,
        "use \"every\"",
    ),
    (
        "style.first_and_foremost",
        "first and foremost",
        "wordy",
        Severity::Medium,
        "use \"first\"",
    ),
    (
        "style.few_and_far_between",
        "few and far between",
        "cliche",
        Severity::Medium,
        "",
    ),
    // Clichés.
    (
        "style.low_hanging_fruit",
        "low hanging fruit",
        "cliche",
        Severity::High,
        "",
    ),
    (
        "style.move_the_needle",
        "move the needle",
        "cliche",
        Severity::High,
        "",
    ),
    (
        "style.circle_back",
        "circle back",
        "cliche",
        Severity::High,
        "",
    ),
    (
        "style.touch_base",
        "touch base",
        "cliche",
        Severity::High,
        "",
    ),
    (
        "style.think_outside_the_box",
        "think outside the box",
        "cliche",
        Severity::High,
        "",
    ),
    (
        "style.best_practice",
        "best {practice|practices}",
        "cliche",
        Severity::Low,
        "",
    ),
    (
        "style.at_the_end_of_the_day",
        "at the end of the day",
        "cliche",
        Severity::Medium,
        "",
    ),
    (
        "style.the_fact_of_the_matter",
        "the fact of the matter",
        "cliche",
        Severity::Medium,
        "",
    ),
    (
        "style.tip_of_the_iceberg",
        "the tip of the iceberg",
        "cliche",
        Severity::Medium,
        "",
    ),
    (
        "style.a_perfect_storm",
        "a perfect storm",
        "cliche",
        Severity::Medium,
        "",
    ),
    (
        "style.par_for_the_course",
        "par for the course",
        "cliche",
        Severity::Medium,
        "",
    ),
    (
        "style.gold_standard",
        "the gold standard",
        "cliche",
        Severity::Low,
        "",
    ),
    // Jargon.
    (
        "style.going_forward",
        "going forward",
        "jargon",
        Severity::Medium,
        "use \"from now on\" or cut",
    ),
    ("style.at_scale", "at scale", "jargon", Severity::Low, ""),
    ("style.value_add", "value add", "jargon", Severity::High, ""),
    (
        "style.core_competency",
        "core {competency|competencies}",
        "jargon",
        Severity::High,
        "",
    ),
    (
        "style.action_item",
        "action {item|items}",
        "jargon",
        Severity::Medium,
        "",
    ),
    (
        "style.deep_dive",
        "deep dive",
        "jargon",
        Severity::Medium,
        "",
    ),
];

/// Sentence-initial "So" is a rule about position, not about the word, so it
/// stays a phrase pattern rather than a term: the pattern language cannot
/// anchor to a sentence start, but the token sequence `so ,` only occurs in
/// that position in practice.
const OPENER_PHRASES: &[PhraseRow] = &[
    (
        "style.opener_so",
        "so basically",
        "opener",
        Severity::Medium,
        "start with the point",
    ),
    (
        "style.opener_so_yeah",
        "so yeah",
        "opener",
        Severity::Medium,
        "",
    ),
    (
        "style.opener_essentially",
        "so essentially",
        "opener",
        Severity::Medium,
        "",
    ),
];

/// Build the pack.
pub fn pack() -> LexiconPack {
    let all: Vec<WordRow> = WEASEL.iter().chain(WORDY).chain(CLICHE).copied().collect();
    let all_phrases: Vec<PhraseRow> = DOC_STYLE_PHRASES
        .iter()
        .chain(OPENER_PHRASES)
        .copied()
        .collect();
    LexiconPack {
        name: "doc-style".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Documentation style-guide rules mined from Vale's MIT packs: weasel words, \
                      wordy phrases with short equivalents, clichés, corporate jargon, \
                      conversational openers."
            .into(),
        license: "MIT".into(),
        redistributable: true,
        sources: vec![
            source(
                "errata-ai Vale style packs",
                "https://github.com/errata-ai",
                "Microsoft, Google, write-good and proselint rule sets (MIT); substitution \
                 rules become Term::alternatives",
            ),
            source(
                "Microsoft Writing Style Guide",
                "https://learn.microsoft.com/en-us/style-guide/welcome/",
                "the guide prefers contractions — a register statement, not only a rule",
            ),
        ],
        terms: terms(&all),
        phrases: phrases(&all_phrases),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordy_terms_carry_the_replacement_the_fix_block_needs() {
        let pack = pack();
        let utilize = pack
            .terms
            .iter()
            .find(|t| t.word == "utilize")
            .expect("utilize");
        assert_eq!(utilize.alternatives, vec!["use".to_string()]);
        assert_eq!(utilize.category, "wordy");
        // Every wordy term must suggest something; a "wordy" flag with no
        // alternative is a finding an agent cannot act on.
        for term in pack.terms.iter().filter(|t| t.category == "wordy") {
            assert!(
                !term.alternatives.is_empty(),
                "{} suggests no replacement",
                term.word
            );
        }
    }

    #[test]
    fn the_categories_are_the_five_documented_ones() {
        let mut categories = pack().categories();
        categories.sort();
        assert_eq!(
            categories,
            vec!["cliche", "jargon", "opener", "weasel", "wordy"]
        );
    }
}
