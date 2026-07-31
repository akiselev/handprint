//! Tokenization policy.
//!
//! Tokenization is *explicit policy*, not an implementation detail: the
//! [`Tokenizer`] is serialized with the fitted model so that a profile computed
//! two years later is computed the same way. It is a plain enum of policies —
//! no closures, no trait objects (design invariant #2).

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::normalize::{self, ScoringText};
use crate::text::Span;

/// How a token's surface form is folded before it becomes a feature dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaseFold {
    /// Lowercase every token. The default: `The` and `the` are the same word,
    /// and casing is captured separately by the typography features.
    #[default]
    Lower,
    /// Preserve case. Useful when casing itself is the object of study and the
    /// corpus is large enough to afford the vocabulary split.
    Preserve,
}

/// How apostrophes inside words are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApostrophePolicy {
    /// Keep contractions whole, folding curly apostrophes to `'`, so `don't`
    /// and `don\u{2019}t` are one dimension. The typography features still see
    /// which glyph was used.
    #[default]
    Unify,
    /// Keep contractions whole and keep the apostrophe glyph distinct.
    Preserve,
    /// Split clitics off: `don't` becomes `don` + `'t`.
    SplitClitics,
}

/// The tokenization policy, serialized with the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Tokenizer {
    /// UAX-29 word boundaries over the scoring text.
    Words {
        /// Case folding policy.
        #[serde(default)]
        case: CaseFold,
        /// Apply NFC to each token's surface form.
        #[serde(default = "crate::util::yes")]
        nfc: bool,
        /// Emit punctuation as its own tokens. On by default: punctuation is
        /// the small-sample workhorse (Grieve 2007).
        #[serde(default = "crate::util::yes")]
        punct_as_tokens: bool,
        /// Apostrophe handling.
        #[serde(default)]
        apostrophes: ApostrophePolicy,
    },
    /// A sliding window of `n` characters over the scoring text, including
    /// spaces and punctuation. Used by the char-n-gram feature.
    Chars {
        /// Window width in characters.
        n: usize,
        /// Case folding policy.
        #[serde(default)]
        case: CaseFold,
    },
}

impl Default for Tokenizer {
    fn default() -> Self {
        Tokenizer::Words {
            case: CaseFold::Lower,
            nfc: true,
            punct_as_tokens: true,
            apostrophes: ApostrophePolicy::Unify,
        }
    }
}

/// What kind of thing a token is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    /// A word: at least one alphabetic character.
    Word,
    /// A number: digits, possibly with separators.
    Number,
    /// A punctuation mark.
    Punct,
    /// A symbol that is not punctuation (currency, math, arrows).
    Symbol,
    /// An emoji or pictograph.
    Emoji,
    /// Anything else that is not whitespace.
    Other,
}

impl TokenKind {
    /// True for kinds that participate in word-level statistics (MFW,
    /// richness, sentence length).
    #[inline]
    pub fn is_lexical(self) -> bool {
        matches!(self, TokenKind::Word | TokenKind::Number)
    }
}

/// One token: where it came from and what it folds to.
///
/// `span` indexes the **source** text, so `&document.text()[token.span.range()]`
/// always yields the original characters — including any homoglyphs that the
/// scoring view folded away.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Byte range in the source text.
    pub span: Span,
    /// What kind of token this is.
    pub kind: TokenKind,
    /// Range into the analysis' normalized-form buffer.
    pub(crate) norm: Span,
}

/// Tokens plus the buffer their normalized forms live in.
#[derive(Debug, Clone, Default)]
pub struct TokenStream {
    tokens: Vec<Token>,
    norm_buf: String,
}

impl TokenStream {
    /// The tokens, in document order.
    #[inline]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Number of tokens of every kind.
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// True when the document produced no tokens.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// The normalized surface form of a token.
    #[inline]
    pub fn form(&self, token: &Token) -> &str {
        &self.norm_buf[token.norm.range()]
    }

