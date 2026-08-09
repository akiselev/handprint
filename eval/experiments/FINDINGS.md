# Findings — problems worth writing about

A running log of things that went wrong, in the order they were found. Kept
separate from the per-experiment RESULTS files because these are *about the
method*, not about a cell, and because a problem recorded only after it was
fixed reads as foresight rather than as a finding.

Each entry: what broke, how it was caught, what it cost, and whether it is
fixed. Entries are not deleted when fixed.

---

## 1. A rate is not a distribution

**Found:** matrix round 1, from four blind judges independently.

Every device band is a rate per thousand tokens. Prose that puts one joke in
every paragraph and prose that puts eight jokes in two paragraphs and none in
the next six have the same rate, and every judge named the difference first and
unprompted: *"nearly every beat gets a simile or aphorism, where the author
would let some plain sentences breathe."* All of that prose sat inside every
band.

**Cost:** the showcase story cleared the gate with zero Medium-or-above findings
on Device, Frames, Syntax and Rhythm, and read as exhausting.

**Fixed, partly.** Four spacing dimensions added (`dev:device_burstiness`,
`dev:device_para_share`, `frame:figure_burstiness`, `frame:figure_para_share`).
They catch 2 of the 4 drafts judges called overwritten. `figure_burstiness` is
under-powered below a dozen figures and `figure_para_share` is confounded by
paragraph length — Melville scores highest of anything because he writes six
enormous paragraphs. The honest version normalises per sentence, not per
paragraph. Not done.

## 2. Content contamination outranks every style signal

**Found:** matrix round 1, named first by every judge.

A rewrite of *Crime and Punishment* into Adams still contains Raskolnikov. The
judges do not care how the sentences move. This is not a bug in the pastiche;
it is a property of the task as arm A designs it, and it means arm A cannot
separate voice from contamination.

**Cost:** 6/6 losses on spot-the-fake at maximum confidence, and a
recommendation to drop the `contrast` family that was half wrong — the family
was flagging exactly this and it was mislabelled "topic, not style".

**Addressed, not fixed:** arm B (`samecontent/`) holds content constant so the
two can be separated. The underlying tension is inherent to voice transfer.

## 3. Rate collision: a voice has rates its content cannot supply

**Found:** twice, independently — dialogue in Poe→Pratchett, similes in
Melville→Jerome.

Where the source's own figure density, dialogue density or narrative person
differs from the target's, style transfer and content preservation are in
direct conflict, and iteration does not dissolve it. Melville is simile-dense
at 4.62/1k; Jerome's band is 0.78–2.39. Reaching Jerome means deleting
Melville's figures, which the drift guard exists to prevent.

**Not fixable.** It is a property of the task. What it changes is what a
corpus-mimic critic can honestly be asked for.

## 4. `p_same_author` is not quality at working length

**Found:** C15 (Jerome → Doyle) — the metric's worst cell (0.184 → 0.090) was
the judges' cleanest prose and the only draft never flagged overwritten.

**Compounded by** the noise floor: a genuine Adams passage of 1,015 tokens
scores **0.533** against its own reference with 7 gated findings. That is the
ceiling, not 1.0, and a rewrite at 0.3 is much closer than it reads.

**Not fixed.** Mitigated by reporting per-family findings as the evidence and
treating the distance as indicative, per `LengthTier::Unreliable`.

## 5. An author with two registers is a bad target

**Found:** Wodehouse, during matrix construction — his sampled passage scored
closer to Twain's reference than to his own, because his narratorial
chapter-openings are mock-philosophical essay and his dialogue is a different
instrument.

**Also true of Adams, mildly.** *The Salmon of Doubt* is 19% of the Adams
corpus and much of it is transcribed speech and interviews. Measured against
the Adams reference:

| Adams material | p_same_author | findings |
|---|---|---|
| Hitchhiker's | 0.906 | 2 |
| Salmon — fiction | 0.931 | 3 |
| Long Dark Tea-Time | 0.760 | 1 |
| Salmon — essays/interviews | **0.559** | 7 |

