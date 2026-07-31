//! Comparison frames — the Adams signature, and the fiction-signature detector.
//!
//! A *comparison frame* is the syntactic scaffolding of a simile: `as X as Y`,
//! `like a Y`, `in much the same way that Y`. The frames themselves are
//! pattern-grade — Niculae & Danescu-Niculescu-Mizil (EMNLP 2014) treat
//! figurative-comparison detection the same way — and what makes them a *style*
//! feature is not the rate of similes but what happens inside the vehicle:
//!
//! * **Negated vehicle** — "…in much the same way that bricks *don't*". A
//!   negator inside the vehicle clause turns a comparison into a joke. This is
//!   the single most recognizable Adams construction and, as far as the
//!   literature goes, nobody has counted it.
//! * **Ironic hedge** — "about as X as Y". Veale & Hao identify the `about as`
//!   hedge as an irony marker: the writer is signalling that the comparison is
//!   not to be taken at face value.
//! * **Frozen share** — what fraction of an author's similes are dead ones
//!   ("as good as gold"). A creative-simile author has a low frozen share; the
//!   Dark & Stormy over-firing failure mode is an LLM producing a *high* novel
//!   share, above the human band, which is exactly what a two-sided band on
//!   this dimension catches.
//! * **Vehicle length** — a one-word vehicle and a clause-long one are
//!   different devices.
//!
//! # What this does not do
//!
//! No tenor↔vehicle semantic distance. That needs embeddings, which would break
//! the data-only constraint; corpus PMI is a weak proxy and is not attempted
//! here. No metaphor detection without a frame — an unframed metaphor is not
//! findable with patterns, and pretending otherwise would produce a rate that
//! measures the pattern list rather than the author.

use serde::{Deserialize, Serialize};

use super::{
    per_1k, ratio, DimInfo, Family, Feature, FitContext, FittedFeature, PackLicense, Unit,
};
use crate::error::Result;
use crate::feature::lexicon::LexiconPack;
use crate::text::{Analysis, Span};
use crate::util;
use crate::vector::{Interner, Symbol, VectorBuilder};

/// Negators that make a vehicle a joke rather than a comparison.
const NEGATORS: &[&str] = &[
    "not",
    "n't",
    "'t",
    "never",
    "no",
    "nothing",
    "nobody",
    "none",
    "nor",
    "neither",
    "hardly",
    "barely",
    "scarcely",
    "don't",
    "doesn't",
    "didn't",
    "isn't",
    "aren't",
    "wasn't",
    "weren't",
    "can't",
    "cannot",
    "won't",
    "wouldn't",
    "couldn't",
    "shouldn't",
    "hasn't",
    "haven't",
    "hadn't",
    "ain't",
];

/// Determiners that can open a `like` vehicle. Requiring one is what keeps the
/// verb `like` ("I like it") and the discourse particle ("it was, like, fine")
/// out of the count.
const VEHICLE_DETERMINERS: &[&str] = &[
    "a", "an", "the", "some", "any", "every", "one", "two", "three", "his", "her", "its", "their",
    "our", "my", "your", "that", "this", "those", "these",
];

/// Clause-final punctuation that ends a vehicle.
const VEHICLE_STOPS: &[&str] = &[".", "!", "?", ";", ":", "\u{2014}", "\u{2013}"];

/// Longest vehicle counted, in lexical tokens. Beyond this the "vehicle" is the
/// rest of the paragraph and the length statistic stops describing the device.
const VEHICLE_WINDOW: usize = 10;

/// Which frame matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameKind {
    /// `as X as Y`.
    AsAs,
    /// `about as X as Y` — Veale & Hao's irony hedge.
    IronicAsAs,
    /// `like a/the Y`.
    LikeA,
    /// `(in) much the same way (that) Y`.
    SameWay,
}

/// One extracted comparison.
#[derive(Debug, Clone)]
struct Frame {
    kind: FrameKind,
    /// Source span covering frame and vehicle.
    span: Span,
    /// Lexical tokens of the vehicle.
    vehicle_len: usize,
    /// Whether a negator appears inside the vehicle.
    negated: bool,
    /// Normalized text of the whole frame, for the frozen-simile lookup.
    normalized: String,
}

