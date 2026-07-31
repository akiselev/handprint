# `handprint` — Implementation Plan

Interpretable stylometry for Rust: profile, compare, and explain writing style.
Three entry points over one core:

1. **Forensic mode** — link/verify authorship (e.g. HN throwaway experiment):
   distances, calibrated percentiles, per-feature contributions. Explicitly
   **not a verdict machine** — outputs are evidence summaries, never "same
   author: yes/no".
2. **Agent-critic, de-AI mode** — an LLM agent scores its own draft against an
   AI-style reference, gets actionable per-feature feedback (rates, spans,
   direction of change), rewrites, re-scores until it passes.
3. **Agent-critic, corpus-mimic mode** — same loop, but the target is a
   user-supplied corpus (e.g. all my HN comments): rewrite until the text falls
   within that author's own stylistic variation.

handprint is the **objective function + explainer**; the agent is the
optimizer. No rewriting inside the crate, no LLM calls in core.

Evidence base: see `RESEARCH.md` (2026-07-31) — all algorithmic corrections and
design decisions below are sourced there.

---

## 0. Ground rules & architecture

**Design invariants**

1. Every score decomposes: any distance must be explainable as a sum of
   per-dimension contributions (plus at most a global normalizer). Rules out
   non-linear kernels in core.
2. Fitted state is data-only and serde-serializable. Fit once, ship the model,
   profile anywhere. No closures/trait objects inside fitted structs.
3. Corpus-relative logic (vocab selection, scaling, calibration) lives only in
   `fit`. `transform`/`profile` is pure and deterministic given a fitted
   reference.
4. Tokenization is explicit policy, serialized with the model.
5. **Scaling is a per-metric property, not a pipeline-global.** Delta-family
   metrics consume z-scores; MinMax/Ruzicka consumes non-negative relative
   frequencies. The type system or the `Metric` trait must enforce this
   (research finding H3 — MinMax on z-scores is ill-defined).
6. **Lexicons and reference profiles are versioned data, never code.** AI-ism
   vocabularies decay (per model, per year — "delve" peaked Q1 2024 and then
   declined); every shipped artifact carries a date, source manifest, and
   model/corpus provenance.
7. Core stays light: `unicode-segmentation`, `serde`, `hashbrown`/`ahash`,
   `string-interner` (or hand-rolled), `thiserror`. Everything heavy is
   feature-gated.

**Workspace layout**

```
handprint/
├── crates/
│   ├── handprint-core/    # phases 1–7
│   ├── handprint-cli/     # phase 8; the agent-facing surface
│   └── handprint-data/    # phase 9; corpus extractors (HN, public sets, local chat logs)
├── eval/                  # phase 10; Python eval harness (non-circular judges)
└── fixtures/              # golden tests, salted docs, generated AI/human samples
```

Module map for `handprint-core`:

```
text/       Document, Span, Tokenizer, normalization, artifact/homoglyph flags
feature/    Feature + FittedFeature traits; punct, sentence, mfw, char_ngram,
            richness, lexicon, surprisal
vector/     Symbol interner, FeatureVector (sparse), dense export
reference/  Corpus, Reference (fitted pipeline + scaling + calibration), serde
compare/    metrics (CosineDelta default), Profile, Comparison, rank helpers
verify/     General Imposters (bootstrap verification, abstention zone)
explain/    per-dim decomposition, span attribution
critique/   CritiqueReport: JSON contract for the agent loop, threshold profiles
contrast/   Fightin' Words (corrected math), ContrastFeature
```

