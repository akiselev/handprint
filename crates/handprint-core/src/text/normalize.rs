//! Normalization policy.
//!
//! handprint keeps two views of every document:
//!
//! * the **source text**, byte-for-byte as supplied. All [`Span`]s that ever
//!   leave the crate index into this. Typography — curly vs straight quotes,
//!   em vs en dash, `…` vs `...`, NBSP — is a first-class feature family, so it
//!   is *never* normalized away before the typography features see it.
//! * the **scoring text**, in which Unicode confusables are folded to their
//!   Latin skeleton and invisible formatting characters are dropped. Tokenizing
//!   this view is what stops homoglyph stuffing from dodging the word features
//!   (SilverSpeak-style attacks); the artifact scan reports the substitution
//!   separately as a finding.
//!
//! NFC and case folding are applied **per word token**, not to the whole text.
//! Whole-text NFC would shift every downstream byte offset for no benefit,
//! and the characters it would fold are exactly the ones the typography
//! features want to see.
//!
//! [`Span`]: crate::text::Span

use crate::text::Span;

/// A view of a document with confusables folded and invisibles removed,
/// carrying the offset map back to the source text.
#[derive(Debug, Clone)]
pub struct ScoringText {
    text: String,
    map: OffsetMap,
    source_len: usize,
}

#[derive(Debug, Clone)]
enum OffsetMap {
    /// Nothing was rewritten: scoring offsets *are* source offsets.
    Identity,
    /// One `(scoring_offset, source_offset)` entry per character, plus a final
    /// sentinel at `(scoring_len, source_len)`. Sorted by scoring offset.
    Table(Vec<(u32, u32)>),
}

impl ScoringText {
    /// Build the scoring view of `source`.
    pub fn new(source: &str) -> Self {
        // Fast path: the overwhelmingly common case is text that needs no
        // rewriting at all, and we can detect it with one scan that allocates
        // nothing.
        if source.is_ascii() || !source.chars().any(|c| needs_rewrite(c).is_some()) {
            return Self {
                text: source.to_owned(),
                map: OffsetMap::Identity,
                source_len: source.len(),
            };
        }

        let mut text = String::with_capacity(source.len());
        let mut table = Vec::new();
        for (src_off, ch) in source.char_indices() {
            table.push((text.len() as u32, src_off as u32));
            match needs_rewrite(ch) {
                Some(Rewrite::Drop) => {}
                Some(Rewrite::To(replacement)) => text.push(replacement),
                None => text.push(ch),
            }
        }
        table.push((text.len() as u32, source.len() as u32));
        Self {
            text,
            map: OffsetMap::Table(table),
            source_len: source.len(),
        }
    }

    /// The folded text. Tokenizers run over this.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// True when the scoring view is byte-identical to the source.
    #[inline]
    pub fn is_identity(&self) -> bool {
        matches!(self.map, OffsetMap::Identity)
    }

    /// Map a byte offset in the scoring text back to the source text.
    ///
    /// Offsets that fall on a character boundary — which is all a tokenizer
    /// ever produces — map exactly.
    pub fn to_source_offset(&self, offset: usize) -> usize {
        match &self.map {
            OffsetMap::Identity => offset.min(self.source_len),
            OffsetMap::Table(table) => {
                match table.binary_search_by_key(&(offset as u32), |e| e.0) {
                    // Several scoring offsets can collide when characters were
                    // dropped; binary_search may land on any of them. Walk back to
                    // the first entry with this offset so a dropped run maps to its
                    // start rather than its end.
                    Ok(i) => {
                        let mut i = i;
                        while i > 0 && table[i - 1].0 == offset as u32 {
                            i -= 1;
                        }
                        table[i].1 as usize
                    }
                    Err(0) => 0,
                    Err(i) => table[i - 1].1 as usize,
                }
            }
        }
    }

    /// Map a span in the scoring text back to the source text.
    pub fn to_source_span(&self, span: Span) -> Span {
        Span::new(
            self.to_source_offset(span.start),
            self.to_source_offset(span.end),
        )
    }
}

enum Rewrite {
    Drop,
    To(char),
}

/// Characters that must not survive into the scoring view.
fn needs_rewrite(c: char) -> Option<Rewrite> {
    if c.is_ascii() {
        return None;
    }
    if is_invisible(c) {
        return Some(Rewrite::Drop);
    }
    fold_confusable(c).map(Rewrite::To)
}

/// Invisible / formatting characters that carry no style signal but do break
/// tokenization. Zero-width joiner is deliberately absent: it is load-bearing
/// inside emoji sequences, so it is flagged by the artifact scan instead.
pub fn is_invisible(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'    // soft hyphen
        | '\u{061C}'  // arabic letter mark
        | '\u{180E}'  // mongolian vowel separator
        | '\u{200B}'  // zero width space
        | '\u{200C}'  // zero width non-joiner
        | '\u{200E}'..='\u{200F}' // LRM / RLM
        | '\u{202A}'..='\u{202E}' // bidi embedding / override
        | '\u{2060}'..='\u{2064}' // word joiner, invisible operators
        | '\u{2066}'..='\u{2069}' // bidi isolates
        | '\u{FEFF}'  // BOM / zero width no-break space
        | '\u{FFF9}'..='\u{FFFB}' // interlinear annotation
    )
}

