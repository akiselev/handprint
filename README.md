# handprint

Interpretable stylometry for Rust: profile, compare, and explain writing style.

handprint measures how text is written — punctuation habits, sentence rhythm,
word choice, character-level fingerprints — and tells you **which features drive
a result and where in the text they came from**. It is an objective function and
an explainer. It is not a verdict machine, and it does not rewrite anything.

Three entry points sit on one core:

1. **Forensic** — link or verify authorship. Distances, length-stratified
   calibrated p-values, per-feature contributions, General Imposters
   verification with an abstention zone.
2. **De-AI critic** — an agent scores its own draft against an AI-style
   reference, gets per-feature feedback with byte spans and target bands,
   rewrites, re-scores.
3. **Corpus-mimic critic** — the same loop, but the target is your own writing:
   rewrite until the draft falls inside that author's own stylistic variation.

handprint is the objective function; the agent is the optimizer. No LLM calls in
core.

```
crates/handprint-core    the library: text, features, references, metrics, critique
crates/handprint-cli     the `handprint` binary
crates/handprint-data    corpus extractors: agent transcripts, Hacker News
eval/                    non-circular evaluation harness (Python)
```

## Install

```sh
cargo install --path crates/handprint-cli          # the `handprint` binary
cargo install --path crates/handprint-cli --features net   # + Hacker News fetching
```

## Worked example 1 — forensic linkage

Fit a reference on a corpus, calibrate it against a background population, then
ask how unusual a distance is.

```sh
# A corpus laid out as one directory per author.
handprint fit corpus/ -o refs/hn.json \
    --name hn-2015-2022 --version 2026.07 \
    --features punct,sentence,function-words,ngrams

# Calibration turns distances into evidence. Without it, `compare` prints a
# number and nothing else.
handprint calibrate refs/hn.json --background background/ --metric cosine

handprint compare -r refs/hn.json accounts/main.txt accounts/throwaway.txt
```

```
cosine_delta distance 0.284913  (5120 vs 412 tokens)
  p_value_vs_unrelated 0.0031   p_same_author 0.4120   likelihood ratio 18.40
  p_value_vs_unrelated is P(distance <= d | different authors). It is NOT the
  probability the authors differ.

by family:
  punct        +0.118204
  sentence     +0.061553
  mfw          +0.058017

top contributions:
dimension                                     a          b      share
punct:em_dash_rate                       0.4102     6.8100  +0.041127
sent:len_stddev                          9.8800     3.1200  +0.028410
```

`rank` scores a query against many candidates; `verify` runs General Imposters
with a grey zone that abstains rather than guessing.

## Worked example 2 — the de-AI loop

Fit one reference on the **union** of an AI corpus and a human corpus, then ask
handprint which direction each dimension pulls.

```sh
handprint fit union/ -o refs/de-ai.json --features punct,sentence,lexicon,mfw
handprint critique -r refs/de-ai.json --mode contrast --away union/ai draft.md
```

Each finding names a dimension, what the draft does, what the human side does,
and which way to move. Byte spans mean an agent can patch precisely.

## Worked example 3 — the corpus-mimic loop

```sh
handprint fit mine/ -o refs/me.json --features punct,sentence,lexicon,ngrams
handprint calibrate refs/me.json --background background/ --metric burrows

while ! handprint critique -r refs/me.json draft.md > report.json; do
    # your agent reads report.json and rewrites draft.md
    rewrite draft.md report.json
done
```

Exit codes follow Vale: **0** pass, **1** findings, **2** error. A report looks
like this:

```json
{
  "handprint": "0.1.0",
  "contract": "1",
  "reference": { "name": "my-corpus", "version": "2026.07", "kind": "corpus" },
  "mode": "corpus-mimic",
  "iteration": 3,
  "doc": {
    "tokens": 412,
    "length_tier": "unreliable",
    "confidence": { "punct": "ok", "sentence": "low", "lexicon": "ok", "char_ngram": "none" },
    "imputed_dims": 4
  },
  "verdict": {
    "pass": false, "gate": "p_same_author >= 0.100",
    "p_same_author": 0.031, "p_value_vs_unrelated": 0.412,
    "distance": 1.8842, "capped_distance": 1.7710, "metric": "burrows_delta"
  },
  "findings": [
    { "id": "punct.em_dash_rate", "family": "punct", "severity": "high",
      "observed": 8.2, "unit": "per_1k_tokens", "target_band": [0.5, 2.1],
      "z": 3.4, "direction": "reduce", "spans": [[120, 123], [338, 341]],
      "message": "8.20 per 1k tokens vs reference band 0.50-2.10 - reduce ~74%" },
    { "id": "lex.ai-slop.word.delve", "family": "lexicon", "severity": "high",
      "observed": 4.9, "unit": "per_1k_tokens", "target_band": [0.0, 0.0],
      "z": 9.7, "direction": "reduce", "spans": [[10, 15]],
      "message": "4.90 per 1k tokens vs reference band 0.00-0.00 - reduce ~100%",
      "fix": { "kind": "consider_replace", "alternatives": ["dig into", "examine"] } }
  ],
  "findings_total": 23,
  "guards": { "canary_ok": true, "drift_ok": true, "content_overlap": 0.87 }
}
```

Note `findings_total`: **everything is scored, only the worst offenders are
reported.** That is deliberate — see below.

## Resisting the loop

An optimizer pointed at a scorer will optimize the scorer. Five mechanisms push
back:

| Guard | What it does |
|---|---|
| **Top-k reporting** | Every dimension is scored; ~7 are shown. The agent fixes what it sees; the gate checks what it cannot. |
| **Canary family** | One family (default: corpus-LM surprisal) is scored and *never reported*. Reported features improving while it worsens sets `canary_ok = false`. |
| **Holistic gate** | Passing is `p_same_author >= threshold` against the calibrated same-author distribution — never "all findings cleared". |
| **Per-dimension cap** | No single dimension may hold more than 25% of the distance magnitude when the gate is evaluated. |
| **Drift guard** | Content-word and character-n-gram overlap against the original draft, so a rewrite that deletes or re-topics the text is caught. |

## Design invariants

1. **Every score decomposes** into per-dimension contributions plus at most one
   documented global normalizer. This rules out non-linear kernels in core.
2. **Fitted state is data-only and `serde`-serializable.** Fit once, ship the
   model, profile anywhere. No closures or trait objects inside fitted structs.
3. **Corpus-relative logic lives only in `fit`.** `transform` is pure.
4. **Tokenization is explicit policy**, serialized with the model.
5. **Scaling is a per-metric property.** Delta metrics consume z-scores;
   MinMax/Ruzicka consumes non-negative relative frequencies. MinMax on z-scores
   is ill-defined, so the `Metric` declares what it needs.
6. **Lexicons and reference profiles are versioned data, never code.** AI-ism
   vocabularies decay per model and per year; every artifact carries a date and
   a source manifest.
7. **Core stays light.** `serde`, `unicode-segmentation`, `unicode-normalization`,
   `rand`, `thiserror`. Everything else is feature-gated.

## Limitations

Read this section before using any number from this crate in an argument.

**This is not a verdict machine.** `p_value_vs_unrelated` is
`P(distance ≤ d | different authors)`. It is *not* the probability that two
texts share an author, and reading it that way is the prosecutor's fallacy. A
p-value of 0.01 means a distance this small occurs in 1% of unrelated pairs;
what that is worth depends entirely on how many unrelated candidates exist. The
reported likelihood ratio is closer to what most people want, and is still only
a *score-based* likelihood ratio.

**Topic confound is the dominant failure mode.** PAN 2020 found topic similarity
correlated with verifier errors across every method examined; Halvani et al.
concluded that none of the evaluated authorship-verification methods is robust
against topical influence. A low distance under topic-matched conditions is weak
evidence. The robust settings are function-word-restricted vocabularies —
`handprint fit --features function-words` and `handprint contrast
--function-words` — and character n-grams, which transfer better across topic
and genre than frequent words do.

