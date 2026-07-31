//! Syntax texture — the shape of the clause, not the meaning of the word.
//!
//! Shared across the Bukowski, Thompson and marketing axes, which is why it is
//! one family rather than three. The two fiction poles are a designed-in
//! contrast pair: both paratactic, profane and first-person, yet opposite on
//! sentence-length variance, intensifier density and punctuation exuberance. A
//! family that could not separate them would be measuring "informal register"
//! and calling it style.
//!
//! # The scarcity dimensions
//!
//! `syn:ly_adverb_rate`, `syn:sub_coord_ratio` and `syn:intensifier_rate` are
//! here for their **low** bands. Bukowski is characterized by what he does not
//! do — adverbs, subordination, intensifiers — and "write minimally" gives an
//! agent no target rate. A calibrated two-sided band does. Every ○ cell in the
//! coverage matrix is a claim of this shape, and it is only testable because
//! absence has a band like anything else.
//!
//! # What each detector gets wrong
//!
//! * `syn:fragment_rate` — a sentence with no finite verb. Closed-class
//!   auxiliary and modal lists plus inflection rules; noisy per sentence,
//!   reliable as a rate, which is all a profile needs. It over-claims on
//!   imperatives (which genuinely have no finite subject-verb pair) and
//!   under-claims on fragments headed by a bare past participle.
//! * `syn:allcaps_exclaim_rate` — ALL-CAPS words in exclamatory context,
//!   acronyms excluded. Overlaps deliberately with `punct:allcaps_rate`: that
//!   one counts every capitalized run, this one counts shouting. A pipeline
//!   with both double-weights the overlap; that is a documented trade, not an
//!   accident.
//! * `syn:person:*` vs `biber:pron:*` — the same closed classes counted twice.
//!   Enforcing "only one at fit" is not practical, so the rule is a
//!   documentation rule: **pick one family for pronouns per pipeline**, and
//!   `short_text_defaults` never includes both.
//! * `syn:rare_word_tail_rate` — the Bukowski anti-allusion axis. Needs a
//!   background [`CountPack`]; without one it is missing rather than zero,
//!   because "this author never reaches for a rare word" and "we have no
//!   frequency table" are different claims.
//!
//! # Escalation rhythm
//!
//! `syn:syllable_slope`, `syn:syllable_autocorr` and `syn:syllable_peak_ratio`
//! treat the per-sentence syllable count as a time series. Wills's reading of
//! Thompson's wave cadence — sentences that build and then break — is a claim
//! about the *shape of that series*, and a slope plus an autocorrelation is the
//! cheapest honest version of it.

use serde::{Deserialize, Serialize};

use super::{
    per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, PackLicense, Unit,
};
use crate::error::Result;
use crate::feature::pack::{CountPack, NormPack};
use crate::text::syllable::{count_syllables, SyllableMethod};
use crate::text::{Analysis, Span};
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Version of the closed-class lists in this module.
pub const SYNTAX_LISTS_VERSION: &str = "2026.08";

/// Auxiliaries and modals: a sentence containing one has a finite verb.
const AUXILIARIES: &[&str] = &[
    "am", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
    "did", "can", "could", "will", "would", "shall", "should", "may", "might", "must", "ought",
    "'s", "'re", "'ve", "'ll", "'d", "'m",
];

/// Subordinating conjunctions.
const SUBORDINATORS: &[&str] = &[
    "although",
    "though",
    "because",
    "since",
    "unless",
    "until",
    "while",
    "whilst",
    "whereas",
    "if",
    "whether",
    "before",
    "after",
    "once",
    "when",
    "whenever",
    "where",
    "wherever",
    "as",
    "than",
    "that",
    "who",
    "whom",
    "whose",
    "which",
    "lest",
    "provided",
    "supposing",
    "albeit",
];

/// Coordinating conjunctions.
const COORDINATORS: &[&str] = &["and", "but", "or", "nor", "for", "yet", "so"];

/// Words ending in `-ly` that are not manner adverbs.
const LY_STOPLIST: &[&str] = &[
    "only",
    "early",
    "family",
    "reply",
    "supply",
    "apply",
    "imply",
    "rely",
    "ugly",
    "silly",
    "holy",
    "jolly",
    "belly",
    "rally",
    "really",
    "ally",
    "bully",
    "fully",
    "hilly",
    "dolly",
    "folly",
    "gully",
    "jelly",
    "lily",
    "melancholy",
    "monopoly",
    "assembly",
    "italy",
    "july",
    "anomaly",
    "panoply",
    "multiply",
    "simply",
];

