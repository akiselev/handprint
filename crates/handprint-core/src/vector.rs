//! Phase 2 — symbols, interning, and sparse feature vectors.
//!
//! Every feature dimension is a human-readable name (`"punct:em_dash_rate"`,
//! `"mfw:the"`, `"c3:prefix:sti"`) interned to a [`Symbol`]. The interner is
//! owned by the [`Reference`](crate::Reference) and serialized with it, so a
//! shipped model can always name the dimension a number refers to — which is
//! the whole point of an interpretable stylometer.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::text::Span;

/// A handle to an interned dimension name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Symbol(pub u32);

impl Symbol {
    /// The symbol's index, for dense-vector addressing.
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Maps dimension names to [`Symbol`]s and back.
///
/// Serializes as the name table alone; the reverse map is rebuilt on load.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<String>", into = "Vec<String>")]
pub struct Interner {
    names: Vec<String>,
    #[allow(clippy::mutable_key_type)]
    lookup: HashMap<String, Symbol>,
}

impl From<Vec<String>> for Interner {
    fn from(names: Vec<String>) -> Self {
        let lookup = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), Symbol(i as u32)))
            .collect();
        Interner { names, lookup }
    }
}

impl From<Interner> for Vec<String> {
    fn from(i: Interner) -> Self {
        i.names
    }
}

impl Interner {
    /// An empty interner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a name, returning its existing symbol if already present.
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&sym) = self.lookup.get(name) {
            return sym;
        }
        let sym = Symbol(self.names.len() as u32);
        self.names.push(name.to_owned());
        self.lookup.insert(name.to_owned(), sym);
        sym
    }

    /// Look up an existing symbol without interning.
    #[inline]
    pub fn get(&self, name: &str) -> Option<Symbol> {
        self.lookup.get(name).copied()
    }

    /// The name a symbol stands for.
    #[inline]
    pub fn name(&self, sym: Symbol) -> Option<&str> {
        self.names.get(sym.index()).map(String::as_str)
    }

    /// The name a symbol stands for, or `"?"` for an unknown symbol.
    #[inline]
    pub fn resolve(&self, sym: Symbol) -> &str {
        self.name(sym).unwrap_or("?")
    }

    /// Number of interned names.
    #[inline]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// True when nothing has been interned.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// All names, indexed by symbol.
    #[inline]
    pub fn names(&self) -> &[String] {
        &self.names
    }
}

/// A sparse vector of feature values, sorted by symbol.
///
/// Sorted-and-merged is not premature cleverness here: profiles are compared
/// pairwise in bulk during calibration (tens of thousands of pairs), and the
/// merge walk is both faster and allocation-free compared to hashing.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    entries: Vec<(Symbol, f64)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    spans: Vec<(Symbol, Vec<Span>)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    missing: Vec<Symbol>,
}

impl FeatureVector {
    /// The `(symbol, value)` pairs, sorted by symbol.
    #[inline]
    pub fn entries(&self) -> &[(Symbol, f64)] {
        &self.entries
    }

    /// Number of non-zero dimensions.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no dimension is set.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The value of one dimension, or zero.
    pub fn get(&self, sym: Symbol) -> f64 {
        self.entries
            .binary_search_by_key(&sym, |e| e.0)
            .map(|i| self.entries[i].1)
            .unwrap_or(0.0)
    }

    /// The spans that produced a dimension, when span tracking was on.
    pub fn spans(&self, sym: Symbol) -> &[Span] {
        self.spans
            .binary_search_by_key(&sym, |e| e.0)
            .map(|i| self.spans[i].1.as_slice())
            .unwrap_or(&[])
    }

    /// True when this vector carries span attribution.
    #[inline]
    pub fn has_spans(&self) -> bool {
        !self.spans.is_empty()
    }

    /// Dimensions a feature declined to compute for this document.
    ///
    /// A missing value is not a zero. MATTR is undefined below its window and
    /// MTLD is meaningless on 30 tokens; emitting a silent zero there would
    /// make a short document look maximally unlike every author. Missing
    /// dimensions are imputed to the reference mean at scaling time — the
    /// neutral choice, which cannot push a distance in either direction — and
    /// the profile carries a warning saying so.
    #[inline]
    pub fn missing(&self) -> &[Symbol] {
        &self.missing
    }