Crate name: `handprint` — confirmed available on crates.io as of 2026-07-31
(`stylo` is taken by Servo's CSS engine). Nearest competitor is a 21-download
Delta-only CLI from July 2026; ship sooner rather than later.

---

## Phase 1 — Text layer

**Deliverables**

- `Document`: owns text; lazily computed token stream; stable byte-offset
  `Span`s.
- `Tokenizer` (enum-of-policies, serializable):
  - `Words { case: CaseFold, nfc: bool, punct_as_tokens: bool, apostrophes: ApostrophePolicy }`
  - `Chars { window over normalized text }`
  - Default: NFC + lowercase + UAX-29 word bounds, punctuation retained as
    tokens.
- Normalization policy, documented: NFC for *word* features only. Typography
  (curly vs straight quotes, em/en dashes, ellipsis char vs `...`, NBSP) is a
  first-class feature family — never normalized away before the typography
  features see it.
- **Artifact scan** (new): flag homoglyphs (mixed-script confusables),
  zero-width characters, and chatbot residue (`oaicite`, `turn0search`,
  `[cite: N]`, `contentReference`). These are tokenizer attacks / generation
  artifacts, not style; they surface as findings, and scoring runs on the
  confusable-normalized text so homoglyph stuffing can't dodge the features.

**Acceptance**

- Round-trip property: every token's `Span` slices the original text to the
  token's source form.
- Snapshot tests over a nasty-Unicode fixture (contractions, curly quotes,
  emoji, CJK mix, homoglyph-salted sample).

## Phase 2 — Feature vectors & interning

**Deliverables**

- `Symbol` = `u32` via per-`Reference` interner. Interner serializes with the
  model.
- `FeatureVector`: sparse `Vec<(Symbol, f64)>` sorted by symbol; builder uses a
  hashmap, freezes to sorted vec.
- Optional span tracking: `Symbol -> SmallVec<Span>` when built with
  `.track_spans(true)`. Off by default.
- `to_dense(&self, dims: &[Symbol]) -> Vec<f64>` + optional `ndarray` feature
  flag.

**Acceptance**

- Merge-based L1/cosine over sorted sparse vecs microbenchmarked vs hashmap
  impl (criterion).

## Phase 3 — Feature set

Traits:

```rust
trait Feature { type Fitted: FittedFeature; fn fit(&self, c: &Corpus) -> Result<Self::Fitted>; }
trait FittedFeature { fn transform(&self, d: &Document, out: &mut VecBuilder); fn dims(&self) -> &[Symbol]; }
```

Implement, in order:

1. **`PunctTypography`** (trivial fit; the small-sample workhorse — Grieve 2007
   found frequent-words + punctuation the strongest combo): rates per 1k tokens
   for `, ; : — – ... " ' ( ) !`, em-dash vs spaced-hyphen style, straight vs
   curly quotes, ellipsis style, ALLCAPS rate, digit style, contraction rate,
   netspeak casing ("imo/IMO"), letter/digit unigram frequencies, word-length
   distribution, typo/misspelling rate (a *human* marker — LLM text is
   abnormally clean).
2. **`SentenceStats`** (trivial fit): mean/stddev/**variance and Fano factor**
   of sentence length in tokens ("burstiness" — low variance is a documented
   AI marker), comma density per sentence, paragraph length stats, sentence-
   **opener** rates ("Additionally,", "Moreover,", "In conclusion"), discourse-
   marker rates, markdown-structure stats (bullet/bold/header density —
   documented post-training tell). Cheap rule-based sentence splitting;
   document its limits.
3. **`MostFrequentWords`**: `top(k)`, `culling(f)` (fraction of corpus docs a
   word must appear in), curated function-word list mode, stopword-only mode
   (the topic-robust setting — document this). Fit selects vocab from corpus.
4. **`CharNgrams::new(3..=4).top(k).typed(true)`**: over normalized text incl.
   spaces & punctuation. **Positional typing** (Sapkota 2015): tag each n-gram
   prefix / suffix / space-prefix / space-suffix / mid-word / beg-punct /
   mid-punct / end-punct. Affix+punct-only matches or beats untyped-everything
   at ~⅔ the features, wins biggest cross-domain, and makes explanations
   meaningful (`sti` as word-prefix is authorial; mid-word it's noise). Dim
   names carry the type: `"c3:prefix:sti"`.
5. **`Richness`** (length-corrected only): **MTLD** (primary — the only index
   found length-invariant in McCarthy & Jarvis 2010), MATTR (window 100;
   **undefined for docs < window — emit missing value + warning, never silent
   whole-doc TTR**), Yule's K, Simpson's D (theoretically length-invariant,
   empirically drift — carry length warnings). *No raw TTR/hapax.*
6. **`Lexicon`** (trivial fit): match versioned AI-ism/slop word- and
   phrase-lists (data packages, phase 8) with spans. Seed sources: berenslab
   excess-vocab 900 words, Liang top-100 adjectives/adverbs, Wikipedia
   signs-of-AI-writing phrase patterns (significance inflation, copula
   avoidance, negative parallelism "not just X, but Y", rule-of-three), vague
   attribution, formulaic transitions.
7. **`SurprisalLM`**: char-trigram (and optionally word-bigram) LM fitted on
   the reference corpus at `fit` time — counts only, serializable. Features:
   mean cross-entropy under the corpus LM, **variance of per-token surprisal**
   (uniformly-low surprisal is the GLTR/Binoculars insight, no LLM needed),
   surprisal autocorrelation. Decomposes per-token → span attribution. Hard to
   fake by word-list stuffing (Goodhart canary candidate).
8. **`ContrastVocab`** — phase 6 output plugged in as a feature.

All fitted features carry human-readable dim names (`"mfw:the"`,
`"c3:midpunct:, a"`, `"punct:em_dash_rate"`, `"sent:len_stddev"`) through the
interner.

**Acceptance**

- Golden tests: MFW z-scores vs a hand-computed fixture; cross-check one corpus
  against R `stylo` for Delta *ranking* agreement (order match, not exact
  values — stylo's shipped Eder code differs from its own docs by a constant
  and an off-by-one weight, so numeric match is not achievable or needed).
- Typed char n-grams: fixture verifying type assignment at word boundaries.
- MATTR fallback behavior on a 60-token doc: missing value + warning, not TTR.

## Phase 4 — Reference, Profile, comparison, calibration

**Deliverables**

- `Corpus`: `Vec<(AuthorId, Vec<Document>)>` + iterators; helper to aggregate
  an author's docs into one pseudo-doc (per-comment HN profiles are too short;
  aggregation is the obvious path).
- `Pipeline::builder().feature(..)….fit(&corpus) -> Reference`. Scaling
  (z-score per dim, mean/stddev over corpus docs or authors — knob, default
  per-text per the Delta literature) is applied per metric family per
  invariant #5.
- `Reference::profile(&Document) -> Profile` and `profile_agg(...)`.
- `Profile`: scaled vector + provenance. **Graded length warnings** tied to the
  literature (Eder 2015/2017): `<2,000` words = unreliable; `2,000–5,000` =
  usable only with clear signal; `≥5,000` = supported. Per-feature-family
  confidence (punct is usable far below MFW/richness floors) — this feeds the
  critique contract's `confidence` block.
- Metrics (trait `Metric` with required
  `decompose(a, b) -> impl Iterator<Item=(Symbol, f64)>`):
  - **`CosineDelta` — the default** (angle between z-score vectors; Evert et
    al.: consistently best, robust to MFW-count up to 10k).
  - `BurrowsDelta` (L1 on z-scores / n_dims — kept for literature
    comparability).
  - `Eder` (rank-weighted Delta).
  - `MinMax` (Ruzicka) — **consumes unscaled relative frequencies** (invariant
    #5). Decomposes via `contribution_i = |a_i − b_i| / Σ_j max(a_j, b_j)`.
- **Calibration — two distributions, length-stratified** (research H4):
  - *Different-author*: sampled unrelated author pairs (~10k max).
  - *Same-author*: split each background author's docs in half, sample
    within-author pairs. Enables (a) an honest score-based likelihood ratio,
    (b) the one-class "does this text belong to this corpus" question — which
    is also the critic loop's stopping criterion.
  - Both stored **binned by token count** (e.g. log-spaced bins), interpolated
    at compare time. Unstratified percentiles are systematically over-confident
    on short texts — the primary use case (verification accuracy collapses to
    chance at 250–500 chars in the literature).
- `Reference::compare(&a, &b) -> Comparison { distance, p_value_vs_unrelated,
  p_same_author, lr, metric }`. The percentile field is named as the p-value it
  is; docs explain the transposed-conditional trap. `rank(&query, &candidates)`
  stays as a thin convenience.

**Acceptance**

- Synthetic test: generated authors → same-author pairs land low, cross-author
  near uniform. **Plus a real-data test on topic-confounded corpora** (two
  authors writing about the same subject) with documented expected degradation
  — synthetic distributions have no topic confound, which is the thing that
  breaks calibration in practice.
- Length stratification: percentile of a 300-word query differs from a
  10k-word query on the same pair, in the correct direction.
- Serde round-trip: `Reference` → bytes → `Reference`, identical profiles.

## Phase 5 — Explanation & critique layer

This is the product core for the agent modes, not a demo.

**Deliverables**

- `Comparison::contributions() -> Vec<Contribution { symbol, name, signed_value }>`
  via `Metric::decompose`, sorted by |value|.
- Span attribution: with span tracking, each contribution carries
  `spans: &[Span]`; for rate-style dims, spans are the occurrences.
- **`CritiqueReport`** — the machine contract (shape modeled on Vale/textlint;
  stable, versioned):

```json
{
  "handprint": "0.x.y",
  "reference": { "name": "hn-akiselev-2015-2022", "version": "2026.07", "kind": "corpus" },
  "mode": "corpus-mimic",
  "doc": { "tokens": 412,
           "confidence": { "punct": "ok", "sentence": "ok", "mfw": "low", "richness": "none" } },
  "verdict": { "pass": false, "p_same_author": 0.03, "p_value_vs_unrelated": 0.41 },
  "findings": [
    { "id": "punct.em_dash_rate", "severity": "high",
      "observed": 8.2, "unit": "per_1k_tokens", "target_band": [0.5, 2.1], "z": 3.4,
      "direction": "reduce", "spans": [[120,123],[338,341]],
      "message": "em-dash rate 8.2/1k vs corpus band 0.5–2.1 — reduce ~75%" },
    { "id": "lexicon.ai-2026-07.delve", "severity": "medium", "spans": [[10,15]],
      "fix": { "kind": "consider_replace", "alternatives": ["dig into", "examine"] } }
  ],
  "guards": { "canary_ok": true, "drift_ok": true, "artifact_flags": [] }
}
```

  - `id` is namespaced (`family.rule` / `family.pack.term`); `target_band` is
    the reference's central interval for that dim; `direction` + spans make
    findings actionable; `confidence` tells the agent which feedback to trust
    at this length.
  - Rate-style families (punct, sentence, lexicon) produce naturally actionable
    findings; MFW/char-ngram distance mostly surfaces through ContrastVocab
    phrase hits instead.
- `classify_contrastive(doc, ref_a, ref_b) -> ContrastReport`: per-dim pull
  toward A vs B (de-AI mode = pull toward AI-reference vs human-reference).
- `render_ansi()` / `render_html()` behind a `render` feature flag —
  highlighted document; still the demo, no longer the point.

**Acceptance**

- Invariant test: sum of contributions == total distance (per metric, 1e-9;
  cosine: plus its global normalizer, documented identity).
- Manual fixture: paragraph salted with "delve", "tapestry", em-dash chains →
  report surfaces exactly those spans with correct target bands.
- JSON schema round-trips and is documented in the book; contract changes are
  semver-major.

## Phase 6 — Fightin' Words (contrastive vocabulary discovery)

Monroe, Colaresi & Quinn (2008), log-odds with informative Dirichlet prior.
**Corrected math** (research H1/H2 + Eq. 19):

For term w, corpora i and j, counts y, totals n, prior α_w with total mass α₀:

```
δ_w = ln[(y_iw + α_w) / (n_i + α₀ − y_iw − α_w)]
    − ln[(y_jw + α_w) / (n_j + α₀ − y_jw − α_w)]

σ²_w = 1/(y_iw + α_w) + 1/(n_i + α₀ − y_iw − α_w)          # full Eq. 19
     + 1/(y_jw + α_w) + 1/(n_j + α₀ − y_jw − α_w)

z_w = δ_w / sqrt(σ²_w)
```

- `Variance::Full` (Eq. 19, default) and `Variance::Reduced` (Eq. 20, the
  common two-term approximation) — the acceptance cross-check against the
  Python `fightin-words` package **must pin `Reduced`** (that's what it
  implements; Eq. 20 inflates |z| by 1/√(1−p), ~4% at function-word
  frequencies).
- **Prior**: `Prior::Background(&Corpus, alpha0)` with α_w = y_w·α₀/n_bg
  (background relative frequency × total mass); `Prior::Uniform(alpha)` where
  **α₀ = alpha·|V| — computed and logged explicitly**, since |V| swings by
  orders of magnitude with term-universe config.
- **α₀ guidance (corrected)**: α₀ is a pseudo-sample size in tokens, calibrated
  against the corpora being compared — α₀ ≈ min(n_i, n_j) is the neutral
  start; α₀ → 0 recovers raw log-odds; α₀ ≫ n_i shrinks everything to zero.
  It is **not** a function of background-corpus size (the old
  "background × 0.01" rule would swamp a 20k-word user corpus with a 5M-word
  background).
- **One term universe per fit** (H2): words, word bigrams/trigrams, char
  n-grams each get their own contrast with their own n_i and α₀ — mixing them
  breaks the multinomial bookkeeping (n_i ill-defined, denominators wrong).
  `ContrastSet` runs several universes and merges ranked lists for
  presentation; z-scores are never compared across universes.
- `ContrastModel::top(n, Side::A | Side::B) -> Vec<Term { name, z, delta, counts }>`
- `ContrastModel::into_feature(z_threshold) -> ContrastVocab` — discovered
  terms become dims for classification + explanation (closes discover →
  classify → highlight).
- **Privacy culling** (for lexicons fitted on private data, e.g. local chat
  logs): restrict retained terms to those above a document-frequency floor
  across distinct sessions/projects, so rare private identifiers (project
  names, paths, people) can never enter a shippable vocabulary. On by default
  when the corpus is flagged private.

**Acceptance**

- Unit test against a hand-computed 3-term toy example, both variance variants.
- Integration: human corpus vs LLM-generated fixture (~200 samples checked into
  fixtures) → top terms include the salted markers; z monotone in salting rate.
- Cross-check the Python `fightin-words` package on the same fixture with
  `Variance::Reduced` + matching prior (catches sign/log-base slips).

## Phase 7 — Verification (General Imposters)

The field-standard authorship verification method (Koppel & Winter 2014; won
PAN 2013/14; implemented in R stylo as `imposters()` for cross-checking).
~40 lines over the existing metric infrastructure:

- `verify(query, target_set, impostor_pool, opts) -> VerifyScore`: over k
  iterations, subsample features (default 50%) and impostors (default 50%);
  score = fraction of iterations where the query is closer to the target set
  than to every sampled impostor.
- Abstention grey zone: `p1/p2` thresholds (stylo-style, optimizable on a
  labeled split via `imposters.optimize` analogue) — scores in (p1, p2) return
  `Verdict::Abstain`, because scores near 0.5 are unreliable.
- Serves both modes: forensic verification (with impostors drawn from the
  background corpus), and an alternative critic-loop gate ("target-set wins at
  score ≥ p2").

**Acceptance**

- Agreement with R stylo `imposters()` on a shared fixture (score within noise
  across seeds; verdict match).
- Topic-confound stress test documented (GI is more robust than global
  percentile — show it, don't claim it).

## Phase 8 — Agent-critic CLI

`handprint-cli`, the agent-facing surface. Subcommands:

- `fit`, `profile`, `compare`, `rank`, `verify`, `contrast`, `explain --html` —
  thin veneers over core (forensic + inspection).
- **`critique --json`** — the loop entry point:
  - `--reference <pack-or-path>` (fitted Reference), `--mode corpus|contrast`
    (mimic a corpus vs. move away from AI-reference toward human-reference).
  - Emits `CritiqueReport` JSON (phase 5). Exit codes Vale-style: 0 pass /
    1 findings / 2 error. `--min-severity`, `--max-findings`.
  - Stdin or file input; byte-offset spans so the agent can patch precisely.
- **Loop protocol & Goodhart resistance** (all cheap, all in core `critique/`):
  - Score **all** features; **report only the top-k offenders** per iteration
    (default ~7). The agent fixes what it sees; the pass/fail gate checks
    everything.
  - **Canary family**: one feature family (default: `SurprisalLM`) is scored
    but never reported. Reported features improving while the canary worsens ⇒
    `guards.canary_ok = false` + a "feedback overfitting" finding.
  - **Pass gate is holistic**: `p_same_author ≥ threshold` (one-class,
    same-author distribution from phase 4) — never "all findings cleared".
    Per-dim contributions are capped so no single dimension can buy a pass.
  - **Drift guard**: crude content check between iterations (content-word
    Jaccard / char-ngram overlap vs. the original draft) to catch degenerate
    rewrites (deleting everything, topic replacement). Semantic-embedding
    meaning preservation lives in the eval harness (phase 10), optionally
    behind an `embeddings` feature flag later — not a core dependency.
- **Packages** (Vale-style): `handprint pack add <name>@<version>` /
  `handprint sync`. Versioned artifacts: AI-ism lexicons (dated, per-model),
  fitted references ("hn-2015-2022", "gpt-4o-wildchat-2025.07"). Each pack
  ships a provenance manifest (sources, licenses, fit date, tokenizer policy,
  privacy-culling attestation for private-derived packs). `handprint.toml`
  declares packs + threshold profile per project.
- `hn` subcommands (from `handprint-data`): `hn fetch --user X`,
  `hn build-reference --random-authors 300 --min-words 5000 --before 2022-11`,
  `hn link --a mainacct --b throwaway`.

**Acceptance**

- End-to-end loop test: scripted "agent" (a template rewriter in the test
  harness) iterates against `critique --json` on a salted draft and converges
  to pass in ≤ N iterations; canary trip test (comma-stuffing fixture flips
  `canary_ok`).
- JSON contract golden files; exit-code semantics.

## Phase 9 — Data pipelines (`handprint-data`)

All extractors emit one normalized JSONL: `{text, author?, model?, source, ts,
register, license}` → corpus directories. Raw corpora never ship; only fitted
packs with provenance manifests do.

1. **HN (human reference + forensic experiments)**: Algolia API
   (`hn.algolia.com/api/v1`, no key, ~10k req/hr) for per-user pulls +
   incremental; BigQuery `bigquery-public-data.hacker_news` for bulk history
   (verify freshness first: `SELECT MAX(timestamp)` — updates broke 2022–24,
   then restored). **Cut the human reference at 2022-11** (contamination:
   post-ChatGPT HN contains LLM-assisted text; AI-isms are leaking into human
   usage). Cache to disk; paginate; rate-limit.
2. **Local agent chat logs — the freshest per-model AI corpus available
   anywhere** (no public dataset covers these models; inventoried on this
   machine 2026-07-31):
   - **Claude Code**: `~/.claude/projects/**/*.jsonl` — 1.4 GB, 147 sessions +
     subagent transcripts. Extract `type == "assistant"` →
     `message.content[].type == "text"` blocks; **per-message `message.model`
     field**: `claude-opus-4-8`, `claude-fable-5`, `claude-sonnet-4-6`.
     ~1.41M words of assistant prose pre-stripping.
   - **Codex CLI**: `~/.codex/sessions/**/*.jsonl` (+ `archived_sessions`) —
     774 MB, 510 rollouts, 2026-02 → 2026-07. Extract
     `type == "response_item"` → `payload.type == "message" &&
     payload.role == "assistant"` → `content[].text`; model from the turn's
     `turn_context.payload.model`: `gpt-5.5`, `gpt-5.6-sol`,
     `gpt-5.3-codex`, `gpt-5.6-terra`. ~911k words pre-stripping. Skip
     `reasoning` items (different register) — or keep as a separate corpus.
   - **Code stripping**: drop fenced code blocks, inline code spans, tool-call
     residue, diffs/paths/URLs; drop messages that are >50% code after
     stripping. Keep the markdown *structure counts* (bullet/bold/header
     density) as features even where the content is stripped — formatting is
     signal. Target: ≥60% of raw words survive as prose.
   - **Register note**: agentic-assistant prose — which is *exactly* the
     deployment register (agents rewriting their own output), so this is a
     feature for the critic mode and a caveat for general-prose claims.
   - **Privacy**: local-only by default. Any pack derived from these logs goes
     through phase-6 privacy culling + manual review before leaving the
     machine. Handle truncated JSONL lines gracefully (some rollouts end
     mid-line).
3. **Public AI corpora** (feature-gated loaders, streaming, no redistribution):
   - `allenai/WildChat-4.8M` (ODC-BY — attribution in NOTICE): gpt-4o / o1 /
     gpt-4.1-mini profiles.
   - `lmarena-ai/arena-human-preference-140k`: multi-vendor 2025 models
     (Gemini 2.5, Claude 3.5/3.7, o3-mini…), per-response labels, same-prompt
     battles → controlled cross-model contrasts.
   - `liamdugan/raid` (MIT): pipeline validation — can handprint separate its
     11 models? — and adversarial robustness splits.
   - `browndw/human-ai-parallel-corpus` (HAP-E, MIT): gold-standard parallel
     human-vs-model design for contrast sanity checks.
4. **Self-generation battery** (optional, for models missing everywhere):
   fixed prompt battery (reuse arena/WildChat prompts, CC-BY) across current
   APIs; ~$10–70 per model per 1M words, halved via batch APIs. Provenance
   recorded per sample.

**Acceptance**

- Extractor golden tests on redacted fixture transcripts (both formats,
  including a truncated-line file).
- Code-stripping precision/recall spot-check on 50 hand-labeled messages.
- `hn build-reference` end-to-end against Algolia with caching.

## Phase 10 — Eval harness (`eval/`, Python)

Non-circular by construction: **the judge is never our own features** (STRAP's
lesson: single automatic metrics get gamed).

- **Away/towards scores** in third-party embedding spaces: LUAR (authorship) +
  `AnnaWegmann/Style-Embedding` or `StyleDistance/styledistance` (style-only).
- **Meaning preservation**: SBERT cosine between draft and rewrite (gate, not
  target).
- **External detector**: Binoculars score before/after de-AI loops; report TPR
  at fixed low FPR on RAID splits (incl. adversarial), never accuracy alone.
- **Forensic eval**: HN linkage experiment — same-author pairs' percentile
  distribution vs unrelated; GI verdict accuracy with abstention rate;
  FPR sanity check on non-native-English text (documented detector bias).
- **Headline experiment for the README**: a real agent (Claude Code) loops
  against `critique --json` on (a) its own fresh drafts → does the external
  Binoculars/Pangram score drop while SBERT similarity stays ≥ 0.9? (b) toward
  the author's HN corpus → does the rewrite land inside the same-author
  percentile band *without* the reported-features-only canary tripping?
  This experiment is the crate's thesis; it either sells it or falsifies it
  early — run a manual pilot as soon as M3 lands.

## Phase 11 — Polish

1. Docs: crate-level book with the three worked examples (HN linkage, de-AI
   loop, corpus-mimic loop); an honest **limitations** section: topic confound
   (the dominant AV failure mode — low percentile under topic-matched
   conditions is weak evidence; stopword-only MFW is the robust setting),
   short-text floors (graded, cited), adversarial pressure (the critic mode
   *is* an attack on the forensic mode — say so), model drift (profiles are
   dated artifacts), non-native-speaker false-positive risk, "not a verdict
   machine" framing throughout.
2. Benchmarks: profile 10k comments, fit on 5M words; transform ≥ 50 MB/s
   single-threaded; `rayon` behind `parallel` flag for corpus fitting only.
3. CI: test + clippy + `cargo semver-checks`; MSRV policy; fixtures via git-lfs
   or generated.

---

## Milestones

| Milestone | Contents | Exit criterion |
|---|---|---|
| M1 | Phases 1–2 | tokenizer snapshots green (incl. artifact flags); sparse vec benched |
| M2 | Phase 3 (feats 1–5) + Phase 4 | HN linkage runs end-to-end with length-stratified percentiles + LR |
| M3 | Phase 5 + minimal Phase 8 (`critique --json` with punct/sentence/lexicon findings, shipped seed lexicon) | scripted-agent loop converges on a salted draft; **manual pilot of the headline experiment** |
| M4 | Phase 6 (corrected) + Phase 3 feat 7–8 | Fightin' Words matches hand-computed + external impl; discover→classify→highlight on human-vs-LLM fixture |
| M5 | Phase 7 + full Phase 8 (guards, packs) | GI agrees with stylo; canary/drift guards trip correctly |
| M6 | Phase 9 | per-model packs fitted from local logs (opus-4.8, fable-5, gpt-5.5…) + HN reference pack, all with provenance |
| M7 | Phase 10–11 | headline experiment numbers in README; docs done |

M2 is the first forensic payoff; M3 is the first agent-critic payoff and the
earliest falsification point — reprioritize everything after it based on what
the pilot shows.

## Resolved decisions

- Name: **`handprint`** (crates.io free as of 2026-07-31).
- Default metric: **Cosine Delta** (Classic Delta kept for comparability).
- Scaling: per-metric property (invariant #5); per-text z-score default.
- Calibration: dual distributions (same-author + unrelated), length-stratified;
  `percentile` renamed `p_value_vs_unrelated`.
- Fightin' Words: full Eq. 19 variance default; one term universe per fit;
  α₀ ≈ min(n_i, n_j) guidance.
- Local chat logs are a first-class per-model corpus source (privacy-culled).

## Open decisions (fine to defer)

- Whether `Corpus` streams from disk (RAM is fine for v0; revisit > 100M
  words).
- MSRV; `no_std` (recommend: no).
- Embedding-based drift guard in-CLI (ONNX/candle) vs eval-harness-only
  (start: eval-only).
- MCP server wrapper around `critique` (post-v1; the JSON contract is designed
  so this is trivial).
- Whether Codex `reasoning` items become a separate register corpus.
