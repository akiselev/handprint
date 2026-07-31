//! Punctuation and typography — the small-sample workhorse.
//!
//! Grieve (2007) found frequent words plus punctuation the strongest single
//! feature combination for authorship attribution, and punctuation keeps
//! working at lengths where frequent-word profiles are still noise. That makes
//! this family the backbone of the critic loop: at 400 words it is the only
//! family with [`Confidence::Ok`](super::Confidence::Ok).
//!
//! Everything here reads the **source** text, not the confusable-folded scoring
//! view, because the distinction between `—` and ` - `, between `"` and `“`, is
//! exactly the signal.

use serde::{Deserialize, Serialize};

use super::{per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::Result;
use crate::text::{Analysis, TokenKind};
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Punctuation marks counted as rates per 1,000 lexical tokens.
///
/// Fullwidth and CJK variants share a dimension with their ASCII counterparts:
/// they are the same authorial choice made on a different keyboard.
const MARKS: &[(&str, &[char])] = &[
    ("comma", &[',', '\u{FF0C}', '\u{3001}']),
    ("semicolon", &[';', '\u{FF1B}']),
    ("colon", &[':', '\u{FF1A}']),
    ("period", &['.', '\u{3002}']),
    ("exclaim", &['!', '\u{FF01}']),
    ("question", &['?', '\u{FF1F}']),
    ("em_dash", &['\u{2014}']),
    ("en_dash", &['\u{2013}']),
    ("hyphen", &['-']),
    ("ellipsis_char", &['\u{2026}']),
    ("quote_double_straight", &['"']),
    ("quote_double_curly", &['\u{201C}', '\u{201D}']),
    ("quote_single_straight", &['\'']),
    ("quote_single_curly", &['\u{2018}', '\u{2019}']),
    ("paren", &['(', ')']),
    ("bracket", &['[', ']']),
    ("brace", &['{', '}']),
    ("slash", &['/']),
    ("backslash", &['\\']),
    ("ampersand", &['&']),
    ("asterisk", &['*']),
    ("percent", &['%']),
    ("hash", &['#']),
    ("at", &['@']),
    ("plus", &['+']),
    ("equals", &['=']),
    ("tilde", &['~']),
    ("pipe", &['|']),
    ("caret", &['^']),
    ("underscore", &['_']),
    ("nbsp", &['\u{00A0}']),
    ("bullet", &['\u{2022}', '\u{00B7}']),
];

/// Ratio-valued typography choices, in `[0, 1]`.
const STYLES: &[&str] = &[
    "em_dash_spaced",
    "dash_em_share",
    "quote_curly_share",
    "apostrophe_curly_share",
    "ellipsis_char_share",
    "serial_comma_share",
    "double_space_after_period",
    "space_before_punct",
    "sentence_initial_lower",
    "number_grouped_share",
];

/// Human-informality markers, as rates per 1,000 lexical tokens.
///
/// These are deliberately *not* a spell checker. Shipping a dictionary would
/// cost megabytes and mislabel jargon, names and code; each rule below fires
/// only on a pattern that needs no dictionary. The signal they carry is the
/// documented one — LLM prose is abnormally clean — so their absence is as
/// informative as their presence.
const INFORMAL: &[&str] = &[
    "repeated_char",
    "missing_apostrophe",
    "common_misspelling",
    "elongated_punct",
    "lowercase_i",
    "total",
];

/// Contractions written without their apostrophe.
const MISSING_APOSTROPHE: &[&str] = &[
    "dont", "cant", "wont", "isnt", "arent", "wasnt", "werent", "didnt", "doesnt", "hasnt",
    "havent", "hadnt", "couldnt", "shouldnt", "wouldnt", "im", "ive", "youre", "theyre", "thats",
    "whats", "lets", "aint", "yall", "shouldve", "couldve", "wouldve",
];

/// Misspellings frequent enough to be worth hard-coding, none of which is a
/// word in its own right.
const MISSPELLINGS: &[&str] = &[
    "teh", "recieve", "recieved", "seperate", "seperated", "definately", "occured", "occurence",
    "alot", "wich", "adn", "thier", "becuase", "untill", "wierd", "arguement", "accomodate",
    "embarass", "existance", "goverment", "independant", "neccessary", "publically", "recomend",
    "refered", "tommorow", "truely", "writting", "acheive", "beleive", "calender", "cemetary",
    "changable", "collegue", "concious", "enviroment", "foriegn", "gaurd", "harrass",
    "immediatly", "knowlege", "liason", "maintainance", "occassion", "perseverence", "priviledge",
    "questionaire", "rythm", "succesful", "supercede", "threshhold", "vaccuum", "visable",
];

/// Clitic endings that mark a token as a contraction.
const CLITICS: &[&str] = &["'t", "'s", "'re", "'ve", "'ll", "'d", "'m"];

/// Configuration for the punctuation and typography family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PunctTypography {
    /// Emit per-letter unigram relative frequencies (Writeprints-static).
    #[serde(default = "crate::util::yes")]
    pub letters: bool,
    /// Emit per-digit unigram relative frequencies.
    #[serde(default = "crate::util::yes")]
    pub digits: bool,
    /// Emit the word-length distribution.
    #[serde(default = "crate::util::yes")]
    pub word_lengths: bool,
    /// Longest word length with its own bucket; longer words share the tail
    /// bucket.
    #[serde(default = "default_max_word_len")]
    pub max_word_len: usize,
}

