#!/usr/bin/env python3
"""The author battery: separation, interpretability, anti-caricature.

Three suites, from WIT.md's acceptance criteria:

**(a) Separation.** Held-out chapters rank their own author first, and General
Imposters verification with the other authors as the impostor pool returns
`TargetFavoured` for the true author. The abstention rate is *reported*, never
suppressed: a verifier that cannot abstain is not a verifier, and hiding how
often it does would make the number meaningless.

**(b) Interpretability.** Each author's top-|z| dimensions against a pooled
background intersect the critic-named trait set in `matrix.py` — including the
○ cells, where absence is the fingerprint.

**(c) Anti-caricature.** The seeded pastiches from `fixtures.py` trip
**upper**-band `Reduce` findings at Medium or above, and fail the same-author
gate.

Corpora are not in the repository. `--corpora` points at a directory of
`{author}/*.md` per-chapter documents; see `corpora/README.md`. Every result is
written to `--build` as JSON, and `results.py` renders the table.

    python3 battery/run.py --corpora corpora --build build
"""

from __future__ import annotations

import argparse
import json
import random
import shutil
from pathlib import Path

from fixtures import FIXTURES
from hp import Handprint, HandprintError, parse_profile_table
from matrix import AUTHORS, matches


def split_corpus(author_dir: Path, build: Path, holdout: int, seed: int) -> tuple[Path, Path]:
    """Split an author's chapters into a fit set and a held-out set.

    Held-out documents are what make suite (a) a test rather than a
    tautology: ranking a document that was in the fitting corpus against the
    reference fitted on it proves nothing.
    """
    chapters = sorted(p for p in author_dir.iterdir() if p.suffix in {".md", ".txt"})
    if len(chapters) < holdout * 3:
        raise SystemExit(
            f"{author_dir.name}: {len(chapters)} chapter(s) is too few to hold "
            f"{holdout} out and still fit; chapterize more books"
        )
    rng = random.Random(f"{seed}:{author_dir.name}")
    rng.shuffle(chapters)
    held, kept = chapters[:holdout], chapters[holdout:]

    fit_dir = build / "fit" / author_dir.name
    hold_dir = build / "holdout" / author_dir.name
    for directory in (fit_dir, hold_dir):
        directory.mkdir(parents=True, exist_ok=True)
    for path in kept:
        shutil.copy2(path, fit_dir / path.name)
    for path in held:
        shutil.copy2(path, hold_dir / path.name)
    return fit_dir, hold_dir


def pooled_background(build: Path, authors: list[str]) -> Path:
    """One directory holding every author, as the background to z-score against.

    An author's "unusual" dimensions are only unusual relative to something.
    Pooling the battery's own authors makes the comparison between the authors
    the battery is about, rather than between an author and English.
    """
    out = build / "background"
    out.mkdir(parents=True, exist_ok=True)
    for author in authors:
        source = build / "fit" / author
        target = out / author
        target.mkdir(parents=True, exist_ok=True)
        for path in source.iterdir():
            shutil.copy2(path, target / path.name)
    return out


def suite_separation(
    hp: Handprint, build: Path, background_ref: Path, authors: list[str]
) -> dict:
    """(a) Cross-author attribution sanity."""
    results = {"rank": {}, "verify": {}, "abstained": 0, "verified": 0}
    for author in authors:
        held = sorted((build / "holdout" / author).iterdir())
        if not held:
            continue
        query = held[0]
        rows = hp.rank(background_ref, query, build / "fit")
        first = rows[0]["label"] if rows else None
        results["rank"][author] = {
            "expected": author,
            "got": first,
            "correct": first == author,
            "distances": {r["label"]: r["distance"] for r in rows},
        }

        impostors = build / "impostors" / author
        impostors.mkdir(parents=True, exist_ok=True)
        for other in authors:
            if other == author:
                continue
            target = impostors / other
            target.mkdir(parents=True, exist_ok=True)
            for path in (build / "fit" / other).iterdir():
                if not (target / path.name).exists():
                    shutil.copy2(path, target / path.name)

        score = hp.verify(
            background_ref, query, build / "fit" / author, impostors
        )
        verdict = score.get("verdict")
        results["verify"][author] = {"verdict": verdict, "score": score.get("score")}
        results["verified"] += 1
        if verdict not in {"target_favoured", "TargetFavoured"}:
            if verdict in {"inconclusive", "Inconclusive"}:
                results["abstained"] += 1
    total = max(1, len(results["rank"]))
    results["rank_accuracy"] = sum(
        1 for r in results["rank"].values() if r["correct"]
    ) / total
    # Reported, never suppressed.
    results["abstention_rate"] = results["abstained"] / max(1, results["verified"])
    return results


