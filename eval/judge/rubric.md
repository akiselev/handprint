# The wit-critic rubric

A per-paragraph judgement, structured after the General Theory of Verbal Humor
and after HumorRank's finding that cross-judge agreement holds (τ ≈ 0.889) when
the task is comparative and collapses when it is absolute.

**Judge pairwise against gold anchors. Never absolute.** TTCW showed that
absolute LLM creativity judgements do not track expert judgement — the model
will happily rate everything a 7 — while pairwise comparison against a fixed
anchor is stable. Every question below is asked of *two* passages at once.

---

## Input

You are given:

* **A**: a passage from the author's own work (the anchor).
* **B**: a passage from a rewrite attempting that author's voice.
* **The intended mechanism**: which comic device the rewrite's plan called for
  in this beat (subverted simile, register clash, absurd precision, digression,
  deadpan escalation, transferred epithet, …).

You are **not** told which passage is which. If you can tell, say so in
`tell` — that is data, not a failure.

## Questions

Answer each for A and for B, then state which is better and why.

### 1. Mechanism

* Is the intended mechanism **present**? Point at the words.
* Is it **the** joke, or decoration on a sentence that would work without it?
* Does the passage use a *different* mechanism instead? Name it.

### 2. Delivery

* **Punchline position.** Does the turn land at the end of its sentence or
  clause, or is it buried mid-sentence with explanation after it?
* **Escalation.** Does a sequence build, or is each item the same size?
* **Concision.** Could the joke survive the deletion of a third of its words?
  If yes, it is over-written.

### 3. Failure flags

Each is a yes/no with a span:

* `cliche` — the vehicle, frame or turn is stock.
* `overexplanation` — the passage explains the joke after making it.
* `both_meanings_stated` — the pun or ambiguity is spelled out rather than
  left to work. HumorRank calls this "lazy generation" and it is the single
  commonest LLM humour failure.
* `unearned` — the escalation arrives without a setup that licensed it.
* `tonal_break` — the register shifts in a way the passage does not use.
* `parroting` — the passage reproduces a phrase from the retrieved exemplars
  rather than the technique. **Check this first**; the exemplar text is
  supplied to you for exactly this purpose.

### 4. Verdict

* `better`: `"A"` or `"B"`.
* `margin`: `clear` | `slight` | `indistinguishable`.
* `reason`: one sentence. Not a summary of the above — the *deciding* factor.

---

## Output

```json
{
  "mechanism": {"a": {"present": true, "span": "...", "is_the_joke": true},
                "b": {"present": false, "span": null, "substituted": "hyperbole"}},
  "delivery": {"a": {"punchline_final": true, "escalates": true, "concise": true},
               "b": {"punchline_final": false, "escalates": false, "concise": false}},
  "flags": {"a": [], "b": ["overexplanation", "cliche"]},
  "verdict": {"better": "A", "margin": "clear",
              "reason": "B states the joke and then explains it."},
  "tell": "B's second sentence names its own device"
}
```

## How the numbers are used

The reported metric is a **win rate against the anchor**, per arm, with the
judge blinded to arm. The claim the capstone makes is about the *ordering* of
the arms — full pipeline beats critique-loop beats zero-shot — not about
beating the real author. The absolute win rate against gold is reported
honestly and is expected to be below 0.5; a run that reports otherwise should
be checked for leakage before it is believed.

Judge model, temperature and anchor count are pinned in the run manifest. A
win rate compared across two different judges is not a comparison.
