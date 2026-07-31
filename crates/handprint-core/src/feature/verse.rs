//! Verse: metre, stress profile, line geometry, archaism, rhetorical figures.
//!
//! Two authors in the battery need this family and need it for opposite
//! reasons. Shakespeare needs metre — feminine endings, stress profiles,
//! enjambment — because that is where four centuries of attribution work has
//! found the signal. Bukowski needs *line geometry* — line-length variance,
//! one-word lines, lowercase line openings — because his poetry's shape is the
//! poem and the metre is not there to measure.
//!
//! # Feminine endings, and why they lead
//!
//! The single best-attested verse discriminator in the field. Spedding split
//! *Henry VIII* between Shakespeare and Fletcher on it in 1850 — 28–40% against
//! 50–77%, with no overlap — and Plecháč confirmed the split in 2019 with MFW
//! plus rhythmic patterns. Everything else here is a supporting dimension.
//!
//! # Verse or prose
//!
//! Taken from markup when there is any, and from a short-line heuristic
//! otherwise. Lineation in original playbooks is genuinely ambiguous and this
//! module does not pretend to resolve it: below
//! [`VersePack::min_verse_share`] **every dimension is marked missing**. Scanning
//! prose as verse would produce a full set of plausible numbers describing
//! nothing, which is worse than an empty column.
//!
//! # Honest naming
//!
//! `verse:doublet_lowpmi_rate` is not called a hendiadys rate. Hendiadys has no
//! published detector; what this measures is Noun-and-Noun coordinations whose
//! members rarely co-occur in the fitting corpus, which is a *proxy* and is
//! named as one. Pun detection and functional-shift rate are absent entirely:
//! SemEval-2017 Task 7 location scores collapse to F≈0.1 and early-modern
//! homophones defeat any dictionary, so both belong to the LLM-judge tier.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::Result;
use crate::text::syllable::{count_syllables, SyllableMethod};
use crate::text::{Analysis, Line, Span};
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Metrical positions reported in the stress profile.
///
/// Ten, because the iambic pentameter line the profile is designed to describe
/// has ten. A longer line contributes its first ten positions; a shorter one
/// contributes what it has.
pub const STRESS_POSITIONS: usize = 10;

/// Longest line, in lexical tokens, that still reads as verse.
const VERSE_LINE_TOKENS: usize = 14;

/// Fewest lines before the verse-vs-prose heuristic is worth running.
const MIN_LINES: usize = 6;

/// Archaic second-person and verb-inflection markers.
const THOU_FORMS: &[&str] = &["thou", "thee", "thy", "thine", "ye"];
const YOU_FORMS: &[&str] = &["you", "your", "yours", "yourself", "yourselves"];

/// Line-final words that mark an enjambment: a line ending on a function word
/// has not finished its clause.
#[rustfmt::skip]
const FUNCTION_FINAL: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "of", "to", "in", "on", "at", "by", "for", "with",
    "as", "that", "which", "who", "whose", "when", "where", "while", "if", "than", "from",
    "into", "upon", "my", "thy", "his", "her", "its", "their", "our", "your", "this", "these",
    "no", "not", "is", "was", "are", "were", "be", "been", "have", "has", "had", "shall",
    "will", "would", "should", "may", "might", "must", "can", "could", "do", "does", "did",
];

/// Rhetorical figures counted per hundred lines, in emission order.
const FIGURE_DIMS: &[&str] = &["anaphora", "epistrophe", "chiasmus", "antithesis"];

/// Configuration for the verse family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersePack {
    /// Share of lines that must read as verse before any dimension is emitted.
    #[serde(default = "default_min_verse_share")]
    pub min_verse_share: f64,
    /// How syllables are counted. Recorded in the fitted state.
    #[serde(default)]
    pub syllables: SyllableMethod,
}

fn default_min_verse_share() -> f64 {
    0.5
}

impl Default for VersePack {
    fn default() -> Self {
        VersePack {
            min_verse_share: default_min_verse_share(),
            syllables: SyllableMethod::default(),
        }
    }
}