The findings on the outlier are spoken-register markers: `sent.marker.essentially`,
`basically`, `specifically`, `sent.opener.so`. It widens the bands; it does not
split the author the way Wodehouse splits.

**Open.** A fiction-only Adams reference is the obvious control and has not
been fitted.

## 6. The corpus had other people's prose in it

**Found:** by auditing the built corpora rather than the scripts, 2026-08-08.

Everything below was inside the Project Gutenberg markers or inside a kept EPUB
spine item, so every existing rule passed it through.

| leak | where | size |
|---|---|---|
| a 1,505-word modern historical essay | Twain, *Pudd'nhead Wilson* | filed as Twain |
| the etext's own provenance page (a 486 running DOS) | Milton, *Paradise Lost* | 325 words |
| transcriber's notes | Melville, Dickinson, Wodehouse | 3 books |
| the older "End of Project Gutenberg's *Title*, by *Author*" sign-off | Twain, Wodehouse | 2 books |
| publication slugs and plate indexes, once per part | Twain, *A Tramp Abroad* | 11 blocks |
| publisher back matter appended to a real chapter | Adams, Pratchett | 4 documents |
| "Books by Douglas Adams" / "Also by Terry Pratchett" pages | Adams, Pratchett | 5 documents |
| an interviewer's prose about Adams | *The Salmon of Doubt* | 1 document |
| duplicate chapters — the YA EPUBs bundle the next book's opening as a preview | Pratchett | 6 groups |
| near-empty front matter: a **1-word** document in the Dickens fitting set | 11 authors | 1 doc each |

**Fixed**, with tests, and every removal is now reported rather than silent.
Residual: the Salmon interview sections are still in, because telling the
interviewer's questions from Adams's answers needs a person.

## 7. Two false positives while fixing #6, both instructive

**A numbered list is not a plate index.** The first version of the plate-list
rule deleted Watson's list of Holmes's limits — *"1. Knowledge of
Literature.—Nil. 2. Philosophy.—Nil."* — one of the most characteristic
passages Doyle wrote. Plate captions are set in capitals and a novelist's list
is in sentence case; that is the whole of the difference, and the rule now
requires capitals.

**"Books by" is a heading, not a substring.** The first version of the
back-matter rule cut 765 words out of *Guards! Guards!* at *"he explored the
spines of the books by his side"* and 4,513 words out of *Monstrous Regiment*.
It now requires a full-line match on a line of eight words or fewer.

**The lesson is the same both times:** a rule written from the shape of the
boilerplate will also match prose, and the only reliable discriminators were
*typographic* (capitals, line-anchoring), not semantic. Both were caught by
diffing corpus word counts before and after, not by reading the rules.

## 8. A silent normalisation failure looks exactly like "nothing was there"

**Found:** the transcriber's-note rule reported zero removals for Moby-Dick
while the note was plainly still in the corpus.

The file is CRLF. `"\r\n\r\n"` does not contain `"\n\n"`, so the paragraph-break
search never matched, the note appeared to run to the end of the book, and the
300-word guard suppressed it. The rule's *report* said nothing had been found,
which is indistinguishable from the file being clean.

**Fixed** by normalising line endings before any structural scan, with a
regression test in DOS line endings.

**The general problem is not fixed:** a guard that suppresses an action also
suppresses the evidence that the action was needed. Every bounded rule here has
this shape.

## 9. A dropped subdirectory is invisible in the output

**Found:** the Pratchett corpus came out 247 documents and 600k words light,
and nothing in any log said so.

The rebuild globbed `corpus/terry-pratchett/*.epub` and the collection keeps
the Tiffany Aching books in `ya/`. Seven books — a fifth of the corpus, and the
fifth containing the duplicated preview chapters the dedupe rule exists for.