/// Configuration for the comparison-frame family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ComparisonFrames {
    /// A pack of frozen (dead) similes, used for `frame:frozen_share`.
    ///
    /// Without one the share is marked missing rather than reported as zero:
    /// "none of this author's similes are clichés" and "we did not check" are
    /// different claims, and only one of them is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_pack: Option<LexiconPack>,
}

impl ComparisonFrames {
    /// Use a frozen-simile pack for the cliché share.
    pub fn with_frozen(mut self, pack: LexiconPack) -> Self {
        self.frozen_pack = Some(pack);
        self
    }
}

/// Fitted [`ComparisonFrames`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedFrames {
    dims: Vec<DimInfo>,
    simile: Symbol,
    negated: Symbol,
    ironic: Symbol,
    frozen_share: Symbol,
    vehicle_len: Symbol,
    /// Lowercased literal frozen similes. Empty when no pack was supplied.
    frozen: Vec<String>,
    frozen_pack: Option<(String, String, bool)>,
}

impl FittedFrames {
    /// The family this feature belongs to.
    pub const FAMILY: Family = Family::Device;

    /// How many frozen similes this was fitted with.
    pub fn frozen_len(&self) -> usize {
        self.frozen.len()
    }
}

impl Feature for ComparisonFrames {
    type Fitted = FittedFrames;

    fn fit(&self, _ctx: &FitContext<'_>, interner: &mut Interner) -> Result<FittedFrames> {
        let mut dims = Vec::new();
        let mut push = |interner: &mut Interner, name: &str, unit: Unit| {
            let sym = interner.intern(name);
            dims.push(DimInfo::new(sym, Family::Device, unit));
            sym
        };
        let simile = push(interner, "frame:simile_rate", Unit::PerThousandTokens);
        let negated = push(
            interner,
            "frame:negated_vehicle_rate",
            Unit::PerThousandTokens,
        );
        let ironic = push(interner, "frame:ironic_hedge_rate", Unit::PerThousandTokens);
        let frozen_share = push(interner, "frame:frozen_share", Unit::Fraction);
        let vehicle_len = push(interner, "frame:vehicle_len_mean", Unit::Tokens);

        let mut frozen: Vec<String> = Vec::new();
        if let Some(pack) = &self.frozen_pack {
            for phrase in &pack.phrases {
                // Only literal patterns: a wildcard has no fixed surface form
                // to compare an extracted frame against.
                if !phrase.pattern.contains(['*', '{']) {
                    frozen.push(phrase.pattern.to_lowercase());
                }
            }
            for term in &pack.terms {
                frozen.push(term.word.to_lowercase());
            }
            frozen.sort();
            frozen.dedup();
        }
        let frozen_pack = self
            .frozen_pack
            .as_ref()
            .map(|p| (p.qualified_name(), p.license.clone(), p.redistributable));

        Ok(FittedFrames {
            dims,
            simile,
            negated,
            ironic,
            frozen_share,
            vehicle_len,
            frozen,
            frozen_pack,
        })
    }
}

impl FittedFeature for FittedFrames {
    fn dims(&self) -> &[DimInfo] {
        &self.dims
    }

    fn family(&self) -> Family {
        Family::Device
    }

    fn pack_licenses(&self) -> Vec<PackLicense> {
        self.frozen_pack
            .iter()
            .map(|(pack, license, redistributable)| PackLicense {
                pack: pack.clone(),
                license: license.clone(),
                redistributable: *redistributable,
            })
            .collect()
    }