/// A small closed interjection list — the Thompson marker.
const INTERJECTIONS: &[&str] = &[
    "ah", "aha", "ahem", "alas", "argh", "aw", "bah", "blah", "boo", "bravo", "christ", "damn",
    "eh", "gee", "gosh", "ha", "hah", "hey", "hm", "hmm", "huh", "hurrah", "jeez", "jesus", "oh",
    "oho", "ooh", "oops", "ouch", "ow", "phew", "pff", "psh", "shh", "ugh", "uh", "um", "umm",
    "well", "whoa", "whoops", "wow", "yay", "yeah", "yeesh", "yikes", "yo", "yow",
];

/// Person deixis, LIWC-style. Duplicated with `biber:pron:*` by design — see
/// the module docs' dedupe rule.
const P1S: &[&str] = &[
    "i", "me", "my", "mine", "myself", "i'm", "i'll", "i've", "i'd",
];
const P2: &[&str] = &[
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "you're",
    "you'll",
    "you've",
];
const P1P: &[&str] = &[
    "we",
    "us",
    "our",
    "ours",
    "ourselves",
    "we're",
    "we'll",
    "we've",
    "let's",
];

/// A background frequency below which a word counts as rare.
///
/// One in a hundred thousand: rare enough that an ordinary paragraph contains
/// none, common enough that a literary vocabulary registers.
const RARE_THRESHOLD: f64 = 1e-5;

/// Fewest sentences before the escalation-rhythm series says anything.
const MIN_RHYTHM_SENTENCES: usize = 6;

/// Configuration for the syntax-texture family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SyntaxTexture {
    /// Booster weights, for the intensifier and chain dimensions.
    ///
    /// `None` uses the bundled VADER weights.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boosters: Option<NormPack>,
    /// Concreteness norms, for `syn:concreteness_mean`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concreteness: Option<NormPack>,
    /// A background word-frequency table, for `syn:rare_word_tail_rate`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<CountPack>,
    /// How syllables are counted for the escalation-rhythm series.
    #[serde(default)]
    pub syllables: SyllableMethod,
}

impl SyntaxTexture {
    /// Supply concreteness norms.
    pub fn with_concreteness(mut self, pack: NormPack) -> Self {
        self.concreteness = Some(pack);
        self
    }

    /// Supply a background frequency table for the rare-word tail.
    pub fn with_background(mut self, pack: CountPack) -> Self {
        self.background = Some(pack);
        self
    }
}

/// Fitted [`SyntaxTexture`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedSyntax {
    dims: Vec<DimInfo>,
    lists_version: String,
    fragment: Symbol,
    sub_coord: Symbol,
    initial_conj: Symbol,
    ly_adverb: Symbol,
    intensifier: Symbol,
    chain_mean: Symbol,
    chain_max: Symbol,
    interjection: Symbol,
    allcaps_exclaim: Symbol,
    polysyndeton: Symbol,
    asyndeton: Symbol,
    comma_chain: Symbol,
    person: Vec<Symbol>,
    concreteness_mean: Symbol,
    rare_tail: Symbol,
    rhythm: Vec<Symbol>,
    boosters: Vec<(String, f64)>,
    concreteness: Vec<(String, f64)>,
    background: Vec<(String, u64)>,
    background_total: u64,
    syllable_method: SyllableMethod,
    packs: Vec<PackLicense>,
}

impl FittedSyntax {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Syntax;

    /// Version of the closed-class lists this was fitted with.
    pub fn lists_version(&self) -> &str {
        &self.lists_version
    }
}

/// Person-deixis dimension names, in emission order.
const PERSON_DIMS: &[(&str, &[&str])] = &[("p1s", P1S), ("p2", P2), ("p1p", P1P)];

/// Escalation-rhythm dimension names, in emission order.
const RHYTHM_DIMS: &[&str] = &["syllable_slope", "syllable_autocorr", "syllable_peak_ratio"];