**Fixed** (recursive find). **Caught only by** comparing corpus totals against
the previous build. There is no assertion anywhere that a rebuild produces at
least as much corpus as it did last time, and there should be.

## 10. Exact-hash dedupe finds nothing after a resize

**Found:** the dedupe rule reported zero duplicates on a corpus that visibly
had six duplicate groups.

The resize cuts every book into 2,500-word pieces counting from the top of the
document. The same three chapters, appearing as a preview at the end of book 30
and as chapters 2–4 of book 32, come out at different offsets, so no two output
files are ever byte-identical.

**Fixed** by deduplicating whole sections *before* the resize. Anything that
survives a re-cut — a near-duplicate, a revised edition — still gets through,
and would need shingling.

## 11. `p_same_author` is not comparable across references

**Found:** scoring the same-content arm, where eight passages with identical
content are scored against all 28 references.

Some references say yes to everything. Mean `p_same_author` assigned to eight
passages their author did not write:

| reference | mean | max | corpus |
|---|---|---|---|
| hermann-hesse | **0.129** | 0.205 | 15 docs |
| l-m-montgomery | **0.103** | 0.161 | 42 docs |
| john-milton | 0.067 | 0.118 | 19 docs |
| … | | | |
| edgar-allan-poe | 0.004 | 0.007 | 119 docs |
| michel-de-montaigne | 0.002 | 0.005 | 198 docs |
| homer | 0.000 | 0.001 | 125 docs |

Two orders of magnitude, on identical text. "0.05 toward Hesse" and "0.05
toward Homer" are not the same statement, and the matrix compared cells against
each other on exactly this number.

**Cause is not simply corpus size** — Machiavelli has 18 documents and is near
the bottom at 0.009. Two mechanisms look live and are not yet separated:
proximity to plain unmarked prose (Hesse in translation, Montgomery, Wodehouse,
Wells all cluster high) and high within-corpus variance widening the bands
(Milton, whose 2,500-word cuts of blank verse are wildly unlike each other).

**Not fixed.** The fix is a per-reference normalisation — report a percentile
against that reference's own background rather than a raw probability — and
until then no cross-cell ranking in `matrix/` should be believed.

## 12. Gate 1 fails once the reference set gets bigger

**Found:** rechecking the "source passage must rank its own author first" gate
against 28 references instead of 14, then running it over the whole set —
56 held-out passages, two per author, ~1,200 words each (`matrix/pairs.py`).

**30 of 56 pass. 18 of 28 authors fail at least one sample.**

| passage | own-author rank | who beat it |
|---|---|---|
| real Adams | 1 | — |
| real Pratchett | 1 | — |
| real Melville | 2 | Milton |
| real Austen | 2 | E. Brontë |
| real Poe | 3 | Hesse, Milton |
| real Jerome | **6** | Montgomery, Hesse, C. Brontë, … |

Three references account for 17 of the 26 failures: Montgomery (6), Milton (6),
Hesse (5). The same gate passed 13 of 14 sampled passages when the comparison
set was 14 references. Adding thin and plain-prose references broke it, without
any of the passages or the authors changing.

**Twain's reference is worse than broken:** it scores foreign passages *higher*
than its own (own 0.067, foreign 0.097, ratio 0.7 — the only ratio below 1).
His corpus spans travel writing, boys' novels, courtroom satire and river
memoir; #5 again, at 283 documents rather than 15.

**This is #11 seen from the other end**, and it means the gate as written
measures the composition of the comparison set as much as it measures the
passage. A gate whose verdict depends on who else is in the room is not a gate.

## 13. Most of arm A's measured movement may be era vocabulary

**Found:** by comparing arms. Arm A rewrites period prose into period voices
and scores 0.08–0.22 toward target. Arm B renders modern content into the same
voices and scores 0.004–0.071 — three to five times lower, with the same
writer, the same references and the same rubric.

