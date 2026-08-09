#!/usr/bin/env python3
"""Score every arm of the transfer against both references, as one table.

The number that matters is not any single cell. It is whether the styled draft
moved *toward* the target reference and *away* from the source reference, and
whether the typography-only arm already accounts for most of the movement. That
last column is why this script exists rather than a shell loop: the first run
of this experiment credited a search-and-replace with a voice transfer.

    python3 arms.py build/refs build/transfer plain plain-typo draft1
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

HANDPRINT = "../target/release/handprint"
GATED_FAMILIES = {"device", "frames", "syntax", "rhythm"}


def critique(reference: Path, document: Path) -> dict | None:
    result = subprocess.run(
        [HANDPRINT, "critique", "-r", str(reference), str(document),
         "--max-findings", "400"],
        capture_output=True, text=True,
    )
    return json.loads(result.stdout) if result.stdout.strip() else None


def main() -> int:
    refs, transfer = Path(sys.argv[1]), Path(sys.argv[2])
    arms = sys.argv[3:]
    target = refs / "terry-pratchett.json"
    source = refs / "douglas-adams.json"

    print(f"{'arm':14s} {'toward PRATCHETT':>26s} {'away from ADAMS':>22s} "
          f"{'gated med+':>11s} {'guards':>14s}")
    print(f"{'':14s} {'p_same':>10s} {'dist':>8s} {'pass':>6s} "
          f"{'p_same':>10s} {'dist':>10s}")
    for arm in arms:
        document = transfer / f"a2p-{arm}.md"
        if not document.exists():
            print(f"{arm:14s} MISSING {document}")
            continue
        t, s = critique(target, document), critique(source, document)
        if t is None or s is None:
            print(f"{arm:14s} SCORING FAILED")
            continue
        tv, sv = t["verdict"], s["verdict"]
        # Gate condition 3 counts Medium-or-above findings in either direction
        # on the four families; being under the ceiling is not the same as
        # being inside the band.
        gated = sum(
            1 for f in t["findings"]
            if f["family"] in GATED_FAMILIES and f["severity"] in ("medium", "high")
        )
        g = t["guards"]
        print(
            f"{arm:14s} {tv.get('p_same_author'):10.4f} {tv.get('distance'):8.4f} "
            f"{str(tv.get('pass')):>6s} "
            f"{sv.get('p_same_author'):10.4f} {sv.get('distance'):10.4f} "
            f"{gated:11d} "
            f"{'canary' if g['canary_ok'] else 'CANARY!':>7s}"
            f"{'/drift' if g['drift_ok'] else '/DRIFT!':>7s}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