impl Feature for SyntaxTexture {
    type Fitted = FittedSyntax;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedSyntax> {
        let syllable_method = self.syllables.resolve_for_fit()?;
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Syntax, unit));
            sym
        };

        let fragment = push(interner, "syn:fragment_rate", Unit::PerHundredSentences);
        let sub_coord = push(interner, "syn:sub_coord_ratio", Unit::Index);
        let initial_conj = push(interner, "syn:initial_conj_rate", Unit::PerHundredSentences);
        let ly_adverb = push(interner, "syn:ly_adverb_rate", Unit::PerThousandTokens);
        let intensifier = push(interner, "syn:intensifier_rate", Unit::PerThousandTokens);
        let chain_mean = push(interner, "syn:booster_chain_mean", Unit::Index);
        let chain_max = push(interner, "syn:booster_chain_max", Unit::Index);
        let interjection = push(interner, "syn:interjection_rate", Unit::PerThousandTokens);
        let allcaps_exclaim = push(
            interner,
            "syn:allcaps_exclaim_rate",
            Unit::PerThousandTokens,
        );
        let polysyndeton = push(interner, "syn:polysyndeton_rate", Unit::PerHundredSentences);
        let asyndeton = push(interner, "syn:asyndeton_rate", Unit::PerHundredSentences);
        let comma_chain = push(interner, "syn:comma_chain_mean", Unit::Index);
        let person = PERSON_DIMS
            .iter()
            .map(|(name, _)| {
                push(
                    interner,
                    &format!("syn:person:{name}"),
                    Unit::PerThousandTokens,
                )
            })
            .collect();
        let concreteness_mean = push(interner, "syn:concreteness_mean", Unit::Index);
        let rare_tail = push(interner, "syn:rare_word_tail_rate", Unit::PerThousandTokens);
        let rhythm = RHYTHM_DIMS
            .iter()
            .map(|name| push(interner, &format!("syn:{name}"), Unit::Index))
            .collect();

        let mut packs: Vec<PackLicense> = Vec::new();
        let booster_pack = match &self.boosters {
            Some(pack) => pack.clone(),
            None => super::packs::vader_boosters(),
        };
        booster_pack.validate()?;
        packs.push(PackLicense {
            pack: booster_pack.qualified_name(),
            license: booster_pack.license.clone(),
            redistributable: booster_pack.redistributable,
        });

        let concreteness = match &self.concreteness {
            Some(pack) => {
                pack.validate()?;
                packs.push(PackLicense {
                    pack: pack.qualified_name(),
                    license: pack.license.clone(),
                    redistributable: pack.redistributable,
                });
                pack.table()
            }
            None => Vec::new(),
        };
        let (background, background_total) = match &self.background {
            Some(pack) => {
                pack.validate()?;
                packs.push(PackLicense {
                    pack: pack.qualified_name(),
                    license: pack.license.clone(),
                    redistributable: pack.redistributable,
                });
                (pack.table(), pack.effective_total())
            }
            None => (Vec::new(), 0),
        };

        Ok(FittedSyntax {
            dims,
            lists_version: SYNTAX_LISTS_VERSION.to_owned(),
            fragment,
            sub_coord,
            initial_conj,
            ly_adverb,
            intensifier,
            chain_mean,
            chain_max,
            interjection,
            allcaps_exclaim,
            polysyndeton,
            asyndeton,
            comma_chain,
            person,
            concreteness_mean,
            rare_tail,
            rhythm,
            boosters: booster_pack.table(),
            concreteness,
            background,
            background_total,
            syllable_method,
            packs,
        })
    }
}

impl FittedFeature for FittedSyntax {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Syntax
    }

    fn pack_licenses(&self) -> Vec<PackLicense> {
        self.packs.clone()
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        self.clause_shape(analysis, tokens, out);
        self.lexical_texture(analysis, tokens, out);
        self.catalogs(analysis, out);
        self.escalation(analysis, out);
    }
}

impl FittedSyntax {
    /// Fragments, parataxis, sentence-initial conjunctions.
    fn clause_shape(&self, analysis: &Analysis<'_>, _tokens: usize, out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let sentences = analysis.structure().sentences.len().max(1);
        let mut fragments = 0usize;
        let mut initial_conj = 0usize;
        let mut subordinators = 0usize;
        let mut coordinators = 0usize;

        for sentence in &analysis.structure().sentences {
            let words: Vec<&str> = stream.tokens()[sentence.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| stream.form(t))
                .collect();
            if words.is_empty() {
                continue;
            }
            if !has_finite_verb(&words) {
                fragments += 1;
                out.note_span(self.fragment, sentence.span);
            }
            if COORDINATORS.contains(&words[0]) {
                initial_conj += 1;
                out.note_span(self.initial_conj, sentence.span);
            }
            for word in &words {
                if SUBORDINATORS.contains(word) {
                    subordinators += 1;
                }
                if COORDINATORS.contains(word) {
                    coordinators += 1;
                }
            }
        }
        out.set(self.fragment, ratio(fragments, sentences) * 100.0);
        out.set(self.initial_conj, ratio(initial_conj, sentences) * 100.0);
        // Ratio, not two rates: the *balance* is the parataxis signal, and a
        // ratio is stable across document lengths in a way two rates are not.
        out.set(
            self.sub_coord,
            if coordinators == 0 {
                subordinators as f64
            } else {
                subordinators as f64 / coordinators as f64
            },
        );
    }

