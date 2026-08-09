# The readability rubric — "would you keep reading?"

`rubric.md` asks whether a rewrite acquired an author's voice. This one asks
whether the result is worth reading. They are not the same question and this
file exists because the project's own results stopped agreeing with each other:
the highest `p_same_author` ever measured here (0.603, Adams → Pratchett) drew
the second-worst judge score, and the cell whose distance moved the *wrong* way
(C15, 0.184 → 0.090) produced the only prose no judge called overwritten.

A pastiche that matches every band and reads like a machine doing an
impression has failed at the thing the artifact is for. The objective function
optimises fidelity because fidelity is what a metric can see. This rubric is
the part it cannot, and when the two disagree, this one is the tiebreaker.

**Pairwise, never absolute** — same reason as `rubric.md`. TTCW showed absolute
LLM creativity scores do not track expert judgement; comparative ones do.

---

## Input

Two passages, **X** and **Y**, unlabelled, in randomised order. You are told
the situation they both render and nothing else — not which is which, not what
either was trying to do, not which one anyone hopes will win.

You are a reader, not a stylometrist. Do not try to identify an author.

## The one question

**You have read both. You can only keep reading one. Which?**

Answer that first, before any analysis, and record it. Everything below is you
explaining a decision you have already made, which is the order that produces
honest answers.

## What to notice, after you have decided

### 1. Pull

* Did you want the next sentence, or did you finish out of duty?
* Where exactly did your attention drop? Quote the sentence.
* Is there a line you would repeat to someone?

### 2. Performance

The failure this whole rubric exists to catch. Prose that is *doing an
impression* rather than telling you something.

* `showing_off` — the writing wants credit for being clever.
* `every_beat_lands` — no plain sentences. A page where every paragraph has a
  joke or an image reads as exhausting, not brilliant; real authors go quiet
  between the good lines.
* `simile_stack` — two figures where one would carry it.
* `explained` — the joke, image or irony is restated in the next clause.
* `costume` — period or authorial mannerisms applied on top of prose whose
  bones are ordinary. Archaic diction is the usual tell.

### 3. Sense

* Does the passage still make sense as a piece of writing — events in order,
  pronouns attached, causes before effects?
* Did anything become confusing that a plainer telling would have kept clear?
* Is anything *false to itself*: a character who acts against what we were
  just told, an object that changes, a joke that contradicts the scene?

### 4. Worth

* If you read this in a book, would you keep going?
* Would you send it to someone?
* Is there one sentence in it that justifies the whole passage?

## Verdict

```json
{
  "keep_reading": "X" | "Y",
  "margin": "clear" | "slight" | "coin-flip",
  "reason": "one sentence — the deciding factor, not a summary",
  "attention_dropped": {"x": "quoted sentence or null", "y": "..."},
  "best_line": {"x": "quoted or null", "y": "..."},
  "flags": {"x": ["every_beat_lands"], "y": []},
  "sense": {"x": "ok" | "confusing" | "contradictory",
            "y": "ok" | "confusing" | "contradictory"},
  "would_send_to_someone": {"x": false, "y": true}
}
```

## How the numbers are used

Reported as a **readability win rate per arm**, judge blinded to arm. Two
things are being measured and they must not be collapsed:

1. **Between arms.** Does the same-content control produce more readable prose
   than the as-designed matrix? That is the comparison the arms exist for.
2. **Against fidelity.** For every passage that carries both a `p_same_author`
   and a readability verdict, report the pair. If they anti-correlate above
   some fidelity, that is the most useful thing this project can find out, and
   it is invisible to either instrument alone.

A readability win rate is not comparable across judge models. Pin the model in
the manifest or do not compare.
