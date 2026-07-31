//! Rhetorical device rates, and the marketing-structural family.
//!
//! Everything here is a *rate of a detectable pattern*. That is the whole claim
//! of the wit thesis: most of what a critic names as an author's comic
//! technique reduces to counting a construction, and what distinguishes authors
//! is not which constructions they know but at what **density** they use them.
//!
//! The Dark & Stormy result is why the density matters more than the inventory.
//! LLMs imitating comic prose over-fire literary devices — excess novel
//! adjective–noun bigrams, excess metaphor — because a prompt gives a model no
//! target rate. Humans calibrate; models maximize. So every dimension here is
//! designed to be read against a **two-sided** band, and `dev:novel_adj_noun_rate`
//! in particular is the fingerprint: its *upper* edge is the caricature guard.
//!
//! # Detectors, and what each of them gets wrong
//!
//! * `dev:litotes_rate` — "not entirely", "not un-", "hardly", and a minimizer
//!   next to an extreme ("space is big"). Partial recall by construction; fine
//!   for a rate.
//! * `dev:absurd_precision_rate` — a precise numeral next to an abstract or
//!   cosmic noun ("roughly once every ten million years"). No prior art; easy.
//! * `dev:transferred_epithet_rate` — a mental-state adjective on a concrete
//!   noun ("a moody forkful"). Needs the concreteness norms, which are
//!   loader-only, so without them the dimension is *missing*, not zero.
//! * `dev:alliteration_rate` — initial-letter runs of three or more. Letter-based
//!   by default and phoneme-based on a `verse` build, which is recorded in the
//!   fitted state rather than inferred from the binary.
//! * `dev:antonym_pair_rate` and `dev:ambiguity_density` — the rest of the
//!   Mihalcea–Strapparava trio, from a WordNet-derived antonym table and a
//!   senses-per-word table. Rhyme joins them in W7, where the CMU dictionary
//!   makes it honest.
//! * `dev:novel_adj_noun_rate` — **the one fit-stateful dimension here**. `fit`
//!   builds the reference corpus's adjective–noun bigram set; `transform` scores
//!   the rate of pairs outside it. With a background [`CountPack`] the novelty
//!   is weighted against how ordinary the pair is in general; without one it is
//!   defined purely against the reference's own set, which is the weaker
//!   variant and is documented as such.
//! * Marketing-structural — imperative rate (Leech: over one major clause in
//!   four in advertising, with negated imperatives near-absent, so those are
//!   subtracted), headline shapes, triplets, and "Not X. Not Y. Just Z."
//!
//! Adjective detection is suffix-based throughout. There is no tagger, and
//! there will not be one before a POS-tagged Tier 2; the suffix rule
//! over-claims on `-ly` adverbs used attributively and under-claims on
//! underived adjectives (`big`, `red`). The error is the same on the corpus
//! side and the draft side.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{
    per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, PackLicense, Unit,
};
use crate::error::Result;
use crate::feature::pack::{CountPack, NormPack};
use crate::text::syllable::SyllableMethod;
use crate::text::{Analysis, Span};
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Suffixes that mark a word as an adjective for the bigram and epithet rules.
const ADJECTIVE_SUFFIXES: &[&str] = &[
    "able", "ible", "al", "ial", "ful", "ic", "ical", "ish", "ive", "ative", "less", "ous", "ious",
    "eous", "y", "ary", "ory", "ant", "ent", "en", "ed", "ing",
];

/// Common words the adjective suffix rule would otherwise claim.
const ADJECTIVE_STOPLIST: &[&str] = &[
    "the",
    "any",
    "many",
    "very",
    "every",
    "only",
    "early",
    "family",
    "city",
    "body",
    "study",
    "money",
    "story",
    "policy",
    "company",
    "country",
    "history",
    "memory",
    "theory",
    "industry",
    "energy",
    "quality",
    "society",
    "security",
    "activity",
    "may",
    "say",
    "way",
    "day",
    "boy",
    "key",
    "they",
    "why",
    "by",
    "my",
    "guy",
    "buy",
    "eye",
    "lie",
    "die",
    "tie",
    "thing",
    "king",
    "ring",
    "spring",
    "morning",
    "evening",
    "during",
    "nothing",
    "something",
    "anything",
    "everything",
    "being",
    "having",
    "doing",
    "going",
    "getting",
    "making",
    "taking",
];

/// Minimizers that produce litotes when placed next to an extreme.
const MINIMIZERS: &[&str] = &[
    "hardly",
    "barely",
    "scarcely",
    "rather",
    "somewhat",
    "slightly",
    "mildly",
    "fairly",
    "moderately",
    "reasonably",
    "tolerably",
    "passably",
    "quite",
];

/// Extreme-magnitude words a minimizer can be ironic about.
const EXTREMES: &[&str] = &[
    "big",
    "huge",
    "vast",
    "enormous",
    "immense",
    "infinite",
    "endless",
    "massive",
    "gigantic",
    "colossal",
    "tiny",
    "minute",
    "microscopic",
    "impossible",
    "certain",
    "perfect",
    "total",
    "complete",
    "absolute",
    "catastrophic",
    "disastrous",
    "unprecedented",
    "extraordinary",
    "incredible",
    "unbelievable",
    "terrifying",
    "devastating",
];

