#!/usr/bin/env python3
"""Score every matrix cell: toward the target, away from the source.

Reports `p_same_author` toward both references and the count of Medium-or-above
findings on the four gated families. At ~1,000 words the whole-profile figure is
below the reliable-attribution floor, so the columns that carry the argument are
the *direction* of movement and the gated-finding count, not the third decimal
of any single number.

    python3 score.py cells.json build/refs/voice build/matrix
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

HANDPRINT = "../target/release/handprint"
GATED = {"device", "frames", "syntax", "rhythm"}


def critique(reference: Path, document: Path) -> dict | None:
    r = subprocess.run(
        [HANDPRINT, "critique", "-r", str(reference), str(document),
         "--max-findings", "400"],
        capture_output=True, text=True,
    )
    return json.loads(r.stdout) if r.stdout.strip() else None


def verify(reference: Path, target: Path, impostors: Path, doc: Path) -> str:
    r = subprocess.run(
        [HANDPRINT, "verify", "-r", str(reference), "--target", str(target),
         "--impostors", str(impostors), str(doc), "--json"],
        capture_output=True, text=True,
    )
    if not r.stdout.strip():
        return "?"
    d = json.loads(r.stdout)
    return {"target_favoured": "favoured", "target_disfavoured": "disfavoured",
            "abstain": "abstain"}.get(d["verdict"], d["verdict"])


def main() -> int:
    cells = json.loads(Path(sys.argv[1]).read_text())
    refs, root = Path(sys.argv[2]), Path(sys.argv[3])
    rows = []
    for c in cells:
        out = root / "out" / f"{c['id']}-{c['src']}-to-{c['dst']}.md"
        src = root / "src" / f"{c['source_author']}.md"
        if not out.exists():
            print(f"{c['id']}  PENDING", flush=True)
            continue
        tgt_ref, src_ref = refs / f"{c['target_author']}.json", refs / f"{c['source_author']}.json"
        if not tgt_ref.exists() or not src_ref.exists():
            print(f"{c['id']}  MISSING REFERENCE", flush=True)
            continue

        # Draft against both poles, and the untouched source against both, so
        # every cell reports movement rather than a bare level.
        d_t, d_s = critique(tgt_ref, out), critique(src_ref, out)
        s_t, s_s = critique(tgt_ref, src), critique(src_ref, src)
        if not all((d_t, d_s, s_t, s_s)):
            print(f"{c['id']}  SCORING FAILED", flush=True)
            continue
        gated = sum(1 for f in d_t["findings"]
                    if f["family"] in GATED and f["severity"] in ("medium", "high"))
        row = {
            "id": c["id"], "axis": c["axis"], "shift": c["shift"],
            "src": c["src"], "dst": c["dst"],
            "src_to_target": s_t["verdict"].get("p_same_author"),
            "draft_to_target": d_t["verdict"].get("p_same_author"),
            "src_to_source": s_s["verdict"].get("p_same_author"),
            "draft_to_source": d_s["verdict"].get("p_same_author"),
            "dist_to_source_before": s_s["verdict"].get("distance"),
            "dist_to_source_after": d_s["verdict"].get("distance"),
            "gated": gated,
            "pass": d_t["verdict"].get("pass"),
            "findings": d_t["findings_total"],
            "tokens": d_t["doc"]["tokens"],
            "canary": d_t["guards"]["canary_ok"],
            "drift": d_t["guards"]["drift_ok"],
        }
        rows.append(row)
        print(f"{row['id']}  {row['src'][:9]:9s}->{row['dst'][:11]:11s} "
              f"toward {row['src_to_target']:.3f}->{row['draft_to_target']:.3f}  "
              f"away {row['src_to_source']:.3f}->{row['draft_to_source']:.3f}  "
              f"gated={row['gated']:2d} n={row['findings']:3d} tok={row['tokens']}",
              flush=True)
    Path(root / "results.json").write_text(json.dumps(rows, indent=2))
    print(f"\nwrote {len(rows)} row(s) to {root/'results.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