    /// Adverbs, intensifiers, interjections, shouting, deixis, concreteness,
    /// rare words.
    fn lexical_texture(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let mut ly = 0usize;
        let mut intensifiers = 0usize;
        let mut interjections = 0usize;
        let mut person = vec![0usize; PERSON_DIMS.len()];
        let mut concreteness: Vec<f64> = Vec::new();
        let mut rare = 0usize;
        let mut chains: Vec<usize> = Vec::new();
        let mut run = 0usize;

        for (token, form) in stream.lexical() {
            if is_ly_adverb(form) {
                ly += 1;
                out.note_span(self.ly_adverb, token.span);
            }
            let boosted = self.booster_of(form).is_some_and(|w| w > 0.0);
            if boosted {
                intensifiers += 1;
                out.note_span(self.intensifier, token.span);
                run += 1;
            } else {
                if run > 0 {
                    chains.push(run);
                }
                run = 0;
            }
            if INTERJECTIONS.contains(&form) {
                interjections += 1;
                out.note_span(self.interjection, token.span);
            }
            for (i, (_, list)) in PERSON_DIMS.iter().enumerate() {
                if list.contains(&form) {
                    person[i] += 1;
                }
            }
            if let Some(score) = self.concreteness_of(form) {
                concreteness.push(score);
            }
            if !self.background.is_empty() && self.is_rare(form) {
                rare += 1;
                out.note_span(self.rare_tail, token.span);
            }
        }
        if run > 0 {
            chains.push(run);
        }

        out.set(self.ly_adverb, per_1k(ly, tokens));
        out.set(self.intensifier, per_1k(intensifiers, tokens));
        out.set(self.interjection, per_1k(interjections, tokens));
        if chains.is_empty() {
            // No boosters at all: a chain length has no value, and zero would
            // read as "chains of length zero" rather than "no chains".
            out.mark_missing(self.chain_mean);
            out.mark_missing(self.chain_max);
        } else {
            let lengths: Vec<f64> = chains.iter().map(|&c| c as f64).collect();
            out.set(self.chain_mean, util::mean(&lengths));
            out.set(
                self.chain_max,
                chains.iter().copied().max().unwrap_or(0) as f64,
            );
        }
        for (i, &sym) in self.person.iter().enumerate() {
            out.set(sym, per_1k(person[i], tokens));
        }
        if self.concreteness.is_empty() || concreteness.len() < MIN_SCORED_WORDS {
            out.mark_missing(self.concreteness_mean);
        } else {
            out.set(self.concreteness_mean, util::mean(&concreteness));
        }
        if self.background.is_empty() {
            out.mark_missing(self.rare_tail);
        } else {
            out.set(self.rare_tail, per_1k(rare, tokens));
        }

        // Shouting: an ALL-CAPS word in a sentence that also carries an
        // exclamation mark. Acronyms are excluded by requiring length and a
        // vowel, which is what separates "SAVAGE" from "API".
        let mut shouts = 0usize;
        for sentence in &analysis.structure().sentences {
            let text = analysis.text(sentence.span);
            if !text.contains('!') {
                continue;
            }
            for token in &stream.tokens()[sentence.tokens.clone()] {
                if !token.kind.is_lexical() {
                    continue;
                }
                let raw = analysis.text(token.span);
                if is_shout(raw) {
                    shouts += 1;
                    out.note_span(self.allcaps_exclaim, token.span);
                }
            }
        }
        out.set(self.allcaps_exclaim, per_1k(shouts, tokens));
    }