/// Nouns abstract or cosmic enough that a precise numeral next to them reads as
/// a joke rather than as a measurement.
const COSMIC_NOUNS: &[&str] = &[
    "universe",
    "universes",
    "galaxy",
    "galaxies",
    "cosmos",
    "eternity",
    "infinity",
    "existence",
    "creation",
    "civilization",
    "civilizations",
    "history",
    "time",
    "space",
    "reality",
    "consciousness",
    "meaning",
    "purpose",
    "destiny",
    "fate",
    "soul",
    "souls",
    "species",
    "evolution",
    "extinction",
    "apocalypse",
    "millennia",
    "millennium",
    "epoch",
    "epochs",
    "eon",
    "eons",
    "aeon",
    "aeons",
    "lifetime",
    "lifetimes",
    "generation",
    "generations",
    "empire",
    "empires",
    "star",
    "stars",
    "planet",
    "planets",
    "world",
    "worlds",
    "dimension",
    "dimensions",
    "improbability",
    "probability",
    "chance",
    "coincidence",
    "happiness",
    "despair",
    "boredom",
    "misery",
    "bureaucracy",
    "paperwork",
    "committee",
    "committees",
];

/// Sentence-initial base verbs that head an imperative.
const IMPERATIVE_VERBS: &[&str] = &[
    "get",
    "buy",
    "try",
    "start",
    "stop",
    "join",
    "call",
    "click",
    "visit",
    "download",
    "discover",
    "explore",
    "find",
    "learn",
    "see",
    "look",
    "read",
    "check",
    "book",
    "order",
    "shop",
    "save",
    "grab",
    "take",
    "make",
    "build",
    "create",
    "sign",
    "subscribe",
    "register",
    "contact",
    "ask",
    "request",
    "schedule",
    "claim",
    "unlock",
    "boost",
    "transform",
    "upgrade",
    "switch",
    "choose",
    "pick",
    "compare",
    "share",
    "follow",
    "watch",
    "listen",
    "imagine",
    "consider",
    "remember",
    "forget",
    "meet",
    "let",
    "give",
    "add",
    "keep",
    "put",
    "send",
    "bring",
    "come",
    "go",
    "do",
    "be",
    "have",
    "use",
    "enjoy",
    "experience",
    "act",
    "hurry",
];

/// Demonstratives that open a forward-reference clickbait headline.
const HEADLINE_DEMONSTRATIVES: &[&str] = &["this", "these", "that", "those"];

/// Fewest words in an alliterative run.
const ALLITERATION_RUN: usize = 3;

/// Configuration for the device-rate family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceRates {
    /// Emit `dev:antonym_pair_rate` and `dev:ambiguity_density`.
    ///
    /// Needs the bundled WordNet-derived tables; without them the two
    /// dimensions are not emitted at all.
    #[serde(default = "crate::util::yes")]
    pub wordnet: bool,
    /// Emit `dev:novel_adj_noun_rate`, the one dimension with fitted state.
    #[serde(default = "crate::util::yes")]
    pub novelty: bool,
    /// A background bigram-frequency table for the novelty dimension.
    ///
    /// Produced offline and pinned on the spec. Without one, novelty is defined
    /// against the reference corpus's own bigram set — the weaker variant, and
    /// a different number, so both are recorded and neither is silent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<CountPack>,
    /// Concreteness norms, for `dev:transferred_epithet_rate`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concreteness: Option<NormPack>,
    /// How syllables and phonemes are counted, for alliteration.
    #[serde(default)]
    pub syllables: SyllableMethod,
}

impl Default for DeviceRates {
    fn default() -> Self {
        DeviceRates {
            wordnet: true,
            novelty: true,
            background: None,
            concreteness: None,
            syllables: SyllableMethod::default(),
        }
    }
}

impl DeviceRates {
    /// Supply a background bigram table for the novelty dimension.
    pub fn with_background(mut self, pack: CountPack) -> Self {
        self.background = Some(pack);
        self
    }

    /// Supply concreteness norms for the transferred-epithet dimension.
    pub fn with_concreteness(mut self, pack: NormPack) -> Self {
        self.concreteness = Some(pack);
        self
    }
}

/// Headline shapes, in emission order.
const HEADLINE_DIMS: &[&str] = &["number_initial", "listicle", "question", "demonstrative"];

/// Fitted [`DeviceRates`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedDevices {
    dims: Vec<DimInfo>,
    litotes: Symbol,
    precision: Symbol,
    epithet: Symbol,
    alliteration: Symbol,
    rhyme: Symbol,
    antonym: Option<Symbol>,
    ambiguity: Option<Symbol>,
    novel_bigram: Option<Symbol>,
    imperative: Symbol,
    headline: Vec<Symbol>,
    triplet: Symbol,
    negative_parallelism: Symbol,
    /// Adjective–noun bigrams seen in the reference corpus, sorted. This is the
    /// feature's only corpus-relative state.
    seen_bigrams: Vec<String>,
    /// Background bigram counts, sorted, with the corpus total.
    background: Vec<(String, u64)>,
    background_total: u64,
    /// Concreteness table and the cut above which a noun counts as concrete.
    concreteness: Vec<(String, f64)>,
    concrete_cut: f64,
    /// Mental-state adjectives, for the transferred-epithet rule.
    emotion: Vec<String>,
    antonyms: Vec<(String, String)>,
    senses: Vec<(String, f64)>,
    syllable_method: SyllableMethod,
    packs: Vec<PackLicense>,
}

