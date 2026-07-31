//! Marketing lexicons: the evaluative/pressure/headline pack and the slop pack.
//!
//! Two packs, split by what they are evidence *of*.
//!
//! * **`marketing-eval`** — the register itself, grounded in Leech's *English
//!   in Advertising* (the favourite-adjective list, the imperative-heavy
//!   syntax), the dark-patterns literature for *vetted* urgency and scarcity
//!   phrasings (Mathur et al. over 11k shopping sites, so these are phrasings
//!   observed in the wild rather than invented), and the clickbait literature
//!   for forward-reference deixis (Blom & Hansen: "This is why", "What happened
//!   next"). A high rate here means "this is advertising", which is a fact
//!   about register, not a fault.
//! * **`marketing-slop`** — the AI-generated-marketing consensus core: unlock,
//!   elevate, seamless, game-changer, "in today's fast-paced world", "look no
//!   further", plus the structural tells. A high rate here means "this was
//!   probably generated", which is a different claim.
//!
//! Keeping them apart matters because a human copywriter scores high on the
//! first and low on the second, and collapsing them would make the family
//! unable to say so.
//!
//! Numeric slots use the `#` wildcard — "only # left", "trusted by # teams" —
//! which is what that wildcard was added for.
//!
//! **The hand lists are the fallback, not the mechanism.** The plan's
//! Kobak-method discovery run (excess-vocabulary contrast of a marketing corpus
//! against a background) supersedes them; when it lands, these become the
//! golden test that the derived pack has to reproduce.

