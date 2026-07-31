//! Readability grades, passive-voice rate, acronym density.
//!
//! Five readability formulas, all of them cheap and none of them a measurement
//! of "how readable" anything is. They are *shape* statistics: Flesch-Kincaid,
//! Gunning Fog and SMOG combine sentence length with a syllable count;
//! Coleman-Liau and ARI use characters instead and so are immune to the
//! syllable estimator's errors. Shipping both kinds is deliberate — where they
//! disagree, the disagreement is itself a signal about the syllable heuristic.
//!
//! # Syllables
//!
//! Vowel-group counting by default: run-length of vowel characters, minus a
//! silent terminal `e`, floored at one. It is accurate to within a syllable on
//! ordinary English and biased low on `-ion`/`-ial` endings. The CMU
//! pronouncing dictionary is available behind the `verse` cargo feature and is
//! opted into at fit time — never implicitly, because
//! [`SyllableMethod`] is recorded in the
//! fitted state and a build that cannot honor a recorded method fails loudly.
//! A reference must profile identically no matter which cargo features built
//! the binary reading it.
//!
//! # Passive voice
//!
//! A be-form followed by a past participle, within a short window. This is the
//! same heuristic PassivePy documents: it over-fires on adjectival participles
//! ("the door was closed" as a state, not an action) and under-fires on
//! participles the `-ed` rule cannot see (`written`, `built`). Both errors are
//! systematic, so the *bias is constant* between the reference corpus and the
//! draft, which is the only property a comparative rate needs. An absolute
//! passive percentage from this dimension would be wrong; a delta is not.

use serde::{Deserialize, Serialize};

use super::{per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::Result;
use crate::text::syllable::{count_syllables, SyllableMethod};
use crate::text::Analysis;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Be-forms that can head a passive.
#[rustfmt::skip]
const BE_FORMS: &[&str] = &[
    "be", "am", "is", "are", "was", "were", "been", "being", "get", "gets", "got", "gotten",
];

/// Irregular past participles the `-ed` rule cannot see.
#[rustfmt::skip]
const IRREGULAR_PARTICIPLES: &[&str] = &[
    "born", "beaten", "become", "begun", "bent", "bound", "bought", "brought", "built",
    "burnt", "caught", "chosen", "come", "cut", "dealt", "done", "drawn", "driven", "eaten",
    "fallen", "fed", "felt", "fought", "found", "given", "gone", "grown", "held", "hidden",
    "hit", "hurt", "kept", "known", "laid", "led", "left", "lent", "lost", "made", "meant",
    "met", "paid", "put", "read", "run", "said", "seen", "sent", "set", "shot", "shown",
    "shut", "slept", "sold", "sought", "sown", "spent", "spoken", "spread", "stolen",
    "struck", "sung", "sunk", "taken", "taught", "thrown", "told", "understood", "won",
    "worn", "written",
];

/// Words ending in `-ed` that are not participles.
#[rustfmt::skip]
const ED_STOPLIST: &[&str] = &[
    "indeed", "embed", "shed", "sled", "bed", "fed", "wed", "red", "need", "seed", "deed",
    "feed", "breed", "creed", "greed", "speed", "freed", "agreed", "exceed", "succeed",
    "proceed", "hundred", "sacred", "naked", "wicked", "united",
];

/// Configuration for the readability family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readability {
    /// Fewest sentences at which a grade level means anything. Below this the
    /// grade dimensions are marked missing rather than computed from one
    /// sentence, which would make the grade a restatement of that sentence's
    /// length.
    #[serde(default = "default_min_sentences")]
    pub min_sentences: usize,
    /// How syllables are counted. Recorded in the fitted state; see
    /// [`SyllableMethod`].
    #[serde(default)]
    pub syllables: SyllableMethod,
}

fn default_min_sentences() -> usize {
    4
}

impl Default for Readability {
    fn default() -> Self {
        Readability {
            min_sentences: default_min_sentences(),
            syllables: SyllableMethod::default(),
        }
    }
}

/// Grade dimensions, in emission order.
#[rustfmt::skip]
const GRADE_DIMS: &[&str] = &[
    "flesch_kincaid", "gunning_fog", "smog", "coleman_liau", "ari", "syllables_per_word",
];

/// Fitted [`Readability`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedReadability {
    dims: Vec<DimInfo>,
    grades: Vec<Symbol>,
    passive: Symbol,
    acronym: Symbol,
    min_sentences: usize,
    /// How syllables were counted at fit time. Honored or refused at transform;
    /// never silently downgraded.
    #[serde(default)]
    syllable_method: SyllableMethod,
}

impl FittedReadability {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Register;

    /// How this reference counts syllables.
    pub fn syllable_method(&self) -> &SyllableMethod {
        &self.syllable_method
    }
}

