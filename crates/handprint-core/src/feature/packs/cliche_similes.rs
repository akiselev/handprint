//! Frozen similes — the dead-metaphor half of the comparison-frame family.
//!
//! A frozen simile is one whose vehicle is fixed by convention: "as good as
//! gold", "like a bull in a china shop". The frozen-simile literature (see
//! arxiv 1511.01756 on distinguishing conventional from creative comparisons)
//! treats the distinction as the interesting variable, and it is: an author's
//! *frozen share* — what fraction of their similes are dead ones — separates
//! the writer reaching for a stock phrase from the one inventing a vehicle.
//!
//! It is also the anti-caricature dimension for comic imitation. The Dark &
//! Stormy finding is that LLMs imitating comic prose over-fire novel
//! comparisons; a frozen share *below* a human author's band is the shape that
//! failure takes, and only a two-sided band can see it.
//!
//! Original curation from the conventional-simile literature and ordinary
//! English idiom. Entries are literal token sequences, because a frozen simile
//! with a wildcard in it is by definition not frozen.

use super::{phrases, source, PhraseRow};
use crate::feature::lexicon::{LexiconPack, Severity};

/// Version of this pack's contents.
pub const VERSION: &str = "2026.08";

// A data table. `rustfmt::skip` keeps one entry per line, which is how data
// wants to be read and diffed.
#[rustfmt::skip]
const FROZEN: &[PhraseRow] = &[
    // `as X as Y`.
    ("simile.good_as_gold", "as good as gold", "frozen", Severity::Low, ""),
    ("simile.white_as_snow", "as white as snow", "frozen", Severity::Low, ""),
    ("simile.black_as_night", "as black as night", "frozen", Severity::Low, ""),
    ("simile.old_as_the_hills", "as old as the hills", "frozen", Severity::Low, ""),
    ("simile.blind_as_a_bat", "as blind as a bat", "frozen", Severity::Low, ""),
    ("simile.busy_as_a_bee", "as busy as a bee", "frozen", Severity::Low, ""),
    ("simile.quiet_as_a_mouse", "as quiet as a mouse", "frozen", Severity::Low, ""),
    ("simile.light_as_a_feather", "as light as a feather", "frozen", Severity::Low, ""),
    ("simile.hard_as_nails", "as hard as nails", "frozen", Severity::Low, ""),
    ("simile.cold_as_ice", "as cold as ice", "frozen", Severity::Low, ""),
    ("simile.clear_as_day", "as clear as day", "frozen", Severity::Low, ""),
    ("simile.clear_as_mud", "as clear as mud", "frozen", Severity::Low, ""),
    ("simile.dry_as_a_bone", "as dry as a bone", "frozen", Severity::Low, ""),
    ("simile.flat_as_a_pancake", "as flat as a pancake", "frozen", Severity::Low, ""),
    ("simile.free_as_a_bird", "as free as a bird", "frozen", Severity::Low, ""),
    ("simile.fresh_as_a_daisy", "as fresh as a daisy", "frozen", Severity::Low, ""),
    ("simile.mad_as_a_hatter", "as mad as a hatter", "frozen", Severity::Low, ""),
    ("simile.pale_as_a_ghost", "as pale as a ghost", "frozen", Severity::Low, ""),
    ("simile.proud_as_a_peacock", "as proud as a peacock", "frozen", Severity::Low, ""),
    ("simile.quick_as_a_flash", "as quick as a flash", "frozen", Severity::Low, ""),
    ("simile.sharp_as_a_tack", "as sharp as a tack", "frozen", Severity::Low, ""),
    ("simile.sick_as_a_dog", "as sick as a dog", "frozen", Severity::Low, ""),
    ("simile.slow_as_molasses", "as slow as molasses", "frozen", Severity::Low, ""),
    ("simile.stubborn_as_a_mule", "as stubborn as a mule", "frozen", Severity::Low, ""),
    ("simile.thick_as_thieves", "as thick as thieves", "frozen", Severity::Low, ""),
    ("simile.tough_as_old_boots", "as tough as old boots", "frozen", Severity::Low, ""),
    ("simile.easy_as_pie", "as easy as pie", "frozen", Severity::Low, ""),
    ("simile.right_as_rain", "as right as rain", "frozen", Severity::Low, ""),
    ("simile.happy_as_a_clam", "as happy as a clam", "frozen", Severity::Low, ""),
    ("simile.strong_as_an_ox", "as strong as an ox", "frozen", Severity::Low, ""),
    ("simile.deaf_as_a_post", "as deaf as a post", "frozen", Severity::Low, ""),
    ("simile.plain_as_the_nose", "as plain as the nose on your face", "frozen", Severity::Low, ""),
    ("simile.snug_as_a_bug", "as snug as a bug", "frozen", Severity::Low, ""),
    ("simile.sober_as_a_judge", "as sober as a judge", "frozen", Severity::Low, ""),
    ("simile.keen_as_mustard", "as keen as mustard", "frozen", Severity::Low, ""),
    ("simile.dead_as_a_doornail", "as dead as a doornail", "frozen", Severity::Low, ""),
    ("simile.safe_as_houses", "as safe as houses", "frozen", Severity::Low, ""),
    ("simile.common_as_dirt", "as common as dirt", "frozen", Severity::Low, ""),
    // `like Y`.
    ("simile.like_a_bull", "like a bull in a china shop", "frozen", Severity::Low, ""),
    ("simile.like_a_fish", "like a fish out of water", "frozen", Severity::Low, ""),
    ("simile.like_a_house_on_fire", "like a house on fire", "frozen", Severity::Low, ""),
    ("simile.like_a_ton_of_bricks", "like a ton of bricks", "frozen", Severity::Low, ""),
    ("simile.like_a_deer", "like a deer in the headlights", "frozen", Severity::Low, ""),
    ("simile.like_a_broken_record", "like a broken record", "frozen", Severity::Low, ""),
    ("simile.like_a_moth", "like a moth to a flame", "frozen", Severity::Low, ""),
    ("simile.like_a_dream", "like a dream", "frozen", Severity::Low, ""),
    ("simile.like_a_charm", "like a charm", "frozen", Severity::Low, ""),
    ("simile.like_a_glove", "like a glove", "frozen", Severity::Low, ""),
    ("simile.like_a_rock", "like a rock", "frozen", Severity::Low, ""),
    ("simile.like_a_hawk", "like a hawk", "frozen", Severity::Low, ""),
    ("simile.like_a_log", "like a log", "frozen", Severity::Low, ""),
    ("simile.like_a_lamb", "like a lamb to the slaughter", "frozen", Severity::Low, ""),
    ("simile.like_a_kid", "like a kid in a candy store", "frozen", Severity::Low, ""),
    ("simile.like_a_sore_thumb", "like a sore thumb", "frozen", Severity::Low, ""),
    ("simile.like_a_wet_blanket", "like a wet blanket", "frozen", Severity::Low, ""),
    ("simile.like_the_back_of_my_hand", "like the back of my hand", "frozen", Severity::Low, ""),
    ("simile.like_a_train_wreck", "like a train wreck", "frozen", Severity::Low, ""),
    ("simile.like_a_bat_out_of_hell", "like a bat out of hell", "frozen", Severity::Low, ""),
];

