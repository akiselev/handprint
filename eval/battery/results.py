#!/usr/bin/env python3
"""Render RESULTS.md from the battery's JSON output.

**Metrics only, never corpus text.** The battery runs against in-copyright
corpora on a local machine; what is committed is what the numbers were, with
enough provenance to re-derive them and no excerpt of anything.

A run that has not happened produces a file that says so. A results table with
plausible-looking placeholder numbers would be worse than an empty one, because
the numbers would end up quoted.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

HEADER = """# Battery results

Metrics only. No corpus text appears in this file, and none ever will: the
author battery runs against in-copyright corpora that exist on one machine.

Regenerate with `make battery` from `eval/`.
"""

NOT_RUN = """
## Status: not run

No battery output found under `build/`. The corpora are not in this repository
and cannot be — see `corpora/README.md` for how to assemble them locally.

Nothing in the documentation quotes a number from this directory, and nothing
should until this file says otherwise.
"""


def render_authors(data: dict) -> str:
    out = ["\n## Author battery\n"]
    separation = data.get("separation", {})
    out.append(
        f"Rank accuracy on held-out chapters: "
        f"**{separation.get('rank_accuracy', 0):.2f}** "
        f"({len(separation.get('rank', {}))} author(s))\n"
    )
    out.append(
        f"General Imposters abstention rate: "
        f"**{separation.get('abstention_rate', 0):.2f}** — reported, not "
        f"suppressed. A verifier that cannot abstain is not a verifier.\n"
    )

    out.append("\n### (a) Separation\n")
    out.append("| author | ranked first | correct |")
    out.append("|---|---|---|")
    for author, row in sorted(separation.get("rank", {}).items()):
        out.append(f"| {author} | {row['got']} | {'yes' if row['correct'] else 'NO'} |")

    out.append("\n### (b) Interpretability\n")
    out.append("| author | named axes hit | required | ○ cells low | pass |")
    out.append("|---|---|---|---|---|")
    for author, row in sorted(data.get("interpretability", {}).items()):
        low = row.get("low_ok", {})
        low_text = ", ".join(f"{k}={'yes' if v else 'NO'}" for k, v in low.items()) or "—"
        out.append(
            f"| {author} | {len(row['high_hit'])} | {row['high_required']} | "
            f"{low_text} | {'yes' if row['passed'] else 'NO'} |"
        )

    out.append("\n### (c) Anti-caricature\n")
    out.append("| pastiche | upper bands tripped | gate | pass |")
    out.append("|---|---|---|---|")
    for name, row in sorted(data.get("anticaricature", {}).items()):
        out.append(
            f"| {name} | {'yes' if row['over_fired_ok'] else 'NO'} | "
            f"{row['gate']} | {'yes' if row['passed'] else 'NO'} |"
        )
    return "\n".join(out) + "\n"


def render_registers(data: dict) -> str:
    out = ["\n## Register battery\n"]
    out.append("| register | named axes hit | required | booster:hedge | pass |")
    out.append("|---|---|---|---|---|")
    ratios = data.get("ratios", {})
    for register, row in sorted(data.get("claims", {}).items()):
        ratio = ratios.get(register, {}).get("booster_hedge")
        ratio_text = f"{ratio:.2f}" if isinstance(ratio, float) else "—"
        out.append(
            f"| {register} | {len(row['high_hit'])} | {row['high_required']} | "
            f"{ratio_text} | {'yes' if row['passed'] else 'NO'} |"
        )
    return "\n".join(out) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", type=Path, default=Path("build"))
    parser.add_argument("--out", type=Path, default=Path("RESULTS.md"))
    args = parser.parse_args()

    authors_path = args.build / "authors.json"
    registers_path = args.build / "registers.json.report"
    body = HEADER

    if not authors_path.exists() and not registers_path.exists():
        args.out.write_text(HEADER + NOT_RUN, encoding="utf-8")
        print(f"wrote {args.out} (no run found)")
        return 0

    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    body += f"\nGenerated {stamp}.\n"
    if authors_path.exists():
        body += render_authors(json.loads(authors_path.read_text(encoding="utf-8")))
    if registers_path.exists():
        body += render_registers(json.loads(registers_path.read_text(encoding="utf-8")))
    args.out.write_text(body, encoding="utf-8")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