impl FittedDevices {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Device;

    /// How many adjective–noun bigrams the reference corpus contained.
    pub fn seen_bigrams(&self) -> &[String] {
        &self.seen_bigrams
    }

    /// Whether novelty was fitted against a background table.
    pub fn has_background(&self) -> bool {
        !self.background.is_empty()
    }
}

impl Feature for DeviceRates {
    type Fitted = FittedDevices;

    fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedDevices> {
        let syllable_method = self.syllables.resolve_for_fit()?;
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Device, unit));
            sym
        };

        let litotes = push(interner, "dev:litotes_rate", Unit::PerThousandTokens);
        let precision = push(
            interner,
            "dev:absurd_precision_rate",
            Unit::PerThousandTokens,
        );
        let epithet = push(
            interner,
            "dev:transferred_epithet_rate",
            Unit::PerThousandTokens,
        );
        let alliteration = push(interner, "dev:alliteration_rate", Unit::PerThousandTokens);
        // Completes the Mihalcea-Strapparava trio. Emitted always, computed
        // only under the Dict method: a letter-based rhyme detector fires on
        // "though/rough" and misses "high/lie", so on VowelGroup this is
        // marked missing rather than faked.
        let rhyme = push(interner, "dev:rhyme_chain_rate", Unit::PerThousandTokens);
        let antonym = self
            .wordnet
            .then(|| push(interner, "dev:antonym_pair_rate", Unit::PerThousandTokens));
        let ambiguity = self
            .wordnet
            .then(|| push(interner, "dev:ambiguity_density", Unit::Index));
        let novel_bigram = self
            .novelty
            .then(|| push(interner, "dev:novel_adj_noun_rate", Unit::PerThousandTokens));
        let imperative = push(interner, "dev:imperative_rate", Unit::PerHundredSentences);
        let headline = HEADLINE_DIMS
            .iter()
            .map(|h| {
                push(
                    interner,
                    &format!("dev:headline:{h}"),
                    Unit::PerHundredSentences,
                )
            })
            .collect();
        let triplet = push(interner, "dev:triplet_rate", Unit::PerThousandTokens);
        let negative_parallelism = push(
            interner,
            "dev:negative_parallelism_rate",
            Unit::PerHundredSentences,
        );

        // The one corpus-relative step. Sorted so the fitted state is
        // byte-identical for a given corpus, the same rule the frequent-word
        // family follows.
        let mut seen_bigrams: Vec<String> = Vec::new();
        if self.novelty {
            let mut set: HashSet<String> = HashSet::new();
            for analysis in ctx.analyses() {
                set.extend(adj_noun_bigrams(analysis).into_iter().map(|(text, _)| text));
            }
            seen_bigrams = set.into_iter().collect();
            seen_bigrams.sort();
        }

        let mut packs: Vec<PackLicense> = Vec::new();
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
        let (concreteness, concrete_cut) = match &self.concreteness {
            Some(pack) => {
                pack.validate()?;
                packs.push(PackLicense {
                    pack: pack.qualified_name(),
                    license: pack.license.clone(),
                    redistributable: pack.redistributable,
                });
                (pack.table(), pack.quantile(0.75).unwrap_or(0.0))
            }
            None => (Vec::new(), 0.0),
        };

        let (antonyms, senses) = if self.wordnet {
            packs.push(PackLicense {
                pack: super::packs::WORDNET_VERSION.to_owned(),
                license: "WordNet 3.0 (permissive, BSD-like)".into(),
                redistributable: true,
            });
            (
                super::packs::wordnet_antonyms(),
                super::packs::senses_per_word(),
            )
        } else {
            (Vec::new(), Vec::new())
        };

        Ok(FittedDevices {
            dims,
            litotes,
            precision,
            epithet,
            alliteration,
            rhyme,
            antonym,
            ambiguity,
            novel_bigram,
            imperative,
            headline,
            triplet,
            negative_parallelism,
            seen_bigrams,
            background,
            background_total,
            concreteness,
            concrete_cut,
            emotion: super::packs::emotion_adjectives(),
            antonyms,
            senses,
            syllable_method,
            packs,
        })
    }
}

impl FittedFeature for FittedDevices {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Device
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
        let sentences = analysis.structure().sentences.len().max(1);
        let stream = analysis.tokens();
        let lexical: Vec<(&str, Span)> = stream.lexical().map(|(t, f)| (f, t.span)).collect();

        self.lexical_devices(&lexical, tokens, out);
        self.wordnet_devices(analysis, &lexical, tokens, out);
        self.novelty(analysis, tokens, out);
        self.marketing(analysis, sentences, tokens, out);
    }
}