/// True for the zero-width joiner, which is legitimate inside emoji sequences
/// and suspicious everywhere else.
#[inline]
pub fn is_zero_width_joiner(c: char) -> bool {
    c == '\u{200D}'
}

/// Fold a confusable character to its Latin/ASCII skeleton.
///
/// This is a curated subset of the Unicode confusables table covering what is
/// actually used to attack text classifiers: Cyrillic and Greek homoglyphs,
/// fullwidth forms, and the mathematical alphanumeric planes. A full
/// `confusables.txt` port would be ~6k entries and is not worth the binary
/// size for the marginal cases.
pub fn fold_confusable(c: char) -> Option<char> {
    // Mathematical alphanumeric symbols and fullwidth forms are dense ranges,
    // so arithmetic beats a table.
    if let Some(folded) = fold_ranged(c) {
        return Some(folded);
    }
    let folded = match c {
        // Cyrillic → Latin
        'А' => 'A',
        'В' => 'B',
        'Е' => 'E',
        'К' => 'K',
        'М' => 'M',
        'Н' => 'H',
        'О' => 'O',
        'Р' => 'P',
        'С' => 'C',
        'Т' => 'T',
        'У' => 'Y',
        'Х' => 'X',
        'Ѕ' => 'S',
        'І' => 'I',
        'Ј' => 'J',
        'Ԁ' => 'D',
        'Ԍ' => 'G',
        'Ԛ' => 'Q',
        'Ԝ' => 'W',
        'Ѵ' => 'V',
        'а' => 'a',
        'в' => 'b',
        'е' => 'e',
        'к' => 'k',
        'м' => 'm',
        'н' => 'h',
        'о' => 'o',
        'р' => 'p',
        'с' => 'c',
        'т' => 't',
        'у' => 'y',
        'х' => 'x',
        'ѕ' => 's',
        'і' => 'i',
        'ј' => 'j',
        'ԛ' => 'q',
        'ԝ' => 'w',
        'ѵ' => 'v',
        'ё' => 'e',
        'ї' => 'i',
        'ґ' => 'r',
        'ѐ' => 'e',
        // Greek → Latin
        'Α' => 'A',
        'Β' => 'B',
        'Ε' => 'E',
        'Ζ' => 'Z',
        'Η' => 'H',
        'Ι' => 'I',
        'Κ' => 'K',
        'Μ' => 'M',
        'Ν' => 'N',
        'Ο' => 'O',
        'Ρ' => 'P',
        'Τ' => 'T',
        'Υ' => 'Y',
        'Χ' => 'X',
        'Ϲ' => 'C',
        'Ϳ' => 'J',
        'α' => 'a',
        'ο' => 'o',
        'ρ' => 'p',
        'ν' => 'v',
        'ϲ' => 'c',
        'ι' => 'i',
        'κ' => 'k',
        'υ' => 'u',
        'τ' => 't',
        'ϳ' => 'j',
        // Armenian / Cherokee / other single-letter lookalikes
        'Ꭺ' => 'A',
        'Ꭼ' => 'E',
        'Ꮋ' => 'H',
        'Ꮖ' => 'I',
        'Ꭻ' => 'J',
        'Ꮶ' => 'K',
        'Ꮮ' => 'L',
        'Ꮇ' => 'M',
        'Ꭴ' => 'O',
        'Ꮲ' => 'P',
        'Ꭱ' => 'R',
        'Ꮪ' => 'S',
        'Ꭲ' => 'T',
        'Ꮩ' => 'V',
        'Ꮃ' => 'W',
        'Ꭹ' => 'Y',
        'Ꮓ' => 'Z',
        'օ' => 'o',
        'ս' => 'u',
        'ց' => 'g',
        'ք' => 'p',
        _ => return None,
    };
    Some(folded)
}

fn fold_ranged(c: char) -> Option<char> {
    let cp = c as u32;
    // Fullwidth ASCII: U+FF01..=U+FF5E ↔ U+0021..=U+007E
    if (0xFF01..=0xFF5E).contains(&cp) {
        return char::from_u32(cp - 0xFF01 + 0x21);
    }
    // Mathematical alphanumeric symbols, U+1D400..=U+1D7FF. Each style block
    // is 26 uppercase then 26 lowercase; the plane has holes (reserved code
    // points whose glyphs live in the BMP letterlike block) which we skip
    // rather than mis-fold.
    if (0x1D400..=0x1D6A3).contains(&cp) {
        let idx = (cp - 0x1D400) % 52;
        return Some(if idx < 26 {
            (b'A' + idx as u8) as char
        } else {
            (b'a' + (idx - 26) as u8) as char
        });
    }
    // Mathematical digits, U+1D7CE..=U+1D7FF, five styles of 0-9.
    if (0x1D7CE..=0x1D7FF).contains(&cp) {
        let idx = (cp - 0x1D7CE) % 10;
        return char::from_u32('0' as u32 + idx);
    }
    None
}

