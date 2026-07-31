# `judge/` — the wit critic

An LLM judge, agent-side, never in the crate. No LLM in core is a standing
invariant, and "is this funny" is not a count.

## The protocol, and why it is this one

**Pairwise against gold anchors. Never absolute.**

TTCW found that absolute LLM creativity judgements do not track expert
judgement — ask a model to rate a passage out of ten and it rates everything a
seven. HumorRank found that the *comparative* task is stable, with cross-judge
agreement around τ = 0.889. So every judgement here is "which of these two is
better, and why", with one side always being the author's own prose.

**Blinded to arm and to side.** Position bias is documented; `run.py`
randomizes which passage is presented as A. The judge is never told which arm
produced the candidate.

**The claim is the ordering.** Full pipeline beats critique-loop beats
zero-shot. The absolute win rate against the real author is reported and is
expected to be below 0.5 — a run that reports otherwise should be checked for
leakage before it is believed.

**Undecided is not folded in.** A judge that cannot choose is information. The
win rate's denominator is decided pairs, and the undecided count is reported
next to it.

## Files

```
judge/
  README.md   this
  rubric.md   the GTVH-structured per-paragraph rubric
  run.py      the harness; the model is injected, not embedded
```

## Why the model is injected

A judge that is part of the harness is a judge nobody can swap out to check the
result. `run.py` takes a `Judge` protocol; the driver supplies it, and the
model id, temperature and anchor count go in the run manifest.

A win rate compared across two different judges is not a comparison.

## Where it fits

Criterion 5 of the capstone. It is one of four gates and not the gate: each of
`p_same_author`, the wit-critic win rate, and the external style embeddings is
Goodhart-able alone. The ensemble is the thing.
