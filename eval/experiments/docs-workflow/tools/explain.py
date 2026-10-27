#!/usr/bin/env python3
"""Explain vs instruct: share of sentences that explain (causal/contrastive/definitional
markers) vs sentences that instruct (open with an imperative verb), per 100 sentences.
A crude, transparent proxy; code is stripped first as for handprint."""
import re, sys, statistics as st
from pathlib import Path
sys.path.insert(0, "/home/user/handprint/eval/registers")
from fetch_docs import strip_code
EXPLAIN = re.compile(r"\b(because|since\b(?! \d)|so that|which means|that means|this means|the reason|is why|that's why|in other words|instead of|rather than|unlike|therefore|under the hood|behind the scenes|what happens|happens when|works by|the idea|the point is|this is how|so the|so it|so you|otherwise|which is why|is what)\b", re.I)
IMPER = re.compile(r"^(?:\d+\.\s*|[-*]\s*)?(click|press|run|type|add|open|choose|create|set|make|select|drag|write|install|edit|change|copy|replace|put|use|save|close|go|start|build|try|look|check|delete|remove|move|enter|pick|draw|place|name|leave|keep|call|import|define|register|pass|give)\b", re.I)
def sentences(text):
    text = re.sub(r"`[^`]*`", "X", text)
    parts = re.split(r"(?<=[.!?])\s+|\n\s*\n|\n(?=\s*(?:\d+\.|[-*])\s)", text)
    return [p.strip() for p in parts if len(p.split()) >= 4]
def measure(path, strip=True):
    t = Path(path).read_text()
    if strip: t = strip_code(t, ".md")
    ss = sentences(t); n = len(ss) or 1
    e = sum(1 for s in ss if EXPLAIN.search(s)); i = sum(1 for s in ss if IMPER.match(s))
    return {"sentences": n, "explain_per100": 100 * e / n, "instruct_per100": 100 * i / n, "words": len(t.split())}
if __name__ == "__main__":
    for p in sys.argv[1:]:
        m = measure(p, strip=not p.split("/")[-1].startswith("docs-"))
        print(f"{Path(p).parent.name}/{Path(p).name}: words={m['words']} explain={m['explain_per100']:.1f} instruct={m['instruct_per100']:.1f}")