    fn transform(&self, analysis: &Analysis<'_>, out: &mut VectorBuilder) {
        let tokens = analysis.lexical_len();
        if tokens == 0 {
            for dim in &self.dims {
                out.mark_missing(dim.symbol);
            }
            return;
        }
        let frames = extract_frames(analysis);

        let mut negated = 0usize;
        let mut ironic = 0usize;
        let mut frozen = 0usize;
        let mut vehicle_lengths: Vec<f64> = Vec::new();
        for frame in &frames {
            out.note_span(self.simile, frame.span);
            vehicle_lengths.push(frame.vehicle_len as f64);
            if frame.negated {
                negated += 1;
                out.note_span(self.negated, frame.span);
            }
            if frame.kind == FrameKind::IronicAsAs {
                ironic += 1;
                out.note_span(self.ironic, frame.span);
            }
            if self.is_frozen(&frame.normalized) {
                frozen += 1;
                out.note_span(self.frozen_share, frame.span);
            }
        }

        out.set(self.simile, per_1k(frames.len(), tokens));
        out.set(self.negated, per_1k(negated, tokens));
        out.set(self.ironic, per_1k(ironic, tokens));

        if frames.is_empty() {
            // No frames is a real observation about the rates above, and no
            // observation at all about the *shape* of frames this author writes.
            out.mark_missing(self.vehicle_len);
        } else {
            out.set(self.vehicle_len, util::mean(&vehicle_lengths));
        }
        if self.frozen.is_empty() || frames.is_empty() {
            out.mark_missing(self.frozen_share);
        } else {
            out.set(self.frozen_share, ratio(frozen, frames.len()));
        }
    }
}

impl FittedFrames {
    /// Whether an extracted frame's text opens with a known dead simile.
    fn is_frozen(&self, normalized: &str) -> bool {
        self.frozen
            .iter()
            .any(|f| normalized == f || normalized.starts_with(&format!("{f} ")))
    }
}

/// Extract every comparison frame in a document.
fn extract_frames(analysis: &Analysis<'_>) -> Vec<Frame> {
    let stream = analysis.tokens();
    let all: Vec<(&str, Span, bool)> = stream
        .iter()
        .map(|(t, f)| (f, t.span, t.kind.is_lexical()))
        .collect();
    // Index of each token in the lexical-only sequence, for vehicle lengths.
    let mut out: Vec<Frame> = Vec::new();
    let mut i = 0usize;
    while i < all.len() {
        if let Some(frame) = frame_at(&all, i) {
            i = frame.1;
            out.push(frame.0);
            continue;
        }
        i += 1;
    }
    out
}

/// Try every frame pattern at one token index, returning the frame and the
/// index to resume scanning from.
fn frame_at(all: &[(&str, Span, bool)], i: usize) -> Option<(Frame, usize)> {
    let form = |k: usize| all.get(k).map(|t| t.0).unwrap_or("");

    // `(about) as X as Y` — X is one to three tokens, so "as thoroughly
    // unremarkable as" matches and "as far as I can tell from the numbers we
    // have as of today" does not chain across the second `as`.
    if form(i) == "as" || (form(i) == "about" && form(i + 1) == "as") {
        let ironic = form(i) == "about";
        let first_as = if ironic { i + 1 } else { i };
        for gap in 1..=3 {
            let second_as = first_as + gap + 1;
            if form(second_as) != "as" {
                continue;
            }
            // The X slot must be lexical throughout: "as, as" is punctuation.
            if !(first_as + 1..second_as).all(|k| all.get(k).is_some_and(|t| t.2)) {
                continue;
            }
            let (vehicle_len, negated, end) = scan_vehicle(all, second_as + 1);
            if vehicle_len == 0 {
                continue;
            }
            let span = Span::new(all[i].1.start, all[end - 1].1.end);
            return Some((
                Frame {
                    kind: if ironic {
                        FrameKind::IronicAsAs
                    } else {
                        FrameKind::AsAs
                    },
                    span,
                    vehicle_len,
                    negated,
                    normalized: normalize(all, i, end),
                },
                end,
            ));
        }
    }

    // `like a/the Y`. The determiner requirement is what excludes the verb
    // ("I like it") and the discourse particle ("it was, like, fine").
    if form(i) == "like" && VEHICLE_DETERMINERS.contains(&form(i + 1)) {
        let (vehicle_len, negated, end) = scan_vehicle(all, i + 1);
        if vehicle_len > 0 {
            let span = Span::new(all[i].1.start, all[end - 1].1.end);
            return Some((
                Frame {
                    kind: FrameKind::LikeA,
                    span,
                    vehicle_len,
                    negated,
                    normalized: normalize(all, i, end),
                },
                end,
            ));
        }
    }

    // `(in) much the same way (that) Y` — the Adams frame, and the one whose
    // vehicle is most often negated.
    if form(i) == "much" && form(i + 1) == "the" && form(i + 2) == "same" && form(i + 3) == "way" {
        let mut start = i + 4;
        if form(start) == "that" {
            start += 1;
        }
        let (vehicle_len, negated, end) = scan_vehicle(all, start);
        if vehicle_len > 0 {
            let open = if i > 0 && form(i - 1) == "in" {
                i - 1
            } else {
                i
            };
            let span = Span::new(all[open].1.start, all[end - 1].1.end);
            return Some((
                Frame {
                    kind: FrameKind::SameWay,
                    span,
                    vehicle_len,
                    negated,
                    normalized: normalize(all, open, end),
                },
                end,
            ));
        }
    }

    None
}