    /// Iterate over `(token, normalized form)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Token, &str)> + '_ {
        self.tokens.iter().map(move |t| (t, self.form(t)))
    }

    /// Iterate over word and number tokens only.
    pub fn lexical(&self) -> impl Iterator<Item = (&Token, &str)> + '_ {
        self.iter().filter(|(t, _)| t.kind.is_lexical())
    }

    /// Count of word and number tokens.
    pub fn lexical_len(&self) -> usize {
        self.tokens.iter().filter(|t| t.kind.is_lexical()).count()
    }
}

impl Tokenizer {
    /// Tokenize the scoring view of a document.
    ///
    /// Spans in the returned stream are mapped back to source offsets.
    pub fn tokenize(&self, scoring: &ScoringText) -> TokenStream {
        match self {
            Tokenizer::Words {
                case,
                nfc,
                punct_as_tokens,
                apostrophes,
            } => tokenize_words(scoring, *case, *nfc, *punct_as_tokens, *apostrophes),
            Tokenizer::Chars { n, case } => tokenize_chars(scoring, *n, *case),
        }
    }
}

fn tokenize_words(
    scoring: &ScoringText,
    case: CaseFold,
    nfc: bool,
    punct_as_tokens: bool,
    apostrophes: ApostrophePolicy,
) -> TokenStream {
    let text = scoring.as_str();
    let lower = matches!(case, CaseFold::Lower);
    let unify_apos = matches!(apostrophes, ApostrophePolicy::Unify);

    let mut out = TokenStream {
        tokens: Vec::with_capacity(text.len() / 4),
        norm_buf: String::with_capacity(text.len()),
    };

    for (offset, piece) in text.split_word_bound_indices() {
        if piece.chars().all(char::is_whitespace) {
            continue;
        }
        let kind = classify(piece);
        if kind == TokenKind::Punct && !punct_as_tokens {
            continue;
        }

        if kind == TokenKind::Word && matches!(apostrophes, ApostrophePolicy::SplitClitics) {
            for (sub_off, sub) in split_clitics(piece) {
                push_token(
                    &mut out,
                    scoring,
                    offset + sub_off,
                    sub,
                    classify(sub),
                    nfc,
                    lower,
                    unify_apos,
                );
            }
            continue;
        }

        push_token(
            &mut out, scoring, offset, piece, kind, nfc, lower, unify_apos,
        );
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn push_token(
    out: &mut TokenStream,
    scoring: &ScoringText,
    offset: usize,
    piece: &str,
    kind: TokenKind,
    nfc: bool,
    lower: bool,
    unify_apos: bool,
) {
    let normalized = normalize::normalize_token(piece, nfc, lower, unify_apos);
    let start = out.norm_buf.len();
    out.norm_buf.push_str(&normalized);
    let norm = Span::new(start, out.norm_buf.len());
    let span = scoring.to_source_span(Span::new(offset, offset + piece.len()));
    out.tokens.push(Token { span, kind, norm });
}

/// UAX-29 keeps `don't` together but not `don` + `'` + `t` in every locale;
/// splitting clitics is therefore done explicitly on the apostrophe.
fn split_clitics(piece: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut parts = Vec::new();
    let mut last = 0;
    for (i, c) in piece.char_indices() {
        if matches!(c, '\'' | '\u{2019}' | '\u{02BC}') && i > 0 {
            parts.push((last, &piece[last..i]));
            last = i;
        }
    }
    parts.push((last, &piece[last..]));
    parts.into_iter().filter(|(_, s)| !s.is_empty())
}

fn classify(piece: &str) -> TokenKind {
    // The overwhelmingly common cases, decided without touching the Unicode
    // tables. Word boundaries never mix letters with punctuation, so one look
    // at the first byte settles an ASCII piece.
    if piece.is_ascii() {
        let bytes = piece.as_bytes();
        let first = bytes[0];
        if first.is_ascii_alphabetic() {
            return TokenKind::Word;
        }
        if first.is_ascii_digit() && bytes.iter().all(|b| !b.is_ascii_alphabetic()) {
            return TokenKind::Number;
        }
    }

    let mut has_alpha = false;
    let mut has_digit = false;
    let mut has_emoji = false;
    let mut has_punct = false;
    let mut has_symbol = false;
    for c in piece.chars() {
        if c.is_alphabetic() {
            has_alpha = true;
        } else if c.is_numeric() {
            has_digit = true;
        } else if is_emoji(c) {
            has_emoji = true;
        } else if is_punctuation(c) {
            has_punct = true;
        } else if !c.is_whitespace() && !normalize::is_zero_width_joiner(c) {
            has_symbol = true;
        }
    }
    match (has_alpha, has_digit, has_emoji, has_punct, has_symbol) {
        (true, _, _, _, _) => TokenKind::Word,
        (false, true, _, _, _) => TokenKind::Number,
        (false, false, true, _, _) => TokenKind::Emoji,
        (false, false, false, true, _) => TokenKind::Punct,
        (false, false, false, false, true) => TokenKind::Symbol,
        _ => TokenKind::Other,
    }
}

/// Unicode general categories P* plus the ASCII marks that `char::is_ascii_punctuation`
/// covers but Unicode files under S*.
pub fn is_punctuation(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_punctuation();
    }
    matches!(c,
        '\u{00A1}' | '\u{00A7}' | '\u{00AB}' | '\u{00B6}' | '\u{00B7}' | '\u{00BB}' | '\u{00BF}'
        | '\u{2010}'..='\u{2027}'   // hyphens, dashes, quotes, ellipsis, bullets
        | '\u{2030}'..='\u{205E}'   // primes, guillemets, brackets, dots
        | '\u{2E00}'..='\u{2E7F}'   // supplemental punctuation
        | '\u{3001}'..='\u{3003}' | '\u{3008}'..='\u{3011}' | '\u{3014}'..='\u{301F}'
        | '\u{FE10}'..='\u{FE19}' | '\u{FE30}'..='\u{FE6B}'
        | '\u{FF01}'..='\u{FF03}' | '\u{FF05}'..='\u{FF0A}' | '\u{FF0C}'..='\u{FF0F}'
        | '\u{FF1A}'..='\u{FF1B}' | '\u{FF1F}'..='\u{FF20}'
    )
}

/// Rough emoji / pictograph test. Exactness is not required: the emoji rate is
/// a feature, and the ranges below cover everything a keyboard emits.
pub fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x1F000..=0x1FAFF
        | 0x2600..=0x27BF
        | 0x2190..=0x21FF
        | 0x2B00..=0x2BFF
        | 0xFE0F | 0xFE0E
    )
}