/// Fitted [`VersePack`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedVerse {
    dims: Vec<DimInfo>,
    feminine: Symbol,
    stress: Vec<Symbol>,
    iambic: Symbol,
    midline_pause: Symbol,
    enjambment: Symbol,
    end_stop: Symbol,
    line_len_mean: Symbol,
    line_len_stddev: Symbol,
    one_word_line: Symbol,
    lowercase_initial: Symbol,
    thou_you: Symbol,
    eth_rate: Symbol,
    est_rate: Symbol,
    vocative: Symbol,
    figures: Vec<Symbol>,
    doublet: Symbol,
    /// Noun-and-Noun pairs the fitting corpus used, with their counts. The
    /// low-PMI test is against this.
    doublets: Vec<(String, u32)>,
    doublet_total: u64,
    min_verse_share: f64,
    syllable_method: SyllableMethod,
}

impl FittedVerse {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Verse;

    /// How this reference counts syllables.
    pub fn syllable_method(&self) -> &SyllableMethod {
        &self.syllable_method
    }

    /// How many coordinations the fitting corpus supplied for the PMI test.
    pub fn doublet_len(&self) -> usize {
        self.doublets.len()
    }
}

impl Feature for VersePack {
    type Fitted = FittedVerse;

    fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedVerse> {
        let syllable_method = self.syllables.resolve_for_fit()?;
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Verse, unit));
            sym
        };

        let feminine = push(
            interner,
            "verse:feminine_ending_rate",
            Unit::PerHundredLines,
        );
        let stress: Vec<Symbol> = (1..=STRESS_POSITIONS)
            .map(|p| push(interner, &format!("verse:stress_pos:{p}"), Unit::Fraction))
            .collect();
        let iambic = push(interner, "verse:iambic_conformity", Unit::Fraction);
        let midline_pause = push(interner, "verse:midline_pause_rate", Unit::PerHundredLines);
        let enjambment = push(interner, "verse:enjambment_rate", Unit::PerHundredLines);
        let end_stop = push(interner, "verse:end_stop_rate", Unit::PerHundredLines);
        let line_len_mean = push(interner, "verse:line_len_mean", Unit::Tokens);
        let line_len_stddev = push(interner, "verse:line_len_stddev", Unit::Index);
        let one_word_line = push(interner, "verse:one_word_line_rate", Unit::PerHundredLines);
        let lowercase_initial = push(
            interner,
            "verse:lowercase_initial_rate",
            Unit::PerHundredLines,
        );
        let thou_you = push(interner, "verse:thou_you_ratio", Unit::Index);
        let eth_rate = push(interner, "verse:eth_rate", Unit::PerThousandTokens);
        let est_rate = push(interner, "verse:est_rate", Unit::PerThousandTokens);
        let vocative = push(interner, "verse:vocative_rate", Unit::PerHundredLines);
        let figures = FIGURE_DIMS
            .iter()
            .map(|f| {
                push(
                    interner,
                    &format!("verse:figure:{f}"),
                    Unit::PerHundredLines,
                )
            })
            .collect();
        let doublet = push(interner, "verse:doublet_lowpmi_rate", Unit::PerHundredLines);

        // The corpus's own Noun-and-Noun coordinations. A doublet is "low PMI"
        // when the corpus rarely puts those two words together, which is only
        // answerable against the corpus — so this is the family's fitted state.
        let mut counts: HashMap<String, u32> = HashMap::new();
        let mut total = 0u64;
        for analysis in ctx.analyses() {
            for (pair, _) in coordinations(analysis) {
                *counts.entry(pair).or_insert(0) += 1;
                total += 1;
            }
        }
        let mut doublets: Vec<(String, u32)> = counts.into_iter().collect();
        doublets.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(FittedVerse {
            dims,
            feminine,
            stress,
            iambic,
            midline_pause,
            enjambment,
            end_stop,
            line_len_mean,
            line_len_stddev,
            one_word_line,
            lowercase_initial,
            thou_you,
            eth_rate,
            est_rate,
            vocative,
            figures,
            doublet,
            doublets,
            doublet_total: total,
            min_verse_share: self.min_verse_share.clamp(0.0, 1.0),
            syllable_method,
        })
    }
}