fn default_max_word_len() -> usize {
    12
}

impl Default for PunctTypography {
    fn default() -> Self {
        PunctTypography {
            letters: true,
            digits: true,
            word_lengths: true,
            max_word_len: default_max_word_len(),
        }
    }
}

/// Fitted [`PunctTypography`]. The fit is trivial — the dimension set is fixed —
/// so this holds nothing but interned symbols.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedPunct {
    dims: Vec<DimInfo>,
    marks: Vec<Symbol>,
    styles: Vec<Symbol>,
    informal: Vec<Symbol>,
    letters: Vec<Symbol>,
    digits: Vec<Symbol>,
    word_len: Vec<Symbol>,
    word_len_mean: Option<Symbol>,
    word_len_stddev: Option<Symbol>,
    allcaps: Symbol,
    contraction: Symbol,
    emoji: Symbol,
    number: Symbol,
    max_word_len: usize,
}

impl FittedPunct {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Punct;
}

impl Feature for PunctTypography {
    type Fitted = FittedPunct;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedPunct> {
        if self.max_word_len == 0 {
            return Err(crate::Error::InvalidConfig {
                what: "PunctTypography::max_word_len",
                detail: "must be at least 1".into(),
            });
        }
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: String, unit: Unit| {
            let sym = interner.intern(&name);
            dims.push(DimInfo::new(sym, Family::Punct, unit));
            sym
        };

        let mut marks = Vec::with_capacity(MARKS.len());
        for (name, _) in MARKS {
            marks.push(push(
                interner,
                format!("punct:{name}_rate"),
                Unit::PerThousandTokens,
            ));
        }
        let mut styles = Vec::with_capacity(STYLES.len());
        for name in STYLES {
            styles.push(push(interner, format!("punct:style:{name}"), Unit::Fraction));
        }
        let mut informal = Vec::with_capacity(INFORMAL.len());
        let mut informal_total = None;
        for name in INFORMAL {
            let sym = push(
                interner,
                format!("punct:informal:{name}"),
                Unit::PerThousandTokens,
            );
            if *name == "total" {
                informal_total = Some(sym);
            }
            informal.push(sym);
        }
        let allcaps = push(
            interner,
            "punct:allcaps_rate".into(),
            Unit::PerThousandTokens,
        );
        let contraction = push(
            interner,
            "punct:contraction_rate".into(),
            Unit::PerThousandTokens,
        );
        let emoji = push(interner, "punct:emoji_rate".into(), Unit::PerThousandTokens);
        let number = push(
            interner,
            "punct:number_rate".into(),
            Unit::PerThousandTokens,
        );
        let mut letters = Vec::new();
        if self.letters {
            for c in b'a'..=b'z' {
                letters.push(push(
                    interner,
                    format!("punct:letter:{}", c as char),
                    Unit::RelativeFrequency,
                ));
            }
        }
        let mut digits = Vec::new();
        if self.digits {
            for c in b'0'..=b'9' {
                digits.push(push(
                    interner,
                    format!("punct:digit:{}", c as char),
                    Unit::RelativeFrequency,
                ));
            }
        }
        let (word_len, word_len_mean, word_len_stddev) = if self.word_lengths {
            let mut buckets = Vec::with_capacity(self.max_word_len + 1);
            for n in 1..=self.max_word_len {
                buckets.push(push(interner, format!("punct:word_len:{n}"), Unit::Fraction));
            }
            buckets.push(push(
                interner,
                format!("punct:word_len:{}+", self.max_word_len + 1),
                Unit::Fraction,
            ));
            (
                buckets,
                Some(push(interner, "punct:word_len_mean".into(), Unit::Index)),
                Some(push(interner, "punct:word_len_stddev".into(), Unit::Index)),
            )
        } else {
            (Vec::new(), None, None)
        };

        // `punct:informal:total` is a rollup of the five markers above it: it
        // belongs in a distance and not in a finding.
        if let Some(total) = informal_total {
            if let Some(dim) = dims.iter_mut().find(|d| d.symbol == total) {
                dim.aggregate = true;
            }
        }

        Ok(FittedPunct {
            dims,
            marks,
            styles,
            informal,
            letters,
            digits,
            word_len,
            word_len_mean,
            word_len_stddev,
            allcaps,
            contraction,
            emoji,
            number,
            max_word_len: self.max_word_len,
        })
    }
}

