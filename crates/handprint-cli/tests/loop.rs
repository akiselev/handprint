//! End-to-end tests for the agent-critic loop.
//!
//! The "agent" here is a template rewriter, not a model: it reads the JSON
//! contract, applies the `fix` alternatives it is given, and re-submits. That
//! is enough to check the thing the loop actually has to guarantee — that the
//! feedback is *mechanically actionable* and that following it converges — with
//! no inference in the test.

use std::path::{Path, PathBuf};
use std::process::Command;

use handprint_core::critique::CritiqueReport;

/// A deterministic pool of "human" clause fragments.
const HUMAN: &[&str] = &[
    "I looked at it again and it still comes out the same",
    "not sure whats going on there honestly",
    "maybe the cache is stale, I dont know",
    "I'll poke at it tomorrow when I have a minute",
    "the numbers are in the sheet if you want them",
    "I think the whole thing needs a rewrite but nobody has time",
    "it worked on my machine which is never a good sign",
    "we shipped it anyway and nothing broke, so far",
    "someone should probably write this down somewhere",
    "the old version did this too, for what its worth",
    "I gave up and just hardcoded it",
    "took me an hour to find, it was a typo",
    "no idea why that helps but it does",
    "I'd rather not touch that file again",
    "we tried that last year and it went badly",
];

const SLOP: &[&str] = &[
    "This comprehensive analysis delves into the intricate tapestry of considerations",
    "Additionally, it is worth noting that experts argue this framework is essential",
    "Moreover, the findings underscore a pivotal shift in the ever evolving landscape",
    "The methodology showcases a robust, scalable, and transformative approach",
    "In conclusion, the results are commendable and unlock the potential of the field",
    "Furthermore, this nuanced and multifaceted subject plays a crucial role throughout",
    "Studies show that the paradigm serves as a cornerstone of modern practice",
    "It is important to note that the realm of possibilities remains unparalleled",
    "This groundbreaking work navigates the complexities with meticulous precision",
    "The tapestry of evidence stands as a testament to a truly holistic methodology",
];

/// A tiny deterministic PRNG, so fixtures are reproducible without a dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn pick<'a>(&mut self, items: &'a [&'a str]) -> &'a str {
        items[(self.next() as usize) % items.len()]
    }
}

fn compose(seed: u64, bits: &[&str], sentences: usize, tail: &str) -> String {
    let mut rng = Rng(seed);
    let mut out = String::new();
    for _ in 0..sentences {
        out.push_str(rng.pick(bits));
        out.push_str(tail);
        out.push(' ');
    }
    out
}

