# The comic-move index

A one-time labelling pass over an ingested corpus, tagging each chunk with the
comic device(s) it uses and the scene function it performs. The index lives
beside the corpus and is re-ingested into `codeindex` as a parallel corpus, so
retrieval becomes **by mechanism, not by topic**:

> give me three passages where this author introduces a character

rather than

> passages about spaceships.

Nothing published does move-indexed pastiche retrieval. The mimicry literature
says demonstration *selection* dominates demonstration count, and that
structural similarity beats topical similarity — which is exactly what a
topic-indexed corpus cannot give you.

## Why this is not in the crate

No LLM in core is a standing invariant. Labelling is a judgement call about
what a passage is *doing*, and there is no count that answers it. So the
labelling pass is agent-side, the labels are data, and handprint never sees the
model.

## Distribution

Sidecars quote passages, so they follow the corpus's own rule:

* **Committed** only where the underlying text is public domain **worldwide** —
  Twain (d. 1910), Poe (d. 1849), Austen (d. 1817).
* **Local only** for everything else, including Wodehouse and Hemingway, whose
  US public-domain status does not extend to life+70 jurisdictions.

`eval/licenses/check.py` fails the build if a sidecar for a local-only author
is ever tracked.

## The parroting guard

Retrieved passages get parroted. That is not a risk to be worried about, it is
the default behaviour, and two things push back:

1. The render prompt asks for the *mechanism*, names it, and supplies the
   exemplar as an illustration of the mechanism rather than as a model
   sentence.
2. handprint's character-n-gram overlap — the same machinery the drift guard
   uses — catches verbatim leakage. A rewrite that lifts a clause scores
   suspiciously high on it.

The wit-critic rubric also carries a `parroting` flag, checked first.

## Files

```
move-index/
  README.md      this
  prompt.md      the labelling prompt, versioned
  label.py       the pass; model injected, like the judge harness
  schema.json    the label schema, so a sidecar is machine-checkable
```

## Reproducibility gate

Before an index is used, run the labelling twice over a 50-chunk sample and
check agreement:

* **Cohen's κ ≥ 0.7** on the device labels, **and** ≥ 80% raw agreement.
* Disagreements are reviewed by hand and the prompt is tightened **once**.
  Tightening repeatedly until the number clears is fitting the prompt to the
  sample, and the sample is 50 chunks.

`label.py --agreement` computes both.