impl FittedFeature for FittedPunct {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Punct
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        let source = analysis.source();

        self.mark_rates(source, tokens, out);
        self.style_ratios(analysis, out);
        self.informality(analysis, tokens, out);
        self.case_and_kind(analysis, tokens, out);
        self.unigrams(source, out);
        self.word_length_distribution(analysis, out);
    }
}

impl FittedPunct {
    fn mark_rates(&self, source: &str, tokens: usize, out: &mut VectorBuilder) {
        let mut counts = vec![0usize; MARKS.len()];
        for c in source.chars() {
            for (i, (_, set)) in MARKS.iter().enumerate() {
                if set.contains(&c) {
                    counts[i] += 1;
                }
            }
        }
        for (i, &sym) in self.marks.iter().enumerate() {
            out.set(sym, per_1k(counts[i], tokens));
        }
    }

    fn style_ratios(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let source = analysis.source();
        let chars: Vec<char> = source.chars().collect();

        let mut em_total = 0usize;
        let mut em_spaced = 0usize;
        let mut spaced_hyphen = 0usize;
        let mut en_total = 0usize;
        let mut double_space_after_period = 0usize;
        let mut single_space_after_period = 0usize;
        let mut space_before_punct = 0usize;
        let mut punct_total = 0usize;

        for i in 0..chars.len() {
            let c = chars[i];
            let prev = i.checked_sub(1).map(|j| chars[j]);
            let next = chars.get(i + 1).copied();
            match c {
                '\u{2014}' => {
                    em_total += 1;
                    if prev.is_some_and(char::is_whitespace) && next.is_some_and(char::is_whitespace)
                    {
                        em_spaced += 1;
                    }
                }
                '\u{2013}' => en_total += 1,
                '-' => {
                    if prev.is_some_and(char::is_whitespace) && next.is_some_and(char::is_whitespace)
                    {
                        spaced_hyphen += 1;
                    }
                }
                '.' => {
                    let after: Vec<char> = chars[i + 1..].iter().take(2).copied().collect();
                    if after.first() == Some(&' ') {
                        if after.get(1) == Some(&' ') {
                            double_space_after_period += 1;
                        } else {
                            single_space_after_period += 1;
                        }
                    }
                }
                _ => {}
            }
            if crate::text::tokenize::is_punctuation(c) && c != '(' && c != '[' && c != '{' {
                punct_total += 1;
                if prev == Some(' ') {
                    space_before_punct += 1;
                }
            }
        }

        let curly_double = count_any(source, &['\u{201C}', '\u{201D}']);
        let straight_double = source.matches('"').count();
        let curly_apos = count_any(source, &['\u{2019}']);
        let straight_apos = source.matches('\'').count();
        let ellipsis_char = source.matches('\u{2026}').count();
        let ellipsis_dots = source.matches("...").count();
        let serial = source.matches(", and ").count() + source.matches(", or ").count();
        let conj = source.matches(" and ").count() + source.matches(" or ").count();

        let mut numbers = 0usize;
        let mut grouped = 0usize;
        for (token, form) in analysis.tokens().iter() {
            if token.kind == TokenKind::Number {
                numbers += 1;
                if form.contains(',') || form.contains('\u{202F}') || form.contains('_') {
                    grouped += 1;
                }
            }
        }

        let sentences = analysis.structure().sentences.len();
        let lower_start = analysis
            .structure()
            .sentences
            .iter()
            .filter(|s| {
                analysis
                    .text(s.span)
                    .chars()
                    .find(|c| c.is_alphabetic())
                    .is_some_and(char::is_lowercase)
            })
            .count();

        let values = [
            ratio(em_spaced, em_total),
            ratio(em_total, em_total + en_total + spaced_hyphen),
            ratio(curly_double, curly_double + straight_double),
            ratio(curly_apos, curly_apos + straight_apos),
            ratio(ellipsis_char, ellipsis_char + ellipsis_dots),
            ratio(serial, conj),
            ratio(
                double_space_after_period,
                double_space_after_period + single_space_after_period,
            ),
            ratio(space_before_punct, punct_total),
            ratio(lower_start, sentences),
            ratio(grouped, numbers),
        ];
        debug_assert_eq!(values.len(), STYLES.len());
        for (&sym, &value) in self.styles.iter().zip(values.iter()) {
            out.set(sym, value);
        }
    }