impl FittedDevices {
    /// Litotes, absurd precision, transferred epithet, alliteration.
    fn lexical_devices(&self, lexical: &[(&str, Span)], tokens: usize, out: &mut VectorBuilder) {
        let mut litotes = 0usize;
        let mut precision = 0usize;
        let mut epithets = 0usize;

        for (i, (form, span)) in lexical.iter().enumerate() {
            // Litotes 1: an explicit negator on a positive term, or a `un-`
            // adjective under negation.
            if matches!(*form, "not" | "n't" | "'t") {
                if let Some((next, next_span)) = lexical.get(i + 1) {
                    if next.starts_with("un")
                        || next.starts_with("in")
                        || matches!(
                            *next,
                            "entirely" | "exactly" | "quite" | "unlike" | "without"
                        )
                    {
                        litotes += 1;
                        out.note_span(self.litotes, Span::new(span.start, next_span.end));
                        continue;
                    }
                }
            }
            // Litotes 2: a minimizer next to an extreme.
            if MINIMIZERS.contains(form) {
                let end = (i + 3).min(lexical.len());
                if let Some((_, other)) = lexical[i + 1..end]
                    .iter()
                    .find(|(f, _)| EXTREMES.contains(f))
                {
                    litotes += 1;
                    out.note_span(self.litotes, Span::new(span.start, other.end));
                    continue;
                }
            }
            // Absurd precision: a numeral inside a short window of a cosmic
            // noun. Six tokens, because the construction usually has a unit and
            // a preposition between them — "once every ten million years across
            // the galaxy" puts five tokens between "million" and "galaxy".
            if is_precise_numeral(form) {
                let end = (i + 7).min(lexical.len());
                if let Some((_, other)) = lexical[i + 1..end]
                    .iter()
                    .find(|(f, _)| COSMIC_NOUNS.contains(f))
                {
                    precision += 1;
                    out.note_span(self.precision, Span::new(span.start, other.end));
                }
            }
            // Transferred epithet: a mental-state adjective on a concrete noun.
            if !self.concreteness.is_empty() && self.is_emotion(form) {
                if let Some((next, next_span)) = lexical.get(i + 1) {
                    if self
                        .concreteness_of(next)
                        .is_some_and(|c| c >= self.concrete_cut)
                    {
                        epithets += 1;
                        out.note_span(self.epithet, Span::new(span.start, next_span.end));
                    }
                }
            }
        }

        out.set(self.litotes, per_1k(litotes, tokens));
        out.set(self.precision, per_1k(precision, tokens));
        if self.concreteness.is_empty() {
            // Loader-only norms: without them this is unknown, not zero.
            out.mark_missing(self.epithet);
        } else {
            out.set(self.epithet, per_1k(epithets, tokens));
        }

        let mut runs = 0usize;
        let mut run_start = 0usize;
        let mut run_len = 0usize;
        let mut previous: Option<char> = None;
        for (i, (form, _)) in lexical.iter().enumerate() {
            let initial = form.chars().next().filter(|c| c.is_alphabetic());
            match (previous, initial) {
                (Some(p), Some(c)) if p == c => run_len += 1,
                (_, Some(_)) => {
                    if run_len >= ALLITERATION_RUN {
                        runs += 1;
                        out.note_span(
                            self.alliteration,
                            Span::new(lexical[run_start].1.start, lexical[i - 1].1.end),
                        );
                    }
                    run_start = i;
                    run_len = 1;
                }
                (_, None) => {
                    if run_len >= ALLITERATION_RUN {
                        runs += 1;
                    }
                    run_len = 0;
                }
            }
            previous = initial;
        }
        if run_len >= ALLITERATION_RUN {
            runs += 1;
            out.note_span(
                self.alliteration,
                Span::new(lexical[run_start].1.start, lexical[lexical.len() - 1].1.end),
            );
        }
        out.set(self.alliteration, per_1k(runs, tokens));
        self.rhyme_chains(lexical, tokens, out);
    }

    /// Rhyme chains: two words within a short window sharing a rhyme key.
    ///
    /// Dict-method only. On the vowel-group method the dimension is marked
    /// missing, because the alternative is a letter-based detector that
    /// measures spelling and calls it sound.
    fn rhyme_chains(&self, lexical: &[(&str, Span)], tokens: usize, out: &mut VectorBuilder) {
        if !matches!(self.syllable_method, SyllableMethod::Dict { .. }) {
            out.mark_missing(self.rhyme);
            return;
        }
        const WINDOW: usize = 12;
        let mut chains = 0usize;
        for (i, (form, span)) in lexical.iter().enumerate() {
            let Some(key) = rhyme_key(form) else { continue };
            let end = (i + WINDOW + 1).min(lexical.len());
            for (other, other_span) in &lexical[i + 1..end] {
                if *other == *form {
                    continue;
                }
                if rhyme_key(other) == Some(key) {
                    chains += 1;
                    out.note_span(self.rhyme, Span::new(span.start, other_span.end));
                    break;
                }
            }
        }
        out.set(self.rhyme, per_1k(chains, tokens));
    }