All eight arm-B renderings moved *toward* their target relative to the
untouched source, 8/8, so voice transfer does register. But the magnitude
collapses when the source stops sharing a century of vocabulary with the
target, which suggests a large share of arm A's number was never voice.

**In both arms the nearest reference is almost never the target.** Every arm-A
cell's nearest is Montgomery or Hesse at 0.32–0.42, well above its 0.08–0.22
toward its actual target.

**Open.** The clean test is a period-neutral source rendered into two targets of
different eras, which arm B can be extended to do.

## 14. The most common defect in everything written here is invisible to the profile

**Found:** blind readability judging, 14 pairs, arms A and B together.

`explained` — the passage restating its own joke, image or irony in the next
clause — was flagged on **8 of 9 distinct passages**, by four independent
judges who could not see each other's answers and were not asked to look for
it. Every one of those passages had already cleared its fidelity gate.

Two others recur: `every_beat_lands` (7 passages) and `costume` — period
mannerism laid over ordinary prose (3 passages, all of them the pre-1900
targets).

**Nothing in the feature set measures any of the three.** The spacing
dimensions added after the last round address `every_beat_lands` and catch
about half of it. `explained` and `costume` have no dimension at all, and
`explained` is the commonest.

## 15. Fidelity carries no information about readability

**Found:** arm B, where seven renderings of identical content have both a
`p_same_author` toward their target and a blind readability record.

Spearman ρ ≈ **+0.28** across seven renderings — zero, at this n. The
lowest-fidelity rendering (Poe, 0.004 toward Poe) has the best readability
record (2/2); the second-lowest (Melville, 0.008) has the worst (0/2).

**One rendering lost to doing nothing.** The Melville version was judged worse
than the untouched flat source, and drew every flag in the rubric. A voice
whose defining move is the expansive gloss, applied to a small administrative
incident, is worse than no voice at all.

The prediction was an anti-correlation above some fidelity. The data is more
awkward than that: at the magnitudes this instrument produces, the objective
function is not harmful, it is **uninformative**. Optimising it neither helps
nor hurts the thing the artifact is for.

## 16. Blending two references beat either alone

**Found:** the same arm, one blended rendering, two judgements. A signal, not a
result — recorded because it is the first evidence for the idea.

The Adams × Pratchett blend went 2/2 and beat **pure Adams clearly**, on
identical content, from the same writer. What it gained was restraint: Adams's
structure with Pratchett's willingness to end a paragraph on a flat sentence.

That is precisely the axis a single-target critique loop pushes the wrong way.
The loop reports `direction: increase` on every under-fired device and has no
way to say "this passage needs fewer good lines", because a rate cannot.

## 17. A blind readability judge catches continuity errors nothing else does

**Found:** the Adams rendering was flagged `sense: confusing` — its opening says
the lift had "spent those eleven days working perfectly", which contradicts
Ellen pressing the button every morning for eleven days and getting nothing.

The drift guard compares against the plain retelling and did not fire; no
dimension in the profile represents narrative consistency at all. This is the
most damaging class of defect for an actual reader and the pipeline is
completely blind to it. The `sense` axis in `judge/readable.md` was added for
`confusing`/`contradictory` verdicts and immediately earned its place.

## 18. CORE has no paragraphs, and that kills four dimensions silently

**Found:** measuring the corpus before loading it. One document in 4,845
contains a newline; none contain markdown.

CORE is an HTML-to-plaintext export, which `../corpora/README.md` already warns
"destroys exactly the signal being measured". Consequences:

* `dev:device_para_share` and `frame:figure_para_share` — the two dimensions
  added in round 2 to catch overwritten prose — measure nothing. The whole
  document is one paragraph, so the share is 1.0 or 0.0.
* `md:link_rate` and `md:footnote_rate` are structurally zero. The `tech-blog`
  claim in `battery/matrix.py` rests on those two and **cannot be tested on
  CORE at all**.

None of that raises an error. A dimension that is structurally zero looks
exactly like a register that never uses the feature.