The effect is easy to see. Contrasting two coding-agent transcript corpora
unrestricted surfaces `decompile`, `ghidra`, `housing`, `city`: the projects,
not the writers. The same contrast with `--function-words` surfaces `let`,
`me`, `now`, `my` on one side and `of`, `your`, `own`, `itself` on the other —
one model narrating its own actions in the first person, the other writing
nominally in the second. Only the second run is about style.

**Short texts are unreliable, and the floors are graded.** Under 2,000 words,
whole-profile attribution is below the length the literature supports (Eder
2015/2017); verification accuracy falls to chance between 250 and 500 characters
with unadjusted thresholds (Halvani et al. 2019). handprint reports a
`length_tier` and *per-family* confidence, because punctuation stays usable far
below the floor that frequent-word profiles need. Note also that with mixed
feature families the length effect can invert at the short end — families that
are undefined below their floors get imputed to the corpus mean, which makes two
*unrelated* short texts look alike. Either way: **a small distance on a short
text is weak evidence.**

**The critic mode is an attack on the forensic mode.** They are the same
machinery pointed in opposite directions. Anything that helps an agent match a
target style also helps someone impersonate an author, and the guards above
raise the cost of gaming the score without eliminating it. This is stated
plainly rather than hedged, because it is the honest description of what this
crate is.

**Profiles are dated artifacts.** "delve" rose roughly fifteenfold in PubMed
abstracts between 2022 and 2024 and then *declined* after public mockery;
em-dash overuse is a GPT-4-era tell that barely existed in GPT-3.5; the whole
marker class is an RLHF artifact that differs per model and per version. The
built-in `ai-slop` lexicon is dated `2026.07` and is a seed, not a fixture. Human
reference corpora should be cut around November 2022 — AI-isms are leaking into
genuinely human usage, so a "human" reference fitted on 2024 text quietly
destroys the thing it exists to measure.

**Non-native-English writing risks false positives.** Detectors of this general
family have been measured at over 50% false-positive rates on real TOEFL essays.
handprint's features are not a detector, but they are correlated with the same
surface properties, and nothing here corrects for it.

**Sentence splitting is rule-based.** Abbreviations outside the built-in list
split early, mid-sentence ellipses split, and unpunctuated chat-register
sentences merge. The errors inflate sentence-length *variance* rather than
biasing the mean, and the same splitter runs over reference and query — so
relative comparisons stay meaningful while absolute burstiness numbers do not.

## Performance

Measured on a 140 KB document (`cargo bench -p handprint-core`):

| Operation | Throughput |
|---|---|
| Sparse L1 distance (merge walk) | ~850 Melem/s, **26× a hashmap implementation** |
| Analysis only (tokenize, segment, artifact scan) | ~12 MiB/s |
| `+ punct` | ~7.5 MiB/s |
| `+ sentence` / `+ mfw` | ~10 MiB/s |
| `+ typed char n-grams (3..=4)` | ~3.4 MiB/s |
| All four families together | ~2.8 MiB/s |

The original plan targeted ≥50 MiB/s single-threaded for `transform`. **That
target is not met.** Typed character n-grams dominate: at orders 3 and 4 they
emit roughly two classified dimensions per character, and the remaining cost is
in the per-position work rather than in anything obviously wasteful. Closing the
gap would need byte-level extraction with an interned n-gram table rather than
string keys. Corpus fitting parallelizes cleanly and is the obvious next lever
(`--features parallel` reserves the flag; the implementation is not written).

## Status

Phases 1–9 of `IDEA.md` are implemented and tested: text layer, feature vectors,
eight feature families, references with dual length-stratified calibration, four
decomposable metrics, explanation and the critique contract, corrected Fightin'
Words, General Imposters, the CLI, and the corpus extractors. `eval/` holds the
non-circular evaluation harness scaffold; the headline experiment in phase 10 has
not been run.

## License

MIT OR Apache-2.0.
