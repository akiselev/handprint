//! Versioned word- and phrase-list matching.
//!
//! **Lexicons are data, never code.** The evidence is unambiguous that AI-ism
//! vocabularies decay: "delve" rose ~15× in PubMed abstracts between 2022 and
//! 2024 and then *declined* after public mockery, em-dash overuse is a
//! GPT-4-era tell that barely existed in GPT-3.5, and the whole class of
//! markers is an RLHF artifact that differs per model and per version. A word
//! list compiled into a binary is wrong within a year of shipping.
//!
//! So every [`LexiconPack`] carries a name, a version, a date and a source
//! manifest, and packs load from JSON at runtime. The built-in
//! [`LexiconPack::ai_slop`] is a *seed* — dated `2026.07`, with its sources
//! listed — not a permanent fixture.
//!
//! Findings from this family are the most directly actionable of any: a term
//! hit has an exact span and, usually, alternatives.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{per_1k, DimInfo, Family, Feature, FitContext, FittedFeature, PackLicense, Unit};
use crate::error::{Error, Result};
use crate::text::{Analysis, Span};
use crate::vector::{Interner, Symbol, VectorBuilder};

/// How much a finding matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Worth knowing about.
    Low,
    /// Worth changing.
    Medium,
    /// Change this first.
    High,
}

impl Severity {
    /// Machine-readable name.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        }
    }
}

/// Where a pack's contents came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackSource {
    /// Human-readable source name.
    pub name: String,
    /// Where to find it.
    pub url: String,
    /// What was taken from it.
    pub note: String,
}

/// A single-word entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Term {
    /// Stable identifier, unique within the pack.
    pub id: String,
    /// The word to match, in tokenizer-normalized form (lowercase).
    pub word: String,
    /// Grouping used for the per-category rates.
    pub category: String,
    /// How much a hit matters.
    pub severity: Severity,
    /// Suggested replacements, surfaced in the critique contract's `fix` block.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

/// A multi-token pattern.
///
/// The pattern language is deliberately tiny: literal words, `*` for a gap of
/// one to four tokens, and `{a|b|c}` for alternatives. Anything needing more
/// than that belongs in a real linter rule, not a lexicon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phrase {
    /// Stable identifier, unique within the pack.
    pub id: String,
    /// The pattern, e.g. `"not just * but *"` or `"plays a {crucial|vital} role"`.
    pub pattern: String,
    /// Grouping used for the per-category rates.
    pub category: String,
    /// How much a hit matters.
    pub severity: Severity,
    /// Explanation shown to the agent.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
}

/// Evidence that a pack derived from private data was culled before shipping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyAttestation {
    /// Minimum number of distinct sessions a term had to appear in.
    pub min_sessions: usize,
    /// How many candidate terms the cull removed.
    pub removed: usize,
    /// Who reviewed the result.
    pub reviewed_by: String,
}

/// A dated, sourced collection of terms and phrases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconPack {
    /// Short name, used as the dimension namespace.
    pub name: String,
    /// Version string; bump when contents change.
    pub version: String,
    /// ISO date the contents were compiled.
    pub date: String,
    /// One-line description.
    #[serde(default)]
    pub description: String,
    /// License of the pack contents.
    #[serde(default)]
    pub license: String,
    /// Whether this pack may be shipped inside a published artifact.
    ///
    /// Defaults to true, which is right for every bundled pack. Set it false on
    /// a pack derived from a research-only or non-commercial resource: a
    /// reference fitted from it embeds the terms and inherits the restriction,
    /// and the CLI warns when it writes such a reference.
    #[serde(default = "crate::util::yes")]
    pub redistributable: bool,
    /// Where the contents came from.
    #[serde(default)]
    pub sources: Vec<PackSource>,
    /// Single-word entries.
    #[serde(default)]
    pub terms: Vec<Term>,
    /// Multi-token patterns.
    #[serde(default)]
    pub phrases: Vec<Phrase>,
    /// Privacy-culling attestation, for packs derived from private corpora.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy: Option<PrivacyAttestation>,
}

impl LexiconPack {
    /// Every distinct category, sorted.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .terms
            .iter()
            .map(|t| t.category.clone())
            .chain(self.phrases.iter().map(|p| p.category.clone()))
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// `name@version`, as it appears in provenance blocks.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Compile every phrase pattern, so a malformed one is found at load time.
    ///
    /// `fit` would find it too, but only after loading a corpus and analyzing
    /// every document. A pack is data a user edits; the error belongs where the
    /// edit happened.
    pub fn validate_patterns(&self) -> Result<()> {
        for phrase in &self.phrases {
            parse_pattern(&phrase.pattern).map_err(|e| match e {
                Error::InvalidConfig { what, detail } => Error::InvalidConfig {
                    what,
                    detail: format!("{}: {detail}", phrase.id),
                },
                other => other,
            })?;
        }
        Ok(())
    }
}

/// Configuration for the lexicon family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconFeature {
    /// The pack to match against.
    pub pack: LexiconPack,
    /// Emit one dimension per term and phrase, not just the category rollups.
    #[serde(default = "crate::util::yes")]
    pub per_term: bool,
    /// Emit the structural rules (rule of three, em-dash chains).
    #[serde(default = "crate::util::yes")]
    pub structural: bool,
    /// Report the *category* rates rather than the per-term rates.
    ///
    /// The AI-slop pack's useful finding is "you wrote `delve`" — a specific
    /// word with a specific replacement. A metadiscourse pack's useful finding
    /// is the opposite: no single hedge occurs often enough at draft length to
    /// have a stable rate, but the hedge *category* does, and "your hedging is
    /// at 4 per 1k against a corpus band of 11–19" is exactly the instruction an
    /// agent can act on.
    ///
    /// Setting this stops the `lex:{pack}:cat:{category}` dimensions being
    /// marked as rollups, so they can become findings, and turns `per_term` off
    /// by default — a hundred near-zero per-term dimensions would otherwise
    /// crowd out everything else in the ranking. Existing packs are unaffected:
    /// the flag defaults to false and serde-defaults on load.
    #[serde(default)]
    pub category_findings: bool,
}