**Not fixable in CORE.** Recorded per register in `build/core-qc.json` so the
caveat travels with the data, and it is why `software-docs` is built from
markdown sources rather than scraped pages.

## 19. Space before punctuation: a scrape artifact that would have been read as style

**Found:** the same audit. Median 1.07 per 1k words across CORE, 20% of
documents above 5, one at 103.

Stripping tags leaves the space that separated them — `Nothing Lasts Forever ,
the sequel`. `punct` is an entire feature family, and this is a property of the
scraper, not of any writer. Repaired deterministically at ingest; residual is
now 0.00 per 1k in every register.

**The general shape is the one worth writing about**, and it is the same as #7:
the artifacts that matter most are *typographic*, they are invisible in a word
count, and they land squarely inside a feature family that would otherwise be
one of the most reliable.

## 20. Counting changed characters is not counting repairs

**Found:** the first version of the CORE loader reported **33 million
typography repairs** on the news register — more repairs than words.

Deleting one space shifts every character after it, so a position-by-position
diff calls the whole document changed. Counting `re.subn` results instead gives
36,033. The wrong number is not merely inflated, it is uninterpretable: the
count exists to show whether a rule fired more often than the corpus can
justify, and at 33 million it cannot show anything.

## 21. The marketing claim fails its first real control

**Found:** by adding the instructional registers the battery never had.

`battery/matrix.py` claims marketing is high on `dev.imperative_rate`,
`lex.marketing-eval.cat.pressure` and `…cat.directive`. With `how-to`,
`recipe`, `advice` and `software-docs` on disk, all three are testable for the
first time. Median rates over 20 documents each:

| register | imperative_rate |
|---|---|
| recipe | **6.73** |
| advice | 4.31 |
| software-docs | 3.00 |
| how-to | 2.25 |
| **marketing** | **0.54** |
| news / encyclopedic / reviews | 0.00 |

Marketing is **below four instructional registers**, and recipe is 12× higher.
`dev:imperative_rate` measures the speech act, not the sell.

The other two dimensions are worse than wrong, they are absent: `pressure`
fires in **1 of 20** marketing documents (and is *higher* on news), `directive`
in 5 of 20. A gate cannot rest on a dimension that is zero in three quarters of
its own register.

**Caveat, stated because it is a real alternative explanation:** this marketing
corpus is CORE's `IP`+`DS` classes — 2012–2014 web persuasion. The
`marketing-eval` pack may have been built against modern SaaS landing copy, in
which case the finding is "the pack does not generalise backwards" rather than
"the pack does not work". Distinguishing those needs a modern marketing corpus,
which the pre-2022 rule forbids.

Either way the claim as written is not supported by the only evidence there is.

## 22. Registers are a space, not a partition — and the thin ones still lose

**Found:** 75 held-out documents, 5 per register, scored against all 15 register
references (`registers/discriminate.py`).

**Top-1 22/75 (29%), top-3 43/75 (57%)** — far below the author equivalent, and
partly for a legitimate reason. An advice column is a kind of blog; a sports
report is a news report; a recipe is a how-to. Authorship is a partition and
register is a neighbourhood, so "beaten by advice" and "beaten by legal-terms"
mean opposite things and the report prints both.

Clean discriminators (median rank 1): **academic, encyclopedic, news,
opinion-blog** — the registers with distinct grammar rather than distinct
subject matter.

The two worst are **recipe (median rank 11, 141 documents)** and **legal-terms
(rank 10, 145 documents)** — the two thinnest corpora, which is finding #11
replicating on a completely different corpus with completely different text.
Whatever is wrong with thin references is not a property of the author battery.

## 23. The gate rejects 96% of the corpus it was fitted on

**Found:** trying to detect AI documentation prose with the `software-docs`
reference.

24 real human documentation pages of 380–700 words, scored against the
reference fitted on their own corpus:

| | p_same_author | gated findings |
|---|---|---|
| real human pages (n=24) | median **0.032** (0.014–0.178) | median **7.5** |
| cold AI draft | **0.082** | 8 |
| AI draft after revision | 0.027 | 15 |

* **The cold AI draft scored higher than 22 of 24 real human pages.**
* **1 of 24 human pages passes the gate** (`p_same_author >= 0.100`).
* Gated-finding counts are indistinguishable: human median 7.5, AI 8.

**Amended after refitting on the complete corpus.** Those numbers came from a
reference fitted on a partial 489-document snapshot taken while the fetch was
still running. Refitted on all 1,077 documents, 30 human pages give a median of
**0.155** and **19 of 30 pass the gate**. So "the gate rejects 96% of its own
corpus" was substantially an artefact of an under-fitted reference, and the
honest version is narrower:

| | p_same_author | gated findings |
|---|---|---|
| 30 human pages (full reference) | median **0.155**, max 0.338 | median 10 |
| cold AI draft | **0.211** | 21 |
| revised AI draft | 0.098 | 17 |

**The inversion survives the fix.** The cold AI draft still scores above 24 of
30 real human pages, and above the revision a blind judge could not tell from a
human page. This is finding #4 in its sharpest form: not "the metric is noisy at
short lengths" but "the metric ranks the machine above the humans".

**Recording the mistake as well as the correction:** I published the 96% figure
from a reference I had built minutes earlier from a directory that was still
being written to, and said so at the time — but reported the number as a
finding anyway rather than waiting for the corpus to finish.

## 24. Acting on the findings made the metric worse, in a new way each pass

**Found:** the same experiment. The cold draft's eight gated findings were all
real and interpretable — zero parentheses, zero asides, over-bolding, private
verbs, freshly coined adjective–noun pairs. I acted on every one.

The revision scored **0.082 → 0.027** with **8 → 15** gated findings, and the
new findings were an almost disjoint set: em-dashes, indefinite pronouns,
synthetic negation, `It`-openers, serial commas, register clash.

Two things happened at once and both are worth separating:

1. **The loop has no fixed point at this length.** Each pass reports whatever
   the current draft's largest deviations are, so acting on all of them
   guarantees a new list rather than a shorter one.
2. **I revised into my own voice, not the corpus's.** Em-dash-heavy, punchy,
   aphoristic. That is further from technical documentation than the AI draft
   was, and the metric was right to say so even though it cannot say why.

The skill's `canary_ok` guard exists for exactly (1) and did not fire, because
the guard watches a held-out family rather than the *identity* of the findings.
A cheap addition: flag when a revision's findings barely overlap the previous
pass's — that is the signature of chasing the report.

## 25. The AI signature is not one thing across registers

**Found:** comparing 60 AI agent-prose documents against 60 human documentation
pages on the dimensions the cold draft was flagged for.

| dimension | human docs | AI prose | AI docs outside human IQR |
|---|---|---|---|
| `md.bold_rate` | 0.56 | 26.80 | **100%** |
| `punct.em_dash_rate` | 0.00 | 18.78 | **100%** |
| `sent.aside_rate` | 14.01 | 52.33 | 95% |
| `md.link_rate` | 6.86 | 0.00 | 93% |
| `punct.paren_rate` | 22.55 | 67.92 | 85% |
| `lex.hyland.cat.hedges` | 7.43 | 2.54 | 82% |

Six dimensions separate the populations cleanly, which is the encouraging half.
The hedging result independently reproduces the published direction the
register matrix borrows — humans hedge *more* — at 3× rather than 40%.

**The awkward half:** the AI *documentation* draft went the opposite way on two
of them. It had **zero** parentheses and **zero** asides, where the AI chat
corpus has three times the human rate. Asked for documentation, the model
strips parentheticals; asked to explain, it drowns in them.