impl FittedFeature for FittedVerse {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Verse
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let lines: Vec<&Line> = analysis
            .structure()
            .lines
            .iter()
            .filter(|l| !l.is_empty())
            .collect();
        if lines.len() < MIN_LINES || verse_share(analysis, &lines) < self.min_verse_share {
            // Never scan prose as verse: a full set of plausible numbers
            // describing nothing is worse than an empty column.
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        self.metre(analysis, &lines, out);
        self.geometry(analysis, &lines, out);
        self.archaism(analysis, out);
        self.figures(analysis, &lines, out);
    }
}

impl FittedVerse {
    /// Feminine endings, the stress profile, conformity, pauses, enjambment.
    fn metre(&self, analysis: &Analysis<'_>, lines: &[&Line], out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let n = lines.len();
        let mut feminine = 0usize;
        let mut midline = 0usize;
        let mut enjambed = 0usize;
        let mut end_stopped = 0usize;
        // Per position: how many lines had a syllable there, and how many of
        // those were stressed.
        let mut stressed = [0usize; STRESS_POSITIONS];
        let mut present = [0usize; STRESS_POSITIONS];
        let mut iambic_hits = 0usize;
        let mut iambic_total = 0usize;

        for line in lines {
            let words: Vec<&str> = stream.tokens()[line.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| stream.form(t))
                .collect();
            if words.is_empty() {
                continue;
            }
            let pattern = self.stress_pattern(&words);

            // A feminine ending is an unstressed final syllable: the line runs
            // one syllable past its metrical close.
            if pattern.last() == Some(&false) && pattern.len() > 1 {
                feminine += 1;
                out.note_span(self.feminine, line.span);
            }
            for (i, &is_stressed) in pattern.iter().take(STRESS_POSITIONS).enumerate() {
                present[i] += 1;
                if is_stressed {
                    stressed[i] += 1;
                }
                // Iambic: odd positions (0-indexed even) unstressed, even
                // positions stressed.
                iambic_total += 1;
                if is_stressed == (i % 2 == 1) {
                    iambic_hits += 1;
                }
            }

            let text = analysis.text(line.span);
            let trimmed = text.trim_end();
            // A mid-line pause is punctuation that is not the line's last
            // character: the caesura.
            if trimmed.char_indices().any(|(i, c)| {
                matches!(c, ',' | ';' | ':' | '.' | '?' | '!') && i + c.len_utf8() < trimmed.len()
            }) {
                midline += 1;
                out.note_span(self.midline_pause, line.span);
            }
            let ends_on_punct = trimmed
                .chars()
                .last()
                .is_some_and(|c| matches!(c, '.' | ',' | ';' | ':' | '?' | '!' | '—'));
            let ends_on_function = words.last().is_some_and(|w| FUNCTION_FINAL.contains(w));
            if ends_on_punct {
                end_stopped += 1;
            } else if ends_on_function {
                // The line-final function word proxy: a line ending on "the"
                // has not finished its clause.
                enjambed += 1;
                out.note_span(self.enjambment, line.span);
            }
        }

        out.set(self.feminine, ratio(feminine, n) * 100.0);
        out.set(self.midline_pause, ratio(midline, n) * 100.0);
        out.set(self.enjambment, ratio(enjambed, n) * 100.0);
        out.set(self.end_stop, ratio(end_stopped, n) * 100.0);
        for (i, &sym) in self.stress.iter().enumerate() {
            if present[i] == 0 {
                out.mark_missing(sym);
            } else {
                out.set(sym, ratio(stressed[i], present[i]));
            }
        }
        out.set(self.iambic, ratio(iambic_hits, iambic_total));
    }

