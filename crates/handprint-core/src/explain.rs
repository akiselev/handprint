//! Phase 5a — explanation: which dimensions drove a result, and where in the
//! text they came from.
//!
//! [`Comparison::contributions`](crate::Comparison::contributions) already
//! decomposes a distance. This module adds the two things a report needs on top
//! of that: **contrastive classification** (is this draft pulled toward A or
//! toward B, and by what?) and **span attribution** (which bytes are
//! responsible).
//!
//! # Why contrastive classification takes one reference, not two
//!
//! The obvious signature is `classify(doc, reference_a, reference_b)`. It does
//! not work: two independently fitted references have different interners,
//! different vocabularies and different corpus statistics, so their dimensions
//! do not line up and their z-scores are not on a common scale. Comparing them
//! would produce numbers, and the numbers would be meaningless.
//!
//! [`ContrastReport::classify`] instead takes **one** reference — fitted on the
//! union of both sides — plus a profile for each side's centroid. Then every
//! dimension is measured the same way for both, the pulls decompose exactly,
//! and `Σ pull = distance_b − distance_a`.

use serde::{Deserialize, Serialize};

use crate::compare::Metric;
use crate::contrast::Side;
use crate::error::{Error, Result};
use crate::feature::{Family, Unit};
use crate::reference::{Profile, Reference};
use crate::text::Span;
use crate::vector::Symbol;

/// One dimension's pull toward one side or the other.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pull {
    /// The dimension.
    pub symbol: Symbol,
    /// Its human-readable name.
    pub name: String,
    /// Which family it came from.
    pub family: Family,
    /// What it is measured in.
    pub unit: Unit,
    /// Signed pull. Positive means the dimension pulls the document toward side
    /// A; negative, toward side B.
    pub pull: f64,
    /// The document's raw value.
    pub observed: f64,
    /// Side A's raw value.
    pub target_a: f64,
    /// Side B's raw value.
    pub target_b: f64,
}

impl Pull {
    /// Which side this dimension favours.
    pub fn side(&self) -> Side {
        if self.pull >= 0.0 {
            Side::A
        } else {
            Side::B
        }
    }

    /// How far the observed value must move to reach the favoured-away side's
    /// value, as a signed delta in the dimension's own unit.
    pub fn delta_to(&self, side: Side) -> f64 {
        match side {
            Side::A => self.target_a - self.observed,
            Side::B => self.target_b - self.observed,
        }
    }
}

/// The result of classifying a document between two reference points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastReport {
    /// Distance to side A.
    pub distance_a: f64,
    /// Distance to side B.
    pub distance_b: f64,
    /// The closer side.
    pub closer: Side,
    /// `distance_b − distance_a`. Positive means closer to A.
    pub margin: f64,
    /// Per-dimension pulls, largest magnitude first. They sum to `margin`.
    pub pulls: Vec<Pull>,
    /// The metric used.
    pub metric: Metric,
}