    fn informality(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        let mut counts = [0usize; 5];
        for (token, form) in analysis.tokens().iter() {
            if token.kind != TokenKind::Word {
                continue;
            }
            if has_triple_repeat(form) {
                counts[0] += 1;
                out.note_span(self.informal[0], token.span);
            }
            if MISSING_APOSTROPHE.contains(&form) {
                counts[1] += 1;
                out.note_span(self.informal[1], token.span);
            }
            if MISSPELLINGS.contains(&form) {
                counts[2] += 1;
                out.note_span(self.informal[2], token.span);
            }
            if form == "i" && analysis.text(token.span) == "i" {
                counts[4] += 1;
                out.note_span(self.informal[4], token.span);
            }
        }
        counts[3] = count_elongated_punct(analysis.source());

        let mut total = 0usize;
        for (i, &sym) in self.informal.iter().take(5).enumerate() {
            out.set(sym, per_1k(counts[i], tokens));
            total += counts[i];
        }
        out.set(self.informal[5], per_1k(total, tokens));
    }

    fn case_and_kind(&self, analysis: &Analysis<'_>, tokens: usize, out: &mut VectorBuilder) {
        let mut allcaps = 0usize;
        let mut contractions = 0usize;
        let mut emoji = 0usize;
        let mut numbers = 0usize;
        for (token, form) in analysis.tokens().iter() {
            match token.kind {
                TokenKind::Emoji => emoji += 1,
                TokenKind::Number => numbers += 1,
                TokenKind::Word => {
                    let raw = analysis.text(token.span);
                    let letters = raw.chars().filter(|c| c.is_alphabetic()).count();
                    if letters >= 2 && raw.chars().filter(|c| c.is_alphabetic()).all(char::is_uppercase)
                    {
                        allcaps += 1;
                        out.note_span(self.allcaps, token.span);
                    }
                    if CLITICS.iter().any(|c| form.ends_with(c)) && form.contains('\'') {
                        contractions += 1;
                    }
                }
                _ => {}
            }
        }
        out.set(self.allcaps, per_1k(allcaps, tokens));
        out.set(self.contraction, per_1k(contractions, tokens));
        out.set(self.emoji, per_1k(emoji, tokens));
        out.set(self.number, per_1k(numbers, tokens));
    }

