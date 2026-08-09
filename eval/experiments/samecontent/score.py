#!/usr/bin/env python3
"""Score every same-content rendering against every fitted reference.

Two numbers per rendering, and the second is the one arm A cannot produce:

* **toward** — `p_same_author` against the target the rendering was written for.
* **elsewhere** — the same score against the *other* targets in the set.

Because the content is identical across renderings, `elsewhere` is a direct
read on how much of a distance is voice rather than subject. In arm A a cell
that scores 0.3 toward its target might owe most of that to the source author's
nouns; here there are no source author's nouns, and every rendering shares the
same ones.

    python3 score.py --refs ../../build/refs/v3 --out results.json
"""

from __future__ import annotations

import argparse
import collections
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent

# Which reference each rendering was written toward. The blend names two, and
# is scored against both.
TARGETS = {
    "adams": ["douglas-adams"],
    "pratchett": ["terry-pratchett"],
    "jerome": ["jerome-k-jerome"],
    "austen": ["jane-austen"],
    "melville": ["herman-melville"],
    "poe": ["edgar-allan-poe"],
    "blend-adams-pratchett": ["douglas-adams", "terry-pratchett"],
    # The untouched source is scored too: it is the floor every rendering has
    # to beat, and without it "0.19 toward Adams" has nothing to mean.
    "source": [],
}


def critique(handprint: Path, reference: Path, document: Path) -> dict | None:
    out = subprocess.run(
        [str(handprint), "critique", "-r", str(reference), str(document),
         "--max-findings", "300"],
        capture_output=True, text=True)
    if not out.stdout.strip():
        return None
    return json.loads(out.stdout)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refs", type=Path, required=True)
    parser.add_argument("--handprint", type=Path,
                        default=HERE.parents[2] / "target/release/handprint")
    parser.add_argument("--out", type=Path, default=HERE / "results.json")
    args = parser.parse_args()

    docs = {"source": HERE / "source.md"}
    for path in sorted((HERE / "out").glob("*.md")):
        docs[path.stem] = path

    references = {p.stem: p for p in sorted(args.refs.glob("*.json"))}
    rows: dict[str, dict] = {}
    for name, path in docs.items():
        targets = TARGETS.get(name, [])
        row = {"targets": targets, "scores": {}, "findings": {}, "tokens": None}
        for ref_name, ref_path in references.items():
            report = critique(args.handprint, ref_path, path)
            if report is None:
                continue
            row["tokens"] = report["doc"]["tokens"]
            row["scores"][ref_name] = report["verdict"].get("p_same_author")
            severity = collections.Counter(
                f.get("severity") for f in report.get("findings", []))
            row["findings"][ref_name] = {
                "total": len(report.get("findings", [])),
                "gated": severity["high"] + severity["medium"],
            }
        rows[name] = row
        best = max(row["scores"].items(), key=lambda kv: kv[1] or -1)
        toward = [f"{t}={row['scores'].get(t):.3f}" for t in targets
                  if row["scores"].get(t) is not None]
        print(f"{name:24} {row['tokens']:5}tok  "
              f"toward: {', '.join(toward) or '-':32} "
              f"nearest: {best[0]} ({best[1]:.3f})")

    args.out.write_text(json.dumps(rows, indent=1), encoding="utf-8")
    print(f"\nwrote {args.out}")

    # The comparison the arm exists for: did writing toward a target actually
    # move this passage toward that target, relative to the untouched source?
    print("\n=== movement against the plain source ===")
    base = rows["source"]["scores"]
    for name, row in rows.items():
        for target in row["targets"]:
            before, after = base.get(target), row["scores"].get(target)
            if before is None or after is None:
                continue
            arrow = "up" if after > before else "DOWN"
            print(f"  {name:24} -> {target:20} {before:.3f} -> {after:.3f}  {arrow}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
