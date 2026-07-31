//! Biber Tier-1 lexico-grammatical rates — the register backbone, no tagger.
//!
//! Biber's (1988) multidimensional analysis factors 67 lexico-grammatical
//! features into interpretable dimensions: D1 involved↔informational, D2
//! narrative, D3 explicit reference, D4 overt persuasion, D5 abstract, D6
//! on-line elaboration. About half of the features need nothing but closed-class
//! word lists and suffix rules, and that half covers D1 and D4 outright. Those
//! are what this module counts. Tier 2 — tense/aspect, agentless passives,
//! that-deletion, attributive adjectives — needs POS tags and is deliberately
//! deferred; a feature-gated perceptron tagger is the clean path later, and v1
//! does not need it.
//!
//! **Double duty as a de-AI family.** Reinhart et al. (PNAS 2025, HAP-E corpus)
//! found the LLM signature *in Biber features and consistent across registers*:
//! 2–5× more present-participial clauses, 1.5–2× more nominalizations, more
//! phrasal coordination and longer words; about half the agentless passives, far
//! fewer contractions and first-person pronouns. Three of those are counted
//! here directly, and the participial-clause finding has an honest no-parser
//! proxy (a sentence-final `, VERBing …` clause) which is what
//! `biber:participial_clause_rate` measures.
//!
//! # Where the lists come from
//!
//! Re-curated from the *published feature definitions* — Biber 1988 Appendix II,
//! the Lancaster MDA tables, and the documented category memberships that every
//! implementation agrees on. Nothing is vendored from `biberpy`, which is
//! GPL-3.0: word lists compiled by hand from a published taxonomy are not that
//! project's code, and the distinction only survives if the re-curation is real.
//! The lists are closed-class and do not decay the way an AI-ism lexicon does,
//! so they live in code with a version string rather than in a dated pack —
//! the same call [`FUNCTION_WORDS`](crate::feature::mfw::FUNCTION_WORDS) makes.
//!
//! # Deliberate omissions
//!
//! * **Type-token ratio** — [`Richness`](crate::feature::Richness) owns it, and
//!   owns it better (MTLD and MATTR are length-corrected; a raw TTR is not).
//! * **Contraction rate** — [`PunctTypography`](crate::feature::PunctTypography)
//!   already emits `punct:contraction_rate`. Counting it twice would
//!   double-weight it in every distance.

use serde::{Deserialize, Serialize};

