#!/usr/bin/env python3
"""Does the imperative rate actually separate marketing from instructions?

`../battery/matrix.py` claims marketing is high on `dev.imperative_rate` and on
the directive and pressure categories. Until now there was no instructional
register in the battery, so the claim could not fail: *buy now* and *run this
command first* are both imperatives, and nothing in the corpus set could tell
them apart.

With `how-to`, `recipe` and `software-docs` on disk the claim is testable. If
marketing is not clearly above the instructional registers on `dev.imperative_rate`
then that dimension is measuring the *speech act*, not the *sell*, and the
marketing claim needs to rest on the pressure and directive categories instead.

    python3 imperatives.py --refs ../build/refs/register
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent

DIMENSIONS = [
    "dev.imperative_rate",
    "lex.marketing-eval.cat.pressure",
    "lex.marketing-eval.cat.directive",
    "biber.pron.p2",
    "lex.hyland.cat.boosters",
    "lex.hyland.cat.hedges",
]


def profile(handprint: Path, reference: Path, document: Path) -> dict[str, float]:
    """Observed rates by dimension, read from the table `profile` prints.

    Not `--json`: that emits the raw vector indexed by interner symbol with no
    names attached, so recovering `dev:imperative_rate` from it would mean
    re-deriving the symbol table here. The printed table is what a person
    reads and it carries the names, which is the same argument
    `battery/hp.py::parse_profile_table` makes.
    """
    out = subprocess.run(
        [str(handprint), "profile", "-r", str(reference), str(document),
         "--top", "4000"],
        capture_output=True, text=True)
    values: dict[str, float] = {}
    started = False
    for line in out.stdout.splitlines():
        if line.startswith("most unusual dimensions"):
            started = True
            continue
        if not started:
            continue
        parts = line.split()
        if len(parts) != 4 or parts[0] == "dimension":
            continue
        try:
            values[parts[0].replace(":", ".")] = float(parts[1])
        except ValueError:
            continue
    return values


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refs", type=Path, default=HERE.parent / "build/refs/register")
    parser.add_argument("--corpora", type=Path, default=HERE.parent / "corpora")
    parser.add_argument("--handprint", type=Path,
                        default=HERE.parents[1] / "target/release/handprint")
    parser.add_argument("--registers", nargs="*", default=[
        "marketing", "how-to", "recipe", "software-docs", "advice",
        "news", "encyclopedic", "reviews"])
    parser.add_argument("--samples", type=int, default=25)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--out", type=Path, default=HERE.parent / "build/imperatives.json")
    parser.add_argument("--anchor", type=Path,
                        default=HERE.parent / "build/refs/register-full-news.json",
                        help="the reference supplying the dimension set. It must "
                             "carry the `marketing` and `hyland` packs or the "
                             "pressure/directive/hedge columns come back empty, "
                             "which reads exactly like a rate of zero.")
    args = parser.parse_args()

    # One reference for everything: these are raw observed rates, and the
    # reference only supplies the dimension set. Using each register's own
    # reference would change the vocabulary-derived dimensions between rows.
    anchor = args.anchor
    if not anchor.exists():
        raise SystemExit(f"missing anchor reference {anchor}")

    jobs = []
    for register in args.registers:
        docs = sorted((args.corpora / register).glob("*.md"))[: args.samples]
        jobs += [(register, d) for d in docs]
    print(f"profiling {len(jobs)} documents across {len(args.registers)} registers")

    rows: dict[str, list[dict[str, float]]] = {r: [] for r in args.registers}

    def run(job):
        register, doc = job
        return register, profile(args.handprint, anchor, doc)

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        for register, values in pool.map(run, jobs):
            if values:
                rows[register].append(values)

    print(f"\n{'register':16}" + "".join(f"{d.split('.')[-1][:13]:>15}" for d in DIMENSIONS))
    summary = {}
    for register in args.registers:
        got = rows[register]
        if not got:
            print(f"{register:16} no data")
            continue
        medians = {}
        cells = ""
        for dim in DIMENSIONS:
            vals = [v[dim] for v in got if dim in v]
            if vals:
                medians[dim] = statistics.median(vals)
                cells += f"{medians[dim]:15.2f}"
            else:
                cells += f"{'-':>15}"
        summary[register] = {"documents": len(got), "medians": medians}
        print(f"{register:16}{cells}")

    imp = "dev.imperative_rate"
    have = {r: s["medians"].get(imp) for r, s in summary.items()
            if s["medians"].get(imp) is not None}
    if "marketing" in have:
        higher = [r for r, v in have.items()
                  if r != "marketing" and v >= have["marketing"]]
        print(f"\nmarketing {imp} = {have['marketing']:.2f}")
        if higher:
            print(f"registers at or above it: {', '.join(higher)}")
            print("=> the imperative rate is measuring the speech act, not the "
                  "sell. The marketing claim cannot rest on this dimension.")
        else:
            print("=> marketing is above every instructional register; the "
                  "claim survives its first real control.")

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(summary, indent=1), encoding="utf-8")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
