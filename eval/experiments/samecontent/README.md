# Arm B — the same-content control

One incident, rendered into every target voice. The matrix (`../matrix/`) is
arm A: thirty different source passages, thirty different target authors, one
cell each. This is the same question with the content held still.

## Why it exists

Every blind judge in the matrix named the same failure first, and named it
before anything about style:

> the underlying content is transparently borrowed Douglas Adams material,
> which sinks it regardless of sentence-level polish

> Pratchett never wrote Holmes pastiche; this is a Doyle scene re-narrated with
> Pratchett-flavored aphorisms bolted on, which is itself the tell

Arm A guarantees that failure in every cell by construction. A rewrite of
*Crime and Punishment* into Adams still contains Raskolnikov, and no amount of
voice work removes him. So arm A measures voice *and* contamination together
and cannot separate them.

The second reason is rate collision, confirmed twice in arm A (dialogue in
Poe→Pratchett, similes in Melville→Jerome): where the source's own density
differs from the target's, style transfer and content preservation are in
direct conflict. With one shared source, that conflict is the same for every
voice, so it becomes a measurement instead of a confound.

## The source

**Not a passage by any author in the corpus.** A source passage by a fitted
author would hand one target an advantage — Jerome rendering a Jerome incident
is not the same task as Jerome rendering a Melville one — and would reintroduce
exactly the contamination this arm removes.

`source.md` is therefore a purpose-written incident in flat, unmarked prose:
plain declaratives, no figures, no jokes, no period markers, no named work.
It is the stage-1 "plain retelling" of the voice-transfer skill, written
directly rather than derived from anything.

It contains, deliberately:

| ingredient | why |
|---|---|
| two people who speak | dialogue-heavy targets need dialogue to be *there* |
| a physical object that misbehaves | something for absurdists to escalate |
| an institution with a rule | bureaucracy for the satirists |
| a small emotional stake | something for the realists |
| a plain sequence of events | so "does it still make sense" is checkable |

Any voice that cannot find purchase on it is telling us something real about
the voice, not about the passage.

## What is measured

Each rendering is scored three ways, and the third is the one this arm was
built for.

1. **`p_same_author` toward the target** — the objective function. Indicative
   only at this length; the per-family findings are the evidence.
2. **Voice fidelity**, blind, under `../../judge/rubric.md`, on the ordering
   question the rubric specifies. Never spot-the-fake: a rewrite carrying no
   author's plot has no plot to be caught by, but the absolute question is
   still the wrong one.
3. **Readability**, blind, under `../../judge/readable.md` — *would you keep
   reading this?* Passages are paired against each other rather than against
   the author, because the interesting comparison is between voices rendering
   the same events, and because the reader's question has no gold answer.

Since content is identical across renderings, a readability judge comparing two
of them is comparing **only** the writing. That is not true anywhere in arm A.

## The comparison with arm A

Both arms feed the same two judges. The arm-level question is:

* Does removing content contamination raise the readability win rate?
* Does it raise or lower fidelity? The prediction is **lower** — a rewrite that
  keeps the source author's nouns scores closer on `mfw` and `char_ngram` for
  reasons that have nothing to do with voice, and this arm gives that up.
* Do fidelity and readability anti-correlate above some fidelity? Arm A's own
  results already hint that they do. This arm can answer it, because it is the
  only place the two can be measured on the same content.

If fidelity falls and readability rises, the objective function is optimising
the wrong thing past a point, and that is the finding — not a disappointment.