use super::{per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::Result;
use crate::text::{Analysis, Span};
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Version of the closed-class lists in this module.
///
/// Bump it when a list changes, so a refit is distinguishable from a reload.
pub const BIBER_LISTS_VERSION: &str = "2026.08";

/// Verbs of private mental state (Biber's "private verbs"): the D1 involved
/// pole. Speech about thinking rather than about the world.
#[rustfmt::skip]
const PRIVATE_VERBS: &[&str] = &[
    "anticipate", "assume", "believe", "conclude", "decide", "demonstrate", "determine",
    "discover", "doubt", "estimate", "fear", "feel", "find", "forget", "guess", "hear",
    "hope", "imagine", "imply", "indicate", "infer", "know", "learn", "mean", "notice",
    "prove", "realize", "recall", "recognize", "remember", "reveal", "see", "show", "suppose",
    "suspect", "think", "understand", "wish", "worry", "assumed", "believed", "concluded",
    "decided", "felt", "found", "guessed", "heard", "hoped", "imagined", "knew", "known",
    "learned", "meant", "noticed", "realized", "remembered", "saw", "seen", "showed", "shown",
    "supposed", "suspected", "thought", "understood", "wished", "worried", "believes",
    "feels", "finds", "knows", "means", "notices", "realizes", "remembers", "sees", "shows",
    "supposes", "suspects", "thinks", "understands",
];

/// Verbs of public speech acts: reported speech, the D2 narrative marker.
#[rustfmt::skip]
const PUBLIC_VERBS: &[&str] = &[
    "admit", "admits", "admitted", "agree", "agreed", "agrees", "announce", "announced",
    "announces", "argue", "argued", "argues", "assert", "asserted", "claim", "claimed",
    "claims", "complain", "complained", "concede", "conceded", "confess", "confessed",
    "contend", "contended", "declare", "declared", "declares", "deny", "denied", "denies",
    "explain", "explained", "explains", "hint", "hinted", "insist", "insisted", "insists",
    "mention", "mentioned", "mentions", "object", "objected", "predict", "predicted",
    "proclaim", "proclaimed", "promise", "promised", "promises", "remark", "remarked",
    "reply", "replied", "report", "reported", "reports", "respond", "responded", "say",
    "said", "says", "state", "stated", "states", "suggest", "suggested", "suggests", "swear",
    "swore", "tell", "told", "warn", "warned", "write", "wrote", "writes",
];

/// Verbs that urge an action: Biber's suasive class, the D4 persuasion pole.
#[rustfmt::skip]
const SUASIVE_VERBS: &[&str] = &[
    "agree", "arrange", "ask", "beg", "command", "commanded", "commands", "concede", "decide",
    "decided", "decides", "demand", "demanded", "demands", "grant", "insist", "insisted",
    "instruct", "instructed", "move", "ordain", "order", "ordered", "orders", "pledge",
    "pledged", "pronounce", "propose", "proposed", "proposes", "recommend", "recommended",
    "recommends", "request", "requested", "requests", "require", "required", "requires",
    "stipulate", "suggest", "suggested", "suggests", "urge", "urged", "urges", "vote",
    "voted",
];

/// Modals of possibility, necessity and prediction — three separate Biber
/// features because they load on different dimensions.
const POSSIBILITY_MODALS: &[&str] = &["can", "may", "might", "could"];
const NECESSITY_MODALS: &[&str] = &["ought", "should", "must"];
const PREDICTION_MODALS: &[&str] = &["will", "would", "shall", "'ll", "'d"];

/// Conjuncts: explicit logical connectors. The D3/D5 explicit-reference marker,
/// and the LLM-prose tell everyone notices before they can name it.
#[rustfmt::skip]
const CONJUNCTS: &[&str] = &[
    "alternatively", "consequently", "conversely", "eg", "furthermore", "hence", "however",
    "instead", "likewise", "moreover", "namely", "nevertheless", "nonetheless",
    "notwithstanding", "otherwise", "rather", "similarly", "therefore", "thus", "viz",
    "accordingly", "additionally", "meanwhile", "subsequently", "ultimately", "overall",
];

/// Amplifiers: boosters that scale a gradable term up.
#[rustfmt::skip]
const AMPLIFIERS: &[&str] = &[
    "absolutely", "altogether", "completely", "enormously", "entirely", "extremely", "fully",
    "greatly", "highly", "intensely", "perfectly", "strongly", "thoroughly", "totally",
    "utterly", "very", "really", "incredibly", "immensely", "hugely", "vastly", "profoundly",
    "deeply",
];

/// Emphatics: markers that assert the truth of a proposition rather than
/// scaling it.
#[rustfmt::skip]
const EMPHATICS: &[&str] = &[
    "just", "really", "most", "more", "so", "such", "sure", "surely", "certainly",
    "definitely", "obviously", "clearly", "indeed", "truly", "actually", "literally",
];

/// Downtoners: the opposite pole, scaling a gradable term down.
#[rustfmt::skip]
const DOWNTONERS: &[&str] = &[
    "almost", "barely", "hardly", "merely", "mildly", "nearly", "only", "partially", "partly",
    "practically", "scarcely", "slightly", "somewhat", "sort", "kind", "rather", "quite",
];

/// Discourse particles: the spoken-register D1 marker.
#[rustfmt::skip]
const DISCOURSE_PARTICLES: &[&str] = &[
    "well", "now", "anyway", "anyhow", "anyways", "ok", "okay", "right", "like", "see",
    "look",
];

/// Person deixis, split the way Biber splits it.
#[rustfmt::skip]
const P1_PRONOUNS: &[&str] = &[
    "i", "me", "my", "mine", "myself", "we", "us", "our", "ours", "ourselves",
];
#[rustfmt::skip]
const P2_PRONOUNS: &[&str] = &[
    "you", "your", "yours", "yourself", "yourselves", "thee", "thy", "thine",
];
#[rustfmt::skip]
const P3_PRONOUNS: &[&str] = &[
    "he", "him", "his", "himself", "she", "her", "hers", "herself", "they", "them", "their",
    "theirs", "themselves",
];
const DEMONSTRATIVE_PRONOUNS: &[&str] = &["this", "that", "these", "those"];
#[rustfmt::skip]
const INDEFINITE_PRONOUNS: &[&str] = &[
    "anybody", "anyone", "anything", "everybody", "everyone", "everything", "nobody", "none",
    "nothing", "nowhere", "somebody", "someone", "something",
];

/// Analytic negation is `not`/`n't`; synthetic negation is the `no`/`neither`/
/// `never` family. Biber separates them because they load oppositely.
#[rustfmt::skip]
const SYNTHETIC_NEGATION: &[&str] = &[
    "no", "neither", "nor", "never", "none", "nothing", "nowhere",
];

/// Nominalization suffixes, in Biber's order.
#[rustfmt::skip]
const NOMINALIZATION_SUFFIXES: &[&str] = &[
    "tion", "tions", "ment", "ments", "ness", "nesses", "ity", "ities",
];

/// Words the nominalization suffix rule would otherwise claim.
///
/// The rule is a suffix test, so every short word ending in `-tion` or `-ity`
/// that is not derived from a verb or adjective has to be listed. `nation` and
/// `station` are the canonical failures — both end in `-tion`, neither is a
/// nominalization — and both are common enough to bias the rate on ordinary
/// prose. `city`, `pity` and `unity` are the `-ity` equivalents.
#[rustfmt::skip]
const NOMINALIZATION_STOPLIST: &[&str] = &[
    "nation", "nations", "station", "stations", "mention", "mentions", "question",
    "questions", "motion", "motions", "notion", "notions", "portion", "portions", "ration",
    "rations", "caution", "cautions", "fiction", "fictions", "friction", "auction",
    "auctions", "option", "options", "potion", "potions", "lotion", "lotions", "city",
    "cities", "pity", "unity", "entity", "entities", "deity", "deities", "parity", "cavity",
    "cavities", "moment", "moments", "comment", "comments", "element", "elements", "segment",
    "segments", "cement", "garment", "garments", "instrument", "instruments", "monument",
    "monuments", "ornament", "ornaments", "regiment", "regiments", "sediment", "witness",
    "witnesses", "business", "businesses", "harness", "wilderness", "fortnight",
];

/// Configuration for the Biber Tier-1 family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiberTier1 {
    /// Emit `biber:participial_clause_rate`, the PNAS regex proxy for
    /// present-participial clauses.
    ///
    /// On by default because it is the single strongest published LLM-vs-human
    /// Biber delta. Switch it off where the proxy's false positives (a
    /// sentence-final gerund complement) would matter more than the signal.
    #[serde(default = "crate::util::yes")]
    pub participial_proxy: bool,
}