impl Default for LexiconFeature {
    fn default() -> Self {
        LexiconFeature {
            pack: LexiconPack::ai_slop(),
            per_term: true,
            structural: true,
            category_findings: false,
        }
    }
}

impl LexiconFeature {
    /// Match against a specific pack.
    pub fn new(pack: LexiconPack) -> Self {
        LexiconFeature {
            pack,
            per_term: true,
            structural: true,
            category_findings: false,
        }
    }

    /// Report category rates instead of per-term rates.
    ///
    /// Also turns off `per_term` and the structural rules, which belong to the
    /// AI-slop pack rather than to every pack. Call `per_term` back on
    /// afterwards if a pack genuinely wants both.
    pub fn category_findings(mut self) -> Self {
        self.category_findings = true;
        self.per_term = false;
        self.structural = false;
        self
    }
}

/// One compiled matcher: either a bare word or a token pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Rule {
    id: String,
    symbol: Symbol,
    category: String,
    severity: Severity,
    parts: Vec<Part>,
    alternatives: Vec<String>,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Part {
    Lit(String),
    OneOf(Vec<String>),
    Gap { min: usize, max: usize },
}

/// Structural rules that need no word list.
const STRUCTURAL: &[&str] = &["rule_of_three", "em_dash_chain", "title_case_heading"];

/// Fitted [`LexiconFeature`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedLexicon {
    dims: Vec<DimInfo>,
    pack_name: String,
    pack_version: String,
    /// The pack's declared license, carried so a fitted reference can report
    /// what it embeds. Serde-defaulted: a pre-rollup artifact loads unchanged
    /// and simply reports nothing.
    #[serde(default)]
    pack_license: String,
    #[serde(default = "crate::util::yes")]
    redistributable: bool,
    rules: Vec<Rule>,
    /// Fast path for single-literal rules.
    words: HashMap<String, usize>,
    categories: Vec<(String, Symbol)>,
    structural: Vec<Symbol>,
    total: Symbol,
}

impl FittedLexicon {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Lexicon;

    /// `name@version` of the pack this was fitted from.
    pub fn pack(&self) -> String {
        format!("{}@{}", self.pack_name, self.pack_version)
    }

    /// The severity declared for a dimension, if it is a lexicon rule.
    pub fn severity_of(&self, symbol: Symbol) -> Option<Severity> {
        self.rules
            .iter()
            .find(|r| r.symbol == symbol)
            .map(|r| r.severity)
    }

    /// Suggested replacements for a dimension, if it has any.
    pub fn alternatives_of(&self, symbol: Symbol) -> &[String] {
        self.rules
            .iter()
            .find(|r| r.symbol == symbol)
            .map(|r| r.alternatives.as_slice())
            .unwrap_or(&[])
    }

    /// The explanation attached to a dimension, if any.
    pub fn message_of(&self, symbol: Symbol) -> Option<&str> {
        self.rules
            .iter()
            .find(|r| r.symbol == symbol)
            .map(|r| r.message.as_str())
            .filter(|m| !m.is_empty())
    }
}

fn parse_pattern(pattern: &str) -> Result<Vec<Part>> {
    let mut parts = Vec::new();
    for piece in pattern.split_whitespace() {
        if piece == "*" {
            parts.push(Part::Gap { min: 1, max: 4 });
        } else if piece.starts_with('{') && !piece.ends_with('}') {
            // Alternatives are matched token by token, so they cannot contain
            // spaces. Catching it here beats silently never matching.
            return Err(Error::InvalidConfig {
                what: "LexiconPack phrase pattern",
                detail: format!(
                    "unterminated or multi-word alternative {piece:?} in {pattern:?}; \
                     alternatives must be single tokens"
                ),
            });
        } else if let Some(inner) = piece.strip_prefix('{').and_then(|p| p.strip_suffix('}')) {
            let options: Vec<String> = inner.split('|').map(|s| s.to_lowercase()).collect();
            if options.iter().any(String::is_empty) {
                return Err(Error::InvalidConfig {
                    what: "LexiconPack phrase pattern",
                    detail: format!("empty alternative in {pattern:?}"),
                });
            }
            parts.push(Part::OneOf(options));
        } else {
            parts.push(Part::Lit(piece.to_lowercase()));
        }
    }
    if parts.is_empty() {
        return Err(Error::InvalidConfig {
            what: "LexiconPack phrase pattern",
            detail: format!("{pattern:?} is empty"),
        });
    }
    Ok(parts)
}