    fn unigrams(&self, source: &str, out: &mut VectorBuilder) {
        if !self.letters.is_empty() {
            let mut counts = [0usize; 26];
            let mut total = 0usize;
            for c in source.chars().flat_map(char::to_lowercase) {
                if c.is_ascii_lowercase() {
                    counts[(c as u8 - b'a') as usize] += 1;
                    total += 1;
                }
            }
            for (i, &sym) in self.letters.iter().enumerate() {
                out.set(sym, ratio(counts[i], total));
            }
        }
        if !self.digits.is_empty() {
            let mut counts = [0usize; 10];
            let mut total = 0usize;
            for c in source.chars() {
                if c.is_ascii_digit() {
                    counts[(c as u8 - b'0') as usize] += 1;
                    total += 1;
                }
            }
            for (i, &sym) in self.digits.iter().enumerate() {
                out.set(sym, ratio(counts[i], total));
            }
        }
    }

    fn word_length_distribution(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        if self.word_len.is_empty() {
            return;
        }
        let mut buckets = vec![0usize; self.word_len.len()];
        let mut lengths: Vec<f64> = Vec::new();
        for (token, _) in analysis.tokens().iter() {
            if token.kind != TokenKind::Word {
                continue;
            }
            let len = analysis
                .text(token.span)
                .chars()
                .filter(|c| c.is_alphanumeric())
                .count();
            if len == 0 {
                continue;
            }
            lengths.push(len as f64);
            let idx = (len - 1).min(self.max_word_len);
            buckets[idx] += 1;
        }
        let total: usize = buckets.iter().sum();
        for (i, &sym) in self.word_len.iter().enumerate() {
            out.set(sym, ratio(buckets[i], total));
        }
        if let Some(sym) = self.word_len_mean {
            out.set(sym, util::mean(&lengths));
        }
        if let Some(sym) = self.word_len_stddev {
            out.set(sym, util::stddev(&lengths));
        }
    }
}

fn count_any(source: &str, set: &[char]) -> usize {
    source.chars().filter(|c| set.contains(c)).count()
}

/// Three or more of the same letter in a row — `soooo`, `ahhh`. No English word
/// does this.
fn has_triple_repeat(form: &str) -> bool {
    let mut run = 1;
    let mut prev = '\0';
    for c in form.chars() {
        if c == prev && c.is_alphabetic() {
            run += 1;
            if run >= 3 {
                return true;
            }
        } else {
            run = 1;
            prev = c;
        }
    }
    false
}