impl Feature for Readability {
    type Fitted = FittedReadability;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedReadability> {
        let method = self.syllables.resolve_for_fit()?;
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Register, unit));
            sym
        };
        let grades = GRADE_DIMS
            .iter()
            .map(|g| push(interner, &format!("read:{g}"), Unit::Index))
            .collect();
        let passive = push(interner, "read:passive_rate", Unit::PerHundredSentences);
        let acronym = push(interner, "read:acronym_rate", Unit::PerThousandTokens);

        Ok(FittedReadability {
            dims,
            grades,
            passive,
            acronym,
            min_sentences: self.min_sentences.max(2),
            syllable_method: method,
        })
    }
}

impl FittedFeature for FittedReadability {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Register
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        let sentences = analysis.structure().sentences.len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }

        let stream = analysis.tokens();
        let mut syllables = 0usize;
        let mut polysyllables = 0usize;
        let mut letters = 0usize;
        let mut acronyms = 0usize;
        for (token, form) in stream.lexical() {
            let n = count_syllables(form, &self.syllable_method);
            syllables += n;
            if n >= 3 {
                polysyllables += 1;
            }
            letters += form.chars().filter(|c| c.is_alphanumeric()).count();
            // Acronyms are read off the *source* text, because the scoring view
            // is case-folded and an acronym is a case pattern.
            let raw = analysis.text(token.span);
            if is_acronym(raw) {
                acronyms += 1;
                out.note_span(self.acronym, token.span);
            }
        }
        out.set(self.acronym, per_1k(acronyms, tokens));

        if sentences >= self.min_sentences {
            let words = tokens as f64;
            let sents = sentences as f64;
            let words_per_sentence = words / sents;
            let syllables_per_word = syllables as f64 / words;
            let letters_per_100 = letters as f64 * 100.0 / words;
            let sentences_per_100 = sents * 100.0 / words;

            let fk = 0.39 * words_per_sentence + 11.8 * syllables_per_word - 15.59;
            let fog = 0.4 * (words_per_sentence + 100.0 * (polysyllables as f64 / words));
            // SMOG is defined over 30-sentence samples; the general form scales
            // the polysyllable count back to that basis.
            let smog = 1.0430 * (polysyllables as f64 * 30.0 / sents).sqrt() + 3.1291;
            let coleman_liau = 0.0588 * letters_per_100 - 0.296 * sentences_per_100 - 15.8;
            let ari = 4.71 * (letters as f64 / words) + 0.5 * words_per_sentence - 21.43;

            let values = [fk, fog, smog, coleman_liau, ari, syllables_per_word];
            debug_assert_eq!(values.len(), GRADE_DIMS.len());
            for (&sym, &value) in self.grades.iter().zip(values.iter()) {
                out.set(sym, value);
            }
        } else {
            for &sym in &self.grades {
                out.mark_missing(sym);
            }
        }

        let passives = passive_spans(analysis);
        for span in &passives {
            out.note_span(self.passive, *span);
        }
        out.set(self.passive, ratio(passives.len(), sentences) * 100.0);
    }
}

/// A token that reads as an acronym in the source text.
///
/// Two or more characters, all uppercase or digits, at least one letter. `I`
/// and `A` are excluded by the length rule; `2026` by the letter rule.
fn is_acronym(raw: &str) -> bool {
    let chars: Vec<char> = raw.chars().collect();
    if chars.len() < 2 || chars.len() > 8 {
        return false;
    }
    let mut has_letter = false;
    for &c in &chars {
        if c.is_alphabetic() {
            if c.is_lowercase() {
                return false;
            }
            has_letter = true;
        } else if !c.is_ascii_digit() {
            return false;
        }
    }
    has_letter
}

/// Spans of `be-form (… ) past-participle` within a four-token window.
fn passive_spans(analysis: &Analysis<'_>) -> Vec<crate::text::Span> {
    let stream = analysis.tokens();
    let lexical: Vec<(&str, crate::text::Span)> =
        stream.lexical().map(|(t, f)| (f, t.span)).collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lexical.len() {
        if !BE_FORMS.contains(&lexical[i].0) {
            i += 1;
            continue;
        }
        // Allow up to two intervening words: "was quickly and quietly closed".
        let end = (i + 4).min(lexical.len());
        for j in (i + 1)..end {
            if is_past_participle(lexical[j].0) {
                out.push(crate::text::Span::new(lexical[i].1.start, lexical[j].1.end));
                i = j;
                break;
            }
        }
        i += 1;
    }
    out
}

