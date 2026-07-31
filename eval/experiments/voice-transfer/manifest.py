#!/usr/bin/env python3
"""Write the capstone run manifest.

A number without a manifest is not reproducible and should not be quoted, so
this runs before the transfers rather than after them. What it records is
everything that would change a result: seeds, source ids, feature set, and the
handprint version that produced the references.

The Gutenberg ids live here rather than in a config file so that the corpus a
run used shows up in the same diff as the code that used it.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

GUTENBERG_IDS = {
    "wodehouse": ["8164", "7471", "2233", "6753"],
    "twain": ["3178", "74", "76", "245", "119"],
    "poe": ["2147", "2148", "2149", "2150", "25525"],
    "austen": ["1342", "161", "141", "158", "121"],
    # Hemingway's US-PD works are not all on Gutenberg. Saying so is better
    # than an id list that half works.
    "hemingway-early": "local; not all of its US-PD works are on Gutenberg",
}

JURISDICTION = (
    "US public domain for all five under the 95-year rule. Worldwide public "
    "domain for twain, poe and austen only — Wodehouse (d. 1975) and Hemingway "
    "(d. 1961) remain in copyright in life+70 jurisdictions until 2046 and "
    "2032. Data-only artifacts (fitted references, calibration, band edges) "
    "publish for all five with this basis stated; text-bearing artifacts "
    "(move-index sidecars, full transcripts) publish only for the "
    "worldwide-PD three."
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("--features", required=True)
    parser.add_argument("--handprint", default="unknown")
    parser.add_argument(
        "--judge",
        default=None,
        help="judge model id, when the transfers have been run",
    )
    args = parser.parse_args()

    manifest = {
        "generated_by": "eval/experiments/voice-transfer/run.sh",
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "seed": args.seed,
        "handprint": args.handprint,
        "features": args.features.split(","),
        "gutenberg_ids": GUTENBERG_IDS,
        "judge_model": args.judge,
        "jurisdiction": JURISDICTION,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