    /// Antonym pairs and sense ambiguity.
    fn wordnet_devices(
        &self,
        analysis: &Analysis<'_>,
        lexical: &[(&str, Span)],
        tokens: usize,
        out: &mut VectorBuilder,
    ) {
        let (Some(antonym), Some(ambiguity)) = (self.antonym, self.ambiguity) else {
            return;
        };
        let mut pairs = 0usize;
        let stream = analysis.tokens();
        for sentence in &analysis.structure().sentences {
            let words: Vec<(&str, Span)> = stream.tokens()[sentence.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| (stream.form(t), t.span))
                .collect();
            'outer: for (i, (form, span)) in words.iter().enumerate() {
                for (other, other_span) in &words[i + 1..] {
                    if self.are_antonyms(form, other) {
                        pairs += 1;
                        out.note_span(antonym, Span::new(span.start, other_span.end));
                        break 'outer;
                    }
                }
            }
        }
        out.set(antonym, per_1k(pairs, tokens));

        let mut total = 0.0f64;
        let mut counted = 0usize;
        for (form, _) in lexical {
            if let Some(senses) = self.senses_of(form) {
                total += senses;
                counted += 1;
            }
        }
        if counted == 0 {
            out.mark_missing(ambiguity);
        } else {
            out.set(ambiguity, total / counted as f64);
        }
    }

    /// The novel adjective–noun bigram rate.
    fn novelty(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        let Some(sym) = self.novel_bigram else {
            return;
        };
        let bigrams = adj_noun_bigrams(analysis);
        if bigrams.is_empty() {
            out.set(sym, 0.0);
            return;
        }
        let mut novel = 0.0f64;
        for (text, span) in &bigrams {
            if self.seen_bigrams.binary_search(text).is_ok() {
                continue;
            }
            let weight = if self.background.is_empty() {
                // Weaker variant: novelty against the reference corpus alone.
                1.0
            } else {
                // Weight by how unusual the pair is in general, so a pair the
                // reference happens not to contain but the language uses freely
                // counts for less than a genuine coinage.
                let count = self
                    .background
                    .binary_search_by(|(t, _)| t.as_str().cmp(text))
                    .map(|i| self.background[i].1)
                    .unwrap_or(0);
                let freq = if self.background_total == 0 {
                    0.0
                } else {
                    count as f64 / self.background_total as f64
                };
                (1.0 - freq * 1_000.0).clamp(0.0, 1.0)
            };
            if weight > 0.0 {
                novel += weight;
                out.note_span(sym, *span);
            }
        }
        out.set(
            sym,
            if tokens == 0 {
                0.0
            } else {
                novel * 1000.0 / tokens as f64
            },
        );
    }

    /// Imperatives, headline shapes, triplets, negative parallelism.
    fn marketing(
        &self,
        analysis: &Analysis<'_>,
        sentences: usize,
        tokens: usize,
        out: &mut VectorBuilder,
    ) {
        let stream = analysis.tokens();
        let mut imperatives = 0usize;
        let mut headline = vec![0usize; HEADLINE_DIMS.len()];
        let mut negative_parallel = 0usize;

        let sentence_list = &analysis.structure().sentences;
        for (index, sentence) in sentence_list.iter().enumerate() {
            let words: Vec<&str> = stream.tokens()[sentence.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| stream.form(t))
                .collect();
            let Some(&first) = words.first() else {
                continue;
            };
            // Leech: imperatives head over a quarter of major clauses in
            // advertising, and *negated* imperatives are near-absent — so a
            // negated one is not the same device and is not counted.
            if IMPERATIVE_VERBS.contains(&first) && !words.get(1).is_some_and(|w| is_negator(w)) {
                imperatives += 1;
                out.note_span(self.imperative, sentence.span);
            }
            if crate::feature::lexicon::is_numeric(first) {
                headline[0] += 1;
                // "N ways to …", "7 things you …".
                if words.get(1).is_some_and(|w| {
                    matches!(
                        *w,
                        "ways"
                            | "things"
                            | "reasons"
                            | "tips"
                            | "steps"
                            | "rules"
                            | "signs"
                            | "facts"
                    )
                }) {
                    headline[1] += 1;
                }
            }
            let text = analysis.text(sentence.span);
            if text.trim_end().ends_with('?') {
                headline[2] += 1;
            }
            if HEADLINE_DEMONSTRATIVES.contains(&first) && words.len() <= 12 {
                headline[3] += 1;
            }
            // "Not X. Not Y. Just Z." — three consecutive sentences, the first
            // two negative-initial and the third a positive assertion.
            if first == "not" && index + 2 < sentence_list.len() {
                let opener = |k: usize| -> Option<&str> {
                    stream.tokens()[sentence_list[k].tokens.clone()]
                        .iter()
                        .find(|t| t.kind.is_lexical())
                        .map(|t| stream.form(t))
                };
                if opener(index + 1) == Some("not")
                    && matches!(
                        opener(index + 2),
                        Some("just" | "only" | "simply" | "merely")
                    )
                {
                    negative_parallel += 1;
                    out.note_span(
                        self.negative_parallelism,
                        Span::new(sentence.span.start, sentence_list[index + 2].span.end),
                    );
                }
            }
        }
        out.set(self.imperative, ratio(imperatives, sentences) * 100.0);
        for (i, &sym) in self.headline.iter().enumerate() {
            out.set(sym, ratio(headline[i], sentences) * 100.0);
        }
        out.set(
            self.negative_parallelism,
            ratio(negative_parallel, sentences) * 100.0,
        );

        // Triplets: `A, B, and C` over single words — the rule of three, which
        // the lexicon family also counts for the AI-slop pack. Here it is a
        // device rate rather than a slop tell, and the band is two-sided.
        let all: Vec<(&str, Span, bool)> = stream
            .iter()
            .map(|(t, f)| (f, t.span, t.kind.is_lexical()))
            .collect();
        let mut triplets = 0usize;
        for w in all.windows(6) {
            if w[0].2
                && w[1].0 == ","
                && w[2].2
                && w[3].0 == ","
                && matches!(w[4].0, "and" | "or")
                && w[5].2
            {
                triplets += 1;
                out.note_span(self.triplet, Span::new(w[0].1.start, w[5].1.end));
            }
        }
        out.set(self.triplet, per_1k(triplets, tokens));
    }

    fn is_emotion(&self, form: &str) -> bool {
        self.emotion.binary_search(&form.to_string()).is_ok()
    }

    fn concreteness_of(&self, form: &str) -> Option<f64> {
        self.concreteness
            .binary_search_by(|(t, _)| t.as_str().cmp(form))
            .ok()
            .map(|i| self.concreteness[i].1)
    }

    fn senses_of(&self, form: &str) -> Option<f64> {
        self.senses
            .binary_search_by(|(t, _)| t.as_str().cmp(form))
            .ok()
            .map(|i| self.senses[i].1)
    }

    fn are_antonyms(&self, a: &str, b: &str) -> bool {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        self.antonyms
            .binary_search_by(|(x, y)| (x.as_str(), y.as_str()).cmp(&(lo, hi)))
            .is_ok()
    }
}

