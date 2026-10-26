#!/usr/bin/env python3
"""Critique a draft the way the corpus was measured.

Applies fetch_docs.strip_code to the draft first (FINDINGS #26: any
normalisation applied to the corpus must be applied to the draft), runs
`handprint critique`, and prints a compact summary: verdict, doc-style
High/Medium counts, top findings. Writes the full JSON next to the draft.

usage: crit.py REFERENCE DRAFT.md [--max N]
"""
import json, subprocess, sys
from pathlib import Path
sys.path.insert(0, "/home/user/handprint/eval/registers")
from fetch_docs import strip_code  # noqa: E402

HP = "/home/user/handprint/target/release/handprint"

def main():
    ref, draft = sys.argv[1], Path(sys.argv[2])
    maxn = sys.argv[sys.argv.index("--max") + 1] if "--max" in sys.argv else "12"
    stripped = strip_code(draft.read_text(), ".md")
    out = subprocess.run([HP, "critique", "-r", ref, "--max-findings", maxn, "-"],
                         input=stripped, capture_output=True, text=True)
    if out.returncode == 2:
        print(out.stderr); sys.exit(2)
    rep = json.loads(out.stdout)
    full = subprocess.run([HP, "critique", "-r", ref, "--max-findings", "500", "-"],
                          input=stripped, capture_output=True, text=True)
    allrep = json.loads(full.stdout)
    jpath = draft.with_suffix(".critique.json")
    jpath.write_text(json.dumps(allrep, indent=1))
    f_all = allrep.get("findings", [])
    ds = [f for f in f_all if f["id"].startswith("lex.doc-style")]
    sev = lambda fs, s: sum(1 for f in fs if f.get("severity") == s)
    v = rep.get("verdict") or {}
    print(f"{draft.name}: tokens={rep['doc']['tokens']} tier={rep['doc']['length_tier']} "
          f"p_same={v.get('p_same_author')} pass={v.get('pass')} "
          f"findings_total={allrep.get('findings_total')} high={sev(f_all,'high')} med={sev(f_all,'medium')} "
          f"doc-style high={sev(ds,'high')} med={sev(ds,'medium')} "
          f"canary_ok={rep.get('guards',{}).get('canary_ok')}")
    for f in rep.get("findings", []):
        print(f"  [{f['severity']}] {f['id']} obs={f['observed']:.2f} band={f['target_band']} {f['direction']}"
              + (f" spans={len(f.get('spans',[]))}" if f.get('spans') else ""))
    # Span text for the agent, from the stripped text the spans index into.
    b = stripped.encode()
    for f in rep.get("findings", [])[:12]:
        for s, e in (f.get("spans") or [])[:3]:
            print(f"     {f['id']}: …{b[max(0,s-30):e+30].decode(errors='replace')!r}…")
    sys.exit(out.returncode)

main()
