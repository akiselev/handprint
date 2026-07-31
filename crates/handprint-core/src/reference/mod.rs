//! Phase 4 — the fitted reference, profiles, and comparison.
//!
//! A [`Reference`] is the whole model: the tokenizer policy, the interner, the
//! fitted features, the per-dimension corpus statistics, and (optionally) the
//! calibration. It is `serde`-serializable end to end, so the usual workflow is
//! *fit once, ship the reference, profile anywhere* — and a profile computed
//! next year is computed the same way as one computed today.

pub mod calibrate;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::compare::{Contribution, Decomposition, DistanceMetric, Metric, Space};
use crate::corpus::Corpus;
use crate::error::{Error, Result};
use crate::feature::{
    Confidence, DimInfo, Family, FeatureSpec, FitContext, FitDoc, Fitted, FittedFeature, Unit,
};
use crate::text::{ArtifactReport, Document, Span, Tokenizer};
use crate::util;
use crate::vector::{FeatureVector, Interner, Symbol, VectorBuilder};

pub use calibrate::{Assessment, Calibration, CalibrationConfig};

/// Per-dimension statistics over the fitting corpus.
///
/// The mean and standard deviation drive z-scoring; the quantiles are what a
/// critique finding's `target_band` is drawn from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimStats {
    /// Corpus mean.
    pub mean: f64,
    /// Corpus sample standard deviation.
    pub stddev: f64,
    /// The denominator z-scoring actually divides by: `max(stddev, floor)`,
    /// where the floor comes from the dimension's unit. See [`DimStats::z`].
    pub scale: f64,
    /// Smallest value seen.
    pub min: f64,
    /// Largest value seen.
    pub max: f64,
    /// 5th, 25th, 50th, 75th and 95th percentiles.
    pub quantiles: [f64; 5],
}

impl DimStats {
    /// The central interval used as a target band, by default the 5th–95th
    /// percentiles.
    pub fn band(&self, width: BandWidth) -> (f64, f64) {
        match width {
            BandWidth::P05P95 => (self.quantiles[0], self.quantiles[4]),
            BandWidth::P25P75 => (self.quantiles[1], self.quantiles[3]),
        }
    }

    /// Z-score of a raw value against this dimension.
    ///
    /// Divides by [`DimStats::scale`], not by the raw standard deviation. Two
    /// failure modes make that necessary, and they pull in opposite directions:
    ///
    /// * **Dividing by noise.** Rates and ratios are computed by division, so a
    ///   dimension that is constant in principle lands with a standard deviation
    ///   around 1e-16 rather than exactly zero. Dividing by that yields z-scores
    ///   around 1e13, and since every Delta-family metric sums absolute
    ///   z-differences, one such dimension silently becomes the whole distance.
    /// * **Discarding the strongest evidence.** Zeroing those dimensions instead
    ///   is worse. "This corpus never uses em dashes and this draft uses eight
    ///   per thousand words" is the single most discriminative thing a
    ///   zero-variance dimension can say, and treating it as no information
    ///   throws away exactly the signal that matters most for a distinctive
    ///   author.
    ///
    /// The floor resolves both: it is derived from the dimension's unit - a
    /// quarter of the smallest difference worth noticing in that unit - so an
    /// unused dimension reports a large but bounded z, and a genuinely constant
    /// one cannot explode.
    pub fn z(&self, value: f64) -> f64 {
        (value - self.mean) / self.scale
    }

    /// Whether the corpus showed no real variation in this dimension.
    ///
    /// Such a dimension still contributes to distances (see [`DimStats::z`]),
    /// but its percentile band collapses to a point, so the critique layer uses
    /// `[min, max]` as its target band instead.
    pub fn is_degenerate(&self) -> bool {
        self.stddev <= DEGENERATE_STDDEV_ABS + DEGENERATE_STDDEV_REL * self.mean.abs()
    }
}

/// Absolute floor below which a standard deviation is floating-point noise.
const DEGENERATE_STDDEV_ABS: f64 = 1e-12;
/// Standard deviation as a fraction of the mean, below which a dimension is
/// constant for every practical purpose.
const DEGENERATE_STDDEV_REL: f64 = 1e-9;

/// Which central interval a target band spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BandWidth {
    /// 5th to 95th percentile — the default; wide enough that ordinary variation
    /// does not produce findings.
    #[default]
    P05P95,
    /// 25th to 75th percentile — for tighter mimicry.
    P25P75,
}