impl Feature for LexiconFeature {
    type Fitted = FittedLexicon;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedLexicon> {
        let pack = &self.pack;
        let ns = &pack.name;
        let mut dims = Vec::new();
        let mut rules = Vec::new();
        let mut words = HashMap::new();

        let push_dim = |interner: &mut Interner, name: String, dims: &mut Vec<DimInfo>| {
            let sym = interner.intern(&name);
            dims.push(DimInfo::new(sym, Family::Lexicon, Unit::PerThousandTokens));
            sym
        };
        let push_rollup = |interner: &mut Interner, name: String, dims: &mut Vec<DimInfo>| {
            let sym = interner.intern(&name);
            dims.push(DimInfo::new(sym, Family::Lexicon, Unit::PerThousandTokens).rollup());
            sym
        };

        if self.per_term {
            for term in &pack.terms {
                let sym = push_dim(interner, format!("lex:{ns}:{}", term.id), &mut dims);
                words.insert(term.word.to_lowercase(), rules.len());
                rules.push(Rule {
                    id: term.id.clone(),
                    symbol: sym,
                    category: term.category.clone(),
                    severity: term.severity,
                    parts: vec![Part::Lit(term.word.to_lowercase())],
                    alternatives: term.alternatives.clone(),
                    message: String::new(),
                });
            }
            for phrase in &pack.phrases {
                let sym = push_dim(interner, format!("lex:{ns}:{}", phrase.id), &mut dims);
                rules.push(Rule {
                    id: phrase.id.clone(),
                    symbol: sym,
                    category: phrase.category.clone(),
                    severity: phrase.severity,
                    parts: parse_pattern(&phrase.pattern)?,
                    alternatives: Vec::new(),
                    message: phrase.message.clone(),
                });
            }
        } else {
            // Still compile the rules; only the per-term dimensions are skipped.
            for term in &pack.terms {
                words.insert(term.word.to_lowercase(), rules.len());
                rules.push(Rule {
                    id: term.id.clone(),
                    symbol: Symbol(u32::MAX),
                    category: term.category.clone(),
                    severity: term.severity,
                    parts: vec![Part::Lit(term.word.to_lowercase())],
                    alternatives: term.alternatives.clone(),
                    message: String::new(),
                });
            }
            for phrase in &pack.phrases {
                rules.push(Rule {
                    id: phrase.id.clone(),
                    symbol: Symbol(u32::MAX),
                    category: phrase.category.clone(),
                    severity: phrase.severity,
                    parts: parse_pattern(&phrase.pattern)?,
                    alternatives: Vec::new(),
                    message: phrase.message.clone(),
                });
            }
        }

        let categories: Vec<(String, Symbol)> = pack
            .categories()
            .into_iter()
            .map(|cat| {
                let name = format!("lex:{ns}:cat:{cat}");
                let sym = if self.category_findings {
                    push_dim(interner, name, &mut dims)
                } else {
                    push_rollup(interner, name, &mut dims)
                };
                (cat, sym)
            })
            .collect();

        let structural = if self.structural {
            STRUCTURAL
                .iter()
                .map(|name| push_dim(interner, format!("lex:{ns}:struct:{name}"), &mut dims))
                .collect()
        } else {
            Vec::new()
        };

        let total = push_rollup(interner, format!("lex:{ns}:total"), &mut dims);

        Ok(FittedLexicon {
            dims,
            pack_name: pack.name.clone(),
            pack_version: pack.version.clone(),
            pack_license: pack.license.clone(),
            redistributable: pack.redistributable,
            rules,
            words,
            categories,
            structural,
            total,
        })
    }
}

impl FittedFeature for FittedLexicon {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Lexicon
    }

    fn pack_licenses(&self) -> Vec<PackLicense> {
        vec![PackLicense {
            pack: self.pack(),
            license: self.pack_license.clone(),
            redistributable: self.redistributable,
        }]
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        let stream = analysis.tokens();
        let lexical: Vec<(&str, Span)> = stream.lexical().map(|(t, f)| (f, t.span)).collect();
        let forms: Vec<&str> = lexical.iter().map(|(f, _)| *f).collect();

        let mut hits: Vec<usize> = vec![0; self.rules.len()];
        let mut spans: Vec<Vec<Span>> = vec![Vec::new(); self.rules.len()];

        for (i, form) in forms.iter().enumerate() {
            if let Some(&rule) = self.words.get(*form) {
                hits[rule] += 1;
                spans[rule].push(lexical[i].1);
            }
        }
        for (ri, rule) in self.rules.iter().enumerate() {
            if rule.parts.len() == 1 && matches!(rule.parts[0], Part::Lit(_)) {
                continue; // already handled by the single-word fast path
            }
            for start in 0..forms.len() {
                if let Some(end) = match_at(&rule.parts, &forms, start) {
                    hits[ri] += 1;
                    spans[ri].push(Span::new(lexical[start].1.start, lexical[end - 1].1.end));
                }
            }
        }

        let mut category_counts: HashMap<&str, usize> = HashMap::new();
        let mut total = 0usize;
        for (ri, rule) in self.rules.iter().enumerate() {
            if hits[ri] == 0 {
                if rule.symbol != Symbol(u32::MAX) {
                    out.set(rule.symbol, 0.0);
                }
                continue;
            }
            total += hits[ri];
            *category_counts.entry(rule.category.as_str()).or_insert(0) += hits[ri];
            if rule.symbol != Symbol(u32::MAX) {
                out.set(rule.symbol, per_1k(hits[ri], tokens));
                for span in &spans[ri] {
                    out.note_span(rule.symbol, *span);
                }
            }
        }
        for (cat, sym) in &self.categories {
            let count = category_counts.get(cat.as_str()).copied().unwrap_or(0);
            out.set(*sym, per_1k(count, tokens));
        }
        out.set(self.total, per_1k(total, tokens));

        self.structural_rules(analysis, tokens, out);
    }
}