fn tokenize_chars(scoring: &ScoringText, n: usize, case: CaseFold) -> TokenStream {
    let text = scoring.as_str();
    let lower = matches!(case, CaseFold::Lower);
    let folded: String = if lower {
        text.to_lowercase()
    } else {
        text.to_owned()
    };
    // Lowercasing can change byte lengths (e.g. `İ`), so index the folded text
    // through its own char offsets and map window starts back through the
    // original character positions.
    let src_starts: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let fold_starts: Vec<usize> = folded.char_indices().map(|(i, _)| i).collect();
    let usable = src_starts.len().min(fold_starts.len());

    let mut out = TokenStream {
        tokens: Vec::new(),
        norm_buf: String::with_capacity(folded.len()),
    };
    if usable < n || n == 0 {
        return out;
    }
    for i in 0..=(usable - n) {
        let fs = fold_starts[i];
        let fe = if i + n < fold_starts.len() {
            fold_starts[i + n]
        } else {
            folded.len()
        };
        let start = out.norm_buf.len();
        out.norm_buf.push_str(&folded[fs..fe]);
        let norm = Span::new(start, out.norm_buf.len());

        let ss = src_starts[i];
        let se = if i + n < src_starts.len() {
            src_starts[i + n]
        } else {
            text.len()
        };
        out.tokens.push(Token {
            span: scoring.to_source_span(Span::new(ss, se)),
            kind: TokenKind::Other,
            norm,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> (ScoringText, TokenStream) {
        let scoring = ScoringText::new(text);
        let stream = Tokenizer::default().tokenize(&scoring);
        (scoring, stream)
    }

    #[test]
    fn spans_round_trip_to_source() {
        let text = "Don\u{2019}t — really — “delve” into 42 things 🙂.";
        let (_, stream) = words(text);
        for (token, _) in stream.iter() {
            // Every span must slice the source at a character boundary.
            assert!(text.is_char_boundary(token.span.start));
            assert!(text.is_char_boundary(token.span.end));
            assert!(!text[token.span.range()].is_empty());
        }
        let forms: Vec<&str> = stream.iter().map(|(_, f)| f).collect();
        assert!(forms.contains(&"don't"), "{forms:?}");
        assert!(forms.contains(&"delve"), "{forms:?}");
        assert!(forms.contains(&"42"), "{forms:?}");
    }

    #[test]
    fn spans_round_trip_through_confusable_folding() {
        // Cyrillic 'о' inside "delve".
        let text = "please delv\u{0435} deeper";
        let (_, stream) = words(text);
        let (token, form) = stream.iter().nth(1).unwrap();
        assert_eq!(form, "delve");
        assert_eq!(&text[token.span.range()], "delv\u{0435}");
    }

    #[test]
    fn punctuation_is_a_token_by_default() {
        let (_, stream) = words("a, b; c");
        let kinds: Vec<TokenKind> = stream.tokens().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Word,
                TokenKind::Punct,
                TokenKind::Word,
                TokenKind::Punct,
                TokenKind::Word
            ]
        );
    }

    #[test]
    fn punctuation_can_be_dropped() {
        let scoring = ScoringText::new("a, b; c");
        let tok = Tokenizer::Words {
            case: CaseFold::Lower,
            nfc: true,
            punct_as_tokens: false,
            apostrophes: ApostrophePolicy::Unify,
        };
        let stream = tok.tokenize(&scoring);
        assert_eq!(stream.len(), 3);
    }

    #[test]
    fn clitic_splitting() {
        let scoring = ScoringText::new("I don't; they're");
        let tok = Tokenizer::Words {
            case: CaseFold::Lower,
            nfc: true,
            punct_as_tokens: false,
            apostrophes: ApostrophePolicy::SplitClitics,
        };
        let forms: Vec<String> = tok
            .tokenize(&scoring)
            .iter()
            .map(|(_, f)| f.to_owned())
            .collect();
        assert_eq!(forms, vec!["i", "don", "'t", "they", "'re"]);
    }

    #[test]
    fn char_ngrams_keep_spaces_and_map_back() {
        let text = "ab cd";
        let scoring = ScoringText::new(text);
        let stream = Tokenizer::Chars {
            n: 3,
            case: CaseFold::Lower,
        }
        .tokenize(&scoring);
        let forms: Vec<&str> = stream.iter().map(|(_, f)| f).collect();
        assert_eq!(forms, vec!["ab ", "b c", " cd"]);
        for (token, form) in stream.iter() {
            assert_eq!(&text[token.span.range()], form);
        }
    }

    #[test]
    fn tokenizer_policy_round_trips_through_serde() {
        let tok = Tokenizer::default();
        let json = serde_json::to_string(&tok).unwrap();
        assert_eq!(serde_json::from_str::<Tokenizer>(&json).unwrap(), tok);
    }

    #[test]
    fn cjk_and_emoji_do_not_panic() {
        let (_, stream) = words("漢字とかな 👨‍👩‍👧 mixed");
        assert!(stream.len() > 1);
    }
}
