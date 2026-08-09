#!/usr/bin/env python3
"""Do held-out chapters prefer their own author's reference?

The references are fitted on the corpora this repository just built, so the
only question that means anything is asked with documents the fit never saw.
Each held-out chapter is scored against both references; the reference with the
higher `p_same_author` wins. Chance is 50%.

This validates the corpora and the references. It says nothing about a
transfer, which is scored on text that is in no corpus at all.

    python3 separation.py build/sep douglas-adams terry-pratchett
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

HANDPRINT = "../target/release/handprint"


def score(reference: Path, document: Path) -> float | None:
    result = subprocess.run(
        [HANDPRINT, "critique", "-r", str(reference), str(document)],
        capture_output=True,
        text=True,
    )
    if not result.stdout.strip():
        return None
    return json.loads(result.stdout)["verdict"].get("p_same_author")


def main() -> int:
    root = Path(sys.argv[1])
    authors = sys.argv[2:]
    references = {a: root / f"{a}.json" for a in authors}
    for a, r in references.items():
        if not r.exists():
            print(f"missing reference {r}", file=sys.stderr)
            return 2

    totals: dict[str, list[int]] = {a: [0, 0] for a in authors}
    for true_author in authors:
        held = sorted((root / "holdout" / true_author).glob("*.md"))
        print(f"\n== {true_author}: {len(held)} held-out document(s)")
        for document in held:
            scores = {a: score(references[a], document) for a in authors}
            if any(v is None for v in scores.values()):
                print(f"  {document.name[:44]:46s} SCORING FAILED")
                continue
            best = max(scores, key=lambda a: scores[a])
            ok = best == true_author
            totals[true_author][0] += int(ok)
            totals[true_author][1] += 1
            detail = "  ".join(f"{a[:9]}={scores[a]:.3f}" for a in authors)
            print(f"  {document.name[:44]:46s} {detail}  -> {'OK' if ok else 'WRONG'}")

    print("\n== summary")
    hit = sum(v[0] for v in totals.values())
    n = sum(v[1] for v in totals.values())
    for a, (correct, count) in totals.items():
        print(f"  {a:18s} {correct}/{count}")
    print(f"  {'overall':18s} {hit}/{n} ({100.0 * hit / n:.0f}%), chance is 50%")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
