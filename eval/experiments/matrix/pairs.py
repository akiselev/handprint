#!/usr/bin/env python3
"""Every author against every reference: the full self-attribution matrix.

The matrix's first gate is "the source passage must rank its own author first".
That gate passed 13 of 14 sampled passages when the comparison set held 14
references and started failing as soon as the set grew, which means it was
partly measuring who else was in the room. This script measures the room.

For each author it takes `--samples` held-out passages of about 1,200 words and
scores each against all references, then reports:

* **self-rank** — where the author's own reference lands. 1 is the gate.
* **margin** — own score minus the best rival. Negative means the gate failed.
* **promiscuity**, per reference — the mean score it gives to passages by
  everyone else. A reference that says yes to everything makes every ranking
  that includes it unreliable.

Random author-to-author pairs are drawn from the authors that *pass*, because a
pair whose source does not rank its own author first is not a transfer cell,
it is a mislabelled one.

    python3 pairs.py --refs ../../build/refs/v3 --corpora ../../corpora \
        --out ../../build/matrix/pairs.json
"""

from __future__ import annotations

import argparse
import json
import random
import re
import statistics
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent


def passage(document: Path, words: int) -> str:
    """A ~`words`-word run of paragraphs from the middle of a document."""
    text = document.read_text(encoding="utf-8", errors="replace")
    paragraphs = [" ".join(p.split()) for p in re.split(r"\n\s*\n", text) if p.strip()]
    # Skip a heading-only first paragraph; keep short ones, they are dialogue.
    if paragraphs and len(paragraphs[0].split()) < 8:
        paragraphs = paragraphs[1:]
    out, total = [], 0
    for para in paragraphs:
        out.append(para)
        total += len(para.split())
        if total >= words:
            break
    return "\n\n".join(out)


def score(handprint: Path, reference: Path, document: Path) -> float | None:
    out = subprocess.run(
        [str(handprint), "critique", "-r", str(reference), str(document)],
        capture_output=True, text=True)
    if not out.stdout.strip():
        return None
    return json.loads(out.stdout)["verdict"].get("p_same_author") or 0.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refs", type=Path, required=True)
    parser.add_argument("--corpora", type=Path, required=True)
    parser.add_argument("--handprint", type=Path,
                        default=HERE.parents[2] / "target/release/handprint")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=2)
    parser.add_argument("--words", type=int, default=1200)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--pairs", type=int, default=12)
    parser.add_argument("--seed", type=int, default=20260808)
    args = parser.parse_args()

    references = {p.stem: p for p in sorted(args.refs.glob("*.json"))}
    work = args.out.parent / "pair-passages"
    work.mkdir(parents=True, exist_ok=True)

    # Sample passages. Seeded by author so re-running compares like with like.
    samples: dict[str, list[Path]] = {}
    for author in references:
        docs = sorted((args.corpora / author).glob("*.md"))
        if not docs:
            continue
        rng = random.Random(f"{args.seed}:{author}")
        rng.shuffle(docs)
        picked = []
        for i, doc in enumerate(docs[: args.samples]):
            body = passage(doc, args.words)
            path = work / f"{author}--{i}.md"
            path.write_text(body + "\n", encoding="utf-8")
            picked.append(path)
        samples[author] = picked

    jobs = [(author, path, ref_name, ref_path)
            for author, paths in samples.items() for path in paths
            for ref_name, ref_path in references.items()]
    print(f"{len(jobs)} scorings across {len(samples)} authors, {args.jobs} at a time")

    def run(job):
        author, path, ref_name, ref_path = job
        return (author, path.name, ref_name,
                score(args.handprint, ref_path, path))

    table: dict[str, dict[str, float]] = {}
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        for author, doc, ref_name, value in pool.map(run, jobs):
            if value is None:
                continue
            table.setdefault(doc, {})[ref_name] = value

    # ---- gate 1, per passage
    gate = []
    for doc, scores in sorted(table.items()):
        author = doc.split("--")[0]
        ranked = sorted(scores.items(), key=lambda kv: -kv[1])
        names = [n for n, _ in ranked]
        rank = names.index(author) + 1 if author in names else None
        rival = next((v for n, v in ranked if n != author), 0.0)
        gate.append({
            "doc": doc, "author": author, "rank": rank,
            "own": scores.get(author), "best_rival": ranked[0][0],
            "margin": (scores.get(author) or 0.0) - rival,
        })

    passed = [g for g in gate if g["rank"] == 1]
    print(f"\ngate 1: {len(passed)}/{len(gate)} passages rank their own author first")
    print(f"{'passage':34} {'rank':>4} {'own':>6} {'margin':>7}  beaten by")
    for g in sorted(gate, key=lambda g: -(g["rank"] or 0)):
        if g["rank"] == 1:
            continue
        print(f"{g['doc']:34} {g['rank']:4} {g['own']:6.3f} {g['margin']:7.3f}  "
              f"{g['best_rival']}")

    # ---- promiscuity, per reference
    promiscuity = {}
    for ref in references:
        foreign = [s[ref] for doc, s in table.items()
                   if doc.split("--")[0] != ref and ref in s]
        promiscuity[ref] = {
            "mean_foreign": statistics.mean(foreign) if foreign else None,
            "max_foreign": max(foreign) if foreign else None,
            "own_mean": statistics.mean(
                [s[ref] for doc, s in table.items()
                 if doc.split("--")[0] == ref and ref in s] or [0.0]),
        }
    print(f"\n{'reference':24} {'own':>6} {'foreign':>8} {'ratio':>7}")
    ordered = sorted(promiscuity.items(),
                     key=lambda kv: kv[1]["mean_foreign"] or 0, reverse=True)
    for ref, row in ordered:
        ratio = (row["own_mean"] / row["mean_foreign"]) if row["mean_foreign"] else float("inf")
        print(f"{ref:24} {row['own_mean']:6.3f} {row['mean_foreign']:8.4f} {ratio:7.1f}")

    # ---- random pairs from the authors that pass
    eligible = sorted({g["author"] for g in passed})
    rng = random.Random(args.seed)
    pairs = []
    while len(pairs) < args.pairs and len(eligible) > 1:
        a, b = rng.sample(eligible, 2)
        if (a, b) in pairs:
            continue
        pairs.append((a, b))
    print(f"\n{args.pairs} random pairs drawn from the {len(eligible)} "
          f"authors whose passages pass gate 1:")
    for a, b in pairs:
        print(f"  {a} -> {b}")

    args.out.write_text(json.dumps({
        "table": table, "gate": gate, "promiscuity": promiscuity,
        "eligible": eligible, "pairs": pairs,
    }, indent=1), encoding="utf-8")
    print(f"\nwrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
