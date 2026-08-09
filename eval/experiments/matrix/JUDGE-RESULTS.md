# What the blind judges said

The stylometric numbers were rising. The judges say that was not enough, and
in one case that it was actively misleading. This file records the result
before any of it is fixed, because a result that only exists after the fix is
not a result.

## Round 1 — "spot the fake" (6/6 against us)

Each judge got one rewrite and one genuine passage by the target author,
unlabelled, in randomised order, and had to say which was genuine. The judges
were subagents with no access to the working conversation.

| cell | target | judge picked real author | confidence | fidelity of the imitation |
|---|---|---|---|---|
| C03 | Adams | ✅ correct | 5/5 | 4/5 |
| C04 | Jerome | ✅ correct | 5/5 | 3/5 |
| C05 | Pratchett | ✅ correct | 5/5 | 3/5 |
| C15 | Conan Doyle | ✅ correct | 5/5 | **1/5** |
| Poe→Pratchett story | Pratchett | ✅ correct | 5/5 | 3/5 |
| Adams→Pratchett | Pratchett | ✅ correct | 5/5 | **2/5** |

**Six for six, every one at maximum confidence.** No qualification available.

### The metric and the judge disagree, and on the top cell they invert

`Adams → Pratchett` scored the **highest** `p_same_author` of anything built
here — 0.603, clearing the Strict gate — and the judge gave it the **second
lowest** fidelity, 2/5. The judge's reason:

> the underlying content is transparently borrowed Douglas Adams material,
> which sinks it regardless of sentence-level polish

Meanwhile `Jerome → Conan Doyle` scored 1/5 with the judge, and it is also the
cell whose `p_same_author` *fell* (0.180 → 0.098). There the two instruments
agree. So the relationship is not "weakly correlated". It is: they agree on
failures and diverge on successes, and the divergence is largest exactly where
the metric is most confident.

## The three failures the judges found, none of which the metric caught

**1. Content contamination is the dominant tell.** Every judge named it first.
A rewrite of *Crime and Punishment* into Adams still contains Raskolnikov; a
rewrite of *The Boscombe Valley Mystery* into Pratchett still contains Holmes.

> Pratchett never wrote Holmes pastiche; this is a Doyle scene re-narrated with
> Pratchett-flavored aphorisms bolted on, which is itself the tell

This has a sharp consequence for
[`../adams-pratchett/RESULTS.md`](../adams-pratchett/RESULTS.md), which
recommended dropping the **contrast** family because it reported topic rather
than style. That recommendation was half wrong. The contrast family was
flagging exactly what the judges flag — that the lexis does not belong to the
target — and calling it "topic, not style" was the mistake. What the family
actually needs is not removal but a way to say *this word is foreign to the
target author* without demanding you delete the noun the passage is about.

**2. The device bands passed prose the judges call overwritten.** The showcase
story cleared gate condition 3 with **zero** Medium-or-above findings on
Device, Frames, Syntax and Rhythm — inside every band. The judge, unprompted:

> Yes. Nearly every paragraph carries an aphorism or simile … there's no plain
> connective tissue; Pratchett lets some sentences just move the story.

The anti-caricature guard is a *rate* check. It counts devices per thousand
tokens and is satisfied. It cannot see that the devices are evenly spread over
every paragraph instead of clustered and then absent, which is what a real
author's page looks like. **A rate is not a distribution.** That is a concrete,
fixable gap in the feature set: the device families need a burstiness or
inter-arrival measure, not just a rate.

**3. Explaining the joke.** Flagged repeatedly, and invisible to every
dimension in the profile:

> Explains the joke instead of trusting it: `They are wrong and the way they
> are wrong is important, because...`

## Round 2 — the protocol correction

Round 1 asked an unwinnable question. `judge/README.md` says so in its own
words: the absolute win rate against the real author is *expected below 0.5*,
and **"the claim is the ordering"**. A rewrite that carries another author's
plot cannot beat the real thing on a spot-the-fake test, and losing that test
6/6 measures the transplant, not the voice.

Round 2 therefore asks the question the rubric actually specifies: given the
untouched source passage and the rewrite, **which is closer to the target
author's voice**, subject matter explicitly excluded. Position is balanced by
construction rather than by chance — the first randomisation put all four
drafts in slot A, which is precisely the position bias the harness warns about.

Results in `results-round2.json`.

### Round 2 result — 4/4 for the rewrites

| cell | target | margin | overwritten? |
|---|---|---|---|
| C03 | Adams | moderate | yes |
| C04 | Jerome | large | yes |
| C05 | Pratchett | large | yes |
| C15 | Conan Doyle | moderate | **no** |