impl FittedLexicon {
    fn structural_rules(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        if self.structural.is_empty() {
            return;
        }
        let stream = analysis.tokens();
        let structure = analysis.structure();

        // Rule of three: `A, B, and C` where A, B and C are single words.
        let mut triples = 0usize;
        let all: Vec<(&str, Span, bool)> = stream
            .iter()
            .map(|(t, f)| (f, t.span, t.kind.is_lexical()))
            .collect();
        for w in all.windows(6) {
            let pattern = [
                w[0].2,
                w[1].0 == ",",
                w[2].2,
                w[3].0 == ",",
                w[4].0 == "and",
                w[5].2,
            ];
            if pattern.iter().all(|&b| b) {
                triples += 1;
                out.note_span(self.structural[0], Span::new(w[0].1.start, w[5].1.end));
            }
        }
        out.set(self.structural[0], per_1k(triples, tokens));

        // Em-dash chains: two or more em dashes inside one sentence.
        let mut chains = 0usize;
        for sentence in &structure.sentences {
            let count = analysis
                .text(sentence.span)
                .chars()
                .filter(|&c| c == '\u{2014}')
                .count();
            if count >= 2 {
                chains += 1;
                out.note_span(self.structural[1], sentence.span);
            }
        }
        out.set(self.structural[1], per_1k(chains, tokens));

        // Title Case Headings: an ATX heading whose content words are all
        // capitalized — a documented formatting tell.
        let mut title_case = 0usize;
        for block in &structure.blocks {
            if !matches!(block.kind, crate::text::BlockKind::Heading(_)) {
                continue;
            }
            let text = analysis.text(block.span).trim_start_matches(['#', ' ']);
            let words: Vec<&str> = text.split_whitespace().collect();
            let capitalized = words
                .iter()
                .filter(|w| w.len() > 3)
                .filter(|w| w.chars().next().is_some_and(char::is_uppercase))
                .count();
            let long_words = words.iter().filter(|w| w.len() > 3).count();
            if long_words >= 2 && capitalized == long_words {
                title_case += 1;
                out.note_span(self.structural[2], block.span);
            }
        }
        out.set(self.structural[2], per_1k(title_case, tokens));
    }
}

/// Match `parts` against `forms` starting at `start`, returning the exclusive
/// end index on success.
fn match_at(parts: &[Part], forms: &[&str], start: usize) -> Option<usize> {
    fn go(parts: &[Part], forms: &[&str], i: usize) -> Option<usize> {
        let Some(part) = parts.first() else {
            return Some(i);
        };
        match part {
            Part::Lit(word) => {
                if forms.get(i) == Some(&word.as_str()) {
                    go(&parts[1..], forms, i + 1)
                } else {
                    None
                }
            }
            Part::OneOf(options) => {
                let form = forms.get(i)?;
                if options.iter().any(|o| o == form) {
                    go(&parts[1..], forms, i + 1)
                } else {
                    None
                }
            }
            Part::Gap { min, max } => {
                for skip in *min..=*max {
                    if i + skip > forms.len() {
                        break;
                    }
                    if let Some(end) = go(&parts[1..], forms, i + skip) {
                        return Some(end);
                    }
                }
                None
            }
        }
    }
    go(parts, forms, start)
}

// ---------------------------------------------------------------------------
// Built-in seed pack
// ---------------------------------------------------------------------------

/// `(id suffix, word, category, severity, alternatives)`.
type SeedTerm = (
    &'static str,
    &'static str,
    Severity,
    &'static [&'static str],
);

