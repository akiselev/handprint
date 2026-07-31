//! Syllable counting, and why the method is fitted state rather than build
//! state.
//!
//! Several families want syllables: readability grades, the escalation-rhythm
//! dimensions in [`syntax`](crate::feature::syntax), alliteration and rhyme in
//! [`device`](crate::feature::device), and every metrical dimension in
//! [`verse`](crate::feature::verse). Two methods are available:
//!
//! * [`SyllableMethod::VowelGroup`] — count runs of vowel characters, drop a
//!   silent terminal `e`, floor at one. No data, no feature flag, accurate to
//!   within a syllable on ordinary English.
//! * [`SyllableMethod::Dict`] — the CMU pronouncing dictionary, behind the
//!   `verse` cargo feature. Accurate, and the only honest basis for phoneme
//!   work like rhyme chains.
//!
//! **The method is recorded in the fitted state and honored or refused, never
//! silently downgraded.** If the dictionary were used whenever the build
//! happened to have it, the same `Reference` JSON would profile differently on
//! a `verse` build than on a plain one, and a calibration fitted under one
//! would silently mis-score under the other. That breaks the transform-purity
//! invariant in the most expensive possible way: quietly, and only in the
//! numbers. So a reference that records `Dict` and is loaded by a build without
//! the dictionary produces an error, not a fallback.
//!
//! Within the `Dict` method, per-word fallback to vowel groups for
//! out-of-dictionary words is fine and expected — it is deterministic given the
//! pinned dictionary version, which is the property that matters.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// How a fitted feature counts syllables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "method", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SyllableMethod {
    /// Vowel-group heuristic. No data dependency; the default everywhere.
    #[default]
    VowelGroup,
    /// CMU pronouncing dictionary, pinned to a version.
    Dict {
        /// The dictionary version this reference was fitted against.
        dict_version: String,
    },
}

impl SyllableMethod {
    /// A short name for provenance and error messages.
    pub fn as_str(&self) -> &str {
        match self {
            SyllableMethod::VowelGroup => "vowel_group",
            SyllableMethod::Dict { .. } => "dict",
        }
    }

    /// Resolve the method a `fit` should record.
    ///
    /// Selecting `Dict` on a build without the dictionary is a configuration
    /// error at fit time, which is the cheap place to find out — far better
    /// than writing a reference nobody else can load.
    pub fn resolve_for_fit(&self) -> Result<SyllableMethod> {
        match self {
            SyllableMethod::VowelGroup => Ok(SyllableMethod::VowelGroup),
            SyllableMethod::Dict { dict_version } => {
                #[cfg(feature = "verse")]
                {
                    let available = crate::text::dict::DICT_VERSION;
                    if !dict_version.is_empty() && dict_version != available {
                        return Err(Error::InvalidConfig {
                            what: "SyllableMethod::Dict",
                            detail: format!(
                                "this build carries dictionary {available}, not {dict_version}"
                            ),
                        });
                    }
                    Ok(SyllableMethod::Dict {
                        dict_version: available.to_owned(),
                    })
                }
                #[cfg(not(feature = "verse"))]
                {
                    let _ = dict_version;
                    Err(Error::InvalidConfig {
                        what: "SyllableMethod::Dict",
                        detail: "dictionary syllables need the `verse` cargo feature; \
                                 rebuild with --features verse or use vowel-group syllables"
                            .into(),
                    })
                }
            }
        }
    }

    /// Check that this build can honor a method recorded in a fitted artifact.
    ///
    /// Called when a reference is loaded. Failing here is the whole design: a
    /// build that cannot reproduce the recorded method must refuse, because the
    /// alternative is numbers that look fine and are not comparable.
    pub fn check_supported(&self) -> Result<()> {
        match self {
            SyllableMethod::VowelGroup => Ok(()),
            SyllableMethod::Dict { dict_version } => {
                #[cfg(feature = "verse")]
                {
                    let available = crate::text::dict::DICT_VERSION;
                    if dict_version == available {
                        Ok(())
                    } else {
                        Err(Error::InvalidConfig {
                            what: "SyllableMethod::Dict",
                            detail: format!(
                                "reference requires pronouncing dictionary {dict_version}, \
                                 this build carries {available}"
                            ),
                        })
                    }
                }
                #[cfg(not(feature = "verse"))]
                {
                    Err(Error::InvalidConfig {
                        what: "SyllableMethod::Dict",
                        detail: format!(
                            "reference requires the `verse` build and pronouncing dictionary \
                             {dict_version}; this build has neither. Rebuild handprint with \
                             --features verse, or refit the reference with vowel-group syllables."
                        ),
                    })
                }
            }
        }
    }
}

