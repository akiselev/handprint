# Poe → Pratchett: a whole short story

"The Pit and the Pendulum" (1842, public domain), 6,004 words, rewritten in
Terry Pratchett's voice. Five critique iterations.

This is the harder test than the Adams transfer in `RESULTS.md`, for three
reasons. The source is by a third author, so nothing can be carried over. The
gap is maximal — first-person gothic terror against third-person comic
fantasy. And at 5,750 tokens it is the first run in the **supported** length
tier (5,000+), rather than the marginal one every earlier number came from.

The critique instrument is `terry-pratchett-voice.json`: the same corpus, fitted
**without** the contrast family. `RESULTS.md` recommended that; this run tested
it. On the same draft the contrast reference reports 174 findings and the voice
reference reports 10, and the 164 difference is almost entirely demands to use
the words *deep*, *pit* and *rats* less often in a story about a pit with rats
in it.

## Iterations

| draft | tokens | p_same | pass | gated med+ | findings | what changed |
|---|---|---|---|---|---|---|
| source-typo | 6,113 | 0.0390 | ✗ | 3 | 46 | Poe, target typography only |
| draft1 | 5,058 | 0.1095 | ✓ | 4 | 29 | full rewrite, third person |
| draft3 | 5,411 | 0.1895 | ✓ | 0 | 17 | dialogue added |
| draft4 | 5,676 | 0.2330 | ✓ | 0 | 12 | more dialogue, commas stripped |
| **final** | **5,750** | **0.2175** | ✓ | **0** | **10** | commas restored, metric plays reverted |

## Against the gate

| criterion | result |
|---|---|
| `verdict.pass` (Balanced ≥ 0.10) | ✅ 0.2175 |
| Strict (≥ 0.25) | ❌ 0.2175 — clears 0.25 on the contrast reference (0.356), not on this one |
| `canary_ok` / `drift_ok` | ✅ both true throughout |
| Zero Medium+ on Device/Frames/Syntax/Rhythm | ✅ from draft3 on |
| Away from source | ✅ distance to Poe 0.2191 → 0.3477; `p_same` 0.3405 → 0.0055 |
| General Imposters | ✅ `target_disfavoured` (0.00) → **`target_favoured`** (0.92) |
| External wit critic | ✅ now run — see [`JUDGE-RESULTS.md`](JUDGE-RESULTS.md) |
| Length tier | ✅ supported |

The away criterion and GI both **pass** here, and both failed in the Adams run.
That is the expected shape: Adams and Pratchett are neighbours, Poe is not.

Replacing the "no judge available" line above: the external wit-critic gate has
since been run — five independent blind LLM judges on two held-out passage
tests, in `JUDGE-RESULTS.md`. The headline: all five judges put the rewrite
closer to Pratchett's voice than the Poe source on 6/6 pairs, and 1–4 of 6
short passages passed as “genuine Pratchett” in a spot-the-fake test, with the
defeats concentrated where the real anchor was a plain interior passage and the
wins-by-stereotype pattern flipped on real signature set-pieces.

## What the run showed

**Third person was the instrument's decision, not a preference.** The baseline
flagged `syn.person.p1s` at 76.39 against a band of 7.54–32.70 and
`biber.pron.p3` at 8.67 against 30.03–59.76. Poe's first-person interior
monologue is the single largest thing separating the two authors.

**First person lives inside quotation marks.** After going third person, draft1
had *zero* first person — and the reference asked for it back. The band wants
both: third-person narration at 30–60 per 1k *and* first person at 7.5–32.7,
which is a corpus of third-person narrative carrying heavy first-person
dialogue. Adding dialogue moved ten dimensions at once: `p1s`, `p1p`, `p2`,
`public_verb_rate`, `question_rate`, `exclaim_rate`, `interjection_rate`,
`quote_single_curly_rate`, `sentence_initial_lower`, and `pron.p3` by dilution.
That co-variance is invisible finding-by-finding.

**The two-sided bands caught over-firing three times.** draft1 over-fired the
punch sentence (`sent.punch_short_rate` 10.04 against a ceiling of 8.07) — the
caricature failure the anti-caricature guard exists for, committed while trying
to sound like Pratchett. Then a blunt comma strip in draft4 pushed
`serial_comma_share` *under* its floor (0.14 against 0.15) and dragged
`comma_rate` under too. The upper and lower edges both did their job.

**Clearing findings is not the same as passing the gate.** draft4 → draft5
(not shown) cut findings 12 → 10 and *lowered* `p_same` 0.2330 → 0.2205. The
skill's warning is correct and it is easy to violate by accident.

**Two dimensions are content-bound, not style-bound.** `punct.quote_single_curly_rate`
wants 31.92–162.47 per 1k; a story about one man alone in a dark room cannot
reach that without inventing characters, which the drift guard exists to
prevent. All dialogue added here is speech the source states or reports — the
sentence, the rumours, the two lines Poe already quotes, and the prisoner
talking to himself. `biber.pron.p3` stayed above band for the same reason: a
single-protagonist narrative has denser third-person pronouns than a corpus of
crowded novels. Neither is a defect in the draft. Both are the target voice
asking for a kind of *scene* the source does not contain.

**`serial_comma_share` is misnamed.** It is
`(", and " + ", or ") / (" and " + " or ")` — every comma before a
conjunction, not the Oxford comma in a list. Driving it down therefore means
deleting commas between independent clauses, which is ungrammatical. The final
draft restores them and reports the dimension rather than satisfying it. Two
other metric plays were reverted for the same reason: a sentence deliberately
started in lower case to feed `sentence_initial_lower` simply read as a typo.

## The one number to distrust

`p_same_author` rose monotonically from 0.039 to 0.233 and then settled at
0.218 while the prose got *better*. Anyone quoting a single figure from this
table is quoting noise at the third decimal. The result that carries weight is
the pair of categorical verdicts: `target_disfavoured` → `target_favoured` on
General Imposters, and the distance to Poe increasing by more than half.