const EXCESS_VOCAB: &[SeedTerm] = &[
    (
        "delve",
        "excess_vocab",
        Severity::High,
        &["dig into", "look at", "examine"],
    ),
    (
        "delves",
        "excess_vocab",
        Severity::High,
        &["digs into", "examines"],
    ),
    (
        "delving",
        "excess_vocab",
        Severity::High,
        &["digging into", "examining"],
    ),
    (
        "intricate",
        "excess_vocab",
        Severity::Medium,
        &["complicated", "detailed"],
    ),
    (
        "intricacies",
        "excess_vocab",
        Severity::Medium,
        &["details", "specifics"],
    ),
    (
        "meticulous",
        "excess_vocab",
        Severity::Medium,
        &["careful", "thorough"],
    ),
    (
        "meticulously",
        "excess_vocab",
        Severity::Medium,
        &["carefully"],
    ),
    (
        "commendable",
        "excess_vocab",
        Severity::High,
        &["good", "worth doing"],
    ),
    (
        "pivotal",
        "excess_vocab",
        Severity::Medium,
        &["key", "central"],
    ),
    (
        "realm",
        "excess_vocab",
        Severity::Medium,
        &["field", "area"],
    ),
    (
        "realms",
        "excess_vocab",
        Severity::Medium,
        &["fields", "areas"],
    ),
    (
        "underscore",
        "excess_vocab",
        Severity::Medium,
        &["show", "stress"],
    ),
    (
        "underscores",
        "excess_vocab",
        Severity::Medium,
        &["shows", "stresses"],
    ),
    (
        "underscoring",
        "excess_vocab",
        Severity::Medium,
        &["showing"],
    ),
    (
        "showcase",
        "excess_vocab",
        Severity::Medium,
        &["show", "demonstrate"],
    ),
    ("showcases", "excess_vocab", Severity::Medium, &["shows"]),
    ("showcasing", "excess_vocab", Severity::Medium, &["showing"]),
    (
        "tapestry",
        "excess_vocab",
        Severity::High,
        &["mix", "range"],
    ),
    (
        "nuanced",
        "excess_vocab",
        Severity::Medium,
        &["subtle", "careful"],
    ),
    (
        "multifaceted",
        "excess_vocab",
        Severity::Medium,
        &["many-sided"],
    ),
    (
        "holistic",
        "excess_vocab",
        Severity::Medium,
        &["overall", "whole"],
    ),
    (
        "myriad",
        "excess_vocab",
        Severity::Medium,
        &["many", "countless"],
    ),
    (
        "plethora",
        "excess_vocab",
        Severity::High,
        &["lots of", "many"],
    ),
    (
        "unwavering",
        "excess_vocab",
        Severity::Medium,
        &["steady", "constant"],
    ),
    (
        "profound",
        "excess_vocab",
        Severity::Medium,
        &["deep", "big"],
    ),
    ("profoundly", "excess_vocab", Severity::Medium, &["deeply"]),
    (
        "foster",
        "excess_vocab",
        Severity::Medium,
        &["encourage", "build"],
    ),
    (
        "fostering",
        "excess_vocab",
        Severity::Medium,
        &["encouraging"],
    ),
    (
        "garner",
        "excess_vocab",
        Severity::Medium,
        &["get", "collect"],
    ),
    (
        "garnered",
        "excess_vocab",
        Severity::Medium,
        &["got", "collected"],
    ),
    ("harness", "excess_vocab", Severity::Medium, &["use"]),
    ("harnessing", "excess_vocab", Severity::Medium, &["using"]),
    (
        "elucidate",
        "excess_vocab",
        Severity::High,
        &["explain", "clarify"],
    ),
    (
        "illuminate",
        "excess_vocab",
        Severity::Medium,
        &["explain", "show"],
    ),
    (
        "embark",
        "excess_vocab",
        Severity::Medium,
        &["start", "begin"],
    ),
    (
        "navigate",
        "excess_vocab",
        Severity::Medium,
        &["handle", "work through"],
    ),
    (
        "navigating",
        "excess_vocab",
        Severity::Medium,
        &["handling"],
    ),
    (
        "unveil",
        "excess_vocab",
        Severity::Medium,
        &["show", "reveal"],
    ),
    ("unveiling", "excess_vocab", Severity::Medium, &["showing"]),
    ("spearhead", "excess_vocab", Severity::Medium, &["lead"]),
    (
        "streamline",
        "excess_vocab",
        Severity::Medium,
        &["simplify"],
    ),
    (
        "streamlining",
        "excess_vocab",
        Severity::Medium,
        &["simplifying"],
    ),
    (
        "bolster",
        "excess_vocab",
        Severity::Medium,
        &["strengthen", "back"],
    ),
    (
        "augment",
        "excess_vocab",
        Severity::Medium,
        &["add to", "extend"],
    ),
    (
        "amplify",
        "excess_vocab",
        Severity::Low,
        &["increase", "boost"],
    ),
    (
        "cultivate",
        "excess_vocab",
        Severity::Medium,
        &["build", "grow"],
    ),
    (
        "encompass",
        "excess_vocab",
        Severity::Medium,
        &["include", "cover"],
    ),
    (
        "encompassing",
        "excess_vocab",
        Severity::Medium,
        &["covering"],
    ),
    (
        "transformative",
        "excess_vocab",
        Severity::High,
        &["big", "far-reaching"],
    ),
    (
        "groundbreaking",
        "excess_vocab",
        Severity::High,
        &["new", "novel"],
    ),
    (
        "unparalleled",
        "excess_vocab",
        Severity::High,
        &["best", "unmatched"],
    ),
    (
        "unprecedented",
        "excess_vocab",
        Severity::Medium,
        &["new", "first"],
    ),
    (
        "invaluable",
        "excess_vocab",
        Severity::Medium,
        &["very useful"],
    ),
    (
        "indispensable",
        "excess_vocab",
        Severity::Medium,
        &["essential"],
    ),
    (
        "paramount",
        "excess_vocab",
        Severity::High,
        &["most important"],
    ),
    (
        "quintessential",
        "excess_vocab",
        Severity::High,
        &["typical", "classic"],
    ),
    (
        "resonate",
        "excess_vocab",
        Severity::Medium,
        &["connect", "land"],
    ),
    ("resonates", "excess_vocab", Severity::Medium, &["connects"]),
    (
        "synergy",
        "excess_vocab",
        Severity::High,
        &["fit", "overlap"],
    ),
    ("granular", "excess_vocab", Severity::Low, &["detailed"]),
    (
        "actionable",
        "excess_vocab",
        Severity::Low,
        &["usable", "concrete"],
    ),
    ("scalable", "excess_vocab", Severity::Low, &["it scales"]),
    ("versatile", "excess_vocab", Severity::Low, &["flexible"]),
    (
        "revolutionize",
        "excess_vocab",
        Severity::High,
        &["change", "improve"],
    ),
    (
        "facilitate",
        "excess_vocab",
        Severity::Medium,
        &["help", "enable"],
    ),
    ("leverage", "excess_vocab", Severity::Medium, &["use"]),
    ("leveraging", "excess_vocab", Severity::Medium, &["using"]),
    (
        "robust",
        "excess_vocab",
        Severity::Low,
        &["solid", "reliable"],
    ),
    ("seamless", "excess_vocab", Severity::Medium, &["smooth"]),
    (
        "seamlessly",
        "excess_vocab",
        Severity::Medium,
        &["smoothly"],
    ),
    (
        "comprehensive",
        "excess_vocab",
        Severity::Low,
        &["complete", "full"],
    ),
    (
        "adeptly",
        "excess_vocab",
        Severity::Medium,
        &["well", "skilfully"],
    ),
    ("deftly", "excess_vocab", Severity::Medium, &["neatly"]),
    ("aptly", "excess_vocab", Severity::Low, &["fittingly"]),
    ("keenly", "excess_vocab", Severity::Low, &["sharply"]),
    (
        "testament",
        "inflation",
        Severity::High,
        &["evidence", "sign"],
    ),
    (
        "cornerstone",
        "inflation",
        Severity::Medium,
        &["basis", "foundation"],
    ),
    ("backbone", "inflation", Severity::Medium, &["core"]),
    (
        "landscape",
        "inflation",
        Severity::Medium,
        &["field", "situation"],
    ),
    (
        "ecosystem",
        "inflation",
        Severity::Low,
        &["set of tools", "community"],
    ),
    (
        "paradigm",
        "inflation",
        Severity::Medium,
        &["model", "approach"],
    ),
    ("beacon", "inflation", Severity::High, &["example"]),
    ("hallmark", "inflation", Severity::Medium, &["sign", "mark"]),
    ("boasts", "copula_avoidance", Severity::High, &["has"]),
    ("boasting", "copula_avoidance", Severity::High, &["with"]),
    (
        "undeniably",
        "hedge",
        Severity::Medium,
        &["clearly", "(cut it)"],
    ),
    ("arguably", "hedge", Severity::Low, &["(cut it)"]),
    ("notably", "hedge", Severity::Low, &["(cut it)"]),
    ("crucially", "hedge", Severity::Medium, &["(cut it)"]),
    ("fundamentally", "hedge", Severity::Low, &["(cut it)"]),
    ("inherently", "hedge", Severity::Low, &["(cut it)"]),
    ("intrinsically", "hedge", Severity::Medium, &["(cut it)"]),
];

