---
name: voice-transfer
description: Rewrite a passage in a target author's voice using a corpus-calibrated critic. Use when asked to rewrite, retell or pastiche text in a specific author's style and a fitted handprint reference for that author exists.
---

# Voice transfer

Rewrite a passage in an author's voice, using a fitted `handprint` reference as
the objective function and the author's own corpus as the source of exemplars.

The reason this is a four-stage loop rather than a prompt is that the two known
failure modes have opposite fixes. Mechanics converge easily and wit does not
(GPT-4o with 15k-word excerpts matches sentence length and stays in its generic
cluster on everything deeper). And imitation over-fires devices — the Dark &
Stormy result — because a prompt gives you no target *rate*. So: plan the
devices at the author's empirical density, render with mechanism-matched
exemplars, and let the calibrated critic pull both ways.

## Before you start

You need:

* `REFERENCE` — a fitted, calibrated reference for the target author.
  `handprint pack show $REFERENCE` should print a calibration line. Without one
  the pass gate cannot be evaluated and this skill has no stopping condition.
* `SOURCE` — the passage to rewrite.
* Optionally a `codeindex` corpus with a move index (see
  `eval/agents/move-index/`). Without it, skip stage 3's retrieval and say so
  in the report; the loop still works, less well.

If the reference is uncalibrated, stop and say so. Do not substitute "all
findings cleared" for the gate — clearing every reported finding is exactly
what an agent optimizing the feedback does, and the gate exists to catch it.

## Stage 1 — plain retelling

Rewrite `SOURCE` with **no style pressure at all**. Preserve every event,
every named entity, every stated fact. Aim for flat, unmarked prose.

This draft is the drift baseline: `content_overlap` is measured against it for
the rest of the loop, so a stage-1 draft that already drops half the content
makes the guard meaningless. Save it as `plain.md`.

## Stage 2 — the device plan, at the author's rate

Read the target rates out of the reference:

```sh
handprint critique -r "$REFERENCE" plain.md --max-findings 200 > baseline.json
```

Every finding carries `target_band`. For each device dimension — `dev:*`,
`frame:*`, `syn:*`, `surp:punch_*` — the band's midpoint is the author's
empirical density. Convert it: a `frame:negated_vehicle_rate` band of 0.8–2.4
per 1k tokens over a 900-word passage means **one or two** negated similes in
the whole piece, not one per paragraph.

Write the plan as a list of beats. For each:

1. Which beat of the source it covers.
2. Which device, if any. Most beats get none — that is what the rate says.
3. A **remote association** step for the ones that do: what distant domain can
   this be yoked to? Bureaucracy for a cosmic event, domestic detail for a
   catastrophe. Generate three candidates and pick the least obvious.
4. The punchline **written out as a one-liner**, before any prose exists.

A plan that puts a device in every beat has already failed. Check it against
the rates before writing a word.

## Stage 3 — render

Per scene, retrieve 2–4 exemplars **by mechanism**:

```sh
codeindex search "$AUTHOR: character introduction using register clash" --limit 4
```

Not by topic. Topical exemplars convey register and not technique, which is the
finding the whole move index exists to act on.

**Imitate the mechanism, not the sentence.** The exemplars are there to show
how the device is built; a phrase lifted from one is parroting and the drift
fingerprint will catch it. If you notice yourself reusing an exemplar's vehicle,
that is the signal to invent a different one.

## Stage 4 — the polish loop

```sh
handprint critique -r "$REFERENCE" draft.md > report.json; echo $?
```

Exit 0 pass, 1 findings, 2 error. Iterate up to **12** times, and sequence the
work:

* **Iterations 1–4: mechanics.** Act on `punct:*`, `sent:*`, `mfw:*`,
  `read:*` findings. These converge fast and there is no point judging wit
  through the wrong sentence rhythm.
* **Iterations 5+: wit.** Act on `dev:*`, `frame:*`, `surp:punch_*`,
  `form:*`. Run the wit critic (`eval/judge/`) if one is available.

Read the whole report each time, not just `findings`:

* `guards.canary_ok == false` — **stop patching**. You have been moving the
  reported numbers while the held-out family got worse, which is the definition
  of optimizing the feedback. Rewrite the passage as prose from the plan
  instead of editing the flagged items.
* `guards.drift_ok == false` — the rewrite has left the content behind. Go back
  to `plain.md` and re-render the beat that drifted.
* `doc.confidence` — a family reading `none` at this length is noise. Do not
  act on its findings.
* **`direction: "increase"` is as real as `"reduce"`.** Under-firing a device is
  a mechanics-only pastiche that never acquired the voice, and the ○ cells of
  the coverage matrix are precisely this claim. An author who never uses
  adverbs has a *low band* on adverbs, and going below it is as much a miss as
  going above.

## The gate

A draft is done when **all** of:

1. `verdict.pass == true` — `p_same_author` at or above threshold. Not "all
   findings cleared".
2. `guards.canary_ok && guards.drift_ok`.
3. **Zero Medium-or-above findings in either direction** on Device, Frames,
   Syntax and Rhythm dimensions. Inside the band, not merely under the ceiling.
4. If a wit critic is available: its pairwise win rate against gold anchors
   improved over the stage-1 baseline.

Each of those alone is Goodhart-able. The ensemble is the gate — that is why
there are four and not one.

## Report

State plainly:

* Which arm ran (plain-only, critique-loop, full pipeline).
* Iterations used, and whether the gate was met or the cap was hit.
* Final `p_same_author`, `content_overlap`, and any guard that tripped.
* Which device dimensions ended outside their band, in which direction.

If the loop did not converge, say so and show the last report. A voice transfer
that hit the iteration cap is a result, and writing it up as a success is the
one failure this whole design exists to prevent.
