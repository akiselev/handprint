# `eval/` — the non-circular evaluation harness

**The judge must never be handprint's own features.** A style scorer evaluated
by its own scorer measures nothing: STRAP showed that single automatic metrics
get gamed, and a rewrite loop driven by handprint that is then scored by
handprint will report success no matter what it did to the text.

So everything here is external:

| Question | Judge | Not |
|---|---|---|
| Did the rewrite move *away* from the source style? | LUAR authorship embeddings; `AnnaWegmann/Style-Embedding` or `StyleDistance/styledistance` for style-only | handprint distances |
| Did it still *say the same thing*? | SBERT cosine between draft and rewrite, used as a **gate**, never as a target | handprint's drift guard, which is a crude proxy |
| Did it fool an external detector? | Binoculars, reported as TPR at fixed low FPR on RAID splits including the adversarial ones | accuracy, which hides the operating point |
| Does forensic linkage work? | Held-out Hacker News account pairs; same-author percentile distribution vs unrelated; GI verdict accuracy *with* abstention rate | any metric that cannot abstain |

## The headline experiment

This is the crate's thesis, and it either sells it or falsifies it early. Run a
manual pilot as soon as a reference and a real agent are available.

An agent loops against `handprint critique --json` on:

**(a) its own fresh drafts.** Does the external Binoculars score drop while
SBERT similarity to the original stays ≥ 0.9? If the detector score drops only
when meaning also drops, the loop is deleting content rather than changing style.

**(b) toward the author's own corpus.** Does the rewrite land inside the
same-author percentile band *without* `guards.canary_ok` going false? A pass
with a tripped canary means the agent optimized the seven reported features and
nothing else — which is the failure this whole design is built to detect.

## Layout

```
eval/
  Makefile             `make licenses` needs nothing; the rest need corpora
  pyproject.toml       dependencies, pinned
  licenses/
    check.py           loader-only tables never bundled, corpora never tracked
  corpora/
    README.md          how to assemble the corpora locally (none are committed)
    chapterize.py      pandoc markdown -> per-chapter documents
  battery/
    matrix.py          the WIT.md coverage matrix, as data
    fixtures.py        seeded over-fired pastiches (original text, committed)
    hp.py              subprocess wrapper around the shipped CLI
    run.py             suites (a) separation, (b) interpretability, (c) anti-caricature
    results.py         renders RESULTS.md — metrics only, never corpus text
  registers/
    run.py             tech-blog vs marketing vs agent prose
  judge/
    README.md          the wit-critic protocol: pairwise, anchored, never absolute
    rubric.md          the GTVH-structured per-paragraph rubric
    run.py             harness; the model is pinned per run in the manifest
  agents/
    move-index/        the comic-move labelling pass and its prompt
  harness/
    embeddings.py      LUAR / Style-Embedding / SBERT wrappers
    detectors.py       Binoculars wrapper
    loop.py            drives `handprint critique --json` against an agent
    report.py          away/towards scores, TPR at fixed FPR, abstention rates
  experiments/
    headline.py        the phase-10 headline experiment
    voice-transfer/    the W10 capstone: PD corpora, three arms, five criteria
```

## Status

The batteries and the license check are implemented and runnable; the corpora
they need are not in the repository and cannot be. The model wrappers in
`harness/` are still interfaces only — LUAR, Style-Embedding, SBERT and
Binoculars are named so the experiment is reproducible when they are available.

**No number from this directory appears anywhere in the documentation, and none
should until `RESULTS.md` says a run happened.** A results table with plausible
placeholder numbers is worse than an empty one, because the numbers get quoted.