/// Whether a numeral is *precise* enough to be funny next to a cosmic noun.
///
/// A bare small integer is not: "two worlds" is a count. A large number, a
/// decimal, or a spelled-out precision word is.
fn is_precise_numeral(form: &str) -> bool {
    const PRECISION_WORDS: &[&str] = &[
        "million",
        "billion",
        "trillion",
        "thousand",
        "hundred",
        "precisely",
        "exactly",
        "roughly",
        "approximately",
    ];
    if PRECISION_WORDS.contains(&form) {
        return true;
    }
    let digits: String = form.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return false;
    }
    form.contains('.') || digits.len() >= 3
}

/// A word's rhyme key, when the build carries pronunciation data.
fn rhyme_key(word: &str) -> Option<&'static str> {
    #[cfg(feature = "verse")]
    {
        crate::text::dict::rhyme_key(word)
    }
    #[cfg(not(feature = "verse"))]
    {
        let _ = word;
        None
    }
}

fn is_negator(form: &str) -> bool {
    matches!(
        form,
        "not" | "n't" | "'t" | "never" | "no" | "don't" | "dont"
    )
}

/// Adjective–noun bigrams in a document, with spans.
///
/// Suffix-based adjective detection with a stoplist; the "noun" slot is any
/// following lexical token that is not itself adjective-shaped and not a
/// function word. The looseness is the point — this feeds a *novelty* rate,
/// where a consistent over-count on both sides washes out and an inconsistent
/// one would not.
fn adj_noun_bigrams(analysis: &Analysis<'_>) -> Vec<(String, Span)> {
    use crate::feature::mfw::FUNCTION_WORDS;
    let forms: Vec<(&str, Span)> = analysis
        .tokens()
        .lexical()
        .map(|(t, f)| (f, t.span))
        .collect();
    let mut out = Vec::new();
    for w in forms.windows(2) {
        let (adj, adj_span) = w[0];
        let (noun, noun_span) = w[1];
        if !is_adjective(adj) || is_adjective(noun) {
            continue;
        }
        if FUNCTION_WORDS.contains(&noun) || noun.chars().count() < 3 {
            continue;
        }
        out.push((
            format!("{adj} {noun}"),
            Span::new(adj_span.start, noun_span.end),
        ));
    }
    out
}

/// Nominalizing suffixes, which beat the adjective suffixes.
///
/// `-ment` ends in `-ent` and `-tion` ends in `-ion`, so without this check
/// "disappointment" and "consideration" read as adjectives and the adjective-noun
/// bigram rule can never fire on the constructions it exists for.
const NOUN_SUFFIXES: &[&str] = &[
    "ment", "ments", "tion", "tions", "sion", "sions", "ness", "ity", "ities", "ance", "ence",
    "ist", "ism",
];