/// Where a reference came from.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Name of the reference, e.g. `"hn-akiselev-2015-2022"`.
    pub name: String,
    /// Version of the artifact, e.g. `"2026.07"`.
    pub version: String,
    /// What the reference models: a specific author's corpus, a model's output,
    /// a background population.
    pub kind: String,
    /// Number of authors in the fitting corpus.
    pub authors: usize,
    /// Number of documents in the fitting corpus.
    pub documents: usize,
    /// Total lexical tokens in the fitting corpus.
    pub tokens: usize,
    /// Free-form notes: source datasets, licenses, cut-off dates, privacy
    /// attestations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Version of handprint that produced the artifact.
    pub handprint: String,
    /// Whether the fitting corpus was flagged private.
    #[serde(default)]
    pub private: bool,
    /// Every data pack this reference's fitted state embeds, with its license.
    ///
    /// A fitted feature that consumed a pack carries the pack's contents inside
    /// itself — it has to, because `transform` is pure and cannot go looking for
    /// a file. So the reference inherits the pack's terms, and a reference
    /// fitted from a non-redistributable table is itself non-redistributable.
    /// Recording it here is what lets a later reader find that out without
    /// re-deriving the pipeline.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<String>,
}

impl Provenance {
    /// Whether any embedded pack forbids redistribution.
    ///
    /// Reads the rollup strings rather than a separate flag so that an artifact
    /// written by an older build — which has no rollup at all — reports `false`
    /// rather than claiming a restriction it knows nothing about.
    pub fn has_non_redistributable_pack(&self) -> bool {
        self.licenses
            .iter()
            .any(|l| l.contains("NOT redistributable"))
    }
}

/// Graded length warnings, tied to Eder's (2015/2017) findings on how much text
/// authorship attribution actually needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthTier {
    /// Under 2,000 words: results are unreliable. Report them as indicative at
    /// best.
    Unreliable,
    /// 2,000–5,000 words: usable when the signal is clear, not otherwise.
    Marginal,
    /// 5,000 words and up: the range the literature supports.
    Supported,
}

impl LengthTier {
    /// Classify a document by its lexical token count.
    pub fn of(tokens: usize) -> LengthTier {
        match tokens {
            0..=1_999 => LengthTier::Unreliable,
            2_000..=4_999 => LengthTier::Marginal,
            _ => LengthTier::Supported,
        }
    }

    /// One-line explanation for a report.
    pub fn message(self) -> &'static str {
        match self {
            LengthTier::Unreliable => {
                "under 2,000 words: below the length at which whole-profile \
                 attribution is reliable (Eder 2015); per-family confidence is \
                 the number to read instead"
            }
            LengthTier::Marginal => {
                "2,000-5,000 words: usable only where the signal is already clear"
            }
            LengthTier::Supported => "5,000+ words: within the range the literature supports",
        }
    }
}

/// A fitted stylometric model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    provenance: Provenance,
    tokenizer: Tokenizer,
    interner: Interner,
    features: Vec<Fitted>,
    /// Every dimension, in feature-declaration order. This order is the dense
    /// vector layout and the rank order [`Metric::Eder`] weights by.
    dims: Vec<DimInfo>,
    stats: Vec<DimStats>,
    /// Per-dimension shift into the non-negative orthant; zero for every
    /// dimension that cannot go negative.
    shift: Vec<f64>,
    metric: Metric,
    band: BandWidth,
    /// Profiles of representative corpus documents.
    ///
    /// Stored so that a shipped reference is self-contained for corpus-mimic
    /// critique: the loop needs something document-shaped to measure against,
    /// and re-deriving it would mean shipping the corpus. Profiles are numeric,
    /// which is the part that is safe to redistribute.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    exemplars: Vec<Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calibration: Option<Calibration>,
}