    /// Polysyndeton, asyndeton, comma chains.
    fn catalogs(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let sentences = analysis.structure().sentences.len().max(1);
        let mut poly = 0usize;
        let mut asyn = 0usize;
        let mut comma_runs: Vec<f64> = Vec::new();

        for sentence in &analysis.structure().sentences {
            let toks: Vec<(&str, Span, bool)> = stream.tokens()[sentence.tokens.clone()]
                .iter()
                .map(|t| (stream.form(t), t.span, t.kind.is_lexical()))
                .collect();
            let conjunctions = toks
                .iter()
                .filter(|(f, _, lex)| *lex && matches!(*f, "and" | "or" | "nor"))
                .count();
            let commas = toks.iter().filter(|(f, _, _)| *f == ",").count();
            let lexical = toks.iter().filter(|(_, _, lex)| *lex).count();

            // Polysyndeton: three or more coordinators in one sentence, and
            // more coordinators than commas — "and this and that and the other"
            // rather than a comma list with one final "and".
            if conjunctions >= 3 && conjunctions > commas {
                poly += 1;
                out.note_span(self.polysyndeton, sentence.span);
            }
            // Asyndeton: a comma list of three or more with no coordinator at
            // all — "I came, I saw, I conquered".
            if commas >= 2 && conjunctions == 0 && lexical >= 6 {
                asyn += 1;
                out.note_span(self.asyndeton, sentence.span);
            }
            if commas > 0 {
                comma_runs.push(commas as f64);
            }
        }
        out.set(self.polysyndeton, ratio(poly, sentences) * 100.0);
        out.set(self.asyndeton, ratio(asyn, sentences) * 100.0);
        if comma_runs.is_empty() {
            out.set(self.comma_chain, 0.0);
        } else {
            out.set(self.comma_chain, util::mean(&comma_runs));
        }
    }

    /// The per-sentence syllable series and its shape.
    fn escalation(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let series: Vec<f64> = analysis
            .structure()
            .sentences
            .iter()
            .map(|s| {
                stream.tokens()[s.tokens.clone()]
                    .iter()
                    .filter(|t| t.kind.is_lexical())
                    .map(|t| count_syllables(stream.form(t), &self.syllable_method) as f64)
                    .sum::<f64>()
            })
            .filter(|&n| n > 0.0)
            .collect();

        if series.len() < MIN_RHYTHM_SENTENCES {
            for &sym in &self.rhythm {
                out.mark_missing(sym);
            }
            return;
        }
        let mean = util::mean(&series);
        out.set(self.rhythm[0], slope(&series));
        out.set(self.rhythm[1], autocorrelation(&series));
        let peak = series.iter().cloned().fold(f64::MIN, f64::max);
        out.set(self.rhythm[2], if mean > 0.0 { peak / mean } else { 0.0 });
    }

    fn booster_of(&self, form: &str) -> Option<f64> {
        self.boosters
            .binary_search_by(|(t, _)| t.as_str().cmp(form))
            .ok()
            .map(|i| self.boosters[i].1)
    }

    fn concreteness_of(&self, form: &str) -> Option<f64> {
        self.concreteness
            .binary_search_by(|(t, _)| t.as_str().cmp(form))
            .ok()
            .map(|i| self.concreteness[i].1)
    }

    /// Whether a word sits in the background's rare tail.
    ///
    /// A word the background has never seen counts as rare only if it is long
    /// enough to be a word rather than a typo or a token-splitting artifact.
    fn is_rare(&self, form: &str) -> bool {
        if form.chars().count() < 5 || crate::feature::lexicon::is_numeric(form) {
            return false;
        }
        let count = self
            .background
            .binary_search_by(|(t, _)| t.as_str().cmp(form))
            .map(|i| self.background[i].1)
            .unwrap_or(0);
        if self.background_total == 0 {
            return false;
        }
        (count as f64 / self.background_total as f64) < RARE_THRESHOLD
    }
}

/// Fewest scored words before a concreteness mean means anything.
const MIN_SCORED_WORDS: usize = 10;

/// Subjects after which an `-s` word is a verb rather than a plural noun.
///
/// "he barks" is finite; "no windows" is not, and a bare `-s` rule cannot tell
/// them apart. Requiring a pronoun-like subject immediately before is the
/// cheapest rule that gets the common cases right.
const S_SUBJECTS: &[&str] = &[
    "he",
    "she",
    "it",
    "who",
    "that",
    "which",
    "one",
    "everyone",
    "someone",
    "nobody",
    "everything",
    "something",
    "nothing",
    "this",
    "there",
];