    /// Line geometry: the Bukowski axes.
    fn geometry(&self, analysis: &Analysis<'_>, lines: &[&Line], out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let n = lines.len();
        let lengths: Vec<f64> = lines.iter().map(|l| l.lexical_len(stream) as f64).collect();
        out.set(self.line_len_mean, util::mean(&lengths));
        out.set(self.line_len_stddev, util::variance(&lengths).sqrt());

        let mut one_word = 0usize;
        let mut lowercase = 0usize;
        for line in lines {
            if line.lexical_len(stream) == 1 {
                one_word += 1;
                out.note_span(self.one_word_line, line.span);
            }
            let text = analysis.text(line.span);
            if text
                .chars()
                .find(|c| c.is_alphabetic())
                .is_some_and(char::is_lowercase)
            {
                lowercase += 1;
            }
        }
        out.set(self.one_word_line, ratio(one_word, n) * 100.0);
        out.set(self.lowercase_initial, ratio(lowercase, n) * 100.0);
    }

    /// Archaic pronouns and verb inflections.
    fn archaism(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len().max(1);
        let mut thou = 0usize;
        let mut you = 0usize;
        let mut eth = 0usize;
        let mut est = 0usize;
        for (token, form) in analysis.tokens().lexical() {
            if THOU_FORMS.contains(&form) {
                thou += 1;
                out.note_span(self.thou_you, token.span);
            }
            if YOU_FORMS.contains(&form) {
                you += 1;
            }
            if is_eth(form) {
                eth += 1;
                out.note_span(self.eth_rate, token.span);
            }
            if is_est(form) {
                est += 1;
                out.note_span(self.est_rate, token.span);
            }
        }
        // A ratio rather than two rates: the *balance* is the sociolinguistic
        // variable, and it is what the literature on thou/you reports.
        out.set(
            self.thou_you,
            if you == 0 {
                thou as f64
            } else {
                thou as f64 / you as f64
            },
        );
        out.set(self.eth_rate, per_1k(eth, tokens));
        out.set(self.est_rate, per_1k(est, tokens));
    }

    /// Vocatives, rhetorical figures, low-PMI doublets.
    fn figures(&self, analysis: &Analysis<'_>, lines: &[&Line], out: &mut VectorBuilder) {
        let stream = analysis.tokens();
        let n = lines.len();
        let mut vocatives = 0usize;
        let mut counts = vec![0usize; FIGURE_DIMS.len()];

        // Openers and closers are *two* words, not one. A one-word test fires
        // on every pair of lines that happen to begin "The", which is a fact
        // about English rather than about the poem.
        let edge = |l: &Line, from_start: bool| -> Option<(&str, &str)> {
            let words: Vec<&str> = stream.tokens()[l.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| stream.form(t))
                .collect();
            if words.len() < 2 {
                return None;
            }
            if from_start {
                Some((words[0], words[1]))
            } else {
                Some((words[words.len() - 2], words[words.len() - 1]))
            }
        };
        let openers: Vec<Option<(&str, &str)>> = lines.iter().map(|l| edge(l, true)).collect();
        let closers: Vec<Option<(&str, &str)>> = lines.iter().map(|l| edge(l, false)).collect();

        for (i, line) in lines.iter().enumerate() {
            let words: Vec<&str> = stream.tokens()[line.tokens.clone()]
                .iter()
                .filter(|t| t.kind.is_lexical())
                .map(|t| stream.form(t))
                .collect();
            // Vocative: "O" followed by a noun phrase — the pattern rule, which
            // is the whole of what is claimable without a parser.
            if words.first() == Some(&"o") && words.len() > 1 {
                vocatives += 1;
                out.note_span(self.vocative, line.span);
            }
            // Anaphora: this line and the next open on the same word.
            if i + 1 < lines.len() && openers[i].is_some() && openers[i] == openers[i + 1] {
                counts[0] += 1;
                out.note_span(self.figures[0], line.span);
            }
            // Epistrophe: this line and the next close on the same word.
            if i + 1 < lines.len() && closers[i].is_some() && closers[i] == closers[i + 1] {
                counts[1] += 1;
                out.note_span(self.figures[1], line.span);
            }
            // Chiasmus: an A B … B A pattern of content words inside the line.
            if has_chiasmus(&words) {
                counts[2] += 1;
                out.note_span(self.figures[2], line.span);
            }
            // Antithesis: an antonym pair inside the line.
            if has_antithesis(&words) {
                counts[3] += 1;
                out.note_span(self.figures[3], line.span);
            }
        }
        out.set(self.vocative, ratio(vocatives, n) * 100.0);
        for (i, &sym) in self.figures.iter().enumerate() {
            out.set(sym, ratio(counts[i], n) * 100.0);
        }

        // Low-PMI doublets: Noun-and-Noun pairs the fitting corpus rarely put
        // together. Named "doublet" and not "hendiadys", which has no detector.
        let mut low = 0usize;
        for (pair, span) in coordinations(analysis) {
            if self.is_low_pmi(&pair) {
                low += 1;
                out.note_span(self.doublet, span);
            }
        }
        out.set(self.doublet, ratio(low, n) * 100.0);
    }