impl Reference {
    /// Start building a pipeline.
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::default()
    }

    /// Where this reference came from.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// The tokenization policy this reference was fitted with.
    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    /// The dimension name table.
    pub fn interner(&self) -> &Interner {
        &self.interner
    }

    /// Every dimension, in dense-vector order.
    pub fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    /// Per-dimension corpus statistics, aligned with [`Reference::dims`].
    pub fn stats(&self) -> &[DimStats] {
        &self.stats
    }

    /// The fitted features.
    pub fn features(&self) -> &[Fitted] {
        &self.features
    }

    /// The default metric for this reference.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// The band width used for target intervals.
    pub fn band_width(&self) -> BandWidth {
        self.band
    }

    /// Profiles of representative corpus documents.
    ///
    /// These are what [`Critic::corpus_mimic`](crate::Critic::corpus_mimic)
    /// should be pointed at. Never a centroid: see that method's docs for why.
    pub fn exemplars(&self) -> &[Profile] {
        &self.exemplars
    }

    /// Replace the exemplar set.
    pub fn set_exemplars(&mut self, exemplars: Vec<Profile>) {
        self.exemplars = exemplars;
    }

    /// The calibration, if the reference carries one.
    pub fn calibration(&self) -> Option<&Calibration> {
        self.calibration.as_ref()
    }

    /// Attach or replace the calibration.
    pub fn set_calibration(&mut self, calibration: Calibration) {
        self.calibration = Some(calibration);
    }

    /// The name of a dimension.
    pub fn name_of(&self, symbol: Symbol) -> &str {
        self.interner.resolve(symbol)
    }

    /// Index of a dimension in the dense layout.
    pub fn index_of(&self, symbol: Symbol) -> Option<usize> {
        self.dims.iter().position(|d| d.symbol == symbol)
    }

    /// Look a dimension up by name.
    pub fn dim_by_name(&self, name: &str) -> Option<(usize, &DimInfo)> {
        let symbol = self.interner.get(name)?;
        let index = self.index_of(symbol)?;
        Some((index, &self.dims[index]))
    }

    /// A short fingerprint identifying this reference, used to reject
    /// cross-reference comparisons.
    pub fn fingerprint(&self) -> String {
        format!(
            "{}@{}/{}d",
            self.provenance.name,
            self.provenance.version,
            self.dims.len()
        )
    }

    /// The families present in this reference.
    pub fn families(&self) -> Vec<Family> {
        let mut families: Vec<Family> = self.dims.iter().map(|d| d.family).collect();
        families.sort_unstable();
        families.dedup();
        families
    }

    /// Profile a document.
    pub fn profile(&self, doc: &Document) -> Profile {
        self.profile_with(doc, false)
    }

    /// Profile a document, keeping span attribution.
    ///
    /// Span tracking roughly doubles the cost of a profile, so it is opt-in and
    /// only worth paying for when a report will point at the text.
    pub fn profile_tracked(&self, doc: &Document) -> Profile {
        self.profile_with(doc, true)
    }

    fn profile_with(&self, doc: &Document, track_spans: bool) -> Profile {
        let analysis = doc.analyze(&self.tokenizer);
        let mut builder = VectorBuilder::new().track_spans(track_spans);
        for feature in &self.features {
            feature.transform(&analysis, &mut builder);
        }
        let vector = builder.build();

        let symbols: Vec<Symbol> = self.dims.iter().map(|d| d.symbol).collect();
        let mut raw = vector.to_dense(&sorted(&symbols));
        // `to_dense` works on a sorted symbol list; re-order into declaration
        // order, which is the layout every metric and stat array assumes.
        raw = self.reorder_from_sorted(&raw);

        let missing: Vec<u32> = self
            .dims
            .iter()
            .enumerate()
            .filter(|(_, d)| vector.is_missing(d.symbol))
            .map(|(i, _)| i as u32)
            .collect();
        // A missing dimension is imputed to the corpus mean: the only choice
        // that cannot push a distance in either direction.
        for &i in &missing {
            raw[i as usize] = self.stats[i as usize].mean;
        }

        Profile {
            reference: self.fingerprint(),
            raw,
            missing,
            tokens: analysis.lexical_len(),
            doc_id: doc.id().map(str::to_owned),
            artifacts: analysis.artifacts().clone(),
            vector: track_spans.then_some(vector),
        }
    }

    fn reorder_from_sorted(&self, sorted_values: &[f64]) -> Vec<f64> {
        let mut symbols: Vec<(Symbol, usize)> = self
            .dims
            .iter()
            .enumerate()
            .map(|(i, d)| (d.symbol, i))
            .collect();
        symbols.sort_unstable_by_key(|(s, _)| *s);
        let mut out = vec![0.0; self.dims.len()];
        for (pos, (_, declared)) in symbols.iter().enumerate() {
            out[*declared] = sorted_values[pos];
        }
        out
    }

    /// Profile an author's documents as one aggregate.
    ///
    /// Individual short documents fall far below every length floor; the
    /// standard remedy is to profile their concatenation.
    pub fn profile_aggregate(&self, docs: &[Document]) -> Profile {
        let mut text = String::new();
        for doc in docs {
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(doc.text());
        }
        self.profile(&Document::new(text))
    }

    /// Project a profile into a metric's input space.
    pub fn vector_in(&self, profile: &Profile, space: Space) -> Vec<f64> {
        match space {
            Space::ZScore => (0..self.dims.len())
                .map(|i| self.stats[i].z(profile.raw[i]))
                .collect(),
            Space::NonNegative => (0..self.dims.len())
                .map(|i| (profile.raw[i] - self.shift[i]).max(0.0))
                .collect(),
        }
    }

    /// Compare two profiles with this reference's default metric.
    pub fn compare(&self, a: &Profile, b: &Profile) -> Result<Comparison> {
        self.compare_with(a, b, self.metric)
    }

    /// Compare two profiles with a specific metric.
    pub fn compare_with(&self, a: &Profile, b: &Profile, metric: Metric) -> Result<Comparison> {
        if a.reference != b.reference {
            return Err(Error::ReferenceMismatch {
                left: a.reference.clone(),
                right: b.reference.clone(),
            });
        }
        if a.reference != self.fingerprint() {
            return Err(Error::ReferenceMismatch {
                left: a.reference.clone(),
                right: self.fingerprint(),
            });
        }
        let space = metric.space();
        let va = self.vector_in(a, space);
        let vb = self.vector_in(b, space);
        let decomposition = metric.decompose(&va, &vb);
        let distance = decomposition.total();

        let assessment = self
            .calibration
            .as_ref()
            .filter(|c| c.metric() == metric)
            .map(|c| c.assess(distance, a.tokens, b.tokens));

        Ok(Comparison {
            distance,
            metric,
            tokens: (a.tokens, b.tokens),
            assessment,
            decomposition,
            raw_a: a.raw.clone(),
            raw_b: b.raw.clone(),
            z_a: self.vector_in(a, Space::ZScore),
            z_b: self.vector_in(b, Space::ZScore),
            dims: self.dims.clone(),
            names: self
                .dims
                .iter()
                .map(|d| self.name_of(d.symbol).to_owned())
                .collect(),
        })
    }

    /// Rank candidates by distance from a query, closest first.
    pub fn rank<'a>(
        &self,
        query: &Profile,
        candidates: impl IntoIterator<Item = (&'a str, &'a Profile)>,
    ) -> Result<Vec<Ranked>> {
        let mut out = Vec::new();
        for (label, candidate) in candidates {
            let comparison = self.compare(query, candidate)?;
            out.push(Ranked {
                label: label.to_owned(),
                distance: comparison.distance,
                assessment: comparison.assessment,
            });
        }
        out.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(out)
    }
}

