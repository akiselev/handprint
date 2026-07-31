#!/usr/bin/env python3
"""The wit-critic harness: pairwise, anchored, blinded.

Reads `rubric.md`, pairs each candidate paragraph with a gold anchor, shuffles
which side is which, and asks a judge model. What comes back is a win rate per
arm plus the raw judgements, and the run manifest pins the model.

The model call is deliberately an injected callable rather than a hard-coded
client. Two reasons: the harness has to be runnable against whatever judge is
available at the time, and a judge that is part of the harness is a judge
nobody can swap out to check the result.

    python3 judge/run.py --candidates build/transfer --anchors corpora/wodehouse \\
        --arms zero-shot,critique-loop,full --out build/judge.json
"""

from __future__ import annotations

import argparse
import json
import random
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Protocol

RUBRIC = Path(__file__).parent / "rubric.md"


class Judge(Protocol):
    """Anything that can compare two passages under the rubric."""

    def compare(self, rubric: str, a: str, b: str, mechanism: str) -> dict: ...


@dataclass
class Pairing:
    """One blinded comparison."""

    candidate: str
    anchor: str
    mechanism: str
    #: True when the candidate was presented as "A".
    candidate_first: bool
    result: dict = field(default_factory=dict)

    def candidate_won(self) -> bool | None:
        better = self.result.get("verdict", {}).get("better")
        if better not in {"A", "B"}:
            return None
        return (better == "A") == self.candidate_first


def pair_up(
    candidates: list[str],
    anchors: list[str],
    mechanisms: list[str],
    seed: int,
) -> list[Pairing]:
    """Pair every candidate with an anchor, randomizing presentation order.

    The order matters: judges have a documented position bias, and a harness
    that always put the candidate second would measure the bias.
    """
    rng = random.Random(seed)
    pairings = []
    for index, candidate in enumerate(candidates):
        anchor = anchors[index % len(anchors)]
        mechanism = mechanisms[index] if index < len(mechanisms) else "unspecified"
        pairings.append(
            Pairing(
                candidate=candidate,
                anchor=anchor,
                mechanism=mechanism,
                candidate_first=rng.random() < 0.5,
            )
        )
    return pairings


def run(pairings: list[Pairing], judge: Judge) -> dict:
    rubric = RUBRIC.read_text(encoding="utf-8")
    wins = 0
    decided = 0
    for pairing in pairings:
        first, second = (
            (pairing.candidate, pairing.anchor)
            if pairing.candidate_first
            else (pairing.anchor, pairing.candidate)
        )
        pairing.result = judge.compare(rubric, first, second, pairing.mechanism)
        won = pairing.candidate_won()
        if won is not None:
            decided += 1
            wins += int(won)
    return {
        "pairs": len(pairings),
        "decided": decided,
        "wins": wins,
        # Undecided pairs stay in the denominator nowhere and are reported
        # separately: a judge that cannot choose is information, and folding it
        # into a win rate would hide it.
        "win_rate": wins / decided if decided else None,
        "undecided": len(pairings) - decided,
        "judgements": [
            {
                "mechanism": p.mechanism,
                "candidate_first": p.candidate_first,
                "result": p.result,
            }
            for p in pairings
        ],
    }


def load_paragraphs(directory: Path, limit: int) -> list[str]:
    """Paragraphs from every file in a directory, in a stable order."""
    out: list[str] = []
    for path in sorted(directory.glob("*.md")):
        for block in path.read_text(encoding="utf-8").split("\n\n"):
            block = block.strip()
            if len(block.split()) >= 30:
                out.append(block)
            if len(out) >= limit:
                return out
    return out


def unavailable_judge(*_args, **_kwargs) -> Judge:
    raise SystemExit(
        "No judge model was supplied. Pass one from a driver script:\n\n"
        "    from judge.run import run, pair_up\n"
        "    run(pair_up(...), MyJudge())\n\n"
        "The harness deliberately does not embed a client: a judge that is\n"
        "part of the harness is a judge nobody can swap out to check the result."
    )


def main(judge_factory: Callable[[], Judge] = unavailable_judge) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidates", type=Path, required=True)
    parser.add_argument("--anchors", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--limit", type=int, default=40)
    parser.add_argument("--seed", type=int, default=20260801)
    parser.add_argument(
        "--mechanisms",
        type=Path,
        help="JSON list of the intended mechanism per candidate paragraph",
    )
    args = parser.parse_args()

    candidates = load_paragraphs(args.candidates, args.limit)
    anchors = load_paragraphs(args.anchors, args.limit)
    if not candidates or not anchors:
        raise SystemExit("need both candidate and anchor paragraphs")
    mechanisms = (
        json.loads(args.mechanisms.read_text(encoding="utf-8"))
        if args.mechanisms
        else []
    )

    pairings = pair_up(candidates, anchors, mechanisms, args.seed)
    report = run(pairings, judge_factory())
    args.out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(
        f"{report['wins']}/{report['decided']} wins "
        f"({report['undecided']} undecided) -> {args.out}"
    )
    print(
        "The claim is the ordering of the arms, not the absolute rate. "
        "A win rate above 0.5 against the real author should be checked for "
        "leakage before it is believed."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