So there is no single "AI signature" to subtract. A de-AI critic has to be
register-matched, and a rule learned from chat transcripts will point the wrong
way on a README. That is the strongest argument yet for the register corpora,
and it is also a warning about `agent-prose` being fitted from chat and applied
to everything.

## 26. Stripping code from the corpus taught the reference that docs contain no code

**Found:** the first critique of the AI draft reported
`md.code_fence_rate` **high, band 0.00–0.00**.

`fetch_docs.py` strips fenced blocks, correctly — a page half made of Python
would otherwise make the reference measure Python. But the draft was critiqued
unstripped, so the comparison was prose-plus-code against prose, and the loudest
finding in the report was an artifact of my own pipeline.

**Fixed** by running the draft through the same `strip_code` before critiquing.
The general rule is the one this keeps re-teaching: **any normalisation applied
to the corpus must be applied to the draft**, and nothing in the tool enforces
that or can detect the mismatch.

## 27. The revision a blind judge could not distinguish is the one the metric downgraded

**Found:** three blind pairwise judgements on the de-AI drafts, judge with no
access to the working conversation, asked which passage was machine-written
before any analysis.

| pair | called machine-written | confidence | preferred |
|---|---|---|---|
| cold AI vs revised AI | **cold AI** — correct | 5/5 | revised |
| real human page vs cold AI | **cold AI** — correct | 5/5 | human |
| real human page vs revised AI | **"cannot tell"** | 2/5 | revised |

The judge caught the cold draft at maximum confidence twice and could not
separate the revised draft from a genuine human documentation page. Over the
same two drafts the metric moved **0.082 → 0.027** and **8 → 15** gated
findings, i.e. it reported the successful revision as a large regression.

This is finding #15 stated as strongly as the evidence allows: the objective
function is not merely uninformative about readability, **it inverted on the
one case where a reader's verdict was unambiguous.**

**Caveat that weakens the third row, raised by the judge itself:** the human
page went through `fetch_docs.py`, so its code blocks are stripped and "roughly
a dozen instructions terminate in a colon with no snippet". That handicapped it.
The judge still rated the two "close to equal on the underlying authority".

## 28. Four of the judge's five tells have no dimension at all

The judge named its tells with quotations. On the cold draft:

1. **`It's <adjective> to` openers** — four of them, three in section-opening
   position.
2. **Template scaffolding** — Overview / Prerequisites / Best Practices /
   Troubleshooting / Conclusion, "rather than headings that name this tool's
   actual stages".
3. **Empty list-closers** — *"Each of these stages plays a crucial role in
   ensuring…"*, a sentence that restates the list above it.
4. **Filler intensifiers** — *"crucial role"*, *"streamlined workflow"*,
   *"important pieces of information"*, *"simply run"*.
5. **A Conclusion that restates the title and delivers a moral.**

Counted directly: the cold draft has **4** `it's <adj> to` constructions, **5**
template headings and **8** filler adjective+noun pairs. The revision has
**0, 0, 0**.

Only (4) reached the critic, as `dev.novel_adj_noun_rate`. Worse, the one
dimension that should have caught (1) — `sent.opener.it` — **was not flagged on
the cold draft and fired at High on the revision**, which has none of them.

And two packs that exist for precisely this were never fitted into the register
references: `doc-style` (weasel words, wordy phrases, clichés) and `marketing`.
My `FEATURES` list in `corpora/refit.sh` omits both. For a *documentation*
register that is the single most obviously relevant pack in the crate.

**Actions, in order of value:**

1. Add `doc-style` to the register feature set and refit. Free, and it targets
   tells 3 and 4 directly.
2. Work out why `sent.opener.it` inverted. A dimension that fires on the clean
   draft and not the dirty one is worse than a missing dimension.
3. There is no dimension for *section-heading vocabulary*. "Overview /
   Prerequisites / Conclusion" versus headings naming the actual subject is a
   strong, cheap, register-specific AI tell and nothing measures it.

