# Docs workflow experiment (2026-10-26)

The first run of the documentation workflow in `../../../DOCS.md` and `../../../DOCS2.md`: four pages
for cadatomic and svelters, written by agents, checked against the code, polished with handprint,
judged blind, cross-checked, and finally checked by running the software. Everything in this
directory is the record; the published pages are private artifacts the user holds.

## The four pages

| id | project | type | topic |
|---|---|---|---|
| `cad-tutorial-led-board` | cadatomic | tutorial | an LED board from an empty project to fabrication files |
| `cad-howto-erc` | cadatomic | how-to | run the schematic ERC, fix or waive what it finds |
| `sv-tutorial-first-skelter-app` | svelters | tutorial | a Svelte counter in a native skelter window |
| `sv-howto-skelter` | svelters | how-to | add a native widget kind and use it from Svelte |

## Pipeline as run

1. **Writer** (one agent per page): fact sheet with `path:line` per claim, then a cold draft
   (`draft0.md`). Rules in `BRIEF.md`.
2. **Verifier** (a different agent): every claim and every factual sentence against the code
   (`verify.md`). For the skelter tutorial the verifier also built and ran it under Xvfb.
3. **Reviser**: fix every verified issue (`draft1.md`), then up to three handprint passes
   (`draft2..4.md`, `final.md`, rules in `LOOP.md`) against a reference fitted on human
   documentation pages written before 2022.
4. **Blind judge** (a different model from the writers): per page two pairs, A = `draft0` vs
   `final` (the whole pipeline) and B = `draft1` vs `final` (handprint's polish alone), each pair
   judged twice in swapped order by separate agents (`JUDGE.md`, `judging.json`).
5. **Cross-page check** against each other and the projects' known-bug lists (`CROSSCHECK.md`).
6. **Live check**: cadatomic built at its pinned sibling commits and driven with real input under
   Xvfb (`live/LIVE.md`), because steps 2 and 5 disagreed about what works.

The published page is `published.md`: the judge-preferred version plus the step 5 and 6
corrections.

## Results

### Verification found what the writers got wrong

| page | claims | verifier issues | worst |
|---|---|---|---|
| LED tutorial | 91 | 18 (5 high) | wrong palette shortcut; a part never renamed so "choose J" fails; a Name form that never appears from the menu |
| ERC how-to | 69 | 18 | a "Go to the place" step on a finding with no place; a pin fix that needs **Update** from the library too; keys that need focus on the sheet |
| skelter tutorial | 63 | 14 | a missing system library that crashes on X11; two troubleshooting messages that don't match what the app prints (found only by running it) |
| skelter how-to | 60 | 22 | the widget spec registered twice (compile error); an undefined theme sheet so the box is invisible; text that never draws without a text shaper |

Fact sheets were accurate (most wrong facts were line citations); the errors were in the *draft*,
in steps a reader would perform. Every page had at least one error that would stop a reader.

### The blind judges: the pipeline wins, the polish doesn't

| comparison | final won | draft won |
|---|---|---|
| A: cold draft vs final (whole pipeline) | **8 of 8** (6 clear, 2 slight) | 0 |
| B: fact-fixed draft vs final (handprint polish only) | 1 (slight) | **7** (4 slight, 3 coin-flip) |

Every A verdict named a fact or a step ("X gives the steps that actually work"). Every B verdict
called the pages near-identical and decided on structure: the judges preferred numbered single
actions, bulleted prerequisites and bold UI labels, which the handprint passes had reduced because
the reference (mostly developer prose) has fewer of them. No judge called any page
machine-written (32 page readings, all "can't tell" at confidence 2).

### What handprint saw

Reference v1: 897 human documentation pages (41 projects, pre-2022), calibrated per project;
30 pages from six further projects held out as the human baseline.

| | high findings | medium | doc-style high | p_same_author |
|---|---|---|---|---|
| held-out human pages (n=30, median) | 3.5 (max 15) | 5 | 1 (max 5) | 0.061 (14/30 ≥ 0.10) |
| skelter tutorial draft1 → final | 3 → 0 | 2 → 3 | 0 → 0 | 0.112 → 0.150 |
| skelter how-to draft1 → final | 4 → 1 | 4 → 2 | 0 → 0 | 0.007 → 0.007 |
| ERC how-to draft1 → final | 6 → 3 | 12 → 8 | 0 → 0 | 0.000 → 0.005 |
| LED tutorial draft1 → final | 13 → 13 | 13 → 12 | 0 → 0 | 0.000 → 0.000 |

(`scores.json` has every version against both the loop's partial reference v0 and v1.)

- The developer pages sit inside the human range from the first draft; the GUI tutorial sits far
  outside it whatever is done to it. That is register, not quality: tables of coordinates, bold UI
  labels and click sequences are what a GUI tutorial is.
- **No draft, cold or polished, produced a single High doc-style finding** — the separator that
  worked in FINDINGS #31 (cold AI 6, human median 1). Writers given the brief's rules (no template
  headings, no restating closer, no filler) don't produce those tells.
- handprint's polish lowered its own counts and the judges didn't prefer the result: FINDINGS #27
  again, at four pages.

### The cross-checks were the most valuable stage after verification

- **Cross-page:** the LED tutorial (from the fix queue) said the palette shortcut and the Problems
  list don't work; the ERC how-to (from code) relied on both. Each page had been verified on its
  own.
- **Known bugs are not ground truth.** The fix queue's entries were recorded on lane branches; on
  master two were already fixed and the third (typing into Svelte fields) is a test-runner
  readback artifact. A page built from the fix queue told readers only step 1 of the tutorial
  could be done. The live run showed typing works.
- **Only the live run found the real blocker**: no row menu opens in Problems, so **Waive…** can't
  be reached, which the ERC page's code reading and the fix queue both missed.

## What to change in the design

1. **Execution beats every reading-based check.** The two pages that were run (skelter tutorial;
   cadatomic for the disputed claims) are the only ones whose statements about behaviour can be
   trusted. Make "run it" the default verifier for any page that says what the reader will see.
2. **Give verifiers the known-bug lists, and treat those lists as claims to check**, not facts.
3. **Polish needs a register-matched reference**, or it fights the conventions of GUI docs. Until
   one exists (KiCad, Blender, FreeCAD manuals, DOCS.md 1a), use handprint for AI-tell detection
   (doc-style High count) and as a report, not as a rewrite loop.
4. **Style passes drift facts.** One pass invented a claim ("examples only as tests") and three
   sentences shifted meaning; the reviser caught them by diffing. That is DOCS2's sentence→claim
   alignment (2b.15) arriving on cue.
5. **The cheapest big win was the brief**: no cold draft tripped the AI-tell detectors.

## Not done

No task agent (DOCS 2c.14) and no question bank: the live runs answered specific disputes, not
"can a fresh reader do this task". No human read the pages. The skelter how-to was never compiled.
Judges were a different Claude model, not a different model family.