    /// True when a dimension was explicitly marked missing.
    pub fn is_missing(&self, sym: Symbol) -> bool {
        self.missing.binary_search(&sym).is_ok()
    }

    /// Project onto a dense layout.
    ///
    /// `dims` must be sorted; missing dimensions become zero. This is the
    /// bridge into the metric layer, which works densely because the reference
    /// dimension set is fixed and shared by every profile.
    pub fn to_dense(&self, dims: &[Symbol]) -> Vec<f64> {
        let mut out = vec![0.0; dims.len()];
        let mut i = 0;
        for (j, &dim) in dims.iter().enumerate() {
            while i < self.entries.len() && self.entries[i].0 < dim {
                i += 1;
            }
            if i < self.entries.len() && self.entries[i].0 == dim {
                out[j] = self.entries[i].1;
            }
        }
        out
    }

    /// Sum of `|a_i - b_i|` over the union of dimensions, by merge walk.
    pub fn l1_distance(&self, other: &FeatureVector) -> f64 {
        let mut total = 0.0;
        merge(self, other, |_, a, b| total += (a - b).abs());
        total
    }

    /// Dot product over the intersection of dimensions.
    pub fn dot(&self, other: &FeatureVector) -> f64 {
        let mut total = 0.0;
        merge(self, other, |_, a, b| total += a * b);
        total
    }

    /// Euclidean norm.
    pub fn norm(&self) -> f64 {
        self.entries.iter().map(|(_, v)| v * v).sum::<f64>().sqrt()
    }
}

/// Walk two sorted sparse vectors in lockstep, calling `f(symbol, a, b)` for
/// every symbol present in either.
pub fn merge(a: &FeatureVector, b: &FeatureVector, mut f: impl FnMut(Symbol, f64, f64)) {
    let (ea, eb) = (&a.entries, &b.entries);
    let (mut i, mut j) = (0, 0);
    while i < ea.len() && j < eb.len() {
        match ea[i].0.cmp(&eb[j].0) {
            std::cmp::Ordering::Less => {
                f(ea[i].0, ea[i].1, 0.0);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                f(eb[j].0, 0.0, eb[j].1);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                f(ea[i].0, ea[i].1, eb[j].1);
                i += 1;
                j += 1;
            }
        }
    }
    for e in &ea[i..] {
        f(e.0, e.1, 0.0);
    }
    for e in &eb[j..] {
        f(e.0, 0.0, e.1);
    }
}

/// Accumulates feature values, then freezes into a [`FeatureVector`].
///
/// Feature implementations only ever see this type, which keeps them from
/// depending on the vector's internal representation.
#[derive(Debug, Clone, Default)]
pub struct VectorBuilder {
    values: HashMap<Symbol, f64>,
    spans: HashMap<Symbol, Vec<Span>>,
    missing: Vec<Symbol>,
    track_spans: bool,
}

impl VectorBuilder {
    /// A builder with span tracking off.
    pub fn new() -> Self {
        Self::default()
    }

    /// Turn span attribution on. Off by default: it roughly doubles the cost of
    /// a profile and is only needed when a report will point at the text.
    pub fn track_spans(mut self, on: bool) -> Self {
        self.track_spans = on;
        self
    }

    /// Whether this builder is recording spans.
    #[inline]
    pub fn tracks_spans(&self) -> bool {
        self.track_spans
    }

    /// Set a dimension, replacing any previous value.
    pub fn set(&mut self, sym: Symbol, value: f64) {
        self.values.insert(sym, value);
    }

    /// Add to a dimension.
    pub fn add(&mut self, sym: Symbol, value: f64) {
        *self.values.entry(sym).or_insert(0.0) += value;
    }

    /// Add to a dimension and record the span that caused it.
    pub fn add_at(&mut self, sym: Symbol, value: f64, span: Span) {
        self.add(sym, value);
        if self.track_spans {
            self.spans.entry(sym).or_default().push(span);
        }
    }