def suite_interpretability(
    hp: Handprint, build: Path, background_ref: Path, authors: list[str], top: int
) -> dict:
    """(b) Do the top deviations match the critic-named traits?"""
    results = {}
    for author in authors:
        axes = AUTHORS[author]
        held = sorted((build / "holdout" / author).iterdir())
        if not held:
            continue
        table = hp.run(
            "profile", "-r", str(background_ref), str(held[0]), "--top", str(top)
        )
        rows = parse_profile_table(table)
        ranked = [name.replace(":", ".") for name, _ in rows]
        by_name = {name.replace(":", "."): z for name, z in rows}

        hit = [c for c in axes.high if any(matches(d, [c]) for d in ranked)]
        # The ○ cells: these must be *below* the background, which is a
        # negative z, not merely absent from the top-k.
        low_ok = {}
        for claim in axes.low:
            zs = [z for name, z in by_name.items() if matches(name, [claim])]
            low_ok[claim] = bool(zs) and min(zs) < 0.0
        results[author] = {
            "top": ranked[:top],
            "high_hit": hit,
            "high_required": axes.required(),
            "high_ok": len(hit) >= axes.required(),
            "low_ok": low_ok,
            "passed": len(hit) >= axes.required() and all(low_ok.values()),
        }
    return results


def suite_anticaricature(hp: Handprint, build: Path, references: dict[str, Path]) -> dict:
    """(c) Do over-fired pastiches trip upper bands and fail the gate?"""
    results = {}
    fixtures = build / "fixtures"
    target_for = {
        "fake-thompson": "hunter-s-thompson",
        "fake-bukowski": "charles-bukowski",
        "fake-adams": "douglas-adams",
    }
    for name, _builder, _units, expected in FIXTURES:
        author = target_for[name]
        reference = references.get(author)
        draft = fixtures / f"{name}.md"
        if reference is None or not draft.exists():
            continue
        report = hp.critique(reference, draft)
        over = [
            f
            for f in report["findings"]
            if f["direction"] == "reduce" and f["severity"] in {"medium", "high"}
        ]
        tripped = [f["id"] for f in over]
        results[name] = {
            "author": author,
            "expected_dims": expected,
            "tripped": tripped,
            # Every expected dimension must appear among the upper-band trips.
            "over_fired_ok": all(
                any(matches(t, [e]) for t in tripped) for e in expected
            ),
            # And the pastiche must not pass the same-author gate.
            "gate": report["verdict"]["pass"],
            "gate_ok": report["verdict"]["pass"] is not True,
        }
        results[name]["passed"] = (
            results[name]["over_fired_ok"] and results[name]["gate_ok"]
        )
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpora", type=Path, default=Path("corpora"))
    parser.add_argument("--build", type=Path, default=Path("build"))
    parser.add_argument("--handprint", default="handprint")
    parser.add_argument("--holdout", type=int, default=3)
    parser.add_argument("--top", type=int, default=40)
    parser.add_argument("--seed", type=int, default=20260801)
    args = parser.parse_args()

    hp = Handprint(command=args.handprint)
    args.build.mkdir(parents=True, exist_ok=True)

    authors = [a for a in AUTHORS if (args.corpora / a).is_dir()]
    if not authors:
        raise SystemExit(
            f"no battery corpora under {args.corpora}. The in-copyright corpora are "
            f"local-only and are never committed; see corpora/README.md for how to "
            f"build them on your own machine."
        )
    missing = [a for a in AUTHORS if a not in authors]
    if missing:
        print(f"note: running without {', '.join(missing)} (corpus absent)")

    for author in authors:
        split_corpus(args.corpora / author, args.build, args.holdout, args.seed)
    background = pooled_background(args.build, authors)

    # One reference over every author, for the z-scores the interpretability
    # suite reads and for the cross-author ranking.
    background_ref = hp.fit(
        background,
        args.build / "background.json",
        name="battery-background",
        features=AUTHORS[authors[0]].features,
    )
    hp.calibrate(background_ref, background)

    # Per-author references, for the anti-caricature critiques.
    references: dict[str, Path] = {}
    for author in authors:
        reference = args.build / f"{author}.json"
        try:
            hp.fit(
                args.build / "fit" / author,
                reference,
                name=author,
                features=AUTHORS[author].features,
            )
            hp.calibrate(reference, background)
            references[author] = reference
        except HandprintError as error:
            print(f"warning: could not fit {author}: {error}")

    report = {
        "authors": authors,
        "separation": suite_separation(hp, args.build, background_ref, authors),
        "interpretability": suite_interpretability(
            hp, args.build, background_ref, authors, args.top
        ),
        "anticaricature": suite_anticaricature(hp, args.build, references),
    }
    out = args.build / "authors.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"wrote {out}")

    failures = []
    if report["separation"]["rank_accuracy"] < 1.0:
        failures.append("separation: a held-out chapter did not rank its own author first")
    for author, result in report["interpretability"].items():
        if not result["passed"]:
            failures.append(f"interpretability: {author} missed its named axes")
    for name, result in report["anticaricature"].items():
        if not result["passed"]:
            failures.append(f"anti-caricature: {name} did not trip its upper bands")
    for failure in failures:
        print(f"FAIL {failure}")
    print(
        f"abstention rate {report['separation']['abstention_rate']:.2f} "
        f"(reported, not a failure)"
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