impl Default for BiberTier1 {
    fn default() -> Self {
        BiberTier1 {
            participial_proxy: true,
        }
    }
}

/// Dimension names, in emission order. One closed-class counter each.
const CLASS_DIMS: &[(&str, &[&str])] = &[
    ("private_verb_rate", PRIVATE_VERBS),
    ("public_verb_rate", PUBLIC_VERBS),
    ("suasive_verb_rate", SUASIVE_VERBS),
    ("modal:possibility", POSSIBILITY_MODALS),
    ("modal:necessity", NECESSITY_MODALS),
    ("modal:prediction", PREDICTION_MODALS),
    ("conjunct_rate", CONJUNCTS),
    ("amplifier_rate", AMPLIFIERS),
    ("emphatic_rate", EMPHATICS),
    ("downtoner_rate", DOWNTONERS),
    ("discourse_particle_rate", DISCOURSE_PARTICLES),
    ("pron:p1", P1_PRONOUNS),
    ("pron:p2", P2_PRONOUNS),
    ("pron:p3", P3_PRONOUNS),
    ("pron:demonstrative", DEMONSTRATIVE_PRONOUNS),
    ("pron:indefinite", INDEFINITE_PRONOUNS),
    ("synthetic_negation_rate", SYNTHETIC_NEGATION),
];

