# Arm B results — and the head-to-head with arm A

One 800-word incident (`source.md`), rendered into six author voices and one
two-author blend. Every rendering carries the same events, the same characters
and the same nouns, so anything that separates them is writing.

Scored against the v3 references (28 authors, rebuilt corpora) and judged blind
by readers with no access to the working conversation, under
[`../../judge/readable.md`](../../judge/readable.md).

## 1. The metric: direction right, magnitude tiny

| rendering | toward target | source's own score | nearest reference |
|---|---|---|---|
| pratchett | 0.071 | 0.029 | l-m-montgomery (0.161) |
| blend → pratchett | 0.058 | 0.029 | p-g-wodehouse (0.059) |
| jerome | 0.046 | 0.034 | hermann-hesse (0.200) |
| austen | 0.020 | 0.004 | hermann-hesse (0.205) |
| blend → adams | 0.012 | 0.005 | — |
| adams | 0.010 | 0.005 | terry-pratchett (0.049) |
| melville | 0.008 | 0.004 | hermann-hesse (0.159) |
| poe | 0.004 | 0.000 | hermann-hesse (0.158) |

**8/8 moved toward their target.** With content held constant, that movement
cannot be the source author's vocabulary, so voice transfer does register on
this instrument.

**The magnitudes are three to five times smaller than arm A's** (0.082–0.215),
written by the same hand against the same references. The thing that changed is
that the source no longer shares a century of vocabulary with the target. See
[`../FINDINGS.md#13`](../FINDINGS.md).

**In no rendering is the target the nearest reference.** Six of eight are
nearest to Hesse or Montgomery, whom nobody was writing toward. This is not a
property of these renderings — arm A's cells do it too — and it is the subject
of finding #11.

## 2. Readability: 14 blind pairwise judgements

| pair | winner | margin |
|---|---|---|
| adams vs plain source | **adams** | slight |
| pratchett vs plain source | **pratchett** | clear |
| jerome vs plain source | **jerome** | slight |
| austen vs plain source | **austen** | slight |
| poe vs plain source | **poe** | slight |
| blend vs plain source | **blend** | clear |
| melville vs plain source | **plain source** | slight |
| pratchett vs adams | **pratchett** | slight |
| jerome vs austen | **jerome** | slight |
| poe vs melville | **poe** | slight |
| blend vs adams | **blend** | clear |

### The plain source lost six of seven — and won one

Rendering into a voice does make prose more readable, six times out of seven,
which is the result the project wanted and had never actually tested.

The seventh is the interesting one. **The Melville rendering lost to doing
nothing at all**, and lost both of its pairs, and no judge would send it to
anyone:

> X trusts the events to be funny and gets out of the way, while Y stops after
> almost every beat to tell me what the beat meant.

Melville drew every flag in the rubric — `costume` ×2, `every_beat_lands` ×2,
`explained` ×2, `simile_stack`. A voice whose defining move is the expansive
gloss, applied to a small administrative incident, produces prose that is worse
than the unstyled telling. **That is the answer to "is fidelity the right
target": no, and here is a case where the honest fidelity move made it worse.**

### The blend won everything it entered

The Adams × Pratchett blend went 2/2 and beat **pure Adams clearly**:

> Almost every sentence the two render differently, Y renders better

This is one blend and two judgements, so it is a signal and not a result. But
it is the first evidence in this repo for the multi-corpus idea, and it points
somewhere specific: the blend was written to Adams's structure with Pratchett's
willingness to end a paragraph on a flat sentence, and what it gained was
*restraint*, which is exactly the axis the single-target loop pushes the wrong
way.

### The judge caught a continuity error the metric cannot see

The Adams rendering was flagged `sense: confusing` — its opening claims the
lift had "spent those eleven days working perfectly", which contradicts Ellen
pressing the button every morning for eleven days and getting nothing. No
dimension in the profile can see that. It is the single most damaging kind of
defect for a reader and it is invisible to the objective function.

## 3. Fidelity and readability are unrelated in this sample

| rendering | toward target | readability |
|---|---|---|
| pratchett | 0.071 | 2/3 |
| blend | 0.058 | 2/2 |
| jerome | 0.046 | 2/3 |
| austen | 0.020 | 1/2 |
| adams | 0.010 | 2/4 |
| melville | 0.008 | 0/2 |
| poe | **0.004** | **2/2** |

Spearman ρ ≈ **+0.28** across seven renderings — indistinguishable from zero at
this n. The lowest-fidelity rendering (Poe, 0.004) has the best readability
record; the second-lowest (Melville, 0.008) has the worst.

The prediction going in was an anti-correlation above some fidelity. What the
data shows is weaker and more awkward: **at the magnitudes this instrument
produces, `p_same_author` carries no information about whether anyone wants to
read the result.** Optimising it is not harmful; it is uninformative.

Poe is worth a sentence on its own, because it was chosen as the *hostile*
pairing — gothic dread applied to a lift — and it went 2/2. The judge's reason:

> X puts a witness behind the events who cannot sleep and cannot get anyone to
> believe him, which gives the same facts a stake the plain telling has no way
> to supply

The mismatch was the point. A voice that supplies a *stance* beat five voices
that supplied a manner.

## 4. Arm A vs arm B, head to head

Three cross-arm pairs, same rubric, judge warned that the two passages tell
different stories.

| pair | winner | margin | judge's note |
|---|---|---|---|
| arm A C03 → Adams vs arm B Adams | **arm B** | clear | arm A "stops a genuinely tense scene every three paragraphs to deliver an aphorism that wants to be admired" |
| arm A C05 → Pratchett vs arm B Pratchett | **arm A** | slight | "I had to work to discount the fact that X has a corpse in it and Y has a lift" |
| arm A C04 → Jerome vs arm B Jerome | **arm A** | clear | "I did have to discount finding whales more interesting than an office lift" |

**Arm A wins 2–1, and the judge named subject-matter interest as the deciding
difficulty in both of its wins.** That is the confound arm B exists to remove,
showing up in the one comparison where it cannot be removed — a cross-arm
readability contest is not a fair test and this is the evidence.

The flags are the fair comparison, because they are content-independent:

| passage | would send | flags |
|---|---|---|
| arm A C03 → Adams | 0/1 | every_beat_lands, showing_off, explained |
| arm B Adams | 3/4 | every_beat_lands ×2, explained ×3 |
| arm A C05 → Pratchett | 1/1 | explained, simile_stack |
| arm B Pratchett | 3/3 | showing_off ×2, explained ×2, every_beat_lands ×2 |

`explained` — the passage restating its own joke — is the near-universal flag,
appearing on **8 of 9 passages across both arms**. It is the single most common
defect in everything written here, it survived every fidelity gate, and no
dimension in the profile measures it.

## 5. What this says about running arm A's remaining 26 cells

They should not be rendered yet. Not because the writing would be wasted, but
because the number they would be compared on has just been shown not to
support the comparison:

* `p_same_author` differs by two orders of magnitude across references for
  identical text (finding #11), so ranking cells against each other is invalid.
* Gate 1 — "the source passage ranks its own author first" — passes only
  **30 of 56** sampled passages against the 28-reference set (finding #12), and
  18 of 28 authors fail at least one sample. Several cells in the as-designed
  matrix have sources that would not pass their own gate.
* Arm B suggests three to five times of arm A's measured movement is era
  vocabulary rather than voice (finding #13).

Fixing #11 is a per-reference normalisation — report a percentile against that
reference's own background rather than a raw probability. That is a small
change and it should come first.