Every rewrite was judged closer to the target voice than its untouched source.
So the transfers do move, and a blind reader can see the movement — it is the
*absolute* claim that fails, not the directional one. This is what the harness
predicted, and it is why the rubric forbids absolute scoring.

C15 is the instructive cell. Its `p_same_author` **fell** (0.180 → 0.098), and
the round-1 judge gave it 1/5. On the ordering question it wins at a moderate
margin and is the **only** cell not flagged as overwritten:

> Aside from that one inherited joke, A carries the scene with plain
> declarative narration … which is closer to how Doyle actually paces incident.

A cell can be the metric's worst result and the judge's cleanest prose. Whatever
`p_same_author` is tracking at 1,000 tokens, on this cell it is not what a
reader responds to.

## The one defect to fix before scaling

Three of four cells, and the showcase story, were independently flagged
**overwritten** by judges who were not asked to look for it and could not see
each other's answers. The wording converges:

> Nearly every paragraph gets its own aphorism-plus-joke … Adams would let more
> of the dialogue run plain.

> Almost every sentence closes on a quip … Real Jerome intersperses plain
> narration between jokes far more than this rewrite allows.

> Nearly every beat gets a simile or aphorism … where Pratchett would let some
> plain sentences breathe between the good lines.

All of that prose is **inside every device band**. The bands are rates per
thousand tokens, and a rate cannot distinguish "eight jokes clustered in two
paragraphs and none in the next six" from "one joke in every paragraph". Real
authors do the first. Everything written here does the second, and the profile
scores them identically.

The gap is not a threshold to retune. It is a missing measurement: the device
families need an **inter-arrival or burstiness statistic** — variance of the gap
between device hits — alongside the rate. That single addition would have
caught, automatically and at zero LLM cost, the one flaw four independent
judges each found first.

Two smaller items in the same class, both invisible to the current profile:
**explaining the joke after making it**, and **stacking two similes on one
beat** where one should carry it.

---

# The fix, and what measuring it actually showed

## Two new dimensions

`dev:device_burstiness`, `dev:device_para_share`, `frame:figure_burstiness`,
`frame:figure_para_share`. The share dimensions are the share of paragraphs
carrying at least one hit; the burstiness ones are the Goh–Barabási
coefficient `(sd - mean)/(sd + mean)` over the gaps between hits, bounded in
`[-1, 1]` so they need no length correction. Both go missing rather than zero
below four hits, because a spread computed from one gap is not a spread.

They went into **two** families, not one. The judges' word was "simile", and
similes are in `frames`, not `device` — a burstiness statistic over the device
detectors alone would have measured alliteration runs, which are not jokes.

## What the numbers say — mixed, and worth stating plainly

**`frame:figure_para_share` works.** Measured on the Jerome cell the judges
called overwritten, against real Jerome chapters:

```
MY Jerome pastiche      figure_para_share = 0.261      simile_rate = 6.47
real Jerome (4 chapters)          0.113 – 0.143              0.78 – 2.39
```

Nearly double the author's ceiling on paragraph share, and roughly three times
his simile rate. The dimension the judges' complaint asked for does flag the
prose the judges complained about. Same for the Pratchett cell on the device
side: `device_para_share` 0.639 against real Pratchett's 0.397–0.504.

**`figure_burstiness` does not work at this length.** Different documents
return identical values (−0.232 recurs across unrelated texts) because a
1,000-word passage contains too few figures to make the paragraph-gap sequence
informative. It is not wrong, it is under-powered, and it should be read only
on documents long enough to carry a dozen figures. Reported rather than quietly
dropped.

**`figure_para_share` is confounded by paragraph length.** The untouched
Melville source scores 0.571 — *higher* than any pastiche — because Melville
writes six enormous paragraphs and nearly every one contains a figure. Breaking
the same material into twenty short paragraphs lowers the share without
removing a single simile. The share is still the right family of measurement,
but the honest version normalises per sentence, or per hundred tokens of
paragraph, rather than per paragraph.

## The constraint underneath the whole exercise

Sparse revision moved the Jerome cell from 6.47 to 5.87 similes per 1k. The
target band is 0.78–2.39. **The source passage is at 4.62** — Melville is
simile-dense and Jerome is not, so reaching Jerome's rate means deleting
Melville's own figures, which is content loss, which the drift guard exists to
prevent.

This is the same wall the Poe→Pratchett story hit on dialogue rate, and it is
now visible twice, which makes it a property of the task rather than an
accident of one cell:

