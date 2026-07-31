//! Character n-grams with positional typing.
//!
//! Untyped character n-grams work, but they conflate different phenomena under
//! one dimension: `sti` as a word prefix is an authorial habit, `sti` in the
//! middle of `question` is noise. Sapkota et al. (NAACL 2015) showed that
//! tagging each n-gram with *where it occurred* — affix, whole-word, mid-word,
//! multi-word, or one of three punctuation positions — and keeping only the
//! affix and punctuation types matches or beats untyped-everything at roughly
//! two thirds of the feature count, with the largest gains cross-domain.
//!
//! The other benefit is the one this crate cares most about: a typed dimension
//! can be explained. `c3:prefix:sti` is a statement about how someone starts
//! words.
//!
//! N-grams are taken over the scoring text **including spaces and
//! punctuation**; stripping them is a known way to throw away most of the
//! signal.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ratio, DimInfo, Family, Feature, FitContext, FittedFeature, Unit};
use crate::error::{Error, Result};
use crate::text::{Analysis, ScoringText, Span};
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Where an n-gram sits relative to word and punctuation boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NgramType {
    /// Covers the first `n` characters of a longer word.
    Prefix,
    /// Covers the last `n` characters of a longer word.
    Suffix,
    /// Starts with a space, continues into a word.
    SpacePrefix,
    /// Ends with a space, preceded by the tail of a word.
    SpaceSuffix,
    /// Is exactly one whole word.
    WholeWord,
    /// Lies strictly inside one word.
    MidWord,
    /// Spans a word boundary without touching its edge.
    MultiWord,
    /// Begins with punctuation, with none in the middle.
    BegPunct,
    /// Has punctuation somewhere other than its first or last character.
    MidPunct,
    /// Ends with punctuation, with none in the middle.
    EndPunct,
}

impl NgramType {
    /// Short tag used in dimension names.
    pub fn as_str(self) -> &'static str {
        match self {
            NgramType::Prefix => "prefix",
            NgramType::Suffix => "suffix",
            NgramType::SpacePrefix => "spaceprefix",
            NgramType::SpaceSuffix => "spacesuffix",
            NgramType::WholeWord => "wholeword",
            NgramType::MidWord => "midword",
            NgramType::MultiWord => "multiword",
            NgramType::BegPunct => "begpunct",
            NgramType::MidPunct => "midpunct",
            NgramType::EndPunct => "endpunct",
        }
    }

    /// Every type, in declaration order.
    pub const ALL: &'static [NgramType] = &[
        NgramType::Prefix,
        NgramType::Suffix,
        NgramType::SpacePrefix,
        NgramType::SpaceSuffix,
        NgramType::WholeWord,
        NgramType::MidWord,
        NgramType::MultiWord,
        NgramType::BegPunct,
        NgramType::MidPunct,
        NgramType::EndPunct,
    ];

    /// The affix + punctuation set from Sapkota et al. 2015.
    pub const AFFIX_PUNCT: &'static [NgramType] = &[
        NgramType::Prefix,
        NgramType::Suffix,
        NgramType::SpacePrefix,
        NgramType::SpaceSuffix,
        NgramType::BegPunct,
        NgramType::MidPunct,
        NgramType::EndPunct,
    ];
}

/// Which n-gram types become dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[derive(Default)]
pub enum NgramTypes {
    /// Keep every type.
    All,
    /// Keep [`NgramType::AFFIX_PUNCT`] — the recommended setting.
    #[default]
    AffixPunct,
    /// Keep an explicit set.
    Only {
        /// Types to keep.
        set: Vec<NgramType>,
    },
}

impl NgramTypes {
    fn contains(&self, ty: NgramType) -> bool {
        match self {
            NgramTypes::All => true,
            NgramTypes::AffixPunct => NgramType::AFFIX_PUNCT.contains(&ty),
            NgramTypes::Only { set } => set.contains(&ty),
        }
    }
}