/// Walk the vehicle from `start`, returning `(lexical length, negated, end)`.
fn scan_vehicle(all: &[(&str, Span, bool)], start: usize) -> (usize, bool, usize) {
    let mut lexical = 0usize;
    let mut negated = false;
    let mut k = start;
    while k < all.len() && lexical < VEHICLE_WINDOW {
        let (form, _, is_lexical) = all[k];
        if VEHICLE_STOPS.contains(&form) {
            break;
        }
        if is_lexical {
            lexical += 1;
            if NEGATORS.contains(&form) {
                negated = true;
            }
        } else if form == "," && lexical > 0 {
            // A comma ends the vehicle unless nothing has been read yet, which
            // keeps "like a brick, obviously" from swallowing the aside.
            break;
        }
        k += 1;
    }
    (lexical, negated, k.max(start + 1))
}

/// The frame's lexical text, lowercased and space-joined, for frozen lookup.
fn normalize(all: &[(&str, Span, bool)], start: usize, end: usize) -> String {
    all[start..end.min(all.len())]
        .iter()
        .filter(|(_, _, lexical)| *lexical)
        .map(|(form, _, _)| *form)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::packs;
    use crate::text::{Document, Tokenizer};
    use crate::FeatureVector;

    fn fit(spec: ComparisonFrames) -> (FittedFrames, Interner) {
        let corpus = Corpus::new();
        let ctx = FitContext::new(&corpus, &[]);
        let mut interner = Interner::new();
        let fitted = spec.fit(&ctx, &mut interner).unwrap();
        (fitted, interner)
    }

    fn transform(spec: ComparisonFrames, text: &str) -> (FeatureVector, Interner) {
        let (fitted, interner) = fit(spec);
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        let mut b = VectorBuilder::new().track_spans(true);
        fitted.transform(&analysis, &mut b);
        (b.build(), interner)
    }

    fn frames_of(text: &str) -> Vec<Frame> {
        let doc = Document::new(text);
        let analysis = doc.analyze(&Tokenizer::default());
        extract_frames(&analysis)
    }

    /// A paragraph with one of each frame, hand-placed.
    const FIXTURE: &str = "The ship hung in the sky in much the same way that bricks don't. \
        It was as big as a small country. The engine sounded like a walrus with a headache. \
        The plan was about as sensible as a chocolate teapot. He was as good as gold.";

    #[test]
    fn every_frame_shape_is_found_exactly_once() {
        let frames = frames_of(FIXTURE);
        let kinds: Vec<FrameKind> = frames.iter().map(|f| f.kind).collect();
        assert_eq!(
            kinds,
            vec![
                FrameKind::SameWay,
                FrameKind::AsAs,
                FrameKind::LikeA,
                FrameKind::IronicAsAs,
                FrameKind::AsAs,
            ],
            "{:?}",
            frames.iter().map(|f| &f.normalized).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_negated_vehicle_is_distinguished_from_a_plain_simile() {
        let frames = frames_of(FIXTURE);
        let negated: Vec<&Frame> = frames.iter().filter(|f| f.negated).collect();
        assert_eq!(negated.len(), 1, "only the Adams construction is negated");
        assert_eq!(negated[0].kind, FrameKind::SameWay);
        assert!(negated[0].normalized.contains("don't"));
    }

    #[test]
    fn frame_spans_are_byte_exact() {
        let (v, i) = transform(ComparisonFrames::default(), FIXTURE);
        let spans = v.spans(i.get("frame:negated_vehicle_rate").unwrap());
        assert_eq!(spans.len(), 1);
        assert_eq!(
            &FIXTURE[spans[0].range()],
            "in much the same way that bricks don't"
        );
        let spans = v.spans(i.get("frame:ironic_hedge_rate").unwrap());
        assert_eq!(
            &FIXTURE[spans[0].range()],
            "about as sensible as a chocolate teapot"
        );
    }

    #[test]
    fn rates_are_exact_per_thousand_tokens() {
        let doc = Document::new(FIXTURE);
        let tokens = doc.analyze(&Tokenizer::default()).lexical_len();
        let (v, i) = transform(ComparisonFrames::default(), FIXTURE);
        let simile = v.get(i.get("frame:simile_rate").unwrap());
        assert!(
            (simile - 5.0 * 1000.0 / tokens as f64).abs() < 1e-9,
            "{simile}"
        );
        let negated = v.get(i.get("frame:negated_vehicle_rate").unwrap());
        assert!((negated - 1000.0 / tokens as f64).abs() < 1e-9);
    }

    #[test]
    fn the_verb_like_and_the_discourse_particle_are_not_similes() {
        assert!(frames_of("I like it and so does everyone else here.").is_empty());
        assert!(frames_of("It was, like, fine, honestly.").is_empty());
        // But a determiner turns it into a vehicle.
        assert_eq!(frames_of("It moved like a wounded animal.").len(), 1);
    }

    #[test]
    fn as_far_as_style_idioms_do_not_chain_into_a_simile() {
        // The X slot is capped at three tokens, so a long `as ... as` span is
        // not read as a comparison.
        let frames = frames_of(
            "As far as anyone at the meeting could tell from the sheet, \
                                the numbers were unchanged.",
        );
        assert!(
            frames.iter().all(|f| f.vehicle_len > 0),
            "extraction must not emit empty vehicles"
        );
    }

    #[test]
    fn the_frozen_share_needs_a_pack_and_says_so_without_one() {
        let (v, i) = transform(ComparisonFrames::default(), FIXTURE);
        assert!(
            v.is_missing(i.get("frame:frozen_share").unwrap()),
            "without a pack the share is unknown, not zero"
        );

        let pack = packs::builtin("cliche-similes").unwrap();
        let (v, i) = transform(ComparisonFrames::default().with_frozen(pack), FIXTURE);
        let share = v.get(i.get("frame:frozen_share").unwrap());
        // One of the five frames — "as good as gold" — is a dead simile.
        assert!((share - 0.2).abs() < 1e-9, "{share}");
    }

    #[test]
    fn vehicle_length_separates_a_one_word_vehicle_from_a_clause() {
        let short = transform(ComparisonFrames::default(), "It was like a brick. It fell.");
        let long = transform(
            ComparisonFrames::default(),
            "It was like a brick falling slowly through a warm afternoon sky. It fell.",
        );
        let sym = |i: &Interner| i.get("frame:vehicle_len_mean").unwrap();
        assert!(short.0.get(sym(&short.1)) < long.0.get(sym(&long.1)));
    }

    #[test]
    fn a_document_with_no_frames_reports_zero_rates_and_unknown_shape() {
        let (v, i) = transform(
            ComparisonFrames::default(),
            "The build broke this morning and we fixed it before lunch.",
        );
        assert_eq!(v.get(i.get("frame:simile_rate").unwrap()), 0.0);
        // A rate of zero is a fact; a mean vehicle length over zero vehicles is
        // not.
        assert!(v.is_missing(i.get("frame:vehicle_len_mean").unwrap()));
    }

    #[test]
    fn the_frozen_pack_license_rolls_up_into_the_reference() {
        let pack = packs::builtin("cliche-similes").unwrap();
        let (fitted, _) = fit(ComparisonFrames::default().with_frozen(pack));
        let licenses = fitted.pack_licenses();
        assert_eq!(licenses.len(), 1);
        assert!(licenses[0].pack.starts_with("cliche-similes@"));
        assert!(licenses[0].redistributable);
    }

    #[test]
    fn the_spec_round_trips_through_serde() {
        let spec = ComparisonFrames::default();
        assert_eq!(
            serde_json::from_str::<ComparisonFrames>("{}").unwrap(),
            spec
        );
        let with =
            ComparisonFrames::default().with_frozen(packs::builtin("cliche-similes").unwrap());
        let json = serde_json::to_string(&with).unwrap();
        assert_eq!(
            serde_json::from_str::<ComparisonFrames>(&json).unwrap(),
            with
        );
    }
}