/// `!!`, `?!?`, `....` — runs of terminal punctuation longer than one mark.
///
/// `!` and `?` share a run because `?!?` is the same gesture as `!!!`; periods
/// need four in a row, since `...` is an ordinary ellipsis.
fn count_elongated_punct(source: &str) -> usize {
    let mut count = 0;
    let mut bang_run = 0usize;
    let mut dot_run = 0usize;
    for c in source.chars() {
        match c {
            '!' | '?' => {
                dot_run = 0;
                bang_run += 1;
                if bang_run == 2 {
                    count += 1;
                }
            }
            '.' => {
                bang_run = 0;
                dot_run += 1;
                if dot_run == 4 {
                    count += 1;
                }
            }
            _ => {
                bang_run = 0;
                dot_run = 0;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::text::{Document, Tokenizer};

    fn fitted() -> (FittedPunct, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = PunctTypography::default().fit(&ctx, &mut interner).unwrap();
        (fitted, interner)
    }

    fn transform(text: &str) -> (crate::FeatureVector, Interner) {
        let (fitted, interner) = fitted();
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
    fn em_dash_rate_is_per_thousand_lexical_tokens() {
        // Ten words, two em dashes → 200 per 1k.
        let text = "one two—three four five six seven eight nine—ten";
        assert!((value(text, "punct:em_dash_rate") - 200.0).abs() < 1e-9);
    }

    #[test]
    fn dash_style_ratios() {
        assert_eq!(value("a — b and c — d", "punct:style:em_dash_spaced"), 1.0);
        assert_eq!(value("a—b", "punct:style:em_dash_spaced"), 0.0);
        assert_eq!(value("a - b", "punct:style:dash_em_share"), 0.0);
        assert_eq!(value("a — b", "punct:style:dash_em_share"), 1.0);
    }

    #[test]
    fn quote_and_ellipsis_style() {
        assert_eq!(
            value("\u{201C}hi\u{201D} said x", "punct:style:quote_curly_share"),
            1.0
        );
        assert_eq!(value("\"hi\" said x", "punct:style:quote_curly_share"), 0.0);
        assert_eq!(value("wait\u{2026} ok", "punct:style:ellipsis_char_share"), 1.0);
        assert_eq!(value("wait... ok", "punct:style:ellipsis_char_share"), 0.0);
    }

    #[test]
    fn informality_markers_fire() {
        assert!(value("that was sooo good", "punct:informal:repeated_char") > 0.0);
        assert!(value("i dont think so", "punct:informal:missing_apostrophe") > 0.0);
        assert!(value("i recieve alot", "punct:informal:common_misspelling") > 0.0);
        assert!(value("really?!? no way!!", "punct:informal:elongated_punct") > 0.0);
        assert_eq!(value("wait... ok. fine.", "punct:informal:elongated_punct"), 0.0);
        assert_eq!(value("clean prose here", "punct:informal:total"), 0.0);
    }

    #[test]
    fn word_length_distribution_sums_to_one() {
        let (v, i) = transform("a bb ccc dddd eeeee");
        let sum: f64 = (1..=13)
            .map(|n| {
                let name = if n <= 12 {
                    format!("punct:word_len:{n}")
                } else {
                    "punct:word_len:13+".to_string()
                };
                v.get(i.get(&name).unwrap())
            })
            .sum();
        assert!((sum - 1.0).abs() < 1e-9, "{sum}");
        assert!((v.get(i.get("punct:word_len_mean").unwrap()) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn letter_frequencies_normalize() {
        let (v, i) = transform("aab");
        assert!((v.get(i.get("punct:letter:a").unwrap()) - 2.0 / 3.0).abs() < 1e-9);
        assert!((v.get(i.get("punct:letter:b").unwrap()) - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(v.get(i.get("punct:letter:z").unwrap()), 0.0);
    }

    #[test]
    fn allcaps_and_contractions() {
        assert!(value("this is NASA speaking", "punct:allcaps_rate") > 0.0);
        assert!(value("i don't think so", "punct:contraction_rate") > 0.0);
        assert_eq!(value("i do not think so", "punct:contraction_rate"), 0.0);
    }

    #[test]
    fn empty_document_marks_everything_missing() {
        let (fitted, _) = fitted();
        let doc = Document::new("!!!");
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&analysis, &mut b);
        let v = b.build();
        assert!(v.is_empty());
        assert_eq!(v.missing().len(), fitted.dims().len());
    }

    #[test]
    fn spans_are_recorded_for_actionable_markers() {
        let text = "i dont think so";
        let (v, i) = transform(text);
        let sym = i.get("punct:informal:missing_apostrophe").unwrap();
        let spans = v.spans(sym);
        assert_eq!(spans.len(), 1);
        assert_eq!(&text[spans[0].range()], "dont");
    }
}