fn sorted(symbols: &[Symbol]) -> Vec<Symbol> {
    let mut s = symbols.to_vec();
    s.sort_unstable();
    s
}

/// One entry of a [`Reference::rank`] result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ranked {
    /// Candidate label.
    pub label: String,
    /// Distance from the query.
    pub distance: f64,
    /// Calibrated assessment, when the reference carries a calibration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment: Option<Assessment>,
}

/// A document's measured style, relative to one reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    reference: String,
    raw: Vec<f64>,
    missing: Vec<u32>,
    tokens: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
    #[serde(default)]
    artifacts: ArtifactReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vector: Option<FeatureVector>,
}

impl Profile {
    /// Fingerprint of the reference this was measured against.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Raw, unscaled dimension values in the reference's declaration order.
    pub fn raw(&self) -> &[f64] {
        &self.raw
    }

    /// Lexical token count of the profiled document.
    pub fn tokens(&self) -> usize {
        self.tokens
    }

    /// The document's identifier, if it had one.
    pub fn doc_id(&self) -> Option<&str> {
        self.doc_id.as_deref()
    }

    /// Tokenizer attacks and generation residue found in the document.
    pub fn artifacts(&self) -> &ArtifactReport {
        &self.artifacts
    }

    /// Indices of dimensions that were imputed to the corpus mean because the
    /// feature could not compute them at this length.
    pub fn missing(&self) -> &[u32] {
        &self.missing
    }

    /// Spans that produced a dimension, when the profile was built with
    /// [`Reference::profile_tracked`].
    pub fn spans(&self, symbol: Symbol) -> &[Span] {
        self.vector.as_ref().map(|v| v.spans(symbol)).unwrap_or(&[])
    }

    /// The document's length tier.
    pub fn length_tier(&self) -> LengthTier {
        LengthTier::of(self.tokens)
    }

    /// Per-family confidence at this document's length.
    pub fn confidence(&self, reference: &Reference) -> Vec<(Family, Confidence)> {
        reference
            .families()
            .into_iter()
            .map(|f| (f, f.confidence(self.tokens)))
            .collect()
    }

    /// Human-readable warnings: length tier, imputed dimensions, artifacts.
    pub fn warnings(&self, reference: &Reference) -> Vec<String> {
        let mut out = vec![format!(
            "{} lexical tokens - {}",
            self.tokens,
            self.length_tier().message()
        )];
        if !self.missing.is_empty() {
            let names: Vec<&str> = self
                .missing
                .iter()
                .take(6)
                .map(|&i| reference.name_of(reference.dims[i as usize].symbol))
                .collect();
            out.push(format!(
                "{} dimension(s) undefined at this length and imputed to the corpus mean: {}{}",
                self.missing.len(),
                names.join(", "),
                if self.missing.len() > names.len() {
                    ", ..."
                } else {
                    ""
                }
            ));
        }
        for kind in self.artifacts.kinds() {
            let n = self.artifacts.of_kind(kind).count();
            out.push(format!(
                "{n} occurrence(s) of {} - scoring ran on the normalized text",
                kind.id()
            ));
        }
        out
    }
}

/// The result of comparing two profiles.
#[derive(Debug, Clone)]
pub struct Comparison {
    /// The distance.
    pub distance: f64,
    /// The metric used.
    pub metric: Metric,
    /// Lexical token counts of the two documents.
    pub tokens: (usize, usize),
    /// Calibrated assessment, when the reference carries a matching
    /// calibration.
    pub assessment: Option<Assessment>,
    decomposition: Decomposition,
    raw_a: Vec<f64>,
    raw_b: Vec<f64>,
    z_a: Vec<f64>,
    z_b: Vec<f64>,
    dims: Vec<DimInfo>,
    names: Vec<String>,
}

