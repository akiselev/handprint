#!/usr/bin/env python3
"""Do the register references tell registers apart?

The author version of this question is gate 1 in `../experiments/matrix/pairs.py`
and it has a clean answer: a passage either does or does not rank its own author
first. Registers are different, and the difference matters before any number
here is read.

**Registers are not mutually exclusive.** An advice column is a kind of blog. A
recipe is a kind of how-to. A sports report is a news report. Authorship is a
partition; register is a space with neighbours, and a reference that puts a
recipe closest to how-to is not wrong, it is describing the space. So the
report below gives both the self-rank *and* what beat it, because "beaten by
how-to" and "beaten by legal-terms" mean opposite things.

    python3 discriminate.py --refs ../build/refs/register --samples 5
"""

from __future__ import annotations

import argparse
import collections
import json
import random
import statistics
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent


def score(handprint: Path, reference: Path, document: Path) -> float:
    out = subprocess.run(
        [str(handprint), "critique", "-r", str(reference), str(document)],
        capture_output=True, text=True)
    if not out.stdout.strip():
        return 0.0
    return json.loads(out.stdout)["verdict"].get("p_same_author") or 0.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refs", type=Path, default=HERE.parent / "build/refs/register")
    parser.add_argument("--corpora", type=Path, default=HERE.parent / "corpora")
    parser.add_argument("--handprint", type=Path,
                        default=HERE.parents[1] / "target/release/handprint")
    parser.add_argument("--out", type=Path, default=HERE.parent / "build/register-discrimination.json")
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--jobs", type=int, default=10)
    parser.add_argument("--seed", type=int, default=20260808)
    args = parser.parse_args()

    references = {p.stem: p for p in sorted(args.refs.glob("*.json"))}
    probes: list[tuple[str, Path]] = []
    for register in references:
        docs = sorted((args.corpora / register).glob("*.md"))
        rng = random.Random(f"{args.seed}:{register}")
        rng.shuffle(docs)
        probes += [(register, d) for d in docs[: args.samples]]

    jobs = [(reg, doc, rn, rp) for reg, doc in probes for rn, rp in references.items()]
    print(f"{len(jobs)} scorings: {len(probes)} probes x {len(references)} references")

    table: dict[str, dict[str, float]] = collections.defaultdict(dict)
    owner: dict[str, str] = {}

    def run(job):
        reg, doc, rn, rp = job
        return doc.name, reg, rn, score(args.handprint, rp, doc)

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        for doc, reg, rn, value in pool.map(run, jobs):
            table[doc][rn] = value
            owner[doc] = reg

    per_register: dict[str, list[int]] = collections.defaultdict(list)
    beaten_by: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for doc, scores in table.items():
        register = owner[doc]
        ranked = sorted(scores.items(), key=lambda kv: -kv[1])
        names = [n for n, _ in ranked]
        per_register[register].append(names.index(register) + 1)
        for name, _ in ranked:
            if name == register:
                break
            beaten_by[register][name] += 1

    print(f"\n{'register':16} {'top-1':>6} {'top-3':>6} {'median rank':>12}  usually beaten by")
    rows = []
    for register, ranks in sorted(per_register.items(),
                                  key=lambda kv: statistics.median(kv[1])):
        top1 = sum(1 for r in ranks if r == 1)
        top3 = sum(1 for r in ranks if r <= 3)
        rivals = ", ".join(f"{n}({c})" for n, c in beaten_by[register].most_common(3))
        print(f"{register:16} {top1}/{len(ranks):<4} {top3}/{len(ranks):<4} "
              f"{statistics.median(ranks):12.1f}  {rivals}")
        rows.append({"register": register, "ranks": ranks, "top1": top1,
                     "top3": top3, "beaten_by": dict(beaten_by[register])})

    total = sum(len(r) for r in per_register.values())
    hits1 = sum(sum(1 for x in r if x == 1) for r in per_register.values())
    hits3 = sum(sum(1 for x in r if x <= 3) for r in per_register.values())
    print(f"\noverall: top-1 {hits1}/{total} ({100*hits1/total:.0f}%), "
          f"top-3 {hits3}/{total} ({100*hits3/total:.0f}%)")

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps({"rows": rows, "table": table}, indent=1),
                        encoding="utf-8")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