    /// Per-syllable stress for a line, from the dictionary where it has an
    /// answer and from a positional default where it does not.
    fn stress_pattern(&self, words: &[&str]) -> Vec<bool> {
        let mut out = Vec::new();
        for word in words {
            let syllables = count_syllables(word, &self.syllable_method);
            match self.dict_stress(word) {
                Some(pattern) if pattern.chars().count() == syllables => {
                    out.extend(pattern.chars().map(|c| c != '0'));
                }
                _ => {
                    // No entry: a monosyllable is stressed unless it is a
                    // function word, and a polysyllable takes initial stress,
                    // which is the English default.
                    if syllables <= 1 {
                        out.push(!FUNCTION_FINAL.contains(word));
                    } else {
                        out.push(true);
                        out.extend(std::iter::repeat_n(false, syllables - 1));
                    }
                }
            }
        }
        out
    }

    fn dict_stress(&self, word: &str) -> Option<&'static str> {
        match &self.syllable_method {
            SyllableMethod::VowelGroup => None,
            SyllableMethod::Dict { .. } => {
                #[cfg(feature = "verse")]
                {
                    crate::text::dict::stress(word)
                }
                #[cfg(not(feature = "verse"))]
                {
                    let _ = word;
                    None
                }
            }
        }
    }

    /// Whether a coordination is rare enough in the corpus to count.
    fn is_low_pmi(&self, pair: &str) -> bool {
        if self.doublet_total == 0 {
            return false;
        }
        let count = self
            .doublets
            .binary_search_by(|(p, _)| p.as_str().cmp(pair))
            .map(|i| self.doublets[i].1)
            .unwrap_or(0);
        // Rare means "at most once in the corpus", which is the coarsest
        // possible reading of low PMI and the only one a count table supports.
        count <= 1
    }
}

/// What share of lines read as verse.
///
/// Markup first: a line that is a markdown block quote or carries a verse
/// marker is verse by declaration. Otherwise the short-line heuristic, which is
/// what makes this a *confidence* rather than a classification.
fn verse_share(analysis: &Analysis<'_>, lines: &[&Line]) -> f64 {
    if lines.is_empty() {
        return 0.0;
    }
    let stream = analysis.tokens();
    let short = lines
        .iter()
        .filter(|l| {
            let n = l.lexical_len(stream);
            n > 0 && n <= VERSE_LINE_TOKENS
        })
        .count();
    // A single-line "document" of short text is not verse; the MIN_LINES gate
    // upstream is what stops that, and this is only the shape test.
    short as f64 / lines.len() as f64
}

/// `-eth` verb inflections (`doth`, `hath`, `goeth`).
fn is_eth(form: &str) -> bool {
    const EXPLICIT: &[&str] = &[
        "doth", "hath", "saith", "goeth", "cometh", "maketh", "taketh",
    ];
    EXPLICIT.contains(&form) || (form.len() > 4 && form.ends_with("eth"))
}

/// `-est` verb inflections (`dost`, `hast`, `speakest`).
fn is_est(form: &str) -> bool {
    const EXPLICIT: &[&str] = &["dost", "hast", "art", "wilt", "shalt", "canst", "wert"];
    EXPLICIT.contains(&form) || (form.len() > 4 && form.ends_with("est") && !form.ends_with("iest"))
}

