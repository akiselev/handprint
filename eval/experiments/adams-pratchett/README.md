# Adams ↔ Pratchett: an in-copyright voice-transfer trial

The capstone in `../voice-transfer/` is built on public-domain authors so its
artifacts can ship. This one is the opposite case and exists for a different
reason: two authors with **large, clean, modern corpora** and a genuinely
narrow gap between them. Both write British comic fantasy in the same decades.
If a critic cannot tell them apart, the number it reports about a pastiche of
either is not worth quoting.

Everything here is local-only. Both authors are in copyright, so no corpus
text, no fitted reference and no transfer transcript is ever committed —
`eval/licenses/check.py` fails the build if one is. What this directory holds
is the method and the metrics.

## Corpus

| author | books | documents | words |
|---|---|---|---|
| Douglas Adams | 3 of 4 | 175 | 426,320 |
| Terry Pratchett | 42 | 1,418 | 3,558,529 |

Built with `corpora/epub2md.py` and `corpora/chapterize.py --target-words 2500`.
Three things the QC pass caught, all of which would have been invisible in the
fitted reference:

* **One Adams EPUB rejected.** *Dirk Gently's Holistic Detective Agency* is a
  reflow of a hard-wrapped source with the spaces not restored — `out of ithad
  slowed`. Its fused-word rate is 5.6/1k against 0.00–0.05 for every other book
  here, and that is a floor, since the detector only catches fusions where both
  halves are function words. Dropped rather than repaired.
* **Other authors' prose removed.** Stephen Fry's introduction and Richard
  Dawkins's epilogue to *The Salmon of Doubt*, Neil Gaiman's foreword to the
  omnibus *Hitchhiker's*, A. S. Byatt's foreword to *A Blink of the Screen*,
  and Rob Wilkins's afterword to *The Shepherd's Crown*. Listed in
  `corpora/exclude.json` because nothing in the markup distinguishes them.
* **Discworld has no chapters.** Fixed-size 2,500-word documents, cut at
  paragraph boundaries.

The background for calibration is 26 authors, 40 documents each, from Project
Gutenberg via `corpora/fetch_gutenberg.py`, normalized to the same 2,500-word
document size so length is not confounded with author.

## Two defects this trial found

Both were invisible against the synthetic fixtures and fired immediately on
real books.

**A panic on any prose containing a multi-byte character.** `link_spans` in
`feature/sentence.rs` walked the source one byte at a time and sliced at the
cursor, which panics the moment it lands inside a curly quote or an em dash.
Every book in this corpus opens with one. Fixed, with a regression test.

**`dev:novel_adj_noun_rate` had a degenerate band.** `fit` built the set of
adjective–noun pairs as the union over the corpus, then estimated the band by
profiling those same documents against a set built to include their own
coinages. Every fitting document therefore scored exactly zero, the band
collapsed to `[0.00, 0.00]`, and **no unseen text could ever be inside it**.

Held-out Adams chapters scored against an Adams reference, before the fix:

```
obs=35.36  band=[0.00,0.00]  z=+59.8  severity=high  direction=reduce
```

— twelve of twelve, the highest-|z| finding in every report. An agent working
the critique loop would have spent every iteration on an unreachable target,
and gate condition 3 ("zero Medium-or-above Device findings") was unsatisfiable
by construction.

The fix is a document-frequency floor: a pair must appear in two documents to
count as one the corpus has seen. It is a cheap stand-in for leave-one-out.
After it, the same twelve held-out chapters:

```
band=[18.71,36.61]   11 of 12 in band, 1 flagged medium at 45.40
```

which is a two-sided band doing its job. **This changes fitted artifacts:
references fitted before it need a refit.**

## The transfer

Source is the Deep Thought chapter from *The Hitchhiker's Guide to the Galaxy*
(2,515 words), rewritten into Pratchett's voice. Three arms, so the claim is
about the pipeline rather than about a model:

1. **plain** — flat retelling, no style pressure. The drift baseline.
2. **plain-typo** — the same words, with the target corpus's quotation
   typography applied by `typography.py` and nothing else.
3. **draft** — the full loop: device plan at Pratchett's measured rates,
   render, critique, revise.

Arm 2 exists because of what arm 1 turned up. See `RESULTS.md`.
