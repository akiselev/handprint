# Results — Adams → Pratchett, Deep Thought chapter

Source: the Deep Thought chapter of *The Hitchhiker's Guide to the Galaxy*
(2,515 words), rewritten into Terry Pratchett's voice. One transfer, three
arms, two iterations of the critique loop. All figures from
`build/refs/*.json`, fitted on the corpora described in `README.md` and
calibrated against 26 background authors.

Every number below is at the **marginal** length tier (2,000–5,000 words).
`handprint` says of that tier: "usable only where the signal is already clear."
Read accordingly.

## The three arms

| arm | → Pratchett `p_same` | dist | → Adams `p_same` | dist | GI verify | gated med+ |
|---|---|---|---|---|---|---|
| plain | 0.2485 | 0.1272 | 0.0045 | 0.2254 | 0.13 disfavoured | 3 |
| plain-typo | 0.3485 | 0.1229 | 0.0870 | 0.1874 | 0.45 abstain | 3 |
| draft2 | **0.6030** | 0.1148 | 0.1015 | 0.1844 | 0.50 abstain | **0** |

`plain` is a flat retelling. `plain-typo` is the *same words* with the target
corpus's quotation typography applied by `typography.py` — curly single quotes,
curly apostrophes, spaced en dash. `draft2` is the full loop.

## Against the gate

The `voice-transfer` skill's gate, honestly:

1. **`verdict.pass`** — ✅ 0.6030 against a Balanced bar of 0.10, and it clears
   **Strict** (0.25) as well.
2. **Guards** — ✅ `canary_ok` and `drift_ok` true throughout. Note that
   `content_overlap` came back `null`: `critique` has no flag for supplying the
   stage-1 baseline, so the drift guard here is the internal one, not the
   measurement against `plain.md` the skill describes.
3. **Zero Medium-or-above on Device, Frames, Syntax, Rhythm** — ✅ after
   iteration 2. Iteration 1 left two: `syn.interjection_rate` under its band
   (0.79 against 2.00–11.12) and `syn.person.p1p` over it (14.56 against
   0.80–12.18). Both were `increase`/`reduce` in the direction the band asked
   for and both were fixable without touching content.
4. **External wit critic** — ❌ **not run.** No judge was available. The claim
   here is mechanical only.

And the capstone's **away** criterion — distance to the source author must
strictly increase — **fails**. `p_same` against Adams went *up*, 0.0045 →
0.1015, and the distance *fell*. GI verification returned `abstain`, not
`TargetFavoured`.

## Three things worth more than the gate

**The typography arm did a third of the work, and most of the GI movement.**
Changing `"` to `‘` and `—` to `–`, with not one word altered, moved
`p_same` against Pratchett from 0.2485 to 0.3485 and General Imposters from
0.13 (`target_disfavoured`) to 0.45 (`abstain`). That is 28% of the total
`p_same` gain and **86% of the total GI gain**, bought with a search-and-replace.

Quote convention is a real, measurable habit — but it is the *publisher's*,
and it is free to match. Any transfer evaluated without this arm is quoting a
number it did not earn. It is also why the `plain` row is misleadingly far from
*both* authors: ASCII quotation marks are unlike anything in either corpus.

**The Adams reference is at chance, and the Pratchett one is not.** Held-out
chapters, scored against references refitted without them:

```
douglas-adams      6/12      <- chance
terry-pratchett   12/12
overall           18/24 (75%), chance is 50%
```

Pratchett's own chapters score 0.27–0.68 against his reference and 0.01–0.09
against Adams's — clean separation. Adams's chapters score 0.04–0.21 against
his own. The likely causes are corpus size (426k words against 3.56M, after
losing a quarter of the Adams corpus to the bad scan) and heterogeneity: the
`Salmon of Doubt` chapters are essays, letters and interview transcripts, and
both held-out ones were misattributed.

This has a direct consequence for the table above. **`p_same_author` from a
426k-word reference and from a 3.56M-word reference are not on the same scale**,
so "0.60 toward Pratchett, 0.10 away from Adams" is not the two-sided reading it
looks like. The Pratchett column is trustworthy. The Adams column is a weak
instrument reporting a small number.

**`contrast.*` findings are unactionable here, and they are 87% of the report.**
110 of the 127 findings on the final draft are contrast-vocabulary dimensions
demanding fewer uses of `deep`, `computer`, `answer`, `universe`, `thought`,
`philosophers`. The passage is *about* a computer called Deep Thought that is
asked for the answer. Acting on them means deleting the content; the drift
guard exists to stop exactly that. They also crowd out the actionable findings
in a `--max-findings`-limited report — the two syntax findings that actually
mattered ranked below fifty topic words.

A voice-transfer profile probably wants the contrast family down-weighted or
off. Its purpose is signature *lexis*, and over a corpus spanning forty books it
has learned Discworld's proper nouns instead.

## Reproducing

```sh
python3 corpora/epub2md.py ~/books/pratchett/*.epub --out /tmp/md \
    --exclude corpora/exclude.json --report /tmp/qc.json
python3 corpora/chapterize.py /tmp/md/*.md --author terry-pratchett \
    --out corpora --target-words 2500
# fit + calibrate as in ../voice-transfer/run.sh, then:
python3 experiments/adams-pratchett/arms.py build/refs build/transfer \
    plain plain-typo draft2
python3 experiments/adams-pratchett/separation.py build/sep \
    douglas-adams terry-pratchett
```

Fitting Pratchett takes ~20 minutes and the reference is 44 MB, most of it the
contrast and n-gram families. Calibration against a 40-document-per-author
background takes ~75 seconds; against the full corpora it takes long enough
that the subsample is the only practical option.