impl Comparison {
    /// Per-dimension contributions, largest magnitude first.
    ///
    /// The sum of `value` over every contribution, plus
    /// [`Comparison::offset`], is exactly [`Comparison::distance`].
    pub fn contributions(&self) -> Vec<Contribution> {
        let mut out: Vec<Contribution> = (0..self.dims.len())
            .map(|i| Contribution {
                symbol: self.dims[i].symbol,
                name: self.names[i].clone(),
                family: self.dims[i].family,
                unit: self.dims[i].unit,
                value: self.decomposition.contributions[i],
                observed_a: self.raw_a[i],
                observed_b: self.raw_b[i],
                z_a: self.z_a[i],
                z_b: self.z_b[i],
            })
            .collect();
        out.sort_by(|a, b| {
            b.magnitude()
                .partial_cmp(&a.magnitude())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    /// The metric's global offset — zero for everything but cosine, where it
    /// is 1.
    pub fn offset(&self) -> f64 {
        self.decomposition.offset
    }

    /// Contributions aggregated by family, largest magnitude first.
    pub fn by_family(&self) -> Vec<(Family, f64)> {
        let mut totals: HashMap<Family, f64> = HashMap::new();
        for (i, dim) in self.dims.iter().enumerate() {
            *totals.entry(dim.family).or_insert(0.0) += self.decomposition.contributions[i];
        }
        let mut out: Vec<(Family, f64)> = totals.into_iter().collect();
        out.sort_by(|a, b| {
            b.1.abs()
                .partial_cmp(&a.1.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }
}

/// Builds a [`Reference`].
#[derive(Debug, Clone, Default)]
pub struct PipelineBuilder {
    features: Vec<FeatureSpec>,
    tokenizer: Tokenizer,
    metric: Metric,
    band: BandWidth,
    name: String,
    version: String,
    kind: String,
    notes: Vec<String>,
    exemplars: Option<usize>,
}

impl PipelineBuilder {
    /// Add a feature.
    pub fn feature(mut self, feature: impl Into<FeatureSpec>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Add several features.
    pub fn features(mut self, features: impl IntoIterator<Item = FeatureSpec>) -> Self {
        self.features.extend(features);
        self
    }

    /// Set the tokenization policy. It is serialized with the reference.
    pub fn tokenizer(mut self, tokenizer: Tokenizer) -> Self {
        self.tokenizer = tokenizer;
        self
    }

    /// Set the default metric.
    pub fn metric(mut self, metric: Metric) -> Self {
        self.metric = metric;
        self
    }

    /// Set the target-band width used by critique findings.
    pub fn band(mut self, band: BandWidth) -> Self {
        self.band = band;
        self
    }

    /// Name the reference.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Version the reference artifact.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Describe what the reference models (`"corpus"`, `"model"`,
    /// `"background"`).
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    /// Record a provenance note: source dataset, license, cut-off date.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// How many representative document profiles to store in the artifact.
    ///
    /// Defaults to 16. Set to `0` to store none, in which case a caller doing
    /// corpus-mimic critique must supply its own targets.
    pub fn exemplars(mut self, count: usize) -> Self {
        self.exemplars = Some(count);
        self
    }

    /// A pipeline that works on short documents: punctuation, sentence
    /// structure and the seed lexicon, with no vocabulary-hungry families.
    pub fn short_text_defaults() -> Self {
        use crate::feature::{LexiconFeature, PunctTypography, SentenceStats};
        PipelineBuilder::default()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(LexiconFeature::default())
    }

    /// Fit against a corpus.
    pub fn fit(self, corpus: &Corpus) -> Result<Reference> {
        if self.features.is_empty() {
            return Err(Error::EmptyPipeline);
        }

        // Analyze every document once, for every feature.
        let docs: Vec<(usize, &Document)> = corpus
            .authors()
            .iter()
            .enumerate()
            .flat_map(|(ai, a)| a.docs.iter().map(move |d| (ai, d)))
            .collect();
        let fit_docs: Vec<FitDoc<'_>> = docs
            .iter()
            .map(|(author, doc)| FitDoc {
                author: *author,
                analysis: doc.analyze(&self.tokenizer),
            })
            .collect();
        let ctx = FitContext::new(corpus, &fit_docs);

        let mut interner = Interner::new();
        let mut features = Vec::with_capacity(self.features.len());
        let mut dims: Vec<DimInfo> = Vec::new();
        for spec in &self.features {
            let fitted = spec.fit(&ctx, &mut interner)?;
            dims.extend_from_slice(fitted.dims());
            features.push(fitted);
        }
        if dims.is_empty() {
            return Err(Error::EmptyPipeline);
        }
        // Two features claiming the same dimension name would intern to one
        // symbol and land twice in the dense layout, so every profile after
        // this point would silently read one of them out of the other's slot.
        // The usual cause is declaring a family twice with different flags.
        {
            let mut seen: Vec<Symbol> = dims.iter().map(|d| d.symbol).collect();
            seen.sort_unstable();
            let before = seen.len();
            seen.dedup();
            if seen.len() != before {
                let mut names: Vec<&str> = Vec::new();
                let mut counts: HashMap<Symbol, usize> = HashMap::new();
                for dim in &dims {
                    *counts.entry(dim.symbol).or_insert(0) += 1;
                }
                for dim in &dims {
                    if counts[&dim.symbol] > 1 {
                        let name = interner.resolve(dim.symbol);
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                }
                names.sort_unstable();
                names.truncate(6);
                return Err(Error::InvalidConfig {
                    what: "Pipeline features",
                    detail: format!(
                        "two features emit the same dimension(s): {}. Declaring one family \
                         twice — usually the same family with different flags — makes the \
                         dense layout ambiguous. Configure it once.",
                        names.join(", ")
                    ),
                });
            }
        }

        // Transform the corpus once more to collect per-dimension statistics.
        // This second pass is what makes `transform` pure: everything
        // corpus-relative is decided here and frozen.
        let mut columns: Vec<Vec<f64>> = vec![Vec::with_capacity(fit_docs.len()); dims.len()];
        let symbols: Vec<Symbol> = dims.iter().map(|d| d.symbol).collect();
        let sorted_symbols = sorted(&symbols);
        let mut order: Vec<usize> = (0..dims.len()).collect();
        order.sort_unstable_by_key(|&i| symbols[i]);

        for fd in &fit_docs {
            let mut builder = VectorBuilder::new();
            for feature in &features {
                feature.transform(&fd.analysis, &mut builder);
            }
            let vector = builder.build();
            let dense = vector.to_dense(&sorted_symbols);
            for (pos, &declared) in order.iter().enumerate() {
                if !vector.is_missing(symbols[declared]) {
                    columns[declared].push(dense[pos]);
                }
            }
        }

        let stats: Vec<DimStats> = columns
            .iter_mut()
            .zip(&dims)
            .map(|(column, dim)| summarize(column, dim.unit))
            .collect();
        let shift: Vec<f64> = stats.iter().map(|s| s.min.min(0.0)).collect();

        let exemplar_count = self.exemplars.unwrap_or(DEFAULT_EXEMPLARS);

        // Roll up the license of every data pack the fitted features embed, so
        // the artifact carries its own redistribution terms.
        let mut licenses: Vec<String> = features
            .iter()
            .flat_map(|f| f.pack_licenses())
            .map(|l| l.describe())
            .collect();
        licenses.sort();
        licenses.dedup();

        let mut reference = Reference {
            provenance: Provenance {
                name: if self.name.is_empty() {
                    "unnamed".into()
                } else {
                    self.name
                },
                version: if self.version.is_empty() {
                    "0".into()
                } else {
                    self.version
                },
                kind: if self.kind.is_empty() {
                    "corpus".into()
                } else {
                    self.kind
                },
                authors: corpus.author_count(),
                documents: corpus.len(),
                tokens: ctx.total_tokens(),
                notes: self.notes,
                handprint: crate::VERSION.to_owned(),
                private: corpus.is_private(),
                licenses,
            },
            tokenizer: self.tokenizer,
            interner,
            features,
            dims,
            stats,
            shift,
            metric: self.metric,
            band: self.band,
            exemplars: Vec::new(),
            calibration: None,
        };

        // Sample exemplars evenly across the corpus so they span its range
        // rather than clustering in whichever author happens to come first.
        if exemplar_count > 0 && !docs.is_empty() {
            let step = (docs.len() as f64 / exemplar_count as f64).max(1.0);
            let mut exemplars = Vec::new();
            let mut at = 0.0f64;
            while (at as usize) < docs.len() && exemplars.len() < exemplar_count {
                exemplars.push(reference.profile(docs[at as usize].1));
                at += step;
            }
            reference.exemplars = exemplars;
        }
        Ok(reference)
    }
}

/// Representative document profiles stored in a reference by default.
pub const DEFAULT_EXEMPLARS: usize = 16;

/// Alias kept because `Pipeline::builder()` reads better than
/// `Reference::builder()` at a call site that is describing a pipeline.
pub struct Pipeline;

impl Pipeline {
    /// Start building a pipeline.
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::default()
    }
}

/// The floor under a dimension's z-scoring denominator: the smallest difference
/// worth noticing in that unit.
fn stddev_floor(unit: Unit) -> f64 {
    unit.noise_floor()
}

fn summarize(values: &mut [f64], unit: Unit) -> DimStats {
    let floor = stddev_floor(unit);
    if values.is_empty() {
        return DimStats {
            mean: 0.0,
            stddev: 0.0,
            scale: floor,
            min: 0.0,
            max: 0.0,
            quantiles: [0.0; 5],
        };
    }
    let mean = util::mean(values);
    let stddev = util::stddev(values);
    util::sort_floats(values);
    let q = |p: f64| util::quantile_sorted(values, p).unwrap_or(0.0);
    DimStats {
        mean,
        stddev,
        scale: stddev.max(floor),
        min: values[0],
        max: values[values.len() - 1],
        quantiles: [q(0.05), q(0.25), q(0.50), q(0.75), q(0.95)],
    }
}

/// Convenience re-export so `Unit` is reachable from the reference module.
pub use crate::feature::Unit as DimUnit;
const _: () = {
    // Keep `Unit` referenced so the re-export cannot drift out of sync.
    let _ = Unit::PerThousandTokens;
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{MostFrequentWords, PunctTypography, SentenceStats};

    fn corpus() -> Corpus {
        let alice: Vec<Document> = (0..6)
            .map(|i| {
                Document::new(format!(
                    "Alice writes plainly. She uses short sentences, mostly. \
                     Document {i} is one of hers, and it reads the same as the rest. \
                     No dashes here at all. Just commas, and periods."
                ))
            })
            .collect();
        let bob: Vec<Document> = (0..6)
            .map(|i| {
                Document::new(format!(
                    "Bob\u{2014}who writes very differently\u{2014}piles on the dashes!!! \
                     Text number {i} shows it. He also uses \u{201C}curly quotes\u{201D} \
                     everywhere\u{2014}always\u{2014}and long rambling sentences that keep \
                     going well past the point where a reasonable person would have stopped."
                ))
            })
            .collect();
        Corpus::new().with("alice", alice).with("bob", bob)
    }

    fn reference() -> Reference {
        Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(MostFrequentWords::default().top(30))
            .name("test")
            .version("1")
            .fit(&corpus())
            .unwrap()
    }

    #[test]
    fn fit_produces_named_dimensions_and_stats() {
        let r = reference();
        assert!(r.dims().len() > 100);
        assert_eq!(r.stats().len(), r.dims().len());
        assert!(r.dim_by_name("punct:em_dash_rate").is_some());
        assert_eq!(r.provenance().authors, 2);
        assert_eq!(r.provenance().documents, 12);
        assert!(r.provenance().tokens > 100);
    }

    #[test]
    fn same_author_documents_are_closer_than_cross_author_ones() {
        let r = reference();
        let c = corpus();
        let a1 = r.profile(&c.author(&"alice".into()).unwrap()[0]);
        let a2 = r.profile(&c.author(&"alice".into()).unwrap()[1]);
        let b1 = r.profile(&c.author(&"bob".into()).unwrap()[0]);
        let within = r.compare(&a1, &a2).unwrap().distance;
        let across = r.compare(&a1, &b1).unwrap().distance;
        assert!(within < across, "within={within} across={across}");
    }

    #[test]
    fn contributions_sum_to_the_distance_for_every_metric() {
        let r = reference();
        let c = corpus();
        let a = r.profile(&c.author(&"alice".into()).unwrap()[0]);
        let b = r.profile(&c.author(&"bob".into()).unwrap()[0]);
        for metric in Metric::ALL {
            let cmp = r.compare_with(&a, &b, *metric).unwrap();
            let sum: f64 = cmp.contributions().iter().map(|c| c.value).sum();
            assert!(
                (sum + cmp.offset() - cmp.distance).abs() < 1e-9,
                "{metric}: {sum} + {} != {}",
                cmp.offset(),
                cmp.distance
            );
        }
    }

    #[test]
    fn em_dash_rate_dominates_the_explanation() {
        let r = reference();
        let c = corpus();
        let a = r.profile(&c.author(&"alice".into()).unwrap()[0]);
        let b = r.profile(&c.author(&"bob".into()).unwrap()[0]);
        let cmp = r.compare_with(&a, &b, Metric::BurrowsDelta).unwrap();
        let contributions = cmp.contributions();
        let top: Vec<&str> = contributions
            .iter()
            .take(12)
            .map(|c| c.name.as_str())
            .collect();
        assert!(
            top.iter().any(|n| n.contains("em_dash")),
            "expected an em-dash dimension in the top contributions, got {top:?}"
        );
    }

    #[test]
    fn family_rollup_covers_the_distance() {
        let r = reference();
        let c = corpus();
        let a = r.profile(&c.author(&"alice".into()).unwrap()[0]);
        let b = r.profile(&c.author(&"bob".into()).unwrap()[0]);
        let cmp = r.compare(&a, &b).unwrap();
        let total: f64 = cmp.by_family().iter().map(|(_, v)| v).sum();
        assert!((total + cmp.offset() - cmp.distance).abs() < 1e-9);
    }

    #[test]
    fn reference_round_trips_through_serde_with_identical_profiles() {
        let r = reference();
        let json = serde_json::to_string(&r).unwrap();
        let back: Reference = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fingerprint(), r.fingerprint());

        let doc = Document::new("A fresh document — with a dash — to profile twice.");
        let p1 = r.profile(&doc);
        let p2 = back.profile(&doc);
        assert_eq!(p1.raw(), p2.raw());
        assert_eq!(p1.tokens(), p2.tokens());
    }

    #[test]
    fn profiles_from_different_references_cannot_be_compared() {
        let r1 = reference();
        let r2 = Pipeline::builder()
            .feature(PunctTypography::default())
            .name("other")
            .fit(&corpus())
            .unwrap();
        let doc = Document::new("Some text to profile.");
        let err = r1
            .compare(&r1.profile(&doc), &r2.profile(&doc))
            .unwrap_err();
        assert!(matches!(err, Error::ReferenceMismatch { .. }));
    }

    #[test]
    fn missing_dimensions_are_imputed_to_the_corpus_mean() {
        let r = reference();
        // Two sentences is below the dispersion floor.
        let p = r.profile(&Document::new("Short. Very short."));
        assert!(!p.missing().is_empty());
        let (idx, _) = r.dim_by_name("sent:len_variance").unwrap();
        assert!(p.missing().contains(&(idx as u32)));
        assert_eq!(p.raw()[idx], r.stats()[idx].mean);
        // Imputation to the mean means a z-score of exactly zero.
        assert_eq!(r.vector_in(&p, Space::ZScore)[idx], 0.0);
        assert!(p.warnings(&r).iter().any(|w| w.contains("imputed")));
    }

    #[test]
    fn minmax_never_sees_a_negative_value() {
        let r = reference();
        let p = r.profile(&Document::new("Anything at all, really — some text."));
        for v in r.vector_in(&p, Space::NonNegative) {
            assert!(v >= 0.0, "{v}");
        }
    }

    #[test]
    fn zero_variance_dimensions_still_carry_evidence() {
        let r = reference();
        let (idx, dim) = r.dim_by_name("punct:em_dash_rate").unwrap();
        let stats = &r.stats()[idx];
        // Whatever the fixture's spread, the z denominator is never below the
        // unit floor and never zero, so z stays finite and informative.
        assert!(stats.scale >= super::stddev_floor(dim.unit));
        assert!(stats.scale > 0.0);

        let plain = r.profile(&Document::new("No dashes here at all, just commas."));
        let dashy = r.profile(&Document::new(
            "Lots\u{2014}of\u{2014}dashes\u{2014}here\u{2014}now.",
        ));
        let z_plain = r.vector_in(&plain, Space::ZScore)[idx];
        let z_dashy = r.vector_in(&dashy, Space::ZScore)[idx];
        assert!(
            z_dashy > z_plain + 1.0,
            "a heavily dashed draft must score far from a plain one: {z_plain} vs {z_dashy}"
        );
        assert!(z_dashy.is_finite() && z_dashy.abs() < 1e6, "{z_dashy}");
    }

    #[test]
    fn length_tiers_are_graded_not_a_cliff() {
        assert_eq!(LengthTier::of(500), LengthTier::Unreliable);
        assert_eq!(LengthTier::of(3_000), LengthTier::Marginal);
        assert_eq!(LengthTier::of(9_000), LengthTier::Supported);
    }

    #[test]
    fn per_family_confidence_beats_one_document_level_number() {
        let r = reference();
        let text = "word ".repeat(400);
        let p = r.profile(&Document::new(text));
        let confidence = p.confidence(&r);
        let get = |f: Family| confidence.iter().find(|(x, _)| *x == f).unwrap().1;
        assert_eq!(get(Family::Punct), Confidence::Ok);
        assert_eq!(get(Family::Mfw), Confidence::None);
    }

    #[test]
    fn ranking_orders_by_distance() {
        let r = reference();
        let c = corpus();
        let query = r.profile(&c.author(&"alice".into()).unwrap()[0]);
        let alice = r.profile(&c.author(&"alice".into()).unwrap()[2]);
        let bob = r.profile(&c.author(&"bob".into()).unwrap()[0]);
        let ranked = r.rank(&query, [("bob", &bob), ("alice", &alice)]).unwrap();
        assert_eq!(ranked[0].label, "alice");
    }

    #[test]
    fn references_carry_exemplar_profiles() {
        let r = reference();
        assert!(!r.exemplars().is_empty());
        assert!(r.exemplars().len() <= super::DEFAULT_EXEMPLARS);
        for exemplar in r.exemplars() {
            assert_eq!(exemplar.reference(), r.fingerprint());
            assert!(exemplar.tokens() > 0);
        }
        // Exemplars survive serialization, so a shipped artifact is
        // self-contained for corpus-mimic critique.
        let json = serde_json::to_string(&r).unwrap();
        let back: Reference = serde_json::from_str(&json).unwrap();
        assert_eq!(back.exemplars().len(), r.exemplars().len());

        let none = Pipeline::builder()
            .feature(PunctTypography::default())
            .exemplars(0)
            .name("bare")
            .fit(&corpus())
            .unwrap();
        assert!(none.exemplars().is_empty());
    }

    #[test]
    fn empty_pipeline_is_rejected() {
        assert!(matches!(
            Pipeline::builder().fit(&corpus()).unwrap_err(),
            Error::EmptyPipeline
        ));
    }

    #[test]
    fn spans_survive_into_the_profile_when_tracked() {
        let r = reference();
        let text = "Additionally, this is a test.";
        let p = r.profile_tracked(&Document::new(text));
        let sym = r.interner().get("sent:opener:additionally").unwrap();
        assert_eq!(p.spans(sym).len(), 1);
        assert!(r.profile(&Document::new(text)).spans(sym).is_empty());
    }
}
