# The Poe → Pratchett story under external LLM judges

`STORY-RESULTS.md` ended one gate short: the wit critic was "not run — no
judge available". This is that gate, run. Five independent blind judges, each
an **openrouter model in a separate headless `pi` session** with no access to
the working conversation, judged the same six blinded passage pairs from the
final story. Model ids are pinned in the run manifest-style table below;
raw judge output and the pair list are in `eval/build/judge4/` (corpus
artifacts, not committed).

## Protocol

Two tests, six pairs each, balanced A/B presentation by construction
(seed `20260809`), target-voice description supplied, subject matter excluded:

1. **Direction** — the rewrite's paragraph vs. the Poe source paragraph:
   *which is closer to Pratchett's voice?* This is the ordering claim the
   harness actually makes.
2. **Fake** — the rewrite's paragraph vs. a genuine held-out Pratchett
   paragraph: *exactly one is genuine — which?*, plus fidelity 1–5 of the
   imitation and an overwritten yes/no.

Real anchors are from two held-out chapters (`Making Money` ch.4,
`Unseen Academicals` ch.25). Source and rewrite paragraph lengths were
balanced (34–70 words) so the judges could not decide on format.

## Results

| judge model | direction (rewrite closer) | fooled by rewrite | fidelity of rewrite* | overwritten flags |
|---|---|---|---|---|
| deepseek/deepseek-v4-flash | **6/6** | 1/6 | 3.4 | — |
| moonshotai/kimi-k3 | **6/6** | 1/6 | 3.6 | B1 |
| z-ai/glm-5.2 | **6/6** | 2/6 | 2.25 | — |
| qwen/qwen3.8-max | **6/6** | **4/6** | 3.5 | — |
| openai/gpt-5.6-luna-pro | **6/6** | 1/6 | 3.6 | — |

\* fidelity of the rewrite averaged over pairs where the judge *correctly*
identified it as the imitation (glm is the outlier: it rates nearly everything
2/5, including, when fooled, the genuine Pratchett passage — its absolute
numbers should be read relative to its own harshness).

## Pair-level verdicts (R = real author picked, F = fooled)

| pair | real anchor | ds | kimi | glm | qwen | luna |
|---|---|---|---|---|---|---|
| B1 | Moist interior (“weeks since he’d designed a stamp”) | R | R | **F** | **F** | R |
| B2 | Moist interior (“then he thought about this morning”) | **F** | **F** | **F** | **F** | R |
| B3 | Gladys / golem tea ritual | R | R | R | **F** | **F** |
| B4 | “Minutes… looked more like hours” | R | R | R | **F** | R |
| B5 | “YOU DON'T HAVE TO BE MAD…” mug/comma joke | R | R | R | R | R |
| B6 | Tiddles ritual (counted steps) | R | R | R | R | R |

## What the judges said, with quotations

Direction, all six pairs, all models, margin clear — representative tells:

> “deadpan bathetic simile (‘a very large fly in a small church’) vs Poe’s
> Gothic cadence” / “short punchy setup-and-payoff: ‘It was, in fact, the only
> one.’” / “flat comic catalogue and deflating final clause vs Poe’s rhetorical
> exclamations” / “Plain connective tissue, comic simile, and deadpan narrator
> aside.”

Fake test, on pairs we lost:

> B2 (all four): the *real* Moist passage was called “flat”, “competent but
> flat… lacks wryness”, “no Pratchett rhythm”. “All in all, it was a win is
> flat where Pratchett would find a sharper angle.”

> B1 (glm, qwen): “the layered simile and deadpan mechanics of genuine
> Pratchett” / “escalating simile (mill wheel → fly in a small church)…
> unmistakably Pratchett’s deadpan specificity.”

> B3 (qwen): “A’s aphoristic setup-and-undercut (‘the only one’) is a signature
> Pratchett move.”

> B4 (qwen): “A’s ‘left his ticket behind’ metaphor is the kind of oblique,
> unsettling image Pratchett reaches for.”

Fake test, on pairs nobody won:

> B5: “A’s multi-layered grammatical joke (comma + exclamation mark + dark
> reinterpretation) is structurally complex in a way imitations rarely achieve.”

> B6: “A’s ritualistic precision (counted to three, counted to nineteen,
> twenty-nine minutes past eleven) is comic through exactness in a way that’s
> very hard to fake.”

Overwritten — one flag in thirty judgements:

> kimi on B1 (the story’s opening): “Imitation stacks three similes in two
> sentences; genuine passage is pure Moist interiority.”

## Reading

**Direction is settled.** All five judges, all six pairs, clear margins: the
rewrite’s voice is unambiguously closer to Pratchett than the Poe source. This
now closes the loop the instrument opened — `p_same` 0.039 → 0.218, GI
`target_disfavoured` → `target_favoured`, away-distance up 0.219 → 0.348, and
a blind reader agrees with the metric.

**The spot-the-fake result is asymmetric, and the pattern is the finding.**
“Fool” rates run 1/6 to 4/6 depending on the judge — highest for qwen, the
strongest reasoning model present. The pairs we win are precisely the ones
where the rewrite is at its most *device-dense* and the genuine anchor is a
plain interior-monologue paragraph: the judges are deciding “is there a joke in
every sentence”, i.e. matching the Pratchett stereotype, not the identity.

**The unwinnable pairs are genuine set-pieces, and that is structural.** No
judge was fooled by B5 or B6 — the mug/comma joke and the Tiddles ritual,
passages whose humour lives in exactness and accumulated specificity that
paraphrase cannot carry. This is the same wall `JUDGE-RESULTS.md` (matrix) hit
with the overwritten complaint: it is not a rate, and no density band will ever
capture it.

**Overwriting nearly disappeared.** One flag in thirty judgements (kimi on the
story’s densest paragraph — the same stacked-similes spot flagged in the
matrix round), against 3/4 cells + the showcase flagged last time. The discourse
dimensions and the sparse-revision habit moved the flag from the norm to a
single known spot.

## Caveats

- Six short pairs sampled in story order from the first ~1,500 words of the
  final draft (the opening is the densest part). Direction is robust; exact
  fool rates are passage-specific.
- Anchors come from two books of one era of Pratchett (Making Money, Unseen
  Academicals). A wider anchor spread would pin the set-piece finding tighter.
- The judge's model, temperature and prompt are fixed per this run and pinned
  in `eval/build/judge4/`; a win rate compared across a different judge is not
  a comparison, per `judge/README.md`.