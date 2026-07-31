#!/usr/bin/env python3
"""The register battery: tech-blog vs marketing vs agent prose.

Closes the loop back to handprint's original use cases. Three claims, from the
register coverage matrix:

* **Blogs** are high on the apparatus dimensions — asides, links, footnotes,
  rhetorical questions — and on hedging.
* **Marketing** is high on the directive and pressure families, and on the
  booster:hedge ratio.
* **Agent prose** reproduces the PNAS LLM signature *directionally*: more
  nominalizations and participial clauses, fewer contractions and first-person
  pronouns. Direction is the claim, not magnitude — the PNAS effect sizes are
  from a different corpus and quoting them here would be borrowing someone
  else's number.

The agent-prose corpus comes from `handprint extract claude-code`, which is
private data. It never leaves the machine and no vocabulary derived from it
ships without the document-frequency cull and a manual review.

    python3 registers/run.py --corpora corpora --build build
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "battery"))

from hp import Handprint, parse_profile_table  # noqa: E402
from matrix import REGISTERS, matches  # noqa: E402


def booster_hedge_ratio(profile_table: list[tuple[str, float]]) -> float | None:
    """Boosters over hedges, from the Hyland category rates.

    The single dimension that separates the marketing register from the blog
    register most cleanly, and the one that doubles as an AI tell: humans hedge
    about 40% more than models in advice text.
    """
    by_name = {name.replace(":", "."): value for name, value in profile_table}
    boosters = by_name.get("lex.hyland.cat.boosters")
    hedges = by_name.get("lex.hyland.cat.hedges")
    if boosters is None or hedges is None:
        return None
    return boosters / hedges if hedges else float("inf")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpora", type=Path, default=Path("corpora"))
    parser.add_argument("--build", type=Path, default=Path("build"))
    parser.add_argument("--handprint", default="handprint")
    parser.add_argument("--top", type=int, default=40)
    args = parser.parse_args()

    hp = Handprint(command=args.handprint)
    args.build.mkdir(parents=True, exist_ok=True)

    registers = [r for r in REGISTERS if (args.corpora / r).is_dir()]
    if not registers:
        raise SystemExit(
            f"no register corpora under {args.corpora}. Expected subdirectories "
            f"named {', '.join(REGISTERS)}; see corpora/README.md."
        )

    # One background over every register, so "unusual" means "unusual for this
    # comparison" rather than "unusual for English".
    background = args.build / "registers"
    background.mkdir(parents=True, exist_ok=True)
    for register in registers:
        target = background / register
        target.mkdir(parents=True, exist_ok=True)
        for path in (args.corpora / register).iterdir():
            if path.is_file():
                (target / path.name).write_bytes(path.read_bytes())

    features = REGISTERS[registers[0]].features
    reference = hp.fit(
        background,
        args.build / "registers.json",
        name="register-background",
        features=features,
    )
    hp.calibrate(reference, background)

    report: dict = {"registers": registers, "claims": {}, "ratios": {}}
    for register in registers:
        axes = REGISTERS[register]
        docs = sorted(p for p in (args.corpora / register).iterdir() if p.is_file())
        if not docs:
            continue
        table = hp.run("profile", "-r", str(reference), str(docs[0]), "--top", str(args.top))
        rows = parse_profile_table(table)
        ranked = [name.replace(":", ".") for name, _ in rows]
        by_name = {name.replace(":", "."): z for name, z in rows}

        hit = [c for c in axes.high if any(matches(d, [c]) for d in ranked)]
        low_ok = {}
        for claim in axes.low:
            zs = [z for name, z in by_name.items() if matches(name, [claim])]
            low_ok[claim] = bool(zs) and min(zs) < 0.0
        report["claims"][register] = {
            "high_hit": hit,
            "high_required": axes.required(),
            "low_ok": low_ok,
            "passed": len(hit) >= axes.required() and all(low_ok.values()),
        }
        raw = hp.profile(reference, docs[0])
        report["ratios"][register] = {
            "booster_hedge": booster_hedge_ratio(rows),
            "tokens": raw.get("tokens"),
        }

    out = args.build / "registers.json.report"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"wrote {out}")

    failures = [r for r, c in report["claims"].items() if not c["passed"]]
    for register in failures:
        print(f"FAIL register {register} missed its named axes")

    # The marketing/blog booster:hedge ordering is the register battery's
    # cheapest single claim, so it is checked explicitly rather than left to
    # the top-k intersection.
    ratios = report["ratios"]
    if "marketing" in ratios and "tech-blog" in ratios:
        marketing = ratios["marketing"]["booster_hedge"]
        blog = ratios["tech-blog"]["booster_hedge"]
        if marketing is not None and blog is not None and not marketing > blog:
            failures.append("booster:hedge")
            print(
                f"FAIL marketing booster:hedge {marketing:.2f} should exceed "
                f"blog {blog:.2f}"
            )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