/// Coarse script classification, used to flag mixed-script words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Script {
    /// Digits, punctuation, whitespace — compatible with any other script.
    Common,
    /// Latin letters.
    Latin,
    /// Cyrillic letters.
    Cyrillic,
    /// Greek letters.
    Greek,
    /// CJK ideographs and kana.
    Han,
    /// Arabic letters.
    Arabic,
    /// Hebrew letters.
    Hebrew,
    /// Anything else with a letter category.
    Other,
}

/// Classify a character's script.
pub fn script_of(c: char) -> Script {
    if c.is_ascii_alphabetic() {
        return Script::Latin;
    }
    if !c.is_alphabetic() {
        return Script::Common;
    }
    let cp = c as u32;
    match cp {
        0x00C0..=0x024F | 0x1E00..=0x1EFF | 0x2C60..=0x2C7F | 0xA720..=0xA7FF => Script::Latin,
        0x0370..=0x03FF | 0x1F00..=0x1FFF => Script::Greek,
        0x0400..=0x052F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => Script::Cyrillic,
        0x0590..=0x05FF => Script::Hebrew,
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => Script::Arabic,
        0x2E80..=0x9FFF | 0xF900..=0xFAFF => Script::Han,
        _ => Script::Other,
    }
}

/// Normalize a token's surface form for *word* features: NFC, then optional
/// case folding and apostrophe unification.
///
/// Returns a borrowed string whenever nothing changed, which is the common case
/// for lowercase ASCII prose.
pub fn normalize_token(
    s: &str,
    nfc: bool,
    lowercase: bool,
    fold_apostrophes: bool,
) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    use unicode_normalization::UnicodeNormalization;

    // ASCII is always NFC, contains no curly apostrophe, and lowercases
    // bytewise. Since almost every token in English prose is ASCII, this fast
    // path removes three whole-token scans from the common case.
    if s.is_ascii() {
        return if lowercase && s.bytes().any(|b| b.is_ascii_uppercase()) {
            Cow::Owned(s.to_ascii_lowercase())
        } else {
            Cow::Borrowed(s)
        };
    }

    let mut out: Cow<'_, str> = Cow::Borrowed(s);
    if nfc
        && !matches!(
            unicode_normalization::is_nfc_quick(s.chars()),
            unicode_normalization::IsNormalized::Yes
        )
    {
        out = Cow::Owned(s.nfc().collect());
    }
    if fold_apostrophes && out.contains(['\u{2019}', '\u{02BC}', '\u{FF07}']) {
        out = Cow::Owned(out.replace(['\u{2019}', '\u{02BC}', '\u{FF07}'], "'"));
    }
    if lowercase && out.chars().any(char::is_uppercase) {
        out = Cow::Owned(out.to_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_view_for_plain_text() {
        let st = ScoringText::new("hello, world — it's fine");
        assert!(st.is_identity());
        assert_eq!(st.as_str(), "hello, world — it's fine");
        assert_eq!(st.to_source_offset(7), 7);
    }

    #[test]
    fn folds_cyrillic_homoglyphs_and_keeps_offsets() {
        // "раypal" with Cyrillic р and а.
        let source = "\u{0440}\u{0430}ypal now";
        let st = ScoringText::new(source);
        assert_eq!(st.as_str(), "paypal now");
        assert!(!st.is_identity());
        // "now" starts at scoring offset 7; in the source the two Cyrillic
        // characters are two bytes each, so it starts at 9.
        assert_eq!(st.to_source_offset(7), 9);
        assert_eq!(&source[st.to_source_span(Span::new(7, 10)).range()], "now");
    }

    #[test]
    fn drops_zero_width_characters() {
        let source = "de\u{200B}lve";
        let st = ScoringText::new(source);
        assert_eq!(st.as_str(), "delve");
        assert_eq!(st.to_source_offset(5), source.len());
        assert_eq!(
            &source[st.to_source_span(Span::new(2, 5)).range()],
            "\u{200B}lve"
        );
    }

    #[test]
    fn folds_fullwidth_and_math_alphanumerics() {
        assert_eq!(ScoringText::new("ｈｅｌｌｏ").as_str(), "hello");
        // 𝐇𝐞𝐥𝐥𝐨 (mathematical bold)
        assert_eq!(ScoringText::new("\u{1D407}\u{1D41E}").as_str(), "He");
        assert_eq!(ScoringText::new("\u{1D7CE}\u{1D7CF}").as_str(), "01");
    }

    #[test]
    fn token_normalization_borrows_when_unchanged() {
        assert!(matches!(
            normalize_token("delve", true, true, true),
            std::borrow::Cow::Borrowed("delve")
        ));
        assert_eq!(normalize_token("Don\u{2019}t", true, true, true), "don't");
    }

    #[test]
    fn script_classification() {
        assert_eq!(script_of('a'), Script::Latin);
        assert_eq!(script_of('\u{0430}'), Script::Cyrillic);
        assert_eq!(script_of('\u{03B1}'), Script::Greek);
        assert_eq!(script_of(','), Script::Common);
        assert_eq!(script_of('漢'), Script::Han);
    }
}