/// `(id, pattern, category, severity, message)`.
type SeedPhrase = (
    &'static str,
    &'static str,
    &'static str,
    Severity,
    &'static str,
);

const SEED_PHRASES: &[SeedPhrase] = &[
    (
        "phrase.not_just_but",
        "not just * but *",
        "negative_parallelism",
        Severity::High,
        "negative parallelism: say the positive claim directly",
    ),
    (
        "phrase.not_only_but_also",
        "not only * but also *",
        "negative_parallelism",
        Severity::High,
        "negative parallelism: say the positive claim directly",
    ),
    (
        "phrase.its_not_x_its_y",
        "it's not * it's *",
        "negative_parallelism",
        Severity::High,
        "negative parallelism: say the positive claim directly",
    ),
    (
        "phrase.worth_noting",
        "it is worth noting that",
        "filler",
        Severity::Medium,
        "delete the frame and state the fact",
    ),
    (
        "phrase.worth_noting_contracted",
        "it's worth noting that",
        "filler",
        Severity::Medium,
        "delete the frame and state the fact",
    ),
    (
        "phrase.important_to_note",
        "it is important to note",
        "filler",
        Severity::Medium,
        "delete the frame and state the fact",
    ),
    (
        "phrase.plays_a_role",
        "plays a {crucial|vital|pivotal|key|significant|central} role",
        "inflation",
        Severity::High,
        "say what it actually does",
    ),
    (
        "phrase.stands_as_testament",
        "stands as a testament",
        "inflation",
        Severity::High,
        "state the evidence instead",
    ),
    (
        "phrase.ever_evolving",
        "ever evolving {landscape|world|field}",
        "inflation",
        Severity::High,
        "",
    ),
    (
        "phrase.in_todays_world",
        "in today's {fast|digital|modern|connected} *",
        "inflation",
        Severity::Medium,
        "",
    ),
    (
        "phrase.wide_range",
        "a wide {range|array|variety} of",
        "filler",
        Severity::Low,
        "",
    ),
    (
        "phrase.delve_into",
        "delve into",
        "excess_vocab",
        Severity::High,
        "use \"look at\" or \"dig into\"",
    ),
    (
        "phrase.dive_into",
        "{let's|lets} {dive|delve} into",
        "excess_vocab",
        Severity::Medium,
        "",
    ),
    (
        "phrase.shed_light",
        "shed light on",
        "filler",
        Severity::Medium,
        "",
    ),
    (
        "phrase.pave_the_way",
        "pave the way",
        "inflation",
        Severity::Medium,
        "",
    ),
    (
        "phrase.navigate_complexities",
        "navigate the {complexities|landscape|challenges}",
        "excess_vocab",
        Severity::High,
        "",
    ),
    (
        "phrase.experts_argue",
        "experts {argue|say|believe|agree|note}",
        "vague_attribution",
        Severity::High,
        "name the expert or drop the claim",
    ),
    (
        "phrase.studies_show",
        "studies {show|suggest|indicate|showed}",
        "vague_attribution",
        Severity::High,
        "cite the study or drop the claim",
    ),
    (
        "phrase.some_argue",
        "some {argue|say|believe|claim}",
        "vague_attribution",
        Severity::Medium,
        "name who",
    ),
    (
        "phrase.research_suggests",
        "research {suggests|shows|indicates}",
        "vague_attribution",
        Severity::High,
        "cite it or drop it",
    ),
    (
        "phrase.one_of_the_most",
        "one of the most",
        "inflation",
        Severity::Low,
        "",
    ),
    (
        "phrase.serves_as",
        "serves as {a|an|the}",
        "copula_avoidance",
        Severity::Medium,
        "use \"is\"",
    ),
    (
        "phrase.when_it_comes_to",
        "when it comes to",
        "filler",
        Severity::Medium,
        "cut the frame",
    ),
    (
        "phrase.at_the_end_of_the_day",
        "at the end of the day",
        "filler",
        Severity::Medium,
        "",
    ),
    (
        "phrase.in_conclusion",
        "in conclusion",
        "formulaic_close",
        Severity::High,
        "stop when you are done",
    ),
    (
        "phrase.to_sum_up",
        "to sum up",
        "formulaic_close",
        Severity::Medium,
        "",
    ),
    (
        "phrase.in_summary",
        "in summary",
        "formulaic_close",
        Severity::Medium,
        "",
    ),
    (
        "phrase.that_being_said",
        "that being said",
        "filler",
        Severity::Low,
        "",
    ),
    (
        "phrase.rest_assured",
        "rest assured",
        "filler",
        Severity::Medium,
        "",
    ),
    (
        "phrase.look_no_further",
        "look no further",
        "filler",
        Severity::High,
        "",
    ),
    (
        "phrase.unlock_the",
        "unlock the {power|potential|secrets|value} of",
        "inflation",
        Severity::High,
        "",
    ),
    (
        "phrase.harness_the_power",
        "harness the power of",
        "inflation",
        Severity::High,
        "",
    ),
    (
        "phrase.game_changer",
        "a game changer",
        "inflation",
        Severity::High,
        "",
    ),
    (
        "phrase.no_denying",
        "there is no denying",
        "filler",
        Severity::Medium,
        "",
    ),
    (
        "phrase.key_takeaway",
        "the key {takeaway|takeaways}",
        "formulaic_close",
        Severity::Low,
        "",
    ),
    (
        "phrase.cutting_edge",
        "cutting edge",
        "inflation",
        Severity::Medium,
        "",
    ),
    (
        "phrase.state_of_the_art",
        "state of the art",
        "inflation",
        Severity::Low,
        "",
    ),
];