/// Configuration for the character n-gram family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharNgrams {
    /// N-gram orders to extract, e.g. `[3, 4]`.
    pub orders: Vec<usize>,
    /// How many n-grams to keep per order.
    #[serde(default = "default_top")]
    pub top: usize,
    /// Tag n-grams by position. With this off, `types` is ignored and all
    /// occurrences of a string share one dimension.
    #[serde(default = "crate::util::yes")]
    pub typed: bool,
    /// Which types to keep when `typed` is on.
    #[serde(default)]
    pub types: NgramTypes,
    /// Lowercase the text before extraction.
    #[serde(default = "crate::util::yes")]
    pub lowercase: bool,
}

fn default_top() -> usize {
    1000
}

impl Default for CharNgrams {
    fn default() -> Self {
        CharNgrams::new(3..=4)
    }
}

impl CharNgrams {
    /// Extract n-grams of the given orders.
    ///
    /// ```
    /// # use handprint_core::feature::CharNgrams;
    /// let f = CharNgrams::new(3..=4).top(2000).typed(true);
    /// assert_eq!(f.orders, vec![3, 4]);
    /// ```
    pub fn new(orders: impl IntoIterator<Item = usize>) -> Self {
        CharNgrams {
            orders: orders.into_iter().collect(),
            top: default_top(),
            typed: true,
            types: NgramTypes::default(),
            lowercase: true,
        }
    }

    /// Keep this many n-grams per order.
    pub fn top(mut self, top: usize) -> Self {
        self.top = top;
        self
    }

    /// Turn positional typing on or off.
    pub fn typed(mut self, typed: bool) -> Self {
        self.typed = typed;
        self
    }

    /// Restrict which types become dimensions.
    pub fn types(mut self, types: NgramTypes) -> Self {
        self.types = types;
        self
    }
}

/// Fitted [`CharNgrams`]: the selected n-gram vocabulary per order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedCharNgrams {
    dims: Vec<DimInfo>,
    /// The n-gram order each entry of `dims` came from.
    dim_orders: Vec<usize>,
    orders: Vec<usize>,
    /// One bucket per `(order, type)`, each mapping a bare n-gram string to its
    /// symbol.
    ///
    /// Bucketing rather than one map keyed by `"order\u{1}type\u{1}gram"`
    /// matters more than it looks: `transform` performs one lookup per n-gram
    /// *occurrence*, so a composite key would mean a `String` allocation per
    /// character of every document profiled. Bucketing lets the lookup borrow
    /// the n-gram straight out of the scan buffer.
    buckets: Vec<Bucket>,
    typed: bool,
    types: NgramTypes,
    lowercase: bool,
}

/// The vocabulary selected for one `(order, type)` pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Bucket {
    order: usize,
    /// `None` when the feature is untyped.
    ty: Option<NgramType>,
    grams: HashMap<String, Symbol>,
}

impl FittedCharNgrams {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::CharNgram;

    /// How many n-gram dimensions were selected.
    pub fn vocab_len(&self) -> usize {
        self.buckets.iter().map(|b| b.grams.len()).sum()
    }

    /// The symbol for one typed n-gram, or `None` if it is not in the fitted
    /// vocabulary.
    fn lookup(&self, order: usize, ty: Option<NgramType>, gram: &str) -> Option<Symbol> {
        self.buckets
            .iter()
            .find(|b| b.order == order && b.ty == ty)
            .and_then(|b| b.grams.get(gram).copied())
    }
}

fn dim_name(order: usize, ty: Option<NgramType>, gram: &str) -> String {
    let escaped = escape(gram);
    match ty {
        Some(ty) => format!("c{order}:{}:{escaped}", ty.as_str()),
        None => format!("c{order}:{escaped}"),
    }
}

