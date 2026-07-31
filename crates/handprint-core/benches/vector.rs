//! Microbenchmarks for the sparse vector layer (phase 2 acceptance) and for
//! whole-document profiling (phase 11 throughput target).
//!
//! The question phase 2 asks is whether sorted-and-merged sparse vectors beat a
//! hashmap for the pairwise operations calibration does tens of thousands of
//! times. `merge_vs_hashmap` answers it directly.

#![allow(missing_docs)] // criterion_group! generates undocumented items

use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use handprint_core::feature::{CharNgrams, MostFrequentWords, PunctTypography, SentenceStats};
use handprint_core::vector::{FeatureVector, Symbol, VectorBuilder};
use handprint_core::{Corpus, Document, Pipeline};

fn sparse(dims: usize, density: f64, seed: u64) -> FeatureVector {
    let mut state = seed | 1;
    let mut builder = VectorBuilder::new();
    for i in 0..dims {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        if ((state >> 33) % 1000) as f64 / 1000.0 < density {
            builder.set(Symbol(i as u32), ((state >> 20) % 1000) as f64 / 1000.0);
        }
    }
    builder.build()
}

/// The hashmap implementation the merge walk replaces.
fn hashmap_l1(a: &FeatureVector, b: &FeatureVector) -> f64 {
    let map: HashMap<Symbol, f64> = a.entries().iter().copied().collect();
    let other: HashMap<Symbol, f64> = b.entries().iter().copied().collect();
    let mut total = 0.0;
    for (symbol, value) in &map {
        total += (value - other.get(symbol).copied().unwrap_or(0.0)).abs();
    }
    for (symbol, value) in &other {
        if !map.contains_key(symbol) {
            total += value.abs();
        }
    }
    total
}

fn merge_vs_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("l1_distance");
    for dims in [1_000usize, 10_000] {
        let a = sparse(dims, 0.2, 1);
        let b = sparse(dims, 0.2, 2);
        // Both implementations must agree before either number means anything.
        assert!((a.l1_distance(&b) - hashmap_l1(&a, &b)).abs() < 1e-9);

        group.throughput(Throughput::Elements(dims as u64));
        group.bench_with_input(BenchmarkId::new("merge", dims), &dims, |bencher, _| {
            bencher.iter(|| a.l1_distance(&b))
        });
        group.bench_with_input(BenchmarkId::new("hashmap", dims), &dims, |bencher, _| {
            bencher.iter(|| hashmap_l1(&a, &b))
        });
    }
    group.finish();
}

fn dense_projection(c: &mut Criterion) {
    let v = sparse(10_000, 0.2, 3);
    let dims: Vec<Symbol> = (0..10_000).map(|i| Symbol(i as u32)).collect();
    c.bench_function("to_dense/10k", |bencher| bencher.iter(|| v.to_dense(&dims)));
}

fn profile_throughput(c: &mut Criterion) {
    let paragraph = "I looked at the failing test again and the problem is in the tokenizer: \
                     it treats the apostrophe as a word boundary, so contractions split in two. \
                     The fix is to fold the curly apostrophe first, which keeps the span mapping \
                     intact. I also updated the snapshot, since the expected output changes for \
                     three of the fixtures and the old ones would fail.\n\n";
    let mut corpus = Corpus::new();
    for i in 0..40 {
        corpus.add(format!("a{i}"), [Document::new(paragraph.repeat(4))]);
    }
    let reference = Pipeline::builder()
        .feature(PunctTypography::default())
        .feature(SentenceStats::default())
        .feature(MostFrequentWords::default().top(500))
        .feature(CharNgrams::new(3..=4).top(1000))
        .name("bench")
        .fit(&corpus)
        .expect("fit");

    let document = Document::new(paragraph.repeat(200));
    let bytes = document.text().len() as u64;
    let mut group = c.benchmark_group("profile");
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("all_families", |bencher| {
        bencher.iter(|| reference.profile(&document))
    });
    group.finish();
}

criterion_group!(
    benches,
    merge_vs_hashmap,
    dense_projection,
    profile_throughput,
    per_family
);
criterion_main!(benches);

/// Per-family throughput, to locate the bottleneck rather than guess at it.
fn per_family(c: &mut Criterion) {
    let paragraph = "I looked at the failing test again and the problem is in the tokenizer: \
                     it treats the apostrophe as a word boundary, so contractions split in two. \
                     The fix is to fold the curly apostrophe first, which keeps the span mapping \
                     intact.\n\n";
    let mut corpus = Corpus::new();
    for i in 0..20 {
        corpus.add(format!("a{i}"), [Document::new(paragraph.repeat(4))]);
    }
    let document = Document::new(paragraph.repeat(200));
    let bytes = document.text().len() as u64;

    let mut group = c.benchmark_group("family");
    group.throughput(Throughput::Bytes(bytes));
    let cases: Vec<(&str, handprint_core::FeatureSpec)> = vec![
        ("punct", PunctTypography::default().into()),
        ("sentence", SentenceStats::default().into()),
        ("mfw", MostFrequentWords::default().top(500).into()),
        ("ngrams", CharNgrams::new(3..=4).top(1000).into()),
    ];
    for (name, spec) in cases {
        let reference = Pipeline::builder()
            .feature(spec)
            .name("bench")
            .exemplars(0)
            .fit(&corpus)
            .expect("fit");
        group.bench_function(name, |bencher| {
            bencher.iter(|| reference.profile(&document))
        });
    }
    // Tokenization alone, as the floor everything else builds on.
    group.bench_function("analyze_only", |bencher| {
        bencher.iter(|| document.analyze(&handprint_core::Tokenizer::default()))
    });
    group.finish();
}