/// Build the pack.
pub fn pack() -> LexiconPack {
    LexiconPack {
        name: "cliche-similes".into(),
        version: VERSION.into(),
        date: "2026-08-01".into(),
        description: "Frozen (conventional) similes, used for frame:frozen_share. An author's \
                      cliché share separates reaching for a stock vehicle from inventing one, \
                      and a share below the human band is what over-fired comic imitation \
                      looks like."
            .into(),
        license: "CC0-1.0".into(),
        redistributable: true,
        sources: vec![
            source(
                "Original curation",
                "https://github.com/akiselev/handprint",
                "conventional similes compiled from ordinary English idiom",
            ),
            source(
                "Frozen vs creative similes",
                "https://arxiv.org/pdf/1511.01756",
                "the conventional/creative distinction this pack operationalizes",
            ),
        ],
        terms: Vec::new(),
        phrases: phrases(FROZEN),
        privacy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_is_a_literal_token_sequence() {
        // A frozen simile with a wildcard in it is not frozen. The frame
        // feature only loads literal patterns, so a wildcard here would be
        // silently dropped rather than matched.
        for phrase in pack().phrases {
            assert!(
                !phrase.pattern.contains(['*', '{']),
                "{} is not literal",
                phrase.id
            );
            assert_eq!(phrase.pattern, phrase.pattern.to_lowercase());
        }
    }

    #[test]
    fn entries_open_with_a_comparison_frame() {
        for phrase in pack().phrases {
            assert!(
                phrase.pattern.starts_with("as ") || phrase.pattern.starts_with("like "),
                "{} is not a simile: {}",
                phrase.id,
                phrase.pattern
            );
        }
    }

    #[test]
    fn the_pack_is_big_enough_for_a_share_to_mean_something() {
        assert!(pack().phrases.len() >= 50);
    }
}