> A voice has *rates* that its content cannot always supply. Where the source's
> own figure density, dialogue density or narrative person differs from the
> target's, style transfer and content preservation are in direct conflict, and
> no amount of iteration dissolves it.

That is worth more than a table of wins. It says what a corpus-mimic critic can
and cannot be asked for: it can move mechanics, rhythm and register a long way,
and it cannot make a solitary confinement scene as dialogue-heavy as Discworld,
or a Melville digression as sparing with similes as Jerome, without ceasing to
be the same passage.

## Round 3 — does the sparse revision actually read better? 2/2 yes

Same text, two revisions differing mainly in device density, shown blind to a
judge that was not told which was which or which direction we hoped for.

| cell | judge picked | margin | overwritten: dense | overwritten: sparse |
|---|---|---|---|---|
| C04 Jerome | **sparse** | slight | yes | **no** |
| C05 Pratchett | **sparse** | slight | yes | **no** |

The flag flipped in both, which is the result that matters: the judge's own
"overwritten" verdict tracks the revision the new dimensions were built to
detect. On C05:

> A piles on a fourth [simile] before the scene has breathed. B trims that to
> one plain line: "Watson had learned to mistrust that word." Quieter, and it
> lets the next beat carry the scene instead of another narrator quip.

**The margins are slight, and the judge called both "genuine near-ties".** The
fix is real and it is small. It removes a tic; it does not turn a pastiche into
the author. The remaining deviations the judge names are not density at all —
they are structural:

> too tidy and epigrammatic for Jerome, who rambles into a joke rather than
> landing it as a clean aphorism

> no narratorial wink; Pratchett would have the narrator comment on Lestrade's
> remark, not just let it sit as period dialogue

Those are the next thing to measure, and neither is a rate.

---

# Rerun against the refitted references

All 23 references refitted with the four spacing dimensions and recalibrated
against the same 26-author background, then every existing cell rescored.

## Does the new dimension fire where the judges complained?

Partly — 2 of the 4 drafts judges called overwritten:

| draft | judges said | new dimension |
|---|---|---|
| C04 → Jerome | overwritten | **`figure_para_share` 0.261, band [0.049, 0.227], reduce, medium** |
| C04 sparse | not overwritten | **cleared — nothing flagged** |
| Adams→Pratchett | overwritten | `figure_burstiness` 0.199, band [−0.481, 0.116], low |
| C05 → Pratchett | overwritten | not flagged — **miss** |
| Poe→Pratchett story | overwritten | not flagged — **miss** |

C04 is the clean end-to-end case: the judge called it overwritten, the new
dimension fires at Medium, the sparse revision clears it, and a second blind
judge then prefers the sparse revision. That is the loop working.

Two misses out of four. The dimension is a real improvement and not a solution.

## The rerun table

| cell | toward target | away from source | gated med+ | findings |
|---|---|---|---|---|
| C03 → Adams | 0.116 → **0.199** | 0.639 → 0.238 | 4 | 21 |
| C03 sparse | 0.116 → 0.118 | 0.639 → 0.228 | 5 | 22 |
| C04 → Jerome | 0.038 → **0.090** | 0.428 → 0.073 | 7 | 34 |
| C04 sparse | 0.038 → 0.091 | 0.428 → 0.102 | **5** | **29** |
| C05 → Pratchett | 0.093 → **0.168** | 0.577 → 0.135 | 4 | 22 |
| C05 sparse | 0.093 → **0.211** | 0.577 → 0.117 | 4 | **20** |
| C15 → Doyle | 0.184 → 0.090 | 0.311 → 0.143 | 2 | 24 |

## The alignment improved, which is the point

Before the fix, metric and judge **inverted** on the strongest cell: the highest
`p_same_author` anywhere (0.603) drew the second-worst fidelity score.

After it, on both cells where a blind judge compared dense against sparse, the
metric now agrees with the judge:

* **C05** — judge picked sparse; metric moved 0.168 → **0.211** and findings
  22 → 20.
* **C04** — judge picked sparse; gated findings 7 → **5**, total 34 → **29**.

They still disagree on **C03**, where the metric prefers the dense draft
(0.199 against 0.118) and gated findings rise 4 → 5 under sparse revision. That
cell was not put to a judge, so there is no tie-breaker and no claim to make.

C15 remains the one cell that moves the wrong way on the metric (0.184 → 0.090)
while winning its ordering comparison and being the only draft never flagged
overwritten. It is the standing counter-example to reading `p_same_author` as
quality at this document length.