use super::{phrases, source, terms, PhraseRow, WordRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of these packs' contents.
pub const VERSION: &str = "2026.08";

/// Leech's favourite advertising adjectives, plus the modern additions.
#[rustfmt::skip]
const AD_ADJECTIVES: &[WordRow] = &[
    ("new", "evaluation", Severity::Low, &[]),
    ("good", "evaluation", Severity::Low, &[]),
    ("better", "evaluation", Severity::Low, &[]),
    ("best", "evaluation", Severity::Low, &[]),
    ("free", "evaluation", Severity::Low, &[]),
    ("fresh", "evaluation", Severity::Low, &[]),
    ("delicious", "evaluation", Severity::Low, &[]),
    ("sure", "evaluation", Severity::Low, &[]),
    ("clean", "evaluation", Severity::Low, &[]),
    ("wonderful", "evaluation", Severity::Low, &[]),
    ("special", "evaluation", Severity::Low, &[]),
    ("crisp", "evaluation", Severity::Low, &[]),
    ("fine", "evaluation", Severity::Low, &[]),
    ("big", "evaluation", Severity::Low, &[]),
    ("great", "evaluation", Severity::Low, &[]),
    ("real", "evaluation", Severity::Low, &[]),
    ("easy", "evaluation", Severity::Low, &[]),
    ("bright", "evaluation", Severity::Low, &[]),
    ("extra", "evaluation", Severity::Low, &[]),
    ("safe", "evaluation", Severity::Low, &[]),
    ("rich", "evaluation", Severity::Low, &[]),
    ("premium", "evaluation", Severity::Low, &[]),
    ("exclusive", "evaluation", Severity::Low, &[]),
    ("proven", "evaluation", Severity::Low, &[]),
    ("trusted", "evaluation", Severity::Low, &[]),
    ("powerful", "evaluation", Severity::Low, &[]),
    ("effortless", "evaluation", Severity::Low, &[]),
    ("instant", "evaluation", Severity::Low, &[]),
    ("unlimited", "evaluation", Severity::Low, &[]),
    ("guaranteed", "evaluation", Severity::Low, &[]),
    ("award-winning", "evaluation", Severity::Low, &[]),
    ("revolutionary", "evaluation", Severity::Low, &[]),
];

/// Universal quantifiers, the other half of Leech's evaluative axis.
#[rustfmt::skip]
const QUANTIFIERS: &[WordRow] = &[
    ("every", "evaluation", Severity::Low, &[]),
    ("everything", "evaluation", Severity::Low, &[]),
    ("everyone", "evaluation", Severity::Low, &[]),
    ("always", "evaluation", Severity::Low, &[]),
    ("never", "evaluation", Severity::Low, &[]),
    ("anywhere", "evaluation", Severity::Low, &[]),
    ("anytime", "evaluation", Severity::Low, &[]),
    ("anyone", "evaluation", Severity::Low, &[]),
    ("all", "evaluation", Severity::Low, &[]),
    ("any", "evaluation", Severity::Low, &[]),
];

#[rustfmt::skip]
const EVAL_PHRASES: &[PhraseRow] = &[
    // Pressure: urgency and scarcity, from the vetted dark-patterns set.
    ("mkt.only_n_left", "only # left", "pressure", Severity::Medium, ""),
    ("mkt.n_left_in_stock", "# left in stock", "pressure", Severity::Medium, ""),
    ("mkt.selling_fast", "selling fast", "pressure", Severity::Medium, ""),
    ("mkt.almost_gone", "almost gone", "pressure", Severity::Medium, ""),
    ("mkt.limited_time", "limited time {offer|only}", "pressure", Severity::Medium, ""),
    ("mkt.act_now", "act {now|fast|today}", "pressure", Severity::Medium, ""),
    ("mkt.dont_miss", "{don't|dont} miss {out|this}", "pressure", Severity::Medium, ""),
    ("mkt.hurry", "hurry while", "pressure", Severity::Medium, ""),
    ("mkt.ends_soon", "{ends|expires} {soon|today|tonight}", "pressure", Severity::Medium, ""),
    ("mkt.last_chance", "last chance", "pressure", Severity::Medium, ""),
    ("mkt.while_supplies", "while supplies last", "pressure", Severity::Medium, ""),
    // Pressure: social proof.
    ("mkt.trusted_by", "trusted by #", "pressure", Severity::Low, ""),
    ("mkt.join_n", "join #", "pressure", Severity::Low, ""),
    ("mkt.n_customers", "# {customers|users|teams|companies}", "pressure", Severity::Low, ""),
    ("mkt.people_are_viewing", "# people are viewing", "pressure", Severity::Medium, ""),
    ("mkt.loved_by", "loved by {thousands|millions}", "pressure", Severity::Low, ""),
    // Pressure: risk reversal.
    ("mkt.money_back", "money back guarantee", "pressure", Severity::Low, ""),
    ("mkt.cancel_anytime", "cancel {anytime|whenever}", "pressure", Severity::Low, ""),
    ("mkt.cancel_any_time", "cancel any time", "pressure", Severity::Low, ""),
    ("mkt.no_credit_card", "no credit card required", "pressure", Severity::Low, ""),
    ("mkt.risk_free", "risk free", "pressure", Severity::Low, ""),
    ("mkt.no_questions", "no questions asked", "pressure", Severity::Low, ""),
    // Benefit connectives.
    ("mkt.so_you_can", "so you can", "benefit", Severity::Low, ""),
    ("mkt.which_means", "which means you", "benefit", Severity::Low, ""),
    ("mkt.that_means", "that means you", "benefit", Severity::Low, ""),
    ("mkt.helping_you", "{helping|helps} you", "benefit", Severity::Low, ""),
    ("mkt.designed_to", "designed to help", "benefit", Severity::Low, ""),
    ("mkt.built_for", "built for {you|teams|speed}", "benefit", Severity::Low, ""),
    // Directive: calls to action.
    ("mkt.get_started", "get started {today|now|free}", "directive", Severity::Low, ""),
    ("mkt.sign_up", "sign up {today|now|free}", "directive", Severity::Low, ""),
    ("mkt.learn_more", "learn more", "directive", Severity::Low, ""),
    ("mkt.try_free", "try it free", "directive", Severity::Low, ""),
    ("mkt.book_a_demo", "book a demo", "directive", Severity::Low, ""),
    ("mkt.start_your", "start your {free|trial}", "directive", Severity::Low, ""),
    ("mkt.click_here", "click here", "directive", Severity::Medium, ""),
    // Headline: clickbait curiosity and forward-reference deixis.
    ("mkt.this_is_why", "this is why", "headline", Severity::Medium, ""),
    ("mkt.heres_why", "{here's|heres} why", "headline", Severity::Medium, ""),
    ("mkt.what_happened_next", "what happened next", "headline", Severity::High, ""),
    ("mkt.you_wont_believe", "you {won't|wont} believe", "headline", Severity::High, ""),
    ("mkt.the_reason_will", "the reason will", "headline", Severity::High, ""),
    ("mkt.nobody_talks_about", "nobody {talks|mentions}", "headline", Severity::Medium, ""),
    ("mkt.nobody_tells_you", "nobody tells you", "headline", Severity::Medium, ""),
    ("mkt.the_one_thing", "the one {thing|trick|secret}", "headline", Severity::High, ""),
    ("mkt.what_they_dont", "what they {don't|dont} want", "headline", Severity::High, ""),
    ("mkt.n_ways_to", "# ways to", "headline", Severity::Low, ""),
    ("mkt.n_things", "# things {you|to|that}", "headline", Severity::Low, ""),
    ("mkt.n_reasons", "# reasons {why|to}", "headline", Severity::Low, ""),
];

/// The AI-generated-marketing consensus core.
#[rustfmt::skip]
const SLOP_WORDS: &[WordRow] = &[
    ("unlock", "slop_vocab", Severity::High, &["open", "get"]),
    ("unlocking", "slop_vocab", Severity::High, &["opening"]),
    ("elevate", "slop_vocab", Severity::High, &["improve", "raise"]),
    ("elevating", "slop_vocab", Severity::High, &["improving"]),
    ("seamless", "slop_vocab", Severity::High, &["smooth"]),
    ("seamlessly", "slop_vocab", Severity::High, &["smoothly"]),
    ("empower", "slop_vocab", Severity::High, &["let", "enable"]),
    ("empowering", "slop_vocab", Severity::High, &["letting"]),
    ("transformative", "slop_vocab", Severity::High, &["big"]),
    ("innovative", "slop_vocab", Severity::Medium, &["new"]),
    ("cutting-edge", "slop_vocab", Severity::High, &["new"]),
    ("best-in-class", "slop_vocab", Severity::High, &["good"]),
    ("next-generation", "slop_vocab", Severity::Medium, &["newer"]),
    ("supercharge", "slop_vocab", Severity::High, &["speed up"]),
    ("streamline", "slop_vocab", Severity::Medium, &["simplify"]),
    ("optimize", "slop_vocab", Severity::Low, &["improve", "tune"]),
    ("maximize", "slop_vocab", Severity::Medium, &["increase"]),
    ("revolutionize", "slop_vocab", Severity::High, &["change"]),
    ("democratize", "slop_vocab", Severity::High, &["open up"]),
    ("curated", "slop_vocab", Severity::Medium, &["chosen"]),
    ("bespoke", "slop_vocab", Severity::Medium, &["custom"]),
    ("turnkey", "slop_vocab", Severity::Medium, &["ready"]),
    ("frictionless", "slop_vocab", Severity::High, &["easy"]),
    ("scalable", "slop_vocab", Severity::Low, &["it scales"]),
    ("robust", "slop_vocab", Severity::Low, &["reliable"]),
    ("holistic", "slop_vocab", Severity::Medium, &["whole"]),
    ("actionable", "slop_vocab", Severity::Medium, &["usable"]),
];

#[rustfmt::skip]
const SLOP_PHRASES: &[PhraseRow] = &[
    ("slop.todays_world", "in {today's|todays} {fast|fast-paced|digital|modern} *", "slop_frame", Severity::High, "name the year or cut it"),
    ("slop.look_no_further", "look no further", "slop_frame", Severity::High, ""),
    ("slop.game_changer", "a game changer", "slop_frame", Severity::High, ""),
    ("slop.game_changing", "game changing", "slop_frame", Severity::High, ""),
    ("slop.take_it_to_the_next", "to the next level", "slop_frame", Severity::High, ""),
    ("slop.unlock_the_power", "unlock the {power|potential|secrets|value}", "slop_frame", Severity::High, ""),
    ("slop.harness_the_power", "harness the power", "slop_frame", Severity::High, ""),
    ("slop.say_goodbye", "say goodbye to", "slop_frame", Severity::High, ""),
    ("slop.imagine_a_world", "imagine a world", "slop_frame", Severity::High, ""),
    ("slop.whether_you_are", "whether {you're|youre} *", "slop_frame", Severity::Medium, ""),
    ("slop.whether_you_are_long", "whether you are *", "slop_frame", Severity::Medium, ""),
    ("slop.the_perfect_blend", "the perfect {blend|balance|mix}", "slop_frame", Severity::High, ""),
    ("slop.at_your_fingertips", "at your fingertips", "slop_frame", Severity::High, ""),
    ("slop.tailored_to_your", "tailored to your", "slop_frame", Severity::Medium, ""),
    ("slop.designed_with_you", "designed with you in mind", "slop_frame", Severity::High, ""),
    ("slop.the_future_of", "the future of", "slop_frame", Severity::Medium, ""),
    ("slop.more_than_just", "more than just", "slop_frame", Severity::Medium, ""),
    ("slop.not_just_but", "not just * but", "slop_structure", Severity::High, "say the positive claim directly"),
    ("slop.its_not_its", "{it's|its} not * {it's|its}", "slop_structure", Severity::High, "say the positive claim directly"),
    ("slop.dive_in", "{let's|lets} dive", "slop_structure", Severity::Medium, ""),
    ("slop.in_conclusion", "in conclusion", "slop_structure", Severity::High, "stop when you are done"),
    ("slop.key_takeaways", "key {takeaway|takeaways}", "slop_structure", Severity::Medium, ""),
    ("slop.that_being_said", "that being said", "slop_structure", Severity::Low, ""),
    ("slop.here_is_the_thing", "{here's|heres} the {thing|deal}", "slop_structure", Severity::Low, ""),
];

/// The advertising-register pack.
pub fn eval_pack() -> LexiconPack {
    let words: Vec<WordRow> = AD_ADJECTIVES.iter().chain(QUANTIFIERS).copied().collect();
    LexiconPack {
        name: "marketing-eval".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "The advertising register: Leech's favourite adjectives, universal \
                      quantifiers, vetted urgency/scarcity/social-proof phrasings, benefit \
                      connectives, calls to action, and clickbait forward-reference deixis. \
                      A high rate here says \"this is advertising\", not \"this is bad\"."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "Leech 1966, English in Advertising",
                "https://archive.org/details/englishinadverti0000leec",
                "favourite-adjective list and the imperative/block-language observations, \
                 re-curated from the published description",
            ),
            source(
                "Mathur et al., Dark Patterns at Scale",
                "https://arxiv.org/pdf/1907.07032",
                "urgency, scarcity and social-proof phrasings observed across 11k shopping \
                 sites — vetted phrasings rather than invented ones",
            ),
            source(
                "Blom & Hansen, forward-reference clickbait",
                "https://doi.org/10.1016/j.pragma.2014.11.010",
                "the forward-reference deixis behind \"This is why\" and \"What happened next\"",
            ),
        ],
        terms: terms(&words),
        phrases: phrases(EVAL_PHRASES),
        privacy: None,
    }
}