/// Common irregular past forms, which no suffix rule can see.
#[rustfmt::skip]
const IRREGULAR_FINITE: &[&str] = &[
    "ate", "became", "began", "bent", "bit", "bled", "blew", "bore", "bought", "bound",
    "broke", "brought", "built", "burnt", "came", "caught", "chose", "clung", "cost", "cut",
    "dealt", "did", "drank", "drew", "drove", "dug", "fed", "fell", "felt", "fled", "flew",
    "forgave", "forgot", "fought", "found", "gave", "got", "grew", "held", "hid", "hit",
    "hung", "hurt", "kept", "knew", "laid", "led", "left", "lent", "let", "lit", "lost",
    "made", "meant", "met", "mistook", "paid", "put", "quit", "ran", "rang", "read", "rode",
    "rose", "sang", "sank", "sat", "saw", "said", "sent", "set", "shook", "shone", "shot",
    "showed", "shut", "slept", "slid", "sold", "sought", "sped", "spent", "spoke", "spun",
    "spread", "stood", "stole", "struck", "stuck", "sung", "swam", "swore", "took", "taught",
    "thought", "threw", "told", "tore", "understood", "went", "wept", "won", "wore", "wrote",
];

/// Whether a sentence contains a finite verb.
///
/// An auxiliary or modal settles it, as does a common irregular past form.
/// Otherwise the rule looks for `-ed`, or `-s` immediately after a
/// pronoun-like subject.
///
/// Documented failure modes: an imperative reads as a fragment (it has no
/// finite subject-verb pair, which is arguably right); a simple-present plural
/// with no auxiliary ("the machines scream") reads as a fragment; and a
/// fragment headed by a bare participle reads as a sentence. All three are the
/// same on the corpus side and the draft side, which is what a rate needs.
fn has_finite_verb(words: &[&str]) -> bool {
    if words
        .iter()
        .any(|w| AUXILIARIES.contains(w) || IRREGULAR_FINITE.contains(w))
    {
        return true;
    }
    for (i, w) in words.iter().enumerate() {
        let n = w.chars().count();
        if n > 3 && w.ends_with("ed") && !w.ends_with("eed") {
            return true;
        }
        if n > 3
            && w.ends_with('s')
            && !w.ends_with("ss")
            && !w.ends_with("us")
            && i > 0
            && S_SUBJECTS.contains(&words[i - 1])
        {
            return true;
        }
    }
    false
}

/// Whether a word is a manner adverb by the `-ly` rule.
fn is_ly_adverb(form: &str) -> bool {
    form.chars().count() > 4 && form.ends_with("ly") && !LY_STOPLIST.contains(&form)
}

