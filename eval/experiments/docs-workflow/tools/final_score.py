#!/usr/bin/env python3
"""Fit v1 on the full corpus minus held-out projects, calibrate, then score
held-out human pages and every draft version. Writes exp/scores.json."""
import json, random, shutil, subprocess, sys
from pathlib import Path
S = Path("/tmp/claude-0/-home-user/67cb6ead-4593-5b3c-af70-6c2db70b2853/scratchpad")
HP = "/home/user/handprint/target/release/handprint"
sys.path.insert(0, "/home/user/handprint/eval/registers")
from fetch_docs import strip_code
NAMES = "rust-book rust-by-example cargo rust-reference node rails-guides jekyll eslint prettier jest babel vue-docs svelte nextjs gatsby storybook tailwind fastapi sqlalchemy flask requests django drf celery scrapy numpy pandas scikit-learn scipy matplotlib jupyter-nb cpython pytorch tensorflow-docs kubernetes helm istio envoy prometheus docker airflow spark hugo-docs typescript llvm openssl grpc terraform".split()
HELD = {"django", "kubernetes", "numpy", "fastapi", "helm", "rails-guides"}
FEATURES = "punct,sentence-rhythm,punchline,biber,readability,frames,formality,devices,syntax,mfw,ngrams,doc-style,hyland,marketing"

def proj(p):
    n = p.name[5:] if p.name.startswith("docs-") else p.name
    return max((x for x in NAMES if n.startswith(x + "-")), key=len, default=None)

fit = S / "fit1" / "software-docs"; bg = S / "fit1" / "bg"; held = S / "fit1" / "held"
shutil.rmtree(S / "fit1", ignore_errors=True)
for d in (fit, bg, held): d.mkdir(parents=True)
pages = {}
for c in ("corpus", "corpus2", "corpus3"):
    for p in (S / c / "software-docs").glob("*.md"):
        pages[p.name] = p
groups = {}
for name, p in pages.items():
    groups.setdefault(proj(p), []).append(p)
heldpages = []
for g, ps in sorted(groups.items(), key=lambda x: str(x[0])):
    ps.sort(); random.Random(str(g)).shuffle(ps)
    if g in HELD:
        heldpages += ps[:5]
        for p in ps[:5]: shutil.copy(p, held / p.name)
        continue
    for p in ps: shutil.copy(p, fit / p.name)
    if len(ps) >= 10:
        (bg / g).mkdir()
        for p in ps[:40]: shutil.copy(p, bg / g / p.name)
nfit = len(list(fit.iterdir()))
ref = S / "refs" / "docs-v1.json"
subprocess.run([HP, "fit", str(fit), "-o", str(ref), "--no-config", "--name", "software-docs",
                "--version", "v1", "--metric", "burrows", "--features", FEATURES], check=True,
               capture_output=True)
subprocess.run([HP, "calibrate", str(ref), "--background", str(bg), "--metric", "burrows"],
               check=True, capture_output=True)

def score(ref, path, stripped=True):
    text = path.read_text()
    if stripped: text = strip_code(text, ".md")
    out = subprocess.run([HP, "critique", "-r", str(ref), "--max-findings", "500", "-"],
                         input=text, capture_output=True, text=True)
    r = json.loads(out.stdout); f = r.get("findings", [])
    sev = lambda s, pre="": sum(1 for x in f if x["severity"] == s and x["id"].startswith(pre))
    return {"tokens": r["doc"]["tokens"], "p_same": (r.get("verdict") or {}).get("p_same_author"),
            "high": sev("high"), "medium": sev("medium"), "total": r.get("findings_total"),
            "docstyle_high": sev("high", "lex.doc-style"), "docstyle_med": sev("medium", "lex.doc-style"),
            "ids_high": sorted(x["id"] for x in f if x["severity"] == "high")}

res = {"fit_docs": nfit, "held_projects": sorted(HELD), "human": {}, "drafts": {}}
for ref_name in ("docs-v0.json", "docs-v1.json"):
    r = S / "refs" / ref_name
    # Held-out human pages are already code-stripped by fetch_docs.
    res["human"][ref_name] = {p.name: score(r, p, stripped=False) for p in heldpages}
    res["drafts"][ref_name] = {}
    for doc in sorted(d.name for d in (S / "exp").iterdir() if d.is_dir()):
        vs = sorted((S / "exp" / doc).glob("draft*.md")) + [S / "exp" / doc / "final.md", S / "exp" / doc / "published.md"]
        res["drafts"][ref_name][doc] = {v.name: score(r, v) for v in vs if v.exists()}
(S / "exp" / "scores.json").write_text(json.dumps(res, indent=1))
print("fit docs", nfit, "held pages", len(heldpages))