fn is_past_participle(form: &str) -> bool {
    if IRREGULAR_PARTICIPLES.contains(&form) {
        return true;
    }
    form.len() > 4 && form.ends_with("ed") && !ED_STOPLIST.contains(&form)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn transform(text: &str) -> (FeatureVector, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = Readability::default().fit(&ctx, &mut interner).unwrap();
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform(text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    /// Four sentences, twenty words, hand-countable syllables.
    const FIXTURE: &str = "The cat sat down. The dog ran away. A bird flew high. The fish swam.";

    #[test]
    fn grades_match_hand_computed_values() {
        // 15 words over 4 sentences = 3.75 words/sentence. Every word here is
        // monosyllabic under vowel-group counting except "away" (2), so
        // syllables = 16 and syllables/word = 16/15.
        let spw = value(FIXTURE, "read:syllables_per_word");
        assert!((spw - 16.0 / 15.0).abs() < 1e-9, "{spw}");
        let expected_fk = 0.39 * 3.75 + 11.8 * (16.0 / 15.0) - 15.59;
        assert!((value(FIXTURE, "read:flesch_kincaid") - expected_fk).abs() < 1e-9);
        // No polysyllables, so Fog is just 0.4 * words-per-sentence.
        assert!((value(FIXTURE, "read:gunning_fog") - 0.4 * 3.75).abs() < 1e-9);
        // The character-based formulas agree that the text is short and plain.
        assert!(value(FIXTURE, "read:ari") < 2.0);
    }

    #[test]
    fn long_latinate_prose_grades_above_short_plain_prose() {
        let plain = FIXTURE;
        let dense = "The implementation of the aforementioned optimization demonstrates \
                     considerable improvement across heterogeneous configurations. \
                     Subsequent instrumentation corroborated the preliminary determination. \
                     Additional experimentation remains necessary. Consequently, the \
                     investigation continues.";
        for grade in [
            "read:flesch_kincaid",
            "read:gunning_fog",
            "read:coleman_liau",
            "read:ari",
        ] {
            assert!(
                value(dense, grade) > value(plain, grade),
                "{grade}: dense={} plain={}",
                value(dense, grade),
                value(plain, grade)
            );
        }
    }

    #[test]
    fn grades_are_missing_below_the_sentence_floor() {
        let (v, i) = transform("Only one sentence here, and it is short.");
        assert!(v.is_missing(i.get("read:flesch_kincaid").unwrap()));
        assert!(v.is_missing(i.get("read:syllables_per_word").unwrap()));
        // Rates that do not need a grade basis are still emitted.
        assert!(!v.is_missing(i.get("read:acronym_rate").unwrap()));
    }

    #[test]
    fn the_passive_proxy_finds_be_plus_participle() {
        let text = "The record was written to disk. The cache was cleared by the worker. \
                    The team shipped it. Nobody complained.";
        // Two passives over four sentences -> 50 per 100 sentences.
        assert!((value(text, "read:passive_rate") - 50.0).abs() < 1e-9);
    }

    #[test]
    fn the_documented_adjectival_false_positive_is_reproducible() {
        // "was closed" is a state here, not an action, and the heuristic
        // claims it anyway. Asserted so the bias stays visible rather than
        // being quietly "fixed" into an inconsistency with the corpus side.
        let text = "The door was closed. The sky was blue. We went home. It rained.";
        assert!(value(text, "read:passive_rate") > 0.0);
    }

    #[test]
    fn passive_spans_cover_the_construction() {
        let text = "The record was written to disk. It worked. We left. Fine.";
        let (v, i) = transform(text);
        let spans = v.spans(i.get("read:passive_rate").unwrap());
        assert_eq!(&text[spans[0].range()], "was written");
    }

    #[test]
    fn acronyms_are_counted_off_the_source_case() {
        // Five lexical tokens, two acronyms -> 400 per 1k.
        let text = "the API and SDK ship";
        assert!((value(text, "read:acronym_rate") - 400.0).abs() < 1e-9);
        // A single capital letter is not an acronym, nor is a bare year.
        assert_eq!(value("I saw it in 2026 again", "read:acronym_rate"), 0.0);
    }

    #[test]
    fn syllable_counting_handles_the_usual_traps() {
        let m = SyllableMethod::VowelGroup;
        assert_eq!(count_syllables("cat", &m), 1);
        assert_eq!(count_syllables("away", &m), 2);
        assert_eq!(count_syllables("table", &m), 2);
        assert_eq!(count_syllables("make", &m), 1);
        assert_eq!(count_syllables("beautiful", &m), 3);
        // Never zero, however odd the input.
        assert_eq!(count_syllables("rhythm", &m), 1);
        assert_eq!(count_syllables("", &m), 0);
    }

    #[test]
    fn empty_text_marks_everything_missing() {
        let (v, i) = transform("");
        assert!(v.is_missing(i.get("read:passive_rate").unwrap()));
    }

    #[test]
    fn the_spec_round_trips_and_defaults_to_vowel_group() {
        let spec: Readability = serde_json::from_str("{}").unwrap();
        assert_eq!(spec, Readability::default());
        assert_eq!(spec.syllables, SyllableMethod::VowelGroup);
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<Readability>(&json).unwrap(), spec);
    }
}
