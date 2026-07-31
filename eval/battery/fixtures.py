#!/usr/bin/env python3
"""Seeded anti-caricature pastiches.

These are *deliberately over-fired* imitations: a fake Thompson stuffed with
boosters and shouting, a fake Bukowski with the adverbs stripped but the
intensifiers left in, a fake Adams whose every sentence is a negated simile.
Each must trip **upper**-band `Reduce` findings at Medium or above on the
device and syntax dimensions, and each must fail the same-author gate.

They are generated rather than written by hand because the generator is the
argument: the failure mode being tested is *maximizing a device rather than
calibrating it*, and a seeded template that emits one device per sentence is
that failure in its purest form. Nothing here is drawn from any author's text —
the sentence frames are original, which is why this file is committed while the
corpora it is measured against are not.

    python3 fixtures.py --out build/fixtures
"""

from __future__ import annotations

import argparse
import random
from pathlib import Path

# --------------------------------------------------------------------------
# Fake Thompson: booster-stuffed, ALL-CAPS, polysyndetic, drug-lexical.
# The real thing calibrates these; this does not.
# --------------------------------------------------------------------------
THOMPSON_OPENERS = [
    "We were somewhere around the ninth floor when the ABSOLUTELY savage truth hit",
    "There was blood and paperwork and coffee and the SCREAMING of the copier",
    "It was totally, utterly, completely INSANE and everyone knew it",
    "By dawn the whole thing had gone bad, terribly and horribly and finally bad",
    "The lawyer was DERANGED and the machines were worse and nobody cared at all",
]
THOMPSON_TAILS = [
    "and there was no way out, none, not one, MY GOD",
    "and the swine kept talking about quarterly targets",
    "and the drugs had not even started working yet",
    "and the whole atavistic parade rolled on regardless",
    "and somebody somewhere was going to pay for it",
]

# --------------------------------------------------------------------------
# Fake Bukowski: the adverbs are gone, which a naive imitator gets right, and
# the intensifiers are still there, which is the tell. The ○-cell failure.
# --------------------------------------------------------------------------
BUKOWSKI_LINES = [
    "the man drank",
    "it was totally quiet",
    "no windows",
    "the rain",
    "he was absolutely finished",
    "the dog did not move",
    "very cold in there",
    "he sat down",
    "extremely bad coffee",
    "nothing happened",
    "the bar was completely empty",
    "he went home",
]

# --------------------------------------------------------------------------
# Fake Adams: a negated simile in every sentence, which the real thing does
# roughly once a chapter.
# --------------------------------------------------------------------------
ADAMS_TENORS = [
    "The committee moved",
    "The ship hung in the sky",
    "The paperwork accumulated",
    "The announcement arrived",
    "The machine considered the question",
    "The silence spread through the room",
]
ADAMS_VEHICLES = [
    "in much the same way that bricks don't",
    "about as gracefully as a filing cabinet in a hurry",
    "like a policy that had never met a person",
    "in much the same way that an apology doesn't",
    "about as convincingly as a weather forecast",
    "like a decision nobody could later be found to have made",
]


def compose_thompson(rng: random.Random, sentences: int) -> str:
    out = []
    for _ in range(sentences):
        out.append(
            f"{rng.choice(THOMPSON_OPENERS)}, {rng.choice(THOMPSON_TAILS)}!"
        )
    return " ".join(out)


def compose_bukowski(rng: random.Random, lines: int) -> str:
    return "\n".join(rng.choice(BUKOWSKI_LINES) for _ in range(lines))


def compose_adams(rng: random.Random, sentences: int) -> str:
    out = []
    for _ in range(sentences):
        out.append(f"{rng.choice(ADAMS_TENORS)} {rng.choice(ADAMS_VEHICLES)}.")
    return " ".join(out)


#: `(name, builder, units, the dimensions this pastiche must over-fire)`
FIXTURES = [
    (
        "fake-thompson",
        compose_thompson,
        60,
        [
            "syn.intensifier_rate",
            "syn.allcaps_exclaim_rate",
            "syn.booster_chain_max",
            "syn.polysyndeton_rate",
        ],
    ),
    (
        "fake-bukowski",
        compose_bukowski,
        200,
        ["syn.intensifier_rate"],
    ),
    (
        "fake-adams",
        compose_adams,
        60,
        ["frame.negated_vehicle_rate", "frame.simile_rate"],
    ),
]


def build(out: Path, seed: int = 20260801) -> dict[str, list[str]]:
    """Write the fixtures and return each one's expected over-fired dimensions."""
    out.mkdir(parents=True, exist_ok=True)
    expectations: dict[str, list[str]] = {}
    for name, builder, units, dims in FIXTURES:
        # One RNG per fixture, seeded from the name, so adding a fixture never
        # changes the text of an existing one.
        rng = random.Random(f"{seed}:{name}")
        (out / f"{name}.md").write_text(builder(rng, units), encoding="utf-8")
        expectations[name] = dims
    return expectations


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=Path("build/fixtures"))
    parser.add_argument("--seed", type=int, default=20260801)
    args = parser.parse_args()
    expectations = build(args.out, args.seed)
    for name, dims in expectations.items():
        print(f"{name}: must over-fire {', '.join(dims)}")
    print(f"wrote {len(expectations)} fixture(s) to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