## 29. `git rev-list --before` is not a UTC cutoff, and the manifest showed it

**Found:** by reading the manifest I had just cited as proof the corpus was
pre-2022.

`fetch_docs.py` selects each project's last commit before `--before=2022-01-01`
and records the timestamp, and I reported that as making the cutoff "a commit
hash and a timestamp rather than a claim about a dataset". Checking the
timestamps rather than trusting them: **17 of 44 projects sat hours into 2022 in
UTC.** Terraform at 2022-01-01T15:05Z, Kubernetes at 16:22Z, Spark at 04:30Z.

`git rev-list --before` resolves a bare date in the machine's *local* timezone
and compares against the committer date, so the boundary lands wherever the
runner happens to be.

**Fixed** by verifying in epoch seconds — where there is no timezone left to be
wrong about — and walking back to the first commit genuinely below the cutoff.
Every project now verifies: latest commit is pandas at 2021-12-31T23:58:47Z.
`committed_epoch` is recorded per project so the check is repeatable.

The contamination risk was negligible — the first few hours of 2022, ten months
before ChatGPT. **The failure that matters is not the dates, it is that I
published a verification claim without running the verification**, and the
evidence was sitting in the file I was quoting from. A constraint the user
states explicitly deserves an assertion, not an argument.

## 30. Three packs the battery's own claims are written in were never fitted

**Found:** chasing why the critic missed four of the blind judge's five tells.

`corpora/refit.sh` fitted every register with a feature list that omits
`doc-style` (weasel words, wordy phrases, clichés), `hyland` (hedges, boosters)
and `marketing` (pressure, directive). All three appear in claims the battery
makes:

* the marketing claim names `lex.marketing-eval.cat.pressure` and `…directive`
* the register matrix's headline separator is the booster:hedge ratio, i.e.
  `lex.hyland.cat.*`
* `doc-style` is the only pack in the crate aimed at the register the de-AI
  mode exists for

A missing pack does not error. Its dimensions are simply absent from the
report, which is indistinguishable from a rate of zero — and that is exactly
how the empty `pressure` and `directive` columns first read.

**Fixed:** the register feature set now adds all three; authors keep the
original list so the v3 author references stay comparable.

## 31. The doc-style HIGH count is the first thing here that actually detects AI prose

**Found:** re-running the de-AI experiment after adding `doc-style` to the
register feature set (#30) and refitting on all 1,077 documentation pages.

Count of **High**-severity `lex.doc-style.*` findings:

| | doc-style HIGH |
|---|---|
| 30 real human pages | median **1**, max 5, **0 of 30 at ≥6** |
| cold AI draft | **6** |
| revised AI draft | **1** — exactly the human median |

Clean separation, on the one axis that was missing an hour earlier. And the
findings name the judge's tells almost word for word:

* `lex.doc-style.style.best_practice` — the judge's "generic doc-template
  headings (Overview/Prerequisites/**Best Practices**/Troubleshooting/Conclusion)"
* `lex.doc-style.word.simply` — the judge's *"simply run"* filler intensifier
* `lex.doc-style.struct.title_case_heading` — the same template-scaffolding tell
* `lex.doc-style.word.several`, `.most`, `.sufficient`, `.rather` — weasel
  words: *"several distinct stages"*, *"Sufficient disk space"*
* `lex.hyland.cat.boosters` — the remaining filler intensifiers

**Four of the blind judge's five tells are now instrumented, up from one.** The
fifth — a Conclusion that restates the title and delivers a moral — is
structural and still has no dimension.

**The scalar and the findings now point in opposite directions on the same
pair,** which is the cleanest statement of the whole problem this repo has:

* `p_same_author`: cold AI **0.211**, revised **0.098** — prefers the machine.
* `doc-style` HIGH: cold AI **6**, revised **1** — prefers the revision, and
  agrees with the blind judge that could not separate it from a human page.

The instrument was never wrong about the prose. It was wrong about which number
to put on the front.