fn escape(gram: &str) -> String {
    let mut out = String::with_capacity(gram.len());
    for c in gram.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

impl Feature for CharNgrams {
    type Fitted = FittedCharNgrams;

    fn fit(&self, ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedCharNgrams> {
        if self.orders.is_empty() || self.orders.iter().any(|&n| n < 2) {
            return Err(Error::InvalidConfig {
                what: "CharNgrams::orders",
                detail: "must be non-empty with every order >= 2".into(),
            });
        }
        if self.top == 0 {
            return Err(Error::InvalidConfig {
                what: "CharNgrams::top",
                detail: "must be at least 1".into(),
            });
        }

        let mut dims = Vec::new();
        let mut dim_orders = Vec::new();
        let mut buckets: Vec<Bucket> = Vec::new();

        // One scan per document, reused across every order.
        let scans: Vec<(&crate::text::ScoringText, Scan)> = ctx
            .analyses()
            .map(|a| (a.scoring(), Scan::new(a.scoring(), self.lowercase)))
            .collect();

        for &order in &self.orders {
            let mut counts: HashMap<(Option<NgramType>, String), usize> = HashMap::new();
            for (scoring, scan) in &scans {
                walk_scan(scan, scoring, order, |ty, gram, _span| {
                    if self.typed && !self.types.contains(ty) {
                        return;
                    }
                    let tagged = if self.typed { Some(ty) } else { None };
                    *counts.entry((tagged, gram.to_owned())).or_insert(0) += 1;
                });
            }
            let mut ranked: Vec<((Option<NgramType>, String), usize)> =
                counts.into_iter().collect();
            ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            ranked.truncate(self.top);

            for ((ty, gram), _) in ranked {
                let sym = interner.intern(&dim_name(order, ty, &gram));
                dims.push(DimInfo::new(
                    sym,
                    Family::CharNgram,
                    Unit::RelativeFrequency,
                ));
                dim_orders.push(order);
                match buckets.iter_mut().find(|b| b.order == order && b.ty == ty) {
                    Some(bucket) => {
                        bucket.grams.insert(gram, sym);
                    }
                    None => {
                        let mut grams = HashMap::new();
                        grams.insert(gram, sym);
                        buckets.push(Bucket { order, ty, grams });
                    }
                }
            }
        }

        if dims.is_empty() {
            return Err(Error::CorpusTooSmall {
                feature: "CharNgrams",
                detail: format!(
                    "no n-grams of orders {:?} survived selection over {} document(s)",
                    self.orders,
                    ctx.len()
                ),
            });
        }

        Ok(FittedCharNgrams {
            dims,
            dim_orders,
            orders: self.orders.clone(),
            buckets,
            typed: self.typed,
            types: self.types.clone(),
            lowercase: self.lowercase,
        })
    }
}

impl FittedFeature for FittedCharNgrams {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::CharNgram
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let mut values: HashMap<Symbol, f64> = HashMap::new();
        let mut spans: Vec<(Symbol, Span)> = Vec::new();
        let mut produced: Vec<usize> = Vec::new();
        let scan = Scan::new(analysis.scoring(), self.lowercase);

        for &order in &self.orders {
            let mut counts: HashMap<Symbol, usize> = HashMap::new();
            let mut total = 0usize;
            walk_scan(&scan, analysis.scoring(), order, |ty, gram, span| {
                total += 1;
                if self.typed && !self.types.contains(ty) {
                    return;
                }
                let tagged = if self.typed { Some(ty) } else { None };
                if let Some(sym) = self.lookup(order, tagged, gram) {
                    *counts.entry(sym).or_insert(0) += 1;
                    spans.push((sym, span));
                }
            });
            if total == 0 {
                // The document is shorter than this order's window; its
                // dimensions are undefined, not zero.
                continue;
            }
            produced.push(order);
            for (sym, count) in counts {
                values.insert(sym, ratio(count, total));
            }
        }

        for (dim, &order) in self.dims.iter().zip(&self.dim_orders) {
            if produced.contains(&order) {
                out.set(dim.symbol, values.get(&dim.symbol).copied().unwrap_or(0.0));
            } else {
                out.mark_missing(dim.symbol);
            }
        }
        for (sym, span) in spans {
            out.note_span(sym, span);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Word,
    Space,
    Punct,
}

fn class_of(c: char) -> CharClass {
    if c.is_whitespace() {
        CharClass::Space
    } else if c.is_alphanumeric() {
        CharClass::Word
    } else {
        // Symbols, emoji and anything else behave like punctuation for typing
        // purposes: they break words the same way.
        CharClass::Punct
    }
}

/// Everything an n-gram walk needs from a document, computed once.
///
/// The naive shape recomputes the case fold, the character vector, the class
/// vector and the offset table for every n-gram order. With the default
/// `3..=4` that is twice the work for no reason, and it made this the slowest
/// family in the pipeline by a wide margin.
struct Scan {
    chars: Vec<char>,
    classes: Vec<CharClass>,
    /// Byte offset in the scoring text for each character index.
    src_starts: Vec<usize>,
    text_len: usize,
}

impl Scan {
    fn new(scoring: &ScoringText, lowercase: bool) -> Scan {
        let text = scoring.as_str();
        let chars: Vec<char> = if lowercase {
            // ASCII case folding is a byte operation; the Unicode path allocates
            // and consults tables for every character.
            if text.is_ascii() {
                text.chars().map(|c| c.to_ascii_lowercase()).collect()
            } else {
                text.to_lowercase().chars().collect()
            }
        } else {
            text.chars().collect()
        };
        let classes: Vec<CharClass> = chars.iter().copied().map(class_of).collect();
        let src_starts: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        Scan {
            chars,
            classes,
            src_starts,
            text_len: text.len(),
        }
    }

    /// Characters usable for n-gram extraction.
    ///
    /// A case fold can change the character count (`\u{0130}` lowercases to two
    /// characters), so spans are only trustworthy up to the shorter of the two.
    fn usable(&self) -> usize {
        self.src_starts.len().min(self.chars.len())
    }
}

/// Walk every n-gram of `order` over a prepared scan, classifying each by
/// position.
fn walk_scan(
    scan: &Scan,
    scoring: &ScoringText,
    order: usize,
    mut f: impl FnMut(NgramType, &str, Span),
) {
    let usable = scan.usable();
    if order == 0 || usable < order {
        return;
    }
    let mut buf = String::with_capacity(order * 4);
    for i in 0..=(usable - order) {
        buf.clear();
        buf.extend(&scan.chars[i..i + order]);
        let ty = classify(&scan.classes, i, order);
        let start = scan.src_starts[i];
        let end = scan
            .src_starts
            .get(i + order)
            .copied()
            .unwrap_or(scan.text_len);
        f(ty, &buf, scoring.to_source_span(Span::new(start, end)));
    }
}

fn classify(classes: &[CharClass], i: usize, n: usize) -> NgramType {
    let window = &classes[i..i + n];
    let has_punct = window.contains(&CharClass::Punct);
    if has_punct {
        let mid_punct = window[1..n.saturating_sub(1)].contains(&CharClass::Punct);
        return if mid_punct {
            NgramType::MidPunct
        } else if window[0] == CharClass::Punct {
            NgramType::BegPunct
        } else {
            NgramType::EndPunct
        };
    }
    if window.contains(&CharClass::Space) {
        return if window[0] == CharClass::Space {
            NgramType::SpacePrefix
        } else if window[n - 1] == CharClass::Space {
            NgramType::SpaceSuffix
        } else {
            NgramType::MultiWord
        };
    }
    // Entirely word characters, therefore entirely inside one word.
    let at_word_start = i == 0 || classes[i - 1] != CharClass::Word;
    let at_word_end = i + n >= classes.len() || classes[i + n] != CharClass::Word;
    match (at_word_start, at_word_end) {
        (true, true) => NgramType::WholeWord,
        (true, false) => NgramType::Prefix,
        (false, true) => NgramType::Suffix,
        (false, false) => NgramType::MidWord,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::FitDoc;
    use crate::text::{Document, Tokenizer};

    fn types_of(text: &str, order: usize) -> Vec<(String, NgramType)> {
        let scoring = ScoringText::new(text);
        let scan = Scan::new(&scoring, true);
        let mut out = Vec::new();
        walk_scan(&scan, &scoring, order, |ty, gram, _| {
            out.push((gram.to_owned(), ty))
        });
        out
    }

    #[test]
    fn typing_at_word_boundaries() {
        // "sting cat." — check each 3-gram's position type.
        let got = types_of("sting cat.", 3);
        let expect = vec![
            ("sti", NgramType::Prefix),
            ("tin", NgramType::MidWord),
            ("ing", NgramType::Suffix),
            ("ng ", NgramType::SpaceSuffix),
            ("g c", NgramType::MultiWord),
            (" ca", NgramType::SpacePrefix),
            ("cat", NgramType::WholeWord),
            ("at.", NgramType::EndPunct),
        ];
        let got: Vec<(&str, NgramType)> = got.iter().map(|(g, t)| (g.as_str(), *t)).collect();
        assert_eq!(got, expect);
    }

    #[test]
    fn punctuation_typing() {
        let got = types_of("a, b", 3);
        let got: Vec<(&str, NgramType)> = got.iter().map(|(g, t)| (g.as_str(), *t)).collect();
        assert_eq!(
            got,
            vec![("a, ", NgramType::MidPunct), (", b", NgramType::BegPunct)]
        );
    }

    fn fit(spec: CharNgrams, texts: &[&str]) -> (FittedCharNgrams, Interner) {
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

    #[test]
    fn affix_punct_mode_drops_mid_word() {
        let (fitted, interner) = fit(CharNgrams::new([3]).top(100), &["sting sting sting"]);
        assert!(interner.get("c3:prefix:sti").is_some());
        assert!(interner.get("c3:suffix:ing").is_some());
        assert!(
            interner.get("c3:midword:tin").is_none(),
            "mid-word n-grams must be excluded in AffixPunct mode"
        );
        assert!(fitted.vocab_len() > 0);
    }

    #[test]
    fn all_types_mode_keeps_mid_word() {
        let (_, interner) = fit(
            CharNgrams::new([3]).top(100).types(NgramTypes::All),
            &["sting sting"],
        );
        assert!(interner.get("c3:midword:tin").is_some());
    }

    #[test]
    fn untyped_mode_merges_positions() {
        let (_, interner) = fit(
            CharNgrams::new([3]).top(100).typed(false),
            &["sting resting"],
        );
        assert!(interner.get("c3:tin").is_some());
        assert!(interner.get("c3:prefix:sti").is_none());
    }

    #[test]
    fn transform_emits_relative_frequencies_within_order() {
        let (fitted, interner) = fit(
            CharNgrams::new([3]).top(100).types(NgramTypes::All),
            &["abcabc"],
        );
        let doc = Document::new("abcabc");
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new();
        fitted.transform(&analysis, &mut b);
        let v = b.build();
        // 4 trigrams: abc, bca, cab, abc. "abc" is a prefix once and mid-word
        // once, so the typed dims split the mass.
        let total: f64 = fitted.dims().iter().map(|d| v.get(d.symbol)).sum();
        assert!((total - 1.0).abs() < 1e-12, "{total}");
        assert!(v.get(interner.get("c3:prefix:abc").unwrap()) > 0.0);
    }

    #[test]
    fn spans_map_back_to_source() {
        let text = "sting";
        let (fitted, interner) = fit(CharNgrams::new([3]).top(100), &[text]);
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        let v = b.build();
        let spans = v.spans(interner.get("c3:prefix:sti").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(&text[spans[0].range()], "sti");
    }
}