    /// Record a span for a dimension without changing its value.
    pub fn note_span(&mut self, sym: Symbol, span: Span) {
        if self.track_spans {
            self.spans.entry(sym).or_default().push(span);
        }
    }

    /// Declare that a dimension could not be computed for this document.
    ///
    /// Use this rather than emitting a placeholder — see
    /// [`FeatureVector::missing`].
    pub fn mark_missing(&mut self, sym: Symbol) {
        self.values.remove(&sym);
        self.missing.push(sym);
    }

    /// Freeze into a sorted sparse vector.
    pub fn build(self) -> FeatureVector {
        let mut entries: Vec<(Symbol, f64)> = self.values.into_iter().collect();
        entries.sort_unstable_by_key(|e| e.0);
        let mut spans: Vec<(Symbol, Vec<Span>)> = self
            .spans
            .into_iter()
            .map(|(sym, mut v)| {
                v.sort_unstable();
                v.dedup();
                (sym, v)
            })
            .collect();
        spans.sort_unstable_by_key(|e| e.0);
        let mut missing = self.missing;
        missing.sort_unstable();
        missing.dedup();
        FeatureVector {
            entries,
            spans,
            missing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_of(pairs: &[(u32, f64)]) -> FeatureVector {
        let mut b = VectorBuilder::new();
        for &(s, v) in pairs {
            b.set(Symbol(s), v);
        }
        b.build()
    }

    #[test]
    fn interner_round_trips() {
        let mut i = Interner::new();
        let a = i.intern("mfw:the");
        let b = i.intern("mfw:of");
        assert_eq!(i.intern("mfw:the"), a);
        assert_eq!(i.resolve(b), "mfw:of");

        let json = serde_json::to_string(&i).unwrap();
        let back: Interner = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get("mfw:of"), Some(b));
        assert_eq!(back.len(), 2);
    }

    #[test]
    fn builder_produces_sorted_entries() {
        let v = vec_of(&[(7, 1.0), (2, 3.0), (5, 2.0)]);
        assert_eq!(
            v.entries().iter().map(|e| e.0 .0).collect::<Vec<_>>(),
            vec![2, 5, 7]
        );
        assert_eq!(v.get(Symbol(5)), 2.0);
        assert_eq!(v.get(Symbol(6)), 0.0);
    }

    #[test]
    fn dense_projection_fills_gaps() {
        let v = vec_of(&[(1, 4.0), (3, 5.0)]);
        let dims = [Symbol(0), Symbol(1), Symbol(2), Symbol(3)];
        assert_eq!(v.to_dense(&dims), vec![0.0, 4.0, 0.0, 5.0]);
    }

    #[test]
    fn merge_walk_matches_hashmap_reference() {
        let a = vec_of(&[(1, 1.0), (3, 2.0), (9, 0.5)]);
        let b = vec_of(&[(2, 1.0), (3, 5.0), (9, 0.25)]);
        // Reference computation via dense projection over the union.
        let dims = [Symbol(1), Symbol(2), Symbol(3), Symbol(9)];
        let (da, db) = (a.to_dense(&dims), b.to_dense(&dims));
        let expect_l1: f64 = da.iter().zip(&db).map(|(x, y)| (x - y).abs()).sum();
        let expect_dot: f64 = da.iter().zip(&db).map(|(x, y)| x * y).sum();
        assert!((a.l1_distance(&b) - expect_l1).abs() < 1e-12);
        assert!((a.dot(&b) - expect_dot).abs() < 1e-12);
    }

    #[test]
    fn span_tracking_is_opt_in() {
        let mut b = VectorBuilder::new();
        b.add_at(Symbol(1), 1.0, Span::new(0, 3));
        assert!(!b.build().has_spans());

        let mut b = VectorBuilder::new().track_spans(true);
        b.add_at(Symbol(1), 1.0, Span::new(0, 3));
        b.add_at(Symbol(1), 1.0, Span::new(9, 12));
        let v = b.build();
        assert_eq!(v.get(Symbol(1)), 2.0);
        assert_eq!(v.spans(Symbol(1)), &[Span::new(0, 3), Span::new(9, 12)]);
    }
}