/// Fitted [`BiberTier1`].
///
/// The fit is trivial — the dimension set is fixed and the lists are consts —
/// which is the point: no corpus-relative state means nothing to go stale
/// between the corpus and a draft.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedBiber {
    dims: Vec<DimInfo>,
    lists_version: String,
    /// One symbol per entry of [`CLASS_DIMS`].
    classes: Vec<Symbol>,
    analytic_negation: Symbol,
    nominalization: Symbol,
    participial: Option<Symbol>,
    mean_word_len: Symbol,
    d1_involved: Symbol,
    d4_persuasion: Symbol,
}

impl FittedBiber {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Register;

    /// Version of the closed-class lists this was fitted with.
    pub fn lists_version(&self) -> &str {
        &self.lists_version
    }
}

impl Feature for BiberTier1 {
    type Fitted = FittedBiber;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedBiber> {
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Register, unit));
            sym
        };

        let classes: Vec<Symbol> = CLASS_DIMS
            .iter()
            .map(|(name, _)| push(interner, &format!("biber:{name}"), Unit::PerThousandTokens))
            .collect();
        let analytic_negation = push(
            interner,
            "biber:analytic_negation_rate",
            Unit::PerThousandTokens,
        );
        let nominalization = push(
            interner,
            "biber:nominalization_rate",
            Unit::PerThousandTokens,
        );
        let participial = self.participial_proxy.then(|| {
            push(
                interner,
                "biber:participial_clause_rate",
                Unit::PerThousandTokens,
            )
        });
        let mean_word_len = push(interner, "biber:mean_word_len", Unit::Index);

        // The composites are informational: they summarize dimensions that are
        // already reported individually, so reporting them too would tell an
        // agent to "reduce your involvedness by 30%" with nothing to act on.
        let mut push_rollup = |interner: &mut Interner, name: &str| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Register, Unit::Index).rollup());
            sym
        };
        let d1_involved = push_rollup(interner, "biber:d1_involved");
        let d4_persuasion = push_rollup(interner, "biber:d4_persuasion");

        Ok(FittedBiber {
            dims,
            lists_version: BIBER_LISTS_VERSION.to_owned(),
            classes,
            analytic_negation,
            nominalization,
            participial,
            mean_word_len,
            d1_involved,
            d4_persuasion,
        })
    }
}

impl FittedFeature for FittedBiber {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Register
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }

        let mut counts = vec![0usize; CLASS_DIMS.len()];
        let mut analytic = 0usize;
        let mut nominalizations = 0usize;
        let mut char_total = 0usize;

        for (token, form) in analysis.tokens().lexical() {
            char_total += form.chars().count();
            for (i, (_, list)) in CLASS_DIMS.iter().enumerate() {
                if list.contains(&form) {
                    counts[i] += 1;
                    out.note_span(self.classes[i], token.span);
                }
            }
            if form == "not" || form == "n't" || form == "'t" {
                analytic += 1;
                out.note_span(self.analytic_negation, token.span);
            }
            if is_nominalization(form) {
                nominalizations += 1;
                out.note_span(self.nominalization, token.span);
            }
        }

        for (i, &sym) in self.classes.iter().enumerate() {
            out.set(sym, per_1k(counts[i], tokens));
        }
        out.set(self.analytic_negation, per_1k(analytic, tokens));
        out.set(self.nominalization, per_1k(nominalizations, tokens));
        out.set(self.mean_word_len, ratio(char_total, tokens));

        if let Some(sym) = self.participial {
            let mut participials = 0usize;
            for span in participial_clauses(analysis) {
                participials += 1;
                out.note_span(sym, span);
            }
            out.set(sym, per_1k(participials, tokens));
        }

        // D1 involved↔informational: person deixis, private verbs, discourse
        // particles and emphatics pull toward involved; long words and
        // nominalizations pull toward informational.
        let rate = |i: usize| per_1k(counts[i], tokens);
        let involved = rate(11) + rate(0) + rate(10) + rate(8);
        let informational = per_1k(nominalizations, tokens) + ratio(char_total, tokens) * 10.0;
        out.set(self.d1_involved, involved - informational);
        // D4 overt persuasion: suasive verbs, necessity and prediction modals,
        // plus the second-person address that carries a directive.
        out.set(
            self.d4_persuasion,
            rate(2) + rate(4) + rate(5) + rate(12) * 0.5,
        );
    }
}