/// The AI-marketing slop pack.
pub fn slop_pack() -> LexiconPack {
    LexiconPack {
        name: "marketing-slop".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "AI-generated marketing copy: the consensus vocabulary core plus the \
                      structural frames. Separate from marketing-eval because a human \
                      copywriter scores high on that and low on this."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "Original curation",
                "https://github.com/akiselev/handprint",
                "consensus core across curated AI-marketing-tell lists; supersede it with a \
                 Kobak-method contrast run against a real marketing corpus",
            ),
            source(
                "Kobak et al., excess vocabulary",
                "https://github.com/berenslab/llm-excess-vocab",
                "the method this list is a stand-in for until the marketing run is done",
            ),
        ],
        terms: terms(SLOP_WORDS),
        phrases: phrases(SLOP_PHRASES),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_packs_do_not_overlap() {
        // The whole reason they are separate packs: a term in both would make
        // "this is advertising" and "this was generated" the same measurement.
        let eval: Vec<String> = eval_pack().terms.into_iter().map(|t| t.word).collect();
        let slop: Vec<String> = slop_pack().terms.into_iter().map(|t| t.word).collect();
        for word in &slop {
            assert!(!eval.contains(word), "{word} is in both marketing packs");
        }
    }

    #[test]
    fn the_numeric_slots_use_the_hash_wildcard() {
        let pack = eval_pack();
        let numeric: Vec<&str> = pack
            .phrases
            .iter()
            .filter(|p| p.pattern.contains('#'))
            .map(|p| p.id.as_str())
            .collect();
        assert!(numeric.len() >= 5, "{numeric:?}");
        pack.validate_patterns().unwrap();
        slop_pack().validate_patterns().unwrap();
    }

    #[test]
    fn the_eval_categories_are_the_five_leech_families() {
        let mut categories = eval_pack().categories();
        categories.sort();
        assert_eq!(
            categories,
            vec!["benefit", "directive", "evaluation", "headline", "pressure"]
        );
    }
}