fn binary() -> PathBuf {
    // The integration test binary lives next to the built CLI.
    let mut path = std::env::current_exe().expect("test binary path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(if cfg!(windows) { "handprint.exe" } else { "handprint" })
}

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Fixture {
        let dir = std::env::temp_dir().join(format!(
            "handprint-loop-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("me")).unwrap();
        std::fs::create_dir_all(dir.join("background")).unwrap();

        for i in 0..24u64 {
            std::fs::write(
                dir.join("me").join(format!("doc{i:02}.txt")),
                compose(100 + i, HUMAN, 12, "."),
            )
            .unwrap();
        }

        // Background authors, each internally consistent and mutually distinct.
        let all: Vec<&str> = HUMAN.iter().chain(SLOP).copied().collect();
        for a in 0..14usize {
            let author = dir.join("background").join(format!("bg{a:02}"));
            std::fs::create_dir_all(&author).unwrap();
            let slice: Vec<&str> = (0..6).map(|k| all[(a * 3 + k) % all.len()]).collect();
            for k in 0..5u64 {
                std::fs::write(
                    author.join(format!("doc{k}.txt")),
                    compose(
                        5_000 + (a as u64) * 10 + k,
                        &slice,
                        16,
                        if a % 3 == 0 { "." } else { "!" },
                    ),
                )
                .unwrap();
            }
        }
        Fixture { dir }
    }

    fn path(&self, rest: &str) -> PathBuf {
        self.dir.join(rest)
    }

    fn run(&self, args: &[&str]) -> (i32, String, String) {
        let output = Command::new(binary())
            .args(args)
            .current_dir(&self.dir)
            .output()
            .unwrap_or_else(|e| panic!("running {:?}: {e}", binary()));
        (
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// Fit and calibrate a reference, returning its path.
    fn reference(&self) -> PathBuf {
        let reference = self.path("me.json");
        let (code, _, err) = self.run(&[
            "fit",
            "me",
            "-o",
            "me.json",
            "--name",
            "test-corpus",
            "--version",
            "1",
            "--features",
            "punct,sentence,lexicon,ngrams",
        ]);
        assert_eq!(code, 0, "fit failed: {err}");

        let (code, _, err) = self.run(&[
            "calibrate",
            "me.json",
            "--background",
            "background",
            "--metric",
            "burrows",
            "--bins",
            "64,128,256",
            "--different-pairs",
            "400",
            "--same-pairs",
            "200",
        ]);
        assert_eq!(code, 0, "calibrate failed: {err}");
        reference
    }

    fn critique(&self, reference: &Path, draft: &str) -> (i32, CritiqueReport) {
        std::fs::write(self.path("draft.txt"), draft).unwrap();
        let (code, out, err) = self.run(&[
            "critique",
            "-r",
            reference.file_name().unwrap().to_str().unwrap(),
            "draft.txt",
        ]);
        assert!(code == 0 || code == 1, "critique errored ({code}): {err}\n{out}");
        let report: CritiqueReport =
            serde_json::from_str(&out).unwrap_or_else(|e| panic!("bad JSON ({e}): {out}"));
        (code, report)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The scripted "agent": apply every `consider_replace` fix the report offers,
/// by replacing the flagged spans with their first suggested alternative.
///
/// Spans are applied back-to-front so earlier offsets stay valid — which is
/// exactly why the contract reports byte spans rather than line numbers.
fn apply_fixes(draft: &str, report: &CritiqueReport) -> String {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for finding in &report.findings {
        let Some(fix) = &finding.fix else { continue };
        if fix.kind != "consider_replace" {
            continue;
        }
        let Some(replacement) = fix.alternatives.first() else {
            continue;
        };
        if replacement.starts_with('(') {
            continue; // "(cut it)" is an instruction, not a replacement
        }
        for span in &finding.spans {
            edits.push((span[0], span[1], replacement.clone()));
        }
    }
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    edits.dedup_by_key(|e| e.0);

    let mut out = draft.to_owned();
    for (start, end, replacement) in edits {
        if start <= end && end <= out.len() && out.is_char_boundary(start) && out.is_char_boundary(end)
        {
            out.replace_range(start..end, &replacement);
        }
    }
    out
}

/// Delete whole sentences that carry a phrase-level finding.
fn drop_flagged_sentences(draft: &str, report: &CritiqueReport) -> String {
    let flagged: Vec<usize> = report
        .findings
        .iter()
        .filter(|f| f.id.contains(".phrase.") || f.id.contains(".word."))
        .flat_map(|f| f.spans.iter().map(|s| s[0]))
        .collect();
    if flagged.is_empty() {
        return draft.to_owned();
    }
    let mut out = String::new();
    let mut offset = 0usize;
    for sentence in draft.split_inclusive('.') {
        let range = offset..offset + sentence.len();
        offset = range.end;
        if !flagged.iter().any(|f| range.contains(f)) {
            out.push_str(sentence);
        }
    }
    if out.trim().is_empty() {
        draft.to_owned()
    } else {
        out
    }
}

#[test]
fn a_scripted_agent_converges_on_a_salted_draft() {
    let fixture = Fixture::new("converge");
    let reference = fixture.reference();

    // Start from a draft that is half corpus-like and half salted, so that
    // removing the salt leaves something that can plausibly pass.
    let mut draft = format!(
        "{} {}",
        compose(41, HUMAN, 14, "."),
        compose(42, SLOP, 6, ".")
    );

    let (first_code, first) = fixture.critique(&reference, &draft);
    assert_eq!(first_code, 1, "the salted draft should fail: {:?}", first.verdict);
    assert!(first.findings_total > 0);
    let initial_findings = first.findings_total;
    let initial_distance = first.verdict.distance;

    let mut report = first;
    let mut iterations = 0usize;
    const MAX_ITERATIONS: usize = 12;

    while iterations < MAX_ITERATIONS {
        iterations += 1;
        // Apply the mechanical fixes, then drop whatever still carries a
        // lexicon hit. A real agent would rewrite; this one deletes.
        let patched = apply_fixes(&draft, &report);
        let patched = drop_flagged_sentences(&patched, &report);
        if patched == draft {
            break;
        }
        draft = patched;
        let (code, next) = fixture.critique(&reference, &draft);
        report = next;
        if code == 0 {
            break;
        }
    }

    assert!(
        report.verdict.distance < initial_distance,
        "the loop must make progress: {initial_distance} -> {}",
        report.verdict.distance
    );
    assert!(
        report.findings_total < initial_findings,
        "findings should fall: {initial_findings} -> {}",
        report.findings_total
    );
    assert_eq!(
        report.verdict.pass,
        Some(true),
        "did not converge in {iterations} iteration(s): {}",
        report.summary()
    );
    assert!(iterations <= MAX_ITERATIONS);
}

#[test]
fn spans_are_byte_exact_so_patches_apply_cleanly() {
    let fixture = Fixture::new("spans");
    let reference = fixture.reference();
    let draft = format!("{} {}", compose(7, HUMAN, 8, "."), compose(8, SLOP, 4, "."));
    let (_, report) = fixture.critique(&reference, &draft);

    let mut checked = 0usize;
    for finding in &report.findings {
        // Per-term lexicon findings name the term in their id; the span must
        // slice the submitted bytes back to exactly that term.
        let Some(term) = finding.id.strip_prefix("lex.ai-slop.word.") else {
            continue;
        };
        for span in &finding.spans {
            assert_eq!(&draft[span[0]..span[1]], term, "{}", finding.id);
            checked += 1;
        }
    }
    assert!(checked > 0, "expected at least one per-term finding to check");
}

#[test]
fn exit_codes_follow_vale() {
    let fixture = Fixture::new("exits");
    let reference = fixture.reference();

    // 1: findings.
    let (code, _) = fixture.critique(&reference, &compose(3, SLOP, 8, "."));
    assert_eq!(code, 1);

    // 0: pass.
    let (code, report) = fixture.critique(&reference, &compose(4, HUMAN, 12, "."));
    assert_eq!(code, 0, "{}", report.summary());
    assert_eq!(report.verdict.pass, Some(true));

    // 2: error. An uncalibrated reference cannot evaluate the gate.
    let (code, _, _) = fixture.run(&[
        "fit",
        "me",
        "-o",
        "bare.json",
        "--name",
        "bare",
        "--features",
        "punct",
    ]);
    assert_eq!(code, 0);
    std::fs::write(fixture.path("draft.txt"), compose(5, HUMAN, 8, ".")).unwrap();
    let (code, out, _) = fixture.run(&["critique", "-r", "bare.json", "draft.txt"]);
    assert_eq!(code, 2, "{out}");
    let report: CritiqueReport = serde_json::from_str(&out).unwrap();
    assert_eq!(report.verdict.pass, None);

    // 2: error. A missing reference.
    let (code, _, err) = fixture.run(&["critique", "-r", "nope.json", "draft.txt"]);
    assert_eq!(code, 2);
    assert!(err.contains("handprint:"), "{err}");
}

#[test]
fn the_project_config_supplies_defaults() {
    let fixture = Fixture::new("config");
    let reference = fixture.reference();
    let _ = reference;
    std::fs::write(
        fixture.path("handprint.toml"),
        "[project]\nreference = \"me.json\"\nprofile = \"strict\"\nmax_findings = 3\n",
    )
    .unwrap();
    std::fs::write(fixture.path("draft.txt"), compose(9, SLOP, 8, ".")).unwrap();

    // No --reference flag: it comes from handprint.toml.
    let (code, out, err) = fixture.run(&["critique", "draft.txt"]);
    assert_eq!(code, 1, "{err}\n{out}");
    let report: CritiqueReport = serde_json::from_str(&out).unwrap();
    assert_eq!(report.reference.name, "test-corpus");
    assert!(report.findings.len() <= 3, "max_findings not applied");
}

#[test]
fn forensic_commands_run() {
    let fixture = Fixture::new("forensic");
    let reference = fixture.reference();
    let name = reference.file_name().unwrap().to_str().unwrap().to_owned();

    std::fs::write(fixture.path("a.txt"), compose(11, HUMAN, 10, ".")).unwrap();
    std::fs::write(fixture.path("b.txt"), compose(12, SLOP, 10, ".")).unwrap();

    // With the metric the reference was calibrated for, the assessment appears
    // together with the caveat about how to read it.
    let (code, out, err) = fixture.run(&[
        "compare", "-r", &name, "a.txt", "b.txt", "--metric", "burrows",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("distance"));
    assert!(out.contains("p_value_vs_unrelated is P("), "the caveat must be printed: {out}");

    // With a metric it was *not* calibrated for, it says exactly that rather
    // than silently omitting the numbers.
    let (code, out, err) = fixture.run(&[
        "compare", "-r", &name, "a.txt", "b.txt", "--metric", "cosine",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("calibrated for burrows_delta"),
        "the metric mismatch must be explained: {out}"
    );

    let (code, out, err) = fixture.run(&["profile", "-r", &name, "a.txt"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("confidence by family"));

    let (code, out, err) =
        fixture.run(&["rank", "-r", &name, "a.txt", "background", "--metric", "burrows"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Ranking is not identification"), "{out}");

    let (code, out, err) = fixture.run(&["pack", "show", &name]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("calibrated for burrows_delta"), "{out}");

    let (code, out, err) = fixture.run(&[
        "contrast",
        "me",
        "background",
        "--min-count",
        "2",
        "--top",
        "5",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("alpha0"), "{out}");
    assert!(out.contains("comparable only within this term universe"), "{out}");

    let (code, out, err) = fixture.run(&["explain", "-r", &name, "b.txt", "--html", "out.html"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("top contributions"));
    let html = std::fs::read_to_string(fixture.path("out.html")).unwrap();
    assert!(html.starts_with("<!doctype html>"));
}

#[test]
fn verify_runs_general_imposters() {
    let fixture = Fixture::new("verify");
    let reference = fixture.reference();
    let name = reference.file_name().unwrap().to_str().unwrap().to_owned();
    std::fs::write(fixture.path("q.txt"), compose(13, HUMAN, 12, ".")).unwrap();

    let (code, out, err) = fixture.run(&[
        "verify",
        "-r",
        &name,
        "q.txt",
        "--target",
        "me",
        "--impostors",
        "background",
        "--iterations",
        "40",
        "--json",
    ]);
    assert_eq!(code, 0, "{err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(value["score"].is_number());
    assert!(value["verdict"].is_string());
    assert!(value["thresholds"]["p1"].is_number());
}