/// Whether a word is a suffix-rule nominalization.
///
/// Documented bias: this is a morphological test with no lexicon behind it, so
/// it claims every `-tion`/`-ment`/`-ness`/`-ity` word that is not on
/// [`NOMINALIZATION_STOPLIST`]. The residual false positives are rare words
/// (`ration`, `sedition`) and the residual false negatives are irregular
/// nominalizations (`analysis`, `growth`, `belief`). Both are *consistent*
/// across the reference corpus and the draft, which is the property a
/// comparative rate needs.
fn is_nominalization(form: &str) -> bool {
    if form.chars().count() < 6 {
        return false;
    }
    if NOMINALIZATION_STOPLIST.contains(&form) {
        return false;
    }
    NOMINALIZATION_SUFFIXES.iter().any(|s| form.ends_with(s))
}

/// Sentence-final `, VERBing …` clauses — the no-parser proxy for present
/// participial clauses.
///
/// Reinhart et al. report 2–5× more of these in LLM prose than in human prose
/// across every register they tested, which makes it the highest-value single
/// dimension in this module. The proxy fires on a comma followed by an `-ing`
/// word in the last half of a sentence: it misses clause-initial participials
/// ("Having said that, …") and over-fires on a coordinated gerund object
/// ("she liked swimming, running and cycling"). Both errors are the same on the
/// reference corpus and on the draft.
fn participial_clauses(analysis: &Analysis<'_>) -> Vec<Span> {
    let stream = analysis.tokens();
    let mut out = Vec::new();
    for sentence in &analysis.structure().sentences {
        let toks = &stream.tokens()[sentence.tokens.clone()];
        if toks.len() < 4 {
            continue;
        }
        // Only the tail half counts: a clause-medial `, VERBing` is usually a
        // list item, and a sentence-final one is the construction of interest.
        let midpoint = toks.len() / 2;
        for i in midpoint..toks.len().saturating_sub(1) {
            if stream.form(&toks[i]) != "," {
                continue;
            }
            let next = &toks[i + 1];
            if !next.kind.is_lexical() {
                continue;
            }
            let form = stream.form(next);
            if is_ing_verb(form) {
                out.push(Span::new(toks[i].span.start, sentence.span.end));
                break;
            }
        }
    }
    out
}