impl ContrastReport {
    /// Classify a document between two centroid profiles measured against the
    /// same reference.
    ///
    /// In de-AI mode, `a` is the AI-style centroid and `b` the human one, so a
    /// negative `margin` is the goal and the pulls with the largest positive
    /// values are what to change first.
    pub fn classify(
        reference: &Reference,
        doc: &Profile,
        a: &Profile,
        b: &Profile,
        metric: Metric,
    ) -> Result<ContrastReport> {
        let to_a = reference.compare_with(doc, a, metric)?;
        let to_b = reference.compare_with(doc, b, metric)?;

        let ca = to_a.contributions();
        let cb = to_b.contributions();
        // Both are sorted by magnitude, so index them by symbol to pair them up.
        let mut pulls: Vec<Pull> = Vec::with_capacity(ca.len());
        for contribution_a in &ca {
            let Some(contribution_b) = cb.iter().find(|c| c.symbol == contribution_a.symbol) else {
                continue;
            };
            pulls.push(Pull {
                symbol: contribution_a.symbol,
                name: contribution_a.name.clone(),
                family: contribution_a.family,
                unit: contribution_a.unit,
                // Cost of being unlike B minus cost of being unlike A: positive
                // when the dimension is cheaper to explain as A.
                pull: contribution_b.value - contribution_a.value,
                observed: contribution_a.observed_a,
                target_a: contribution_a.observed_b,
                target_b: contribution_b.observed_b,
            });
        }
        pulls.sort_by(|x, y| {
            y.pull
                .abs()
                .partial_cmp(&x.pull.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let margin = to_b.distance - to_a.distance;
        Ok(ContrastReport {
            distance_a: to_a.distance,
            distance_b: to_b.distance,
            closer: if margin >= 0.0 { Side::A } else { Side::B },
            margin,
            pulls,
            metric,
        })
    }

    /// The `n` dimensions pulling hardest toward a side.
    pub fn top(&self, n: usize, side: Side) -> Vec<&Pull> {
        self.pulls
            .iter()
            .filter(|p| p.side() == side)
            .take(n)
            .collect()
    }

    /// Pulls aggregated by family.
    pub fn by_family(&self) -> Vec<(Family, f64)> {
        let mut totals: std::collections::HashMap<Family, f64> = Default::default();
        for pull in &self.pulls {
            *totals.entry(pull.family).or_insert(0.0) += pull.pull;
        }
        let mut out: Vec<(Family, f64)> = totals.into_iter().collect();
        out.sort_by(|x, y| {
            y.1.abs()
                .partial_cmp(&x.1.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }
}

/// A highlighted region of a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Highlight {
    /// Where in the source text.
    pub span: Span,
    /// Which dimension put it there.
    pub name: String,
    /// How much that dimension mattered.
    pub weight: f64,
}

/// Collect the spans behind the highest-weight dimensions of a profile.
///
/// Requires a profile built with
/// [`Reference::profile_tracked`](crate::Reference::profile_tracked);
/// otherwise the result is empty, because no spans were recorded.
pub fn highlights(
    reference: &Reference,
    profile: &Profile,
    weights: &[(Symbol, f64)],
    limit: usize,
) -> Vec<Highlight> {
    let mut ranked: Vec<(Symbol, f64)> = weights.to_vec();
    ranked.sort_by(|a, b| {
        b.1.abs()
            .partial_cmp(&a.1.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut out = Vec::new();
    for (symbol, weight) in ranked {
        for &span in profile.spans(symbol) {
            out.push(Highlight {
                span,
                name: reference.name_of(symbol).to_owned(),
                weight,
            });
            if out.len() >= limit {
                return out;
            }
        }
    }
    out
}

/// Locate the most reference-unlike window of a long document.
///
/// A rolling profile over overlapping windows, in the spirit of stylo's
/// `rolling.delta`. Use it to answer "which paragraph is the problem?" rather
/// than re-reading a whole draft.
pub fn rolling(
    reference: &Reference,
    text: &str,
    target: &Profile,
    window_tokens: usize,
    step_tokens: usize,
    metric: Metric,
) -> Result<Vec<(Span, f64)>> {
    if window_tokens == 0 || step_tokens == 0 {
        return Err(Error::InvalidConfig {
            what: "explain::rolling",
            detail: "window and step must both be positive".into(),
        });
    }
    let doc = crate::text::Document::new(text);
    let analysis = doc.analyze(reference.tokenizer());
    let ends: Vec<usize> = analysis
        .tokens()
        .lexical()
        .map(|(t, _)| t.span.end)
        .collect();
    if ends.len() < window_tokens {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut start_tok = 0usize;
    while start_tok + window_tokens <= ends.len() {
        let start = if start_tok == 0 {
            0
        } else {
            ends[start_tok - 1]
        };
        let end = ends[start_tok + window_tokens - 1];
        let profile = reference.profile(&crate::text::Document::new(&text[start..end]));
        let distance = reference.compare_with(&profile, target, metric)?.distance;
        out.push((Span::new(start, end), distance));
        start_tok += step_tokens;
    }
    Ok(out)
}

#[cfg(feature = "render")]
mod render {
    use super::Highlight;

    /// Render a document with highlighted spans, using ANSI colour.
    ///
    /// Overlapping highlights are resolved by taking the highest weight, which
    /// keeps the output readable at the cost of hiding the fact that two
    /// dimensions both fired on one span.
    pub fn render_ansi(text: &str, highlights: &[Highlight]) -> String {
        let mut out = String::with_capacity(text.len() * 2);
        let mut cursor = 0usize;
        for h in resolve(highlights) {
            if h.span.start < cursor || h.span.end > text.len() {
                continue;
            }
            out.push_str(&text[cursor..h.span.start]);
            out.push_str("\x1b[43m\x1b[30m");
            out.push_str(&text[h.span.range()]);
            out.push_str("\x1b[0m");
            cursor = h.span.end;
        }
        out.push_str(&text[cursor..]);
        out
    }

    /// Render a document with highlighted spans as an HTML fragment.
    pub fn render_html(text: &str, highlights: &[Highlight]) -> String {
        let mut out = String::with_capacity(text.len() * 2);
        let mut cursor = 0usize;
        for h in resolve(highlights) {
            if h.span.start < cursor || h.span.end > text.len() {
                continue;
            }
            escape_into(&text[cursor..h.span.start], &mut out);
            out.push_str(&format!(
                "<mark class=\"handprint\" title=\"{} ({:+.4})\">",
                escape(&h.name),
                h.weight
            ));
            escape_into(&text[h.span.range()], &mut out);
            out.push_str("</mark>");
            cursor = h.span.end;
        }
        escape_into(&text[cursor..], &mut out);
        out
    }

    fn resolve(highlights: &[Highlight]) -> Vec<Highlight> {
        let mut sorted = highlights.to_vec();
        sorted.sort_by(|a, b| {
            a.span.start.cmp(&b.span.start).then_with(|| {
                b.weight
                    .abs()
                    .partial_cmp(&a.weight.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        let mut out: Vec<Highlight> = Vec::new();
        for h in sorted {
            if out.last().is_some_and(|p| p.span.end > h.span.start) {
                continue;
            }
            out.push(h);
        }
        out
    }

    fn escape(s: &str) -> String {
        let mut out = String::new();
        escape_into(s, &mut out);
        out
    }

    fn escape_into(s: &str, out: &mut String) {
        for c in s.chars() {
            match c {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                c => out.push(c),
            }
        }
    }
}

#[cfg(feature = "render")]
pub use render::{render_ansi, render_html};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use crate::feature::{LexiconFeature, PunctTypography, SentenceStats};
    use crate::reference::Pipeline;
    use crate::text::Document;

    fn ai_text(i: usize) -> String {
        format!(
            "This comprehensive analysis delves into the intricate tapestry of \
             considerations—a truly multifaceted subject. Additionally, it is worth noting \
             that experts argue the framework plays a crucial role in run {i}. Moreover, \
             the results underscore a pivotal shift. In conclusion, the findings are \
             commendable and showcase a robust methodology throughout."
        )
    }

    fn human_text(i: usize) -> String {
        format!(
            "I ran it again on batch {i} and got the same answer, which is annoying. \
             Not sure whats going on. Maybe the cache? I'll poke at it tomorrow. \
             Anyway the numbers are in the sheet if you want to look."
        )
    }

    fn setup() -> (Reference, Profile, Profile) {
        let mut corpus = Corpus::new();
        corpus.add(
            "ai",
            (0..12)
                .map(|i| Document::new(ai_text(i)))
                .collect::<Vec<_>>(),
        );
        corpus.add(
            "human",
            (0..12)
                .map(|i| Document::new(human_text(i)))
                .collect::<Vec<_>>(),
        );
        let reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .feature(SentenceStats::default())
            .feature(LexiconFeature::default())
            .name("ai-vs-human")
            .fit(&corpus)
            .unwrap();
        let ai = reference.profile_aggregate(corpus.author(&"ai".into()).unwrap());
        let human = reference.profile_aggregate(corpus.author(&"human".into()).unwrap());
        (reference, ai, human)
    }

    #[test]
    fn classification_picks_the_right_side() {
        let (reference, ai, human) = setup();
        let draft = reference.profile(&Document::new(ai_text(99)));
        let report =
            ContrastReport::classify(&reference, &draft, &ai, &human, Metric::CosineDelta).unwrap();
        assert_eq!(report.closer, Side::A, "{report:?}");
        assert!(report.margin > 0.0);

        let human_draft = reference.profile(&Document::new(human_text(99)));
        let report =
            ContrastReport::classify(&reference, &human_draft, &ai, &human, Metric::CosineDelta)
                .unwrap();
        assert_eq!(report.closer, Side::B);
        assert!(report.margin < 0.0);
    }

    #[test]
    fn pulls_sum_to_the_margin() {
        let (reference, ai, human) = setup();
        let draft = reference.profile(&Document::new(ai_text(7)));
        for metric in Metric::ALL {
            let report =
                ContrastReport::classify(&reference, &draft, &ai, &human, *metric).unwrap();
            let sum: f64 = report.pulls.iter().map(|p| p.pull).sum();
            assert!(
                (sum - report.margin).abs() < 1e-9,
                "{metric}: {sum} != {}",
                report.margin
            );
        }
    }

    #[test]
    fn the_ai_pull_is_explained_by_the_expected_families() {
        let (reference, ai, human) = setup();
        let draft = reference.profile(&Document::new(ai_text(3)));
        let report =
            ContrastReport::classify(&reference, &draft, &ai, &human, Metric::CosineDelta).unwrap();
        let names: Vec<&str> = report
            .top(20, Side::A)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        // The formulaic-opener dimensions are the loudest single signal here.
        assert!(
            names.iter().any(|n| n.starts_with("sent:opener:")),
            "expected formulaic-opener dimensions among the AI pulls: {names:?}"
        );

        // The lexicon family pulls toward A as a whole, even though under
        // cosine no single per-term rate outweighs the letter-frequency
        // dimensions: individual lexicon hits are small in magnitude, which is
        // exactly why the critique layer reports them by severity and band
        // rather than by raw contribution.
        let families = report.by_family();
        let lexicon = families
            .iter()
            .find(|(f, _)| *f == Family::Lexicon)
            .expect("lexicon family should be present");
        assert!(lexicon.1 > 0.0, "{families:?}");
        assert!(
            report
                .pulls
                .iter()
                .any(|p| p.name.starts_with("lex:") && p.pull > 0.0),
            "no lexicon dimension pulled toward A"
        );
    }

    #[test]
    fn delta_to_reports_the_move_needed() {
        let (reference, ai, human) = setup();
        let draft = reference.profile(&Document::new(ai_text(1)));
        let report =
            ContrastReport::classify(&reference, &draft, &ai, &human, Metric::CosineDelta).unwrap();
        let tapestry = report
            .pulls
            .iter()
            .find(|p| p.name == "lex:ai-slop:word.tapestry")
            .expect("tapestry dimension should exist");
        // The draft uses it; the human side does not, so moving toward B means
        // reducing it.
        assert!(tapestry.observed > 0.0, "{tapestry:?}");
        assert!(tapestry.target_b < tapestry.observed, "{tapestry:?}");
        assert!(tapestry.delta_to(Side::B) < 0.0, "{tapestry:?}");
    }

    #[test]
    fn highlights_need_a_tracked_profile() {
        let (reference, _, _) = setup();
        let text = "We delve into the tapestry of it.";
        let sym = reference.interner().get("lex:ai-slop:word.delve").unwrap();
        let weights = [(sym, 1.0)];

        let untracked = reference.profile(&Document::new(text));
        assert!(highlights(&reference, &untracked, &weights, 10).is_empty());

        let tracked = reference.profile_tracked(&Document::new(text));
        let got = highlights(&reference, &tracked, &weights, 10);
        assert_eq!(got.len(), 1);
        assert_eq!(&text[got[0].span.range()], "delve");
        assert_eq!(got[0].name, "lex:ai-slop:word.delve");
    }

    #[test]
    fn rolling_finds_the_odd_window() {
        let (reference, ai, human) = setup();
        let mut text = String::new();
        for i in 0..4 {
            text.push_str(&human_text(i));
            text.push(' ');
        }
        let ai_start = text.len();
        text.push_str(&ai_text(0));
        for i in 4..8 {
            text.push(' ');
            text.push_str(&human_text(i));
        }
        let windows = rolling(&reference, &text, &human, 40, 20, Metric::CosineDelta).unwrap();
        assert!(windows.len() > 3);
        let worst = windows
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        // The most human-unlike window should overlap the AI paragraph.
        assert!(
            worst.0.end > ai_start && worst.0.start < ai_start + ai_text(0).len(),
            "worst window {:?} misses the AI region at {}",
            worst.0,
            ai_start
        );
        let _ = ai;
    }

    #[test]
    fn rolling_rejects_a_zero_window() {
        let (reference, _, human) = setup();
        assert!(rolling(&reference, "some text", &human, 0, 1, Metric::CosineDelta).is_err());
    }

    #[cfg(feature = "render")]
    #[test]
    fn renderers_escape_and_do_not_overlap() {
        let text = "a <b> delve delve c";
        let highlights = [
            Highlight {
                span: Span::new(6, 11),
                name: "lex:x".into(),
                weight: 1.0,
            },
            Highlight {
                span: Span::new(8, 14),
                name: "lex:y".into(),
                weight: 0.5,
            },
        ];
        let html = render_html(text, &highlights);
        assert!(html.contains("&lt;b&gt;"));
        assert_eq!(html.matches("<mark").count(), 1, "{html}");
        let ansi = render_ansi(text, &highlights);
        assert!(ansi.contains("\x1b[43m"));
    }
}