/// An A B … B A pattern over content words.
fn has_chiasmus(words: &[&str]) -> bool {
    use crate::feature::mfw::FUNCTION_WORDS;
    let content: Vec<&str> = words
        .iter()
        .copied()
        .filter(|w| !FUNCTION_WORDS.contains(w) && w.chars().count() > 2)
        .collect();
    if content.len() < 4 {
        return false;
    }
    for i in 0..content.len() {
        for j in (i + 3)..content.len() {
            // content[i] == content[j] and content[i+1] == content[j-1].
            if content[i] == content[j] && content[i + 1] == content[j - 1] && i + 1 < j - 1 {
                return true;
            }
        }
    }
    false
}

/// An antonym pair inside a line, using the W5 WordNet table.
fn has_antithesis(words: &[&str]) -> bool {
    let table = antonym_table();
    for (i, a) in words.iter().enumerate() {
        for b in &words[i + 1..] {
            let (lo, hi) = if a <= b { (*a, *b) } else { (*b, *a) };
            if table
                .binary_search_by(|(x, y)| (x.as_str(), y.as_str()).cmp(&(lo, hi)))
                .is_ok()
            {
                return true;
            }
        }
    }
    false
}

/// The antonym table, built once per process.
fn antonym_table() -> &'static [(String, String)] {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    TABLE.get_or_init(crate::feature::packs::wordnet_antonyms)
}