/// Words ending in `-ing` that are plausibly verbal.
///
/// Excludes the short nouns that happen to end in `-ing` (`thing`, `king`,
/// `ring`, `string`) — without the exclusion "the thing, thing after thing"
/// style prose would register as participial.
fn is_ing_verb(form: &str) -> bool {
    const ING_NOUNS: &[&str] = &[
        "thing",
        "things",
        "king",
        "kings",
        "ring",
        "rings",
        "string",
        "strings",
        "wing",
        "wings",
        "spring",
        "springs",
        "ceiling",
        "morning",
        "evening",
        "everything",
        "something",
        "anything",
        "nothing",
        "during",
        "sibling",
        "siblings",
        "shilling",
        "willing",
        "darling",
    ];
    form.len() > 4 && form.ends_with("ing") && !ING_NOUNS.contains(&form)
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
        let fitted = BiberTier1::default().fit(&ctx, &mut interner).unwrap();
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

    #[test]
    fn closed_class_rates_are_exact_per_thousand() {
        // Ten lexical tokens, two private verbs -> 200 per 1k.
        let text = "i think you know the answer to that one now";
        assert!((value(text, "biber:private_verb_rate") - 200.0).abs() < 1e-9);
        // "i" and "you" are one first-person and one second-person pronoun.
        assert!((value(text, "biber:pron:p1") - 100.0).abs() < 1e-9);
        assert!((value(text, "biber:pron:p2") - 100.0).abs() < 1e-9);
    }

    #[test]
    fn negation_is_split_into_analytic_and_synthetic() {
        let analytic = value(
            "i do not agree with any of that at all",
            "biber:analytic_negation_rate",
        );
        assert!(analytic > 0.0);
        let synthetic = value(
            "no one ever agrees and never will here",
            "biber:synthetic_negation_rate",
        );
        assert!(synthetic > 0.0);
        assert_eq!(
            value(
                "no one ever agrees and never will here",
                "biber:analytic_negation_rate"
            ),
            0.0
        );
    }

    #[test]
    fn the_nominalization_suffix_rule_handles_its_stoplist() {
        // `nation` and `station` end in -tion and are not nominalizations.
        assert!(!is_nominalization("nation"));
        assert!(!is_nominalization("station"));
        assert!(!is_nominalization("city"));
        assert!(!is_nominalization("moment"));
        // Genuine derivations are claimed.
        assert!(is_nominalization("optimization"));
        assert!(is_nominalization("deployment"));
        assert!(is_nominalization("robustness"));
        assert!(is_nominalization("complexity"));
        // Too short to be a derived form at all.
        assert!(!is_nominalization("tion"));

        // Five lexical tokens, one nominalization -> 200 per 1k.
        let text = "the nation saw one optimization";
        assert!((value(text, "biber:nominalization_rate") - 200.0).abs() < 1e-9);
    }

    #[test]
    fn the_participial_proxy_fires_on_sentence_final_ing_clauses() {
        let text = "The system writes the record to disk, ensuring the update survives a crash.";
        assert!(value(text, "biber:participial_clause_rate") > 0.0);
        // No comma, no clause.
        let plain = "The system writes the record to disk and the update survives a crash.";
        assert_eq!(value(plain, "biber:participial_clause_rate"), 0.0);
        // `-ing` nouns are not verbs.
        let noun = "We looked at the whole system again, thing after thing after thing.";
        assert_eq!(value(noun, "biber:participial_clause_rate"), 0.0);
    }

    #[test]
    fn participial_spans_cover_the_clause() {
        let text = "The system writes the record to disk, ensuring the update survives a crash.";
        let (v, i) = transform(text);
        let spans = v.spans(i.get("biber:participial_clause_rate").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(
            &text[spans[0].range()],
            ", ensuring the update survives a crash."
        );
    }

    #[test]
    fn closed_class_hits_carry_spans() {
        let text = "however we think this matters quite a lot";
        let (v, i) = transform(text);
        let spans = v.spans(i.get("biber:conjunct_rate").unwrap());
        assert_eq!(&text[spans[0].range()], "however");
    }

    #[test]
    fn mean_word_length_is_characters_per_lexical_token() {
        // "aa bb cc" -> 6 chars over 3 tokens.
        assert!((value("aa bb cc", "biber:mean_word_len") - 2.0).abs() < 1e-9);
    }

    #[test]
    fn composites_are_rollups_and_never_reportable() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = BiberTier1::default().fit(&ctx, &mut interner).unwrap();
        for name in ["biber:d1_involved", "biber:d4_persuasion"] {
            let sym = interner.get(name).unwrap();
            let dim = fitted.dims().iter().find(|d| d.symbol == sym).unwrap();
            assert!(dim.aggregate, "{name} must be a rollup");
        }
        // And everything else must not be.
        let reportable = fitted.dims().iter().filter(|d| !d.aggregate).count();
        assert_eq!(reportable, fitted.dims().len() - 2);
    }

    #[test]
    fn involved_prose_scores_above_informational_prose_on_d1() {
        let involved = value(
            "well i think you know i really felt like it was fine anyway",
            "biber:d1_involved",
        );
        let informational = value(
            "the implementation of the optimization demonstrates considerable improvement",
            "biber:d1_involved",
        );
        assert!(
            involved > informational,
            "involved={involved} informational={informational}"
        );
    }

    #[test]
    fn empty_text_marks_everything_missing_rather_than_zero() {
        let (v, i) = transform("");
        assert!(v.is_missing(i.get("biber:private_verb_rate").unwrap()));
    }

    #[test]
    fn the_spec_round_trips_through_serde() {
        let spec = BiberTier1::default();
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<BiberTier1>(&json).unwrap(), spec);
        // Absent flags default on, so an older spec loads unchanged.
        assert_eq!(
            serde_json::from_str::<BiberTier1>("{}").unwrap(),
            BiberTier1::default()
        );
    }

    #[test]
    fn switching_the_participial_proxy_off_removes_its_dimension() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = BiberTier1 {
            participial_proxy: false,
        }
        .fit(&ctx, &mut interner)
        .unwrap();
        assert!(interner.get("biber:participial_clause_rate").is_none());
        assert!(fitted.participial.is_none());
    }
}