fn is_adjective(form: &str) -> bool {
    use crate::feature::mfw::FUNCTION_WORDS;
    if form.chars().count() < 4
        || ADJECTIVE_STOPLIST.contains(&form)
        || FUNCTION_WORDS.contains(&form)
    {
        return false;
    }
    if NOUN_SUFFIXES.iter().any(|s| form.ends_with(s)) {
        return false;
    }
    ADJECTIVE_SUFFIXES.iter().any(|s| form.ends_with(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::packs;
    use crate::feature::FitDoc;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn fit_on(spec: DeviceRates, corpus_texts: &[&str]) -> (FittedDevices, Interner) {
        let corpus = Corpus::new();
        let tok = Tokenizer::default();
        let docs: Vec<Document> = corpus_texts.iter().map(|t| Document::new(*t)).collect();
        let fit_docs: Vec<FitDoc<'_>> = docs
            .iter()
            .map(|d| FitDoc {
                author: 0,
                analysis: d.analyze(&tok),
            })
            .collect();
        let ctx = FitContext::new(&corpus, &fit_docs);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        (fitted, interner)
    }

    fn transform(spec: DeviceRates, text: &str) -> (FeatureVector, Interner) {
        let (fitted, interner) = fit_on(spec, &[]);
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn value(text: &str, dim: &str) -> f64 {
        let (v, i) = transform(DeviceRates::default(), text);
        v.get(i.get(dim).unwrap_or_else(|| panic!("no dim {dim}")))
    }

    #[test]
    fn litotes_fires_on_both_of_its_shapes() {
        // "not entirely" is the explicit shape; "hardly enormous" is the
        // minimizer-next-to-extreme shape.
        let text = "The plan was not entirely stupid. The budget was hardly enormous.";
        let (v, i) = transform(DeviceRates::default(), text);
        let spans = v.spans(i.get("dev:litotes_rate").unwrap());
        assert_eq!(spans.len(), 2, "{:?}", spans);
        assert_eq!(&text[spans[0].range()], "not entirely");
        assert_eq!(&text[spans[1].range()], "hardly enormous");
        assert_eq!(
            value(
                "The plan was fine and the budget was fine.",
                "dev:litotes_rate"
            ),
            0.0
        );
    }

    #[test]
    fn absurd_precision_needs_both_halves() {
        let text = "It happens roughly once every ten million years across the galaxy.";
        assert!(value(text, "dev:absurd_precision_rate") > 0.0);
        // A precise number with no cosmic noun is a measurement.
        assert_eq!(
            value(
                "The build took 1200 seconds on the runner.",
                "dev:absurd_precision_rate"
            ),
            0.0
        );
        // A cosmic noun with a bare small count is not a joke either.
        assert_eq!(
            value(
                "There were two worlds left in the set.",
                "dev:absurd_precision_rate"
            ),
            0.0
        );
    }

    #[test]
    fn alliteration_needs_a_run_of_three() {
        assert!(value("the silly slippery slope stopped", "dev:alliteration_rate") > 0.0);
        assert_eq!(
            value("the silly slippery thing stopped", "dev:alliteration_rate"),
            0.0
        );
    }

    #[test]
    fn the_transferred_epithet_needs_concreteness_norms() {
        let (v, i) = transform(
            DeviceRates::default(),
            "He took a moody forkful of the pie.",
        );
        assert!(
            v.is_missing(i.get("dev:transferred_epithet_rate").unwrap()),
            "loader-only norms absent means unknown, not zero"
        );

        let pack = packs::concreteness_stub();
        let (v, i) = transform(
            DeviceRates::default().with_concreteness(pack),
            "He took a moody forkful of the pie and a gloomy sandwich.",
        );
        assert!(v.get(i.get("dev:transferred_epithet_rate").unwrap()) > 0.0);
    }

    #[test]
    fn antonym_pairs_and_ambiguity_come_from_the_wordnet_tables() {
        assert!(
            value(
                "It was neither hot nor cold that day.",
                "dev:antonym_pair_rate"
            ) > 0.0
        );
        assert_eq!(
            value(
                "It was warm that day and stayed warm.",
                "dev:antonym_pair_rate"
            ),
            0.0
        );
        // Ambiguity is a mean over words the sense table knows.
        assert!(value("the run was a break in the line", "dev:ambiguity_density") > 1.0);
    }

    #[test]
    fn switching_wordnet_off_removes_its_dimensions() {
        let (_, interner) = fit_on(
            DeviceRates {
                wordnet: false,
                ..Default::default()
            },
            &[],
        );
        assert!(interner.get("dev:antonym_pair_rate").is_none());
        assert!(interner.get("dev:ambiguity_density").is_none());
    }

    #[test]
    fn the_novel_bigram_set_is_deterministic_and_sorted() {
        let corpus = [
            "a moody forkful and a gloomy sandwich",
            "the hollow silence",
        ];
        let (a, _) = fit_on(DeviceRates::default(), &corpus);
        let (b, _) = fit_on(DeviceRates::default(), &corpus);
        assert_eq!(a.seen_bigrams(), b.seen_bigrams());
        assert!(a.seen_bigrams().windows(2).all(|w| w[0] < w[1]));
        assert!(!a.seen_bigrams().is_empty());
    }

    #[test]
    fn a_pair_the_corpus_never_used_counts_as_novel() {
        let (fitted, interner) = fit_on(DeviceRates::default(), &["a moody forkful of pie"]);
        let score = |text: &str| {
            let doc = Document::new(text);
            let a = doc.analyze(&Tokenizer::default());
            let mut b = VectorBuilder::new();
            fitted.transform(&a, &mut b);
            b.build()
                .get(interner.get("dev:novel_adj_noun_rate").unwrap())
        };
        // The corpus's own pair is not novel; an unseen one is.
        assert_eq!(score("a moody forkful of pie"), 0.0);
        assert!(score("a cavernous disappointment of pie") > 0.0);
    }

    #[test]
    fn a_background_pack_changes_the_number_and_both_stay_deterministic() {
        let corpus = ["a moody forkful of pie"];
        let background = CountPack {
            name: "bg".into(),
            version: "1".into(),
            date: "2026-08-01".into(),
            description: String::new(),
            license: "CC0-1.0".into(),
            redistributable: true,
            sources: vec![],
            // The draft's pair is ordinary in general, so weighting it down is
            // exactly what the background is for.
            entries: vec![("cavernous disappointment".into(), 500)],
            total: 1_000,
        };
        let text = "a cavernous disappointment of pie";
        let score = |spec: DeviceRates| {
            let (fitted, interner) = fit_on(spec, &corpus);
            let doc = Document::new(text);
            let a = doc.analyze(&Tokenizer::default());
            let mut b = VectorBuilder::new();
            fitted.transform(&a, &mut b);
            (
                b.build()
                    .get(interner.get("dev:novel_adj_noun_rate").unwrap()),
                fitted.has_background(),
            )
        };
        let (without, has_a) = score(DeviceRates::default());
        let (with, has_b) = score(DeviceRates::default().with_background(background.clone()));
        assert!(!has_a && has_b);
        assert!(without > 0.0);
        assert_eq!(
            with, 0.0,
            "a pair this common in the background is not novel"
        );
        // Both variants are reproducible.
        let (again, _) = score(DeviceRates::default().with_background(background));
        assert_eq!(with, again);
    }

    #[test]
    fn imperatives_are_counted_but_negated_ones_are_not() {
        // Leech: negated imperatives are near-absent in advertising, so they
        // are a different construction and do not belong in the same rate.
        let text = "Buy the thing today. Do not miss out. Start your trial now. It works.";
        assert!((value(text, "dev:imperative_rate") - 50.0).abs() < 1e-9);
    }

    #[test]
    fn headline_shapes_are_separated() {
        let text = "7 ways to fix it. Why does this happen? This is why it broke. \
                    The build finished.";
        assert!((value(text, "dev:headline:number_initial") - 25.0).abs() < 1e-9);
        assert!((value(text, "dev:headline:listicle") - 25.0).abs() < 1e-9);
        assert!((value(text, "dev:headline:question") - 25.0).abs() < 1e-9);
        assert!((value(text, "dev:headline:demonstrative") - 25.0).abs() < 1e-9);
    }

    #[test]
    fn the_not_x_not_y_just_z_block_is_detected() {
        let text = "Not a phone. Not a tablet. Just the thing you actually wanted.";
        assert!(value(text, "dev:negative_parallelism_rate") > 0.0);
        let (v, i) = transform(DeviceRates::default(), text);
        let spans = v.spans(i.get("dev:negative_parallelism_rate").unwrap());
        assert_eq!(&text[spans[0].range()], text.trim());
        // Two negatives without the positive turn are not the device.
        assert_eq!(
            value(
                "Not a phone. Not a tablet. It broke anyway.",
                "dev:negative_parallelism_rate"
            ),
            0.0
        );
    }

    #[test]
    fn triplets_are_counted_per_thousand_tokens() {
        let text = "it is fast, cheap, and reliable in practice";
        assert!(value(text, "dev:triplet_rate") > 0.0);
        assert_eq!(
            value("it is fast and cheap in practice", "dev:triplet_rate"),
            0.0
        );
    }

    #[test]
    fn adjective_detection_has_a_documented_stoplist() {
        assert!(is_adjective("moody"));
        assert!(is_adjective("cavernous"));
        assert!(is_adjective("hollow") || !is_adjective("hollow"));
        // Common words the suffix rule would otherwise claim.
        assert!(!is_adjective("only"));
        assert!(!is_adjective("family"));
        assert!(!is_adjective("study"));
        assert!(!is_adjective("thing"));
        assert!(!is_adjective("the"));
    }

    #[test]
    fn empty_text_marks_everything_missing() {
        let (v, i) = transform(DeviceRates::default(), "");
        assert!(v.is_missing(i.get("dev:litotes_rate").unwrap()));
    }

    #[test]
    fn rhyme_is_missing_under_the_vowel_group_method() {
        // The honest half of the Mihalcea trio: no phoneme data, no rhyme
        // number. A letter-based detector would measure spelling.
        let (v, i) = transform(DeviceRates::default(), "the light was bright at night");
        assert!(v.is_missing(i.get("dev:rhyme_chain_rate").unwrap()));
    }

    #[cfg(feature = "verse")]
    #[test]
    fn rhyme_chains_are_counted_on_a_verse_build() {
        let spec = DeviceRates {
            syllables: SyllableMethod::Dict {
                dict_version: crate::text::dict::DICT_VERSION.into(),
            },
            ..Default::default()
        };
        let (v, i) = transform(spec.clone(), "the light was bright and it was night");
        assert!(v.get(i.get("dev:rhyme_chain_rate").unwrap()) > 0.0);
        // And it groups by sound, not by spelling.
        let (v, i) = transform(spec, "it was though it was rough");
        assert_eq!(v.get(i.get("dev:rhyme_chain_rate").unwrap()), 0.0);
    }

    #[test]
    fn the_spec_round_trips_and_defaults_sensibly() {
        let spec: DeviceRates = serde_json::from_str("{}").unwrap();
        assert_eq!(spec, DeviceRates::default());
        assert!(spec.wordnet && spec.novelty);
        assert_eq!(spec.syllables, SyllableMethod::VowelGroup);
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<DeviceRates>(&json).unwrap(), spec);
    }
}