impl LexiconPack {
    /// The built-in AI-style seed pack, dated 2026-07.
    ///
    /// A starting point, not a fixture: refit or replace it against a current
    /// per-model corpus (see `handprint-data`) before relying on it for
    /// anything load-bearing. Terms and rates both decay.
    pub fn ai_slop() -> Self {
        let terms = EXCESS_VOCAB
            .iter()
            .map(|(word, category, severity, alternatives)| Term {
                id: format!("word.{word}"),
                word: (*word).to_owned(),
                category: (*category).to_owned(),
                severity: *severity,
                alternatives: alternatives.iter().map(|s| (*s).to_owned()).collect(),
            })
            .collect();
        let phrases = SEED_PHRASES
            .iter()
            .map(|(id, pattern, category, severity, message)| Phrase {
                id: (*id).to_owned(),
                pattern: (*pattern).to_owned(),
                category: (*category).to_owned(),
                severity: *severity,
                message: (*message).to_owned(),
            })
            .collect();
        LexiconPack {
            name: "ai-slop".into(),
            version: "2026.07".into(),
            date: "2026-07-31".into(),
            description: "Seed AI-style vocabulary and phrase patterns.".into(),
            license: "CC0-1.0".into(),
            redistributable: true,
            sources: vec![
                PackSource {
                    name: "berenslab/llm-excess-vocab".into(),
                    url: "https://github.com/berenslab/llm-excess-vocab".into(),
                    note: "excess-vocabulary word list (Kobak et al., Science Advances 2025)"
                        .into(),
                },
                PackSource {
                    name: "Liang et al., ICML 2024".into(),
                    url: "https://arxiv.org/abs/2403.07183".into(),
                    note: "top adjectives and adverbs over-used in LLM-assisted peer reviews"
                        .into(),
                },
                PackSource {
                    name: "Wikipedia: Signs of AI writing".into(),
                    url: "https://en.wikipedia.org/wiki/Wikipedia:WikiProject_AI_Cleanup/Guide"
                        .into(),
                    note: "phrase patterns: significance inflation, copula avoidance, negative \
                           parallelism, vague attribution, formulaic conclusions"
                        .into(),
                },
            ],
            terms,
            phrases,
            privacy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn transform(text: &str) -> (FeatureVector, Interner, FittedLexicon) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = LexiconFeature::default().fit(&ctx, &mut interner).unwrap();
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner, fitted)
    }