/// Whether a raw token reads as shouting rather than as an acronym.
///
/// Three or more letters, all uppercase, containing a vowel. `API` and `SQL`
/// are excluded by the vowel rule and the length rule respectively; `SAVAGE`
/// and `NEVER` are not.
fn is_shout(raw: &str) -> bool {
    let letters: Vec<char> = raw.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() < 4 {
        return false;
    }
    if !letters.iter().all(|c| c.is_uppercase()) {
        return false;
    }
    letters
        .iter()
        .any(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
}

/// Least-squares slope of a series against its index, normalized by the mean.
///
/// Normalized so that a document of long sentences and one of short sentences
/// with the same *shape* score the same: the dimension is about escalation, not
/// about length.
fn slope(series: &[f64]) -> f64 {
    let n = series.len() as f64;
    let mean_x = (n - 1.0) / 2.0;
    let mean_y = util::mean(series);
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, &y) in series.iter().enumerate() {
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    if den <= f64::EPSILON || mean_y <= f64::EPSILON {
        return 0.0;
    }
    (num / den) / mean_y
}

/// Lag-1 Pearson autocorrelation.
fn autocorrelation(series: &[f64]) -> f64 {
    if series.len() < 3 {
        return 0.0;
    }
    let mean = util::mean(series);
    let denom: f64 = series.iter().map(|x| (x - mean).powi(2)).sum();
    if denom <= f64::EPSILON {
        return 0.0;
    }
    let numer: f64 = series[..series.len() - 1]
        .iter()
        .zip(&series[1..])
        .map(|(a, b)| (a - mean) * (b - mean))
        .sum();
    numer / denom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::packs;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn transform(spec: SyntaxTexture, text: &str) -> (FeatureVector, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform(SyntaxTexture::default(), text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    #[test]
    fn fragments_are_counted_exactly_on_the_fixture() {
        // Four sentences: two verbless fragments, two finite controls.
        let text = "No windows. The rain outside. The man walked home. It was cold.";
        assert!((value(text, "syn:fragment_rate") - 50.0).abs() < 1e-9);
        let controls = "The man walked home. It was cold. She opened the door. He waited.";
        assert_eq!(value(controls, "syn:fragment_rate"), 0.0);
    }

    #[test]
    fn fragment_spans_point_at_the_sentence() {
        let text = "No windows. The man walked home. It was cold. She waited.";
        let (v, i) = transform(SyntaxTexture::default(), text);
        let spans = v.spans(i.get("syn:fragment_rate").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(&text[spans[0].range()], "No windows.");
    }

    #[test]
    fn parataxis_shows_up_as_a_low_subordinator_ratio() {
        let paratactic = "The man walked home. It was cold and the rain fell. \
                          He opened the door and went inside.";
        let hypotactic = "Although the man walked home because it was cold, the rain \
                          which fell while he waited meant that he opened the door.";
        assert!(
            value(paratactic, "syn:sub_coord_ratio") < value(hypotactic, "syn:sub_coord_ratio")
        );
    }

    #[test]
    fn sentence_initial_conjunctions_are_counted_per_hundred() {
        let text = "And then it broke. But nobody noticed. The build was fine. It passed.";
        assert!((value(text, "syn:initial_conj_rate") - 50.0).abs() < 1e-9);
    }

    #[test]
    fn the_ly_adverb_rule_has_a_documented_stoplist() {
        assert!(is_ly_adverb("quietly"));
        assert!(is_ly_adverb("savagely"));
        assert!(!is_ly_adverb("only"));
        assert!(!is_ly_adverb("early"));
        assert!(!is_ly_adverb("family"));
        assert!(!is_ly_adverb("reply"));
        // Ten tokens, one adverb -> 100 per 1k.
        let text = "he walked quietly across the room and shut the door";
        assert!((value(text, "syn:ly_adverb_rate") - 100.0).abs() < 1e-9);
    }

    #[test]
    fn booster_chains_are_measured_and_missing_when_there_are_none() {
        // "absolutely completely totally" is a run of three.
        let text = "it was absolutely completely totally wrong and nobody cared at all";
        assert!((value(text, "syn:booster_chain_max") - 3.0).abs() < 1e-9);
        assert!(value(text, "syn:intensifier_rate") > 0.0);

        let (v, i) = transform(SyntaxTexture::default(), "the man walked home in the rain");
        assert!(
            v.is_missing(i.get("syn:booster_chain_max").unwrap()),
            "no boosters means no chain length, not a chain of length zero"
        );
    }

    #[test]
    fn shouting_is_separated_from_acronyms() {
        assert!(is_shout("SAVAGE"));
        assert!(is_shout("NEVER"));
        assert!(!is_shout("API"));
        assert!(!is_shout("SQL"));
        assert!(!is_shout("Savage"));
        // Only in exclamatory context.
        assert!(value("It was SAVAGE out there!", "syn:allcaps_exclaim_rate") > 0.0);
        assert_eq!(
            value("It was SAVAGE out there.", "syn:allcaps_exclaim_rate"),
            0.0
        );
    }

    #[test]
    fn polysyndeton_and_asyndeton_are_distinguished() {
        let poly = "There was blood and vomit and gin and the sound of the machines. \
                    It was quiet. Nothing happened. Fine.";
        assert!(value(poly, "syn:polysyndeton_rate") > 0.0);
        assert_eq!(value(poly, "syn:asyndeton_rate"), 0.0);

        let asyn = "I came, I saw, I conquered the whole thing. It was quiet. \
                    Nothing happened. Fine.";
        assert!(value(asyn, "syn:asyndeton_rate") > 0.0);
        assert_eq!(value(asyn, "syn:polysyndeton_rate"), 0.0);
    }

    #[test]
    fn catalog_spans_point_at_the_sentence() {
        let text = "There was blood and vomit and gin and the machines. It stopped.";
        let (v, i) = transform(SyntaxTexture::default(), text);
        let spans = v.spans(i.get("syn:polysyndeton_rate").unwrap());
        assert_eq!(
            &text[spans[0].range()],
            "There was blood and vomit and gin and the machines."
        );
    }

    #[test]
    fn the_rare_word_tail_needs_a_background_and_says_so_without_one() {
        let (v, i) = transform(SyntaxTexture::default(), "the atavistic swine returned");
        assert!(
            v.is_missing(i.get("syn:rare_word_tail_rate").unwrap()),
            "no background table means unknown, not zero"
        );

        let background = CountPack {
            name: "toy-bg".into(),
            version: "1".into(),
            date: "2026-08-01".into(),
            description: String::new(),
            license: "CC0-1.0".into(),
            redistributable: true,
            sources: vec![],
            entries: vec![
                ("the".into(), 500_000),
                ("returned".into(), 20_000),
                ("swine".into(), 3),
            ],
            total: 1_000_000,
        };
        let (v, i) = transform(
            SyntaxTexture::default().with_background(background),
            "the atavistic swine returned",
        );
        // "atavistic" is unseen and "swine" is below the threshold; "the" and
        // "returned" are common.
        let rate = v.get(i.get("syn:rare_word_tail_rate").unwrap());
        assert!((rate - 500.0).abs() < 1e-9, "{rate}");
    }

    #[test]
    fn concreteness_needs_its_norms() {
        let text = "the brick the chair the table the window the door the hammer the shoe \
                    the dog the cat the tree";
        let (v, i) = transform(SyntaxTexture::default(), text);
        assert!(v.is_missing(i.get("syn:concreteness_mean").unwrap()));

        let (v, i) = transform(
            SyntaxTexture::default().with_concreteness(packs::concreteness_stub()),
            text,
        );
        assert!(v.get(i.get("syn:concreteness_mean").unwrap()) > 4.0);
    }

    #[test]
    fn escalation_rhythm_reads_the_shape_of_the_series() {
        // Six sentences of growing length: a positive slope.
        let building = "One. Two words here. Three more words here now. \
                        Four and more words here again now. \
                        Five and yet more words are here again now today. \
                        Six and even more words are here again now today as well.";
        assert!(value(building, "syn:syllable_slope") > 0.0);
        // The same sentences reversed: a negative slope.
        let falling = "Six and even more words are here again now today as well. \
                       Five and yet more words are here again now today. \
                       Four and more words here again now. Three more words here now. \
                       Two words here. One.";
        assert!(value(falling, "syn:syllable_slope") < 0.0);
    }

    #[test]
    fn escalation_dims_are_missing_on_a_short_document() {
        let (v, i) = transform(SyntaxTexture::default(), "One. Two. Three.");
        assert!(v.is_missing(i.get("syn:syllable_slope").unwrap()));
    }

    #[test]
    fn the_bukowski_thompson_contrast_separates_on_the_designed_axes() {
        // The unit-scale prototype of the W8 battery golden test. Two synthetic
        // registers, opposite on exactly the axes the coverage matrix claims.
        let bukowski = "The man drank. No windows. The rain outside. He slept on the floor. \
                        The dog barked. Nothing moved. He got up and went out. \
                        The bar was open. He sat down. The beer was cold.";
        let thompson = "There was blood and vomit and gin and the sound of the machines! \
                        It was ABSOLUTELY savage out there and the whole thing was completely \
                        totally insane and the machines screamed and nobody moved! \
                        We were somewhere around Barstow when the drugs took hold. \
                        Jesus, it was wild!";

        // Bukowski: fragments high, adverbs and intensifiers absent.
        assert!(value(bukowski, "syn:fragment_rate") > value(thompson, "syn:fragment_rate"));
        assert!(value(bukowski, "syn:intensifier_rate") < value(thompson, "syn:intensifier_rate"));
        // Thompson: shouting, polysyndeton, interjections.
        assert!(value(thompson, "syn:allcaps_exclaim_rate") > 0.0);
        assert_eq!(value(bukowski, "syn:allcaps_exclaim_rate"), 0.0);
        assert!(
            value(thompson, "syn:polysyndeton_rate") > value(bukowski, "syn:polysyndeton_rate")
        );
        assert!(
            value(thompson, "syn:interjection_rate") > value(bukowski, "syn:interjection_rate")
        );
    }

    #[test]
    fn person_deixis_is_split_three_ways() {
        let text = "i think you should ask us about it";
        assert!(value(text, "syn:person:p1s") > 0.0);
        assert!(value(text, "syn:person:p2") > 0.0);
        assert!(value(text, "syn:person:p1p") > 0.0);
    }

    #[test]
    fn the_spec_round_trips_and_defaults_to_the_bundled_boosters() {
        let spec: SyntaxTexture = serde_json::from_str("{}").unwrap();
        assert_eq!(spec, SyntaxTexture::default());
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<SyntaxTexture>(&json).unwrap(), spec);

        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        assert_eq!(fitted.lists_version(), SYNTAX_LISTS_VERSION);
        assert!(fitted
            .pack_licenses()
            .iter()
            .any(|l| l.pack.starts_with("vader-boosters@")));
    }

    #[test]
    fn empty_text_marks_everything_missing() {
        let (v, i) = transform(SyntaxTexture::default(), "");
        assert!(v.is_missing(i.get("syn:fragment_rate").unwrap()));
    }
}
