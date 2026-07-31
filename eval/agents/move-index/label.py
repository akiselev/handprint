#!/usr/bin/env python3
"""The comic-move labelling pass, and its reproducibility gate.

Two modes:

    label.py --corpus corpora/twain --out index/twain    # label
    label.py --agreement index/run-a index/run-b         # gate

The gate is the interesting half. A labelling pass whose two runs disagree
produces an index that retrieves by noise, and "the labels looked reasonable"
is not a check. Cohen's κ ≥ 0.7 *and* ≥ 80% raw agreement, both, because κ
alone is unstable when one label dominates and raw agreement alone is inflated
by exactly that.

The model is injected, like the judge harness: a labeller embedded in the
harness is one nobody can swap out to check the result.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Protocol

PROMPT = Path(__file__).parent / "prompt.md"
SCHEMA = Path(__file__).parent / "schema.json"

#: Chunk size for the labelling pass, in words. Matches codeindex's
#: retrieval-grade chunking closely enough that a label and a retrieved chunk
#: describe the same text.
CHUNK_WORDS = 220


class Labeller(Protocol):
    """Anything that can label a passage under the prompt."""

    def label(self, prompt: str, passage: str) -> dict: ...


def chunk(text: str, words: int = CHUNK_WORDS) -> list[str]:
    """Split on paragraph boundaries, accumulating to roughly `words`.

    Paragraph boundaries rather than a sliding window: a device that spans a
    paragraph break is rare, and a chunk that starts mid-sentence gives the
    labeller no setup to judge the payoff against.
    """
    out: list[str] = []
    buffer: list[str] = []
    count = 0
    for paragraph in text.split("\n\n"):
        paragraph = paragraph.strip()
        if not paragraph:
            continue
        buffer.append(paragraph)
        count += len(paragraph.split())
        if count >= words:
            out.append("\n\n".join(buffer))
            buffer, count = [], 0
    if buffer:
        out.append("\n\n".join(buffer))
    return out


def cohens_kappa(a: list[str], b: list[str]) -> float:
    """Cohen's κ for two label sequences over the same items."""
    if len(a) != len(b) or not a:
        raise ValueError("label sequences must be the same non-zero length")
    n = len(a)
    observed = sum(1 for x, y in zip(a, b) if x == y) / n
    count_a, count_b = Counter(a), Counter(b)
    expected = sum(
        (count_a[label] / n) * (count_b[label] / n)
        for label in set(count_a) | set(count_b)
    )
    if expected >= 1.0:
        # Every item got the same label from both runs. κ is undefined; raw
        # agreement is the honest number and the caller checks it separately.
        return float("nan")
    return (observed - expected) / (1 - expected)


def primary_labels(index: list[dict]) -> list[str]:
    """The first device of each entry, for the agreement calculation.

    Low-confidence entries are excluded rather than counted: including a
    labeller's own admission that it was guessing would inflate disagreement
    and make the gate measure the wrong thing.
    """
    return [
        (entry.get("devices") or ["none"])[0]
        for entry in index
        if entry.get("confidence") != "low"
    ]


def agreement(a_path: Path, b_path: Path) -> dict:
    a = json.loads(a_path.read_text(encoding="utf-8"))
    b = json.loads(b_path.read_text(encoding="utf-8"))
    if len(a) != len(b):
        raise SystemExit(
            f"the two runs labelled different numbers of chunks "
            f"({len(a)} vs {len(b)}); they are not comparable"
        )
    labels_a, labels_b = primary_labels(a), primary_labels(b)
    raw = sum(1 for x, y in zip(labels_a, labels_b) if x == y) / max(1, len(labels_a))
    kappa = cohens_kappa(labels_a, labels_b)
    passed = raw >= 0.80 and (kappa >= 0.70 or kappa != kappa)
    return {
        "chunks": len(a),
        "compared": len(labels_a),
        "raw_agreement": raw,
        "cohens_kappa": kappa,
        "passed": passed,
        "gate": "kappa >= 0.70 and raw >= 0.80",
    }


def unavailable_labeller() -> Labeller:
    raise SystemExit(
        "No labeller model was supplied. Drive this from a script that provides\n"
        "one; the harness deliberately does not embed a client."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument(
        "--agreement",
        nargs=2,
        type=Path,
        metavar=("RUN_A", "RUN_B"),
        help="compare two labelling runs and check the reproducibility gate",
    )
    parser.add_argument("--sample", type=int, default=50)
    args = parser.parse_args()

    if args.agreement:
        report = agreement(*args.agreement)
        print(json.dumps(report, indent=2))
        if not report["passed"]:
            print(
                "\nGATE FAILED. Review the disagreements by hand and tighten the "
                "prompt ONCE. Tightening repeatedly until the number clears is "
                "fitting the prompt to a 50-chunk sample."
            )
            return 1
        return 0

    if not args.corpus or not args.out:
        parser.error("--corpus and --out are required unless --agreement is given")

    chunks: list[tuple[str, str]] = []
    for path in sorted(args.corpus.glob("*.md")):
        for index, piece in enumerate(chunk(path.read_text(encoding="utf-8"))):
            chunks.append((f"{path.stem}#{index}", piece))
    print(f"{len(chunks)} chunk(s) from {args.corpus}")

    labeller = unavailable_labeller()
    prompt = PROMPT.read_text(encoding="utf-8")
    index = []
    for chunk_id, passage in chunks:
        entry = labeller.label(prompt, passage)
        entry["id"] = chunk_id
        index.append(entry)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(index, indent=2), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