    #[test]
    fn seed_pack_is_well_formed() {
        let pack = LexiconPack::ai_slop();
        assert!(!pack.sources.is_empty());
        assert_eq!(pack.qualified_name(), "ai-slop@2026.07");
        let mut ids: Vec<&str> = pack
            .terms
            .iter()
            .map(|t| t.id.as_str())
            .chain(pack.phrases.iter().map(|p| p.id.as_str()))
            .collect();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), before, "duplicate rule ids in the seed pack");
        for phrase in &pack.phrases {
            parse_pattern(&phrase.pattern).unwrap_or_else(|e| panic!("{}: {e}", phrase.id));
        }
    }

    #[test]
    fn pack_round_trips_through_json() {
        let pack = LexiconPack::ai_slop();
        let json = serde_json::to_string(&pack).unwrap();
        let back: LexiconPack = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pack);
    }

    #[test]
    fn word_hits_carry_spans() {
        let text = "we should delve into the tapestry of options";
        let (v, i, _) = transform(text);
        let sym = i.get("lex:ai-slop:word.delve").unwrap();
        assert!(v.get(sym) > 0.0);
        let spans = v.spans(sym);
        assert_eq!(&text[spans[0].range()], "delve");
        assert!(v.get(i.get("lex:ai-slop:word.tapestry").unwrap()) > 0.0);
    }

    #[test]
    fn phrase_patterns_match_with_gaps_and_alternatives() {
        let (v, i, _) = transform("it is not just a tool but a platform for everyone");
        assert!(v.get(i.get("lex:ai-slop:phrase.not_just_but").unwrap()) > 0.0);

        let (v, i, _) = transform("this plays a crucial role in the system");
        assert!(v.get(i.get("lex:ai-slop:phrase.plays_a_role").unwrap()) > 0.0);

        let (v, i, _) = transform("this plays a minor role in the system");
        assert_eq!(
            v.get(i.get("lex:ai-slop:phrase.plays_a_role").unwrap()),
            0.0
        );
    }

    #[test]
    fn phrase_spans_cover_the_whole_match() {
        let text = "studies show that this works";
        let (v, i, _) = transform(text);
        let spans = v.spans(i.get("lex:ai-slop:phrase.studies_show").unwrap());
        assert_eq!(&text[spans[0].range()], "studies show");
    }

    #[test]
    fn category_and_total_rollups() {
        let (v, i, _) = transform("delve into the tapestry and the intricate realm of things");
        let cat = v.get(i.get("lex:ai-slop:cat:excess_vocab").unwrap());
        let total = v.get(i.get("lex:ai-slop:total").unwrap());
        assert!(cat > 0.0);
        assert!(total >= cat);
    }

    #[test]
    fn structural_rules_fire() {
        let (v, i, _) = transform("it is fast, cheap, and reliable in practice");
        assert!(v.get(i.get("lex:ai-slop:struct:rule_of_three").unwrap()) > 0.0);

        let (v, i, _) = transform("the thing — which matters — is here — mostly.");
        assert!(v.get(i.get("lex:ai-slop:struct:em_dash_chain").unwrap()) > 0.0);

        let (v, i, _) = transform("# Getting Started With Handprint\n\nsome body text here now");
        assert!(v.get(i.get("lex:ai-slop:struct:title_case_heading").unwrap()) > 0.0);
    }

    #[test]
    fn clean_text_scores_zero() {
        let (v, i, _) = transform("I ran the numbers again and they still look wrong to me.");
        assert_eq!(v.get(i.get("lex:ai-slop:total").unwrap()), 0.0);
    }

    #[test]
    fn alternatives_are_available_for_findings() {
        let (_, i, fitted) = transform("delve");
        let sym = i.get("lex:ai-slop:word.delve").unwrap();
        assert!(fitted
            .alternatives_of(sym)
            .contains(&"dig into".to_string()));
        assert_eq!(fitted.severity_of(sym), Some(Severity::High));
    }

    #[test]
    fn category_findings_makes_the_category_rates_reportable() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);

        // Default: categories are rollups and never become findings.
        let mut interner = Interner::new();
        let default = LexiconFeature::default().fit(&ctx, &mut interner).unwrap();
        let sym = interner.get("lex:ai-slop:cat:hedge").unwrap();
        let dim = default.dims().iter().find(|d| d.symbol == sym).unwrap();
        assert!(dim.aggregate);

        // With the flag: reportable, and the per-term dimensions are gone.
        let mut interner = Interner::new();
        let fitted = LexiconFeature::default()
            .category_findings()
            .fit(&ctx, &mut interner)
            .unwrap();
        let sym = interner.get("lex:ai-slop:cat:hedge").unwrap();
        let dim = fitted.dims().iter().find(|d| d.symbol == sym).unwrap();
        assert!(!dim.aggregate, "category rate must be reportable");
        assert!(
            interner.get("lex:ai-slop:word.delve").is_none(),
            "per-term dimensions should be off"
        );
        // The pack total stays a rollup either way: "reduce your total lexicon
        // rate" is not an instruction.
        let total = interner.get("lex:ai-slop:total").unwrap();
        assert!(
            fitted
                .dims()
                .iter()
                .find(|d| d.symbol == total)
                .unwrap()
                .aggregate
        );
    }

    #[test]
    fn category_rates_still_count_hits_when_per_term_is_off() {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = LexiconFeature::default()
            .category_findings()
            .fit(&ctx, &mut interner)
            .unwrap();
        let doc = Document::new("a tapestry of meticulous work");
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&analysis, &mut b);
        let v = b.build();
        // Five lexical tokens, two excess-vocab hits -> 400 per 1k. The rules
        // are still compiled and matched with per_term off; only the per-term
        // dimensions are gone.
        let cat = v.get(interner.get("lex:ai-slop:cat:excess_vocab").unwrap());
        assert!((cat - 400.0).abs() < 1e-9, "{cat}");
    }

    #[test]
    fn the_spec_flag_serde_defaults_so_old_artifacts_load_unchanged() {
        let json = serde_json::to_string(&LexiconFeature::default()).unwrap();
        let stripped = json.replace(",\"category_findings\":false", "");
        let back: LexiconFeature = serde_json::from_str(&stripped).unwrap();
        assert!(!back.category_findings);
        assert_eq!(back, LexiconFeature::default());
    }

    #[test]
    fn custom_pack_namespaces_dimensions() {
        let pack = LexiconPack {
            name: "mypack".into(),
            version: "1".into(),
            date: "2026-07-31".into(),
            description: String::new(),
            license: String::new(),
            redistributable: true,
            sources: vec![],
            terms: vec![Term {
                id: "word.foo".into(),
                word: "foo".into(),
                category: "test".into(),
                severity: Severity::Low,
                alternatives: vec![],
            }],
            phrases: vec![],
            privacy: None,
        };
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = LexiconFeature::new(pack).fit(&ctx, &mut interner).unwrap();
        assert_eq!(fitted.pack(), "mypack@1");
        assert!(interner.get("lex:mypack:word.foo").is_some());
    }
}