/// Count the syllables in a normalized (lowercase) word form.
pub fn count_syllables(word: &str, method: &SyllableMethod) -> usize {
    if word.is_empty() {
        return 0;
    }
    match method {
        SyllableMethod::VowelGroup => vowel_groups(word),
        SyllableMethod::Dict { .. } => {
            #[cfg(feature = "verse")]
            {
                crate::text::dict::syllables(word).unwrap_or_else(|| vowel_groups(word))
            }
            #[cfg(not(feature = "verse"))]
            {
                // Unreachable in practice: `check_supported` rejects a `Dict`
                // reference on this build before any transform runs.
                vowel_groups(word)
            }
        }
    }
}

/// Vowel-group syllable estimate.
///
/// Runs of vowels count once. A terminal `e` is silent unless it is the only
/// vowel group (`the`) or follows a consonant + `le` (`table`). Never returns
/// zero for a non-empty word: a word with no vowels at all (`rhythm`, `nth`) is
/// still pronounced.
fn vowel_groups(word: &str) -> usize {
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    if chars.is_empty() {
        // Digits and symbols: one "syllable" so that a numeral does not make a
        // sentence look shorter than it reads.
        return usize::from(!word.is_empty());
    }
    let is_vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
    let mut count = 0usize;
    let mut previous_vowel = false;
    for &c in &chars {
        let vowel = is_vowel(c);
        if vowel && !previous_vowel {
            count += 1;
        }
        previous_vowel = vowel;
    }
    let n = chars.len();
    if n >= 2 && chars[n - 1] == 'e' && !is_vowel(chars[n - 2]) {
        // Silent terminal `e`, except in the `-Cle` pattern where it carries a
        // syllable of its own: `table`, `little`, `cycle`.
        let syllabic_le = n >= 3 && chars[n - 2] == 'l' && !is_vowel(chars[n - 3]);
        if !syllabic_le && count > 1 {
            count -= 1;
        }
    }
    count.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(word: &str) -> usize {
        count_syllables(word, &SyllableMethod::VowelGroup)
    }

    #[test]
    fn ordinary_words_count_correctly() {
        assert_eq!(n("cat"), 1);
        assert_eq!(n("water"), 2);
        assert_eq!(n("beautiful"), 3);
        assert_eq!(n("computer"), 3);
        assert_eq!(n("away"), 2);
        assert_eq!(n("the"), 1);
    }

    #[test]
    fn silent_terminal_e_is_dropped_but_syllabic_le_is_not() {
        assert_eq!(n("make"), 1);
        assert_eq!(n("hope"), 1);
        assert_eq!(n("table"), 2);
        assert_eq!(n("little"), 2);
        assert_eq!(n("cycle"), 2);
    }

    #[test]
    fn a_word_is_never_zero_syllables() {
        assert_eq!(n("rhythm"), 1);
        assert_eq!(n("nth"), 1);
        assert_eq!(n("42"), 1);
        assert_eq!(n(""), 0);
    }

    #[test]
    fn the_method_round_trips_and_defaults_to_vowel_group() {
        let method: SyllableMethod = serde_json::from_str(r#"{"method":"vowel_group"}"#).unwrap();
        assert_eq!(method, SyllableMethod::VowelGroup);
        assert_eq!(SyllableMethod::default(), SyllableMethod::VowelGroup);
        let dict = SyllableMethod::Dict {
            dict_version: "cmudict@0.7b-2026.08".into(),
        };
        let json = serde_json::to_string(&dict).unwrap();
        assert_eq!(serde_json::from_str::<SyllableMethod>(&json).unwrap(), dict);
    }

    #[test]
    fn vowel_group_is_always_supported() {
        assert!(SyllableMethod::VowelGroup.check_supported().is_ok());
        assert_eq!(
            SyllableMethod::VowelGroup.resolve_for_fit().unwrap(),
            SyllableMethod::VowelGroup
        );
    }

    #[cfg(not(feature = "verse"))]
    #[test]
    fn a_dict_reference_fails_loudly_on_a_build_without_the_dictionary() {
        // This is the invariant: refuse, never fall back. A silent downgrade
        // would make the same reference score differently per build.
        let dict = SyllableMethod::Dict {
            dict_version: "cmudict@0.7b-2026.08".into(),
        };
        let err = dict.check_supported().unwrap_err();
        assert!(format!("{err}").contains("verse"), "{err}");
        assert!(dict.resolve_for_fit().is_err());
    }

    #[cfg(feature = "verse")]
    #[test]
    fn a_dict_reference_is_honored_on_a_verse_build() {
        let dict = SyllableMethod::Dict {
            dict_version: crate::text::dict::DICT_VERSION.into(),
        };
        assert!(dict.check_supported().is_ok());
        // A pinned version this build does not carry is still an error.
        let stale = SyllableMethod::Dict {
            dict_version: "cmudict@0.0".into(),
        };
        assert!(stale.check_supported().is_err());
    }
}
