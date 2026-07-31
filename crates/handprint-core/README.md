# handprint-core

Interpretable stylometry: profile, compare, and explain writing style.

`handprint` is an objective function and an explainer, not a verdict machine and
not a rewriter. It answers three questions:

1. **Forensic** — how far apart are these two texts stylistically, how unusual is
   that distance against a calibrated background, and *which features* drive it?
2. **De-AI** — how far is this draft from an AI-style reference, and what
   concretely should change?
3. **Corpus-mimic** — does this draft fall inside a particular author's own
   stylistic variation yet?

Modes 2 and 3 are designed to be driven by an LLM agent in a loop:
`handprint critique --json` emits per-feature findings with byte spans, observed
rates, target bands and a direction of change. handprint scores; the agent
rewrites.

## Design invariants

1. **Every score decomposes.** Any distance is a sum of per-dimension
   contributions (plus, for cosine, one documented global normalizer). No
   non-linear kernels in core.
2. **Fitted state is data-only and `serde`-serializable.** Fit once, ship the
   model, profile anywhere. No closures or trait objects inside fitted structs.
3. **Corpus-relative logic lives only in `fit`.** `transform`/`profile` is pure
   and deterministic given a fitted reference.
4. **Tokenization is explicit policy**, serialized with the model.
5. **Scaling is a per-metric property.** Delta-family metrics consume z-scores;
   MinMax/Ruzicka consumes non-negative relative frequencies. The `Metric` trait
   declares which, and the reference supplies it.
6. **Lexicons and reference profiles are versioned data, never code.** AI-ism
   vocabularies decay per model and per year; every shipped artifact carries a
   date and a provenance manifest.
7. **Core stays light.** Everything heavy is feature-gated.

## Feature families

Each family is a group of dimensions with a shared confidence floor, because a
document long enough to measure punctuation is not long enough to measure
frequent-word distributions and the report says so per family rather than once.

| family | what it measures | usable from |
|---|---|---|
| `punct` | punctuation, typography, casing, contractions | 40 tokens |
| `lexicon` | versioned word- and phrase-list hits, with spans and fixes | 40 |
| `sentence` | sentence dispersion, openers, markdown apparatus, asides, punch rhythm | 60 |
| `register` | Biber Tier-1 rates, readability grades, formality variance and clash, norm densities | 150 |
| `syntax` | fragments, parataxis, adverb scarcity, catalogs, intensifier chains, escalation rhythm | 150 |
| `richness` | length-corrected lexical richness (MTLD, MATTR, Yule's K) | 150 |
| `contrast` | contrastively discovered signature vocabulary | 150 |
| `surprisal` | corpus language-model surprisal — the Goodhart canary | 100 |
| `rhythm` | where surprisal spikes land inside a sentence | 200 |
| `verse` | metre, stress profile, line geometry, archaism | 200 |
| `char_ngram` | typed character n-grams | 300 |
| `device` | comparison frames, litotes, absurd precision, novel bigrams, marketing structure | 300 |
| `mfw` | most-frequent-word relative frequencies | 500 |

Several families are there for their **low** bands. An author characterized by
what they *don't* do — no adverbs, no subordination, no intensifiers — is only
describable by a two-sided band, because "write minimally" gives an agent no
target rate.

Two-sided bands are also the anti-caricature machinery. Imitation over-fires
salient markers, and a critic that only had a ceiling would call that success.

### Data packs

Lexicons and weighted norms are versioned data with a provenance envelope, in
three formats: `LexiconPack` (terms and phrase patterns), `NormPack` (per-word
scores), `CountPack` (background frequency tables — the only way a background
distribution reaches a feature, because `fit` sees the reference corpus alone).

`handprint pack list` shows what a binary carries; `pack export` writes one to
JSON so a project can pin its own. Resources whose terms forbid redistribution
are **loader-only**: the feature and the loader ship, the table does not, and
the dimension reports *missing* rather than zero without it.

## Not a verdict machine

Outputs are evidence summaries. `p_value_vs_unrelated` is the probability of
seeing a distance this small *between unrelated authors* — it is not the
probability that two texts share an author, and reading it that way is the
prosecutor's fallacy. The dominant failure mode in authorship verification is
topic confound; short texts are unreliable below documented floors. See the
`limitations` section of the crate docs.

## Example

```rust
use handprint_core::{Corpus, Document, Pipeline, feature::*};

let corpus = Corpus::new()
    .with("alice", (0..8).map(|i| Document::new(format!(
        "Alice writes plainly, and she keeps her sentences short. \
         This is document {i}, and it reads much like the rest of them. \
         No dashes anywhere. Just commas, and full stops."
    ))))
    .with("bob", (0..8).map(|i| Document::new(format!(
        "Bob\u{2014}who writes very differently\u{2014}piles on the dashes!!! \
         Text number {i} shows it, with \u{201C}curly quotes\u{201D} \
         everywhere\u{2014}always\u{2014}and sentences that ramble well past the \
         point where a reasonable person would have stopped writing."
    ))));

let reference = Pipeline::builder()
    .feature(PunctTypography::default())
    .feature(SentenceStats::default())
    .name("demo")
    .fit(&corpus)?;

let a = reference.profile(&Document::new("Alice writes plainly, mostly. Short sentences."));
let b = reference.profile(&Document::new("Bob\u{2014}writes\u{2014}differently!!! Always has."));
let comparison = reference.compare(&a, &b)?;

println!("distance {:.4}", comparison.distance);
for c in comparison.contributions().iter().take(5) {
    println!("{:>28}  {:+.4}  ({:.2} vs {:.2})", c.name, c.value, c.observed_a, c.observed_b);
}
# Ok::<(), handprint_core::Error>(())
```

## The agent-critic loop

```rust
use handprint_core::{Corpus, Critic, Document, Pipeline, feature::*};

# let corpus = Corpus::new().with("me", (0..8).map(|i| Document::new(format!(
#     "I looked at it again and it still comes out the same, which is annoying. \
#      Not sure whats going on in run {i}. I'll poke at it tomorrow."))));
let reference = Pipeline::builder()
    .feature(PunctTypography::default())
    .feature(SentenceStats::default())
    .feature(LexiconFeature::default())
    .name("my-corpus")
    .fit(&corpus)?;

// Representative documents, not a centroid - see `Critic::corpus_mimic`.
let targets = corpus.documents().take(6).map(|d| reference.profile(d)).collect();
let mut critic = Critic::corpus_mimic(&reference, targets);

let report = critic.review("This comprehensive analysis delves into the intricate tapestry \
                            of considerations. Moreover, it is worth noting that experts argue \
                            the framework plays a crucial role.")?;
println!("{}", report.to_json());
# Ok::<(), handprint_core::Error>(())
```

Each `Finding` carries a namespaced id, the observed value, the reference's
target band, a direction, and byte spans — enough for an agent to patch the text
and re-submit. `guards.canary_ok` and `guards.drift_ok` say whether the loop is
still honest.