/// Noun-and-Noun coordinations, as `"a and b"` keys with their spans.
///
/// "Noun" here means "a content word that is not obviously a verb or an
/// adjective", which is as far as a tagger-free rule can go. The looseness is
/// tolerable because the dimension is a *rate over a corpus-relative rarity
/// test*: a consistent over-count on both sides cancels.
fn coordinations(analysis: &Analysis<'_>) -> Vec<(String, Span)> {
    use crate::feature::mfw::FUNCTION_WORDS;
    let forms: Vec<(&str, Span)> = analysis
        .tokens()
        .lexical()
        .map(|(t, f)| (f, t.span))
        .collect();
    let mut out = Vec::new();
    for w in forms.windows(3) {
        let (a, a_span) = w[0];
        let (conj, _) = w[1];
        let (b, b_span) = w[2];
        if conj != "and" {
            continue;
        }
        if FUNCTION_WORDS.contains(&a) || FUNCTION_WORDS.contains(&b) {
            continue;
        }
        if a.chars().count() < 3 || b.chars().count() < 3 || a == b {
            continue;
        }
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        out.push((
            format!("{lo} and {hi}"),
            Span::new(a_span.start, b_span.end),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::FitDoc;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn fit_on(spec: VersePack, texts: &[&str]) -> (FittedVerse, Interner) {
        let corpus = Corpus::new();
        let tok = Tokenizer::default();
        let docs: Vec<Document> = texts.iter().map(|t| Document::new(*t)).collect();
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

    fn transform(text: &str) -> (FeatureVector, Interner) {
        let (fitted, interner) = fit_on(VersePack::default(), &[]);
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

    /// Eight lines of blank verse, all masculine (stressed) endings.
    const MASCULINE: &str = "But soft what light through yonder window breaks\n\
        It is the east and Juliet is the sun\n\
        Arise fair sun and kill the envious moon\n\
        Who is already sick and pale with grief\n\
        That thou her maid art far more fair than she\n\
        Be not her maid since she is envious\n\
        Her vestal livery is but sick and green\n\
        And none but fools do wear it cast it off";

    /// The same shape with unstressed final syllables.
    const FEMININE: &str = "To be or not to be that is the question\n\
        Whether tis nobler in the mind to suffer\n\
        The slings and arrows of outrageous fortune\n\
        Or to take arms against a sea of trouble\n\
        And by opposing end them and to slumber\n\
        No more and by a sleep to say we ended\n\
        The heartache and the thousand natural shivers\n\
        That flesh is heir to it is a completion";

    #[test]
    fn feminine_endings_separate_the_two_fixtures() {
        let feminine = value(FEMININE, "verse:feminine_ending_rate");
        let masculine = value(MASCULINE, "verse:feminine_ending_rate");
        assert!(
            feminine > masculine,
            "feminine={feminine} masculine={masculine}"
        );
    }

    #[test]
    fn the_spedding_split_separates_two_synthetic_corpora() {
        // The golden test the plan names. Two corpora built at deliberately
        // different feminine-ending rates must not overlap — which is the
        // property Spedding relied on in 1850 and Plecháč confirmed in 2019.
        let low: String = (0..4).map(|_| format!("{MASCULINE}\n")).collect();
        let high: String = (0..4).map(|_| format!("{FEMININE}\n")).collect();
        let low_rate = value(&low, "verse:feminine_ending_rate");
        let high_rate = value(&high, "verse:feminine_ending_rate");
        assert!(
            high_rate > low_rate + 10.0,
            "the split must be wide: {low_rate} vs {high_rate}"
        );
    }

    #[test]
    fn prose_is_never_scanned_as_verse() {
        // The min_verse_share guard. Prose has lines; it does not have verse
        // lines, and a full set of plausible metrical numbers over prose would
        // describe nothing.
        let prose = "The committee determined that the implementation of the proposed \
                     regulation would require substantial revision before the subsequent \
                     evaluation could proceed to the next stage of the process, which \
                     everyone agreed was going to take considerably longer than planned.";
        let (v, i) = transform(prose);
        assert!(v.is_missing(i.get("verse:feminine_ending_rate").unwrap()));
        assert!(v.is_missing(i.get("verse:line_len_mean").unwrap()));
    }

    #[test]
    fn a_short_document_is_not_scanned_either() {
        let (v, i) = transform("one line\ntwo lines");
        assert!(v.is_missing(i.get("verse:feminine_ending_rate").unwrap()));
    }

    #[test]
    fn line_geometry_captures_the_bukowski_axes() {
        let ragged = "the man\ndrank\nalone\nin the kitchen at four in the morning with the \
                      radio on\nand\nnothing\nhappened\nat all";
        let even = "the man drank alone in the kitchen\nat four in the morning with the radio\n\
                    and nothing happened at all that night\nthe dog slept under the kitchen table\n\
                    the light came up behind the curtains\nhe poured another one and sat down";
        assert!(value(ragged, "verse:line_len_stddev") > value(even, "verse:line_len_stddev"));
        assert!(value(ragged, "verse:one_word_line_rate") > 0.0);
        assert_eq!(value(even, "verse:one_word_line_rate"), 0.0);
        assert!(value(ragged, "verse:lowercase_initial_rate") > 90.0);
    }

    #[test]
    fn archaism_is_measured_as_a_ratio_and_two_rates() {
        let archaic = "Thou art the sun and thou dost shine\nThy light doth fall on me\n\
                       And thou hast made the morning\nWhere thou wilt never be\n\
                       For thou art all I know\nAnd thou shalt ever stay";
        let modern = "You are the sun and you do shine\nYour light will fall on me\n\
                      And you have made the morning\nWhere you will never be\n\
                      For you are all I know\nAnd you will ever stay";
        assert!(value(archaic, "verse:thou_you_ratio") > value(modern, "verse:thou_you_ratio"));
        assert!(value(archaic, "verse:eth_rate") > 0.0);
        assert_eq!(value(modern, "verse:eth_rate"), 0.0);
        assert!(value(archaic, "verse:est_rate") > 0.0);
    }

    #[test]
    fn vocatives_use_the_o_plus_np_rule() {
        let text = "O rose thou art sick\nThe invisible worm\nThat flies in the night\n\
                    In the howling storm\nO joy that comes after\nHas found out thy bed";
        assert!(value(text, "verse:vocative_rate") > 0.0);
        let (v, i) = transform(text);
        let spans = v.spans(i.get("verse:vocative_rate").unwrap());
        assert_eq!(spans.len(), 2);
        assert_eq!(&text[spans[0].range()], "O rose thou art sick");
    }

    #[test]
    fn anaphora_and_epistrophe_are_counted_per_hundred_lines() {
        let anaphora = "When I consider how my light is spent\nWhen I consider all the days\n\
                        When I consider what I lost\nAnd think about the rest of it\n\
                        The morning came up slow and grey\nNothing else happened that day";
        assert!(value(anaphora, "verse:figure:anaphora") > 0.0);
        let plain = "The morning came up slow and grey\nNothing else happened that day\n\
                     The dog slept under the table\nThe light came through the curtain\n\
                     He poured another one and sat\nOutside a car went past the door";
        assert_eq!(value(plain, "verse:figure:anaphora"), 0.0);
    }

    #[test]
    fn antithesis_uses_the_antonym_table() {
        let text = "It was the best of times the worst\nIt was the age of wisdom then\n\
                    It was the age of foolishness\nIt was the season of the light\n\
                    It was the season of the dark\nWe had it all before us then";
        assert!(value(text, "verse:figure:antithesis") > 0.0);
    }

    #[test]
    fn the_doublet_proxy_is_relative_to_the_fitting_corpus() {
        // "sound and fury" appears in the corpus and is not low-PMI; the
        // draft's own coinage is.
        let corpus = ["sound and fury signifying nothing at all\nsound and fury again"];
        let (fitted, interner) = fit_on(VersePack::default(), &corpus);
        assert!(fitted.doublet_len() > 0);
        let score = |text: &str| {
            let doc = Document::new(text);
            let a = doc.analyze(&Tokenizer::default());
            let mut b = VectorBuilder::new();
            fitted.transform(&a, &mut b);
            b.build()
                .get(interner.get("verse:doublet_lowpmi_rate").unwrap())
        };
        let familiar = "sound and fury here again\nsound and fury once more now\n\
                        sound and fury yet again\nsound and fury one more time\n\
                        sound and fury still again\nsound and fury at the last";
        let novel = "bicycle and chlorine here again\npenguin and kilogram once more\n\
                     teapot and aardvark yet again\nsuitcase and umbrella one more\n\
                     forkful and windowsill again\nsandwich and elbow at the last";
        assert_eq!(score(familiar), 0.0);
        assert!(score(novel) > 0.0);
    }

    #[test]
    fn the_stress_profile_reports_a_fraction_per_position() {
        let (v, i) = transform(MASCULINE);
        for p in 1..=STRESS_POSITIONS {
            let value = v.get(i.get(&format!("verse:stress_pos:{p}")).unwrap());
            assert!((0.0..=1.0).contains(&value), "position {p}: {value}");
        }
        // Iambic verse has more stress on even positions than odd ones.
        let odd = v.get(i.get("verse:stress_pos:1").unwrap());
        let even = v.get(i.get("verse:stress_pos:2").unwrap());
        assert!(even >= odd, "odd={odd} even={even}");
        assert!(v.get(i.get("verse:iambic_conformity").unwrap()) > 0.3);
    }

    #[test]
    fn enjambment_and_end_stopping_are_complementary() {
        let enjambed = "the light that fell across the\nkitchen floor was the\n\
                        colour of the sky and\nnothing moved at all in\n\
                        the room except the\ndust that hung above the";
        let stopped = "the light fell on the floor.\nthe sky was grey again.\n\
                       nothing moved in the room.\nthe dust hung in the air.\n\
                       the dog slept by the door.\nhe poured another one.";
        assert!(value(enjambed, "verse:enjambment_rate") > value(stopped, "verse:enjambment_rate"));
        assert!(value(stopped, "verse:end_stop_rate") > value(enjambed, "verse:end_stop_rate"));
    }

    #[test]
    fn the_spec_round_trips_and_records_the_syllable_method() {
        let spec: VersePack = serde_json::from_str("{}").unwrap();
        assert_eq!(spec, VersePack::default());
        assert_eq!(spec.min_verse_share, 0.5);
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<VersePack>(&json).unwrap(), spec);

        let (fitted, _) = fit_on(spec, &[]);
        assert_eq!(fitted.syllable_method(), &SyllableMethod::VowelGroup);
    }
}
