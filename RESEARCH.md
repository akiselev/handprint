# handprint — Research Findings (2026-07-31)

Synthesis of four parallel research passes over IDEA.md: (1) algorithm verification
against primary literature, (2) AI-text style markers, (3) datasets, (4) prior art
and gaps. Everything below is sourced; URLs inline.

**Product framing** (decided, not in IDEA.md yet): handprint serves *both*
- the **forensic mode** in IDEA.md (account linkage, verification — distances,
  percentiles, contributions; never verdicts), and
- an **agent-critic mode**: a CLI that LLM agents call in a rewrite loop —
  (a) score a draft for AI-ness with actionable per-feature feedback and rewrite
  until it passes; (b) same loop targeting a user-supplied corpus (e.g. an HN
  comment history). handprint is the objective function + explainer; the agent is
  the optimizer. No rewriting inside the crate.

---

## 1. Algorithm verification — corrections to IDEA.md

Bottom line: nothing invalidates the design. The Fightin' Words δ formula is
exact, Burrows's Delta is correctly stated, and the feature-family choices
(punctuation-first, keep-the-spaces char n-grams) are notably well supported.
Four findings are high-severity; fix before implementation.

### HIGH severity

**H1. α₀ guidance is anchored to the wrong quantity** (IDEA.md "recommend α₀ ≈
background size × 0.01"). In Monroe, Colaresi & Quinn 2008, α₀ is a *pseudo-sample
size in tokens*, calibrated against the size of the corpora being compared
(they used ≈ average group size). Shrinkage is governed by α₀/n_i; scaling to the
background makes regularization depend on an unrelated quantity — a 5M-word HN
background gives α₀ = 50k, swamping a 20k-word user corpus. Corrected guidance:
α₀ ≈ min(n_i, n_j) as a neutral start; α₀ → 0 recovers raw log-odds; α₀ ≫ n_i
shrinks everything to zero. Prior per word: α_w = y_w·α₀/n from the background
(paper Eq. 23). Also: α₀ must equal Σα_w over the term universe — with
`Prior::Uniform(alpha)`, α₀ = alpha·|V| changes silently with vocab config, so
make α₀ an explicit logged quantity in `ContrastModel`.
(Paper PDF: https://languagelog.ldc.upenn.edu/myl/Monroe.pdf, Eqs. 15–23.)

**H2. Mixed term universes break the multinomial bookkeeping.** If one position
emits a unigram + bigram + trigram + char n-grams, n_i (number of draws) is
ill-defined and the `n_i + α₀ − y − α` denominators are wrong. Fit each term
universe (words / word bigrams / char 4-grams / …) as a separate contrast with
its own n_i and α₀; merge ranked lists afterward. Cross-order z-scores are not
on a common scale.

**H3. MinMax/Ruzicka on z-scores is ill-defined** — it requires non-negative
input (`stylo::dist.minmax` takes no scale argument; Kestemont et al. 2016's best
configs are minmax-tf / minmax-tf-idf). Feed it unscaled relative frequencies;
make `Scaling` a per-metric property or type-gate it. It *does* decompose
(invariant #1 holds): contribution_i = |a_i − b_i| / Σ_j max(a_j, b_j).
(https://www.clips.uantwerpen.be/~walter/papers/2016/kskkd16.pdf)

**H4. Calibration is half of what's needed and over-confident on short texts.**
The unrelated-pair percentile estimates only the different-author (H_d) score
distribution — a p-value, not evidence strength. Fixes:
- Also sample **same-author** distances (split each background author's docs in
  half; the HN corpus supports this) → report a score-based likelihood ratio
  and enable the one-class "does this draft belong to the corpus" gate the
  critic loop needs as its stopping criterion.
- **Length-stratify both distributions** (bin by token count, interpolate or
  resample at compare time). Distances shrink as texts lengthen; a flat 10k-pair
  sample of full profiles gives systematically over-confident percentiles for
  400-word queries — the primary use case. Quantified collapse: Halvani et al.
  2019 (https://arxiv.org/abs/1906.10551) show verification c@1 dropping to
  chance at 250–500 chars with unadjusted thresholds.
- Rename `percentile` → something like `p_value_vs_unrelated`; the doc-comment
  "evidence of linkage" phrasing invites the prosecutor's fallacy.
- Add the **General Imposters method** as a `verify` entry point (~40 lines over
  existing metric infra): bootstrap over feature subsets (50%) + impostor
  subsets (50%), score = proportion of iterations the target wins, with a
  p1/p2 abstention grey zone. Field standard (won PAN 2013/14); implemented in
  R stylo for cross-checking. (Koppel & Winter 2014:
  https://asistdl.onlinelibrary.wiley.com/doi/10.1002/asi.22954;
  https://computationalstylistics.github.io/blog/imposters/)
- Known dominant failure mode to document: **topic confound** — PAN 2020 found
  topic similarity correlates with verifier errors (R²≈0.16 overall;
  misclassifications are where topic misleads); "none of the examined AV methods
  is robust against topical influence" (Halvani et al.).
  (https://ceur-ws.org/Vol-2696/paper_264.pdf)

### Medium / low severity

- **Make Cosine Delta the default metric** (cosine/angle on z-score vectors).
  Evert et al.: it consistently outperforms Classic and Quadratic Delta and is
  robust up to n_w = 10,000 ("vector normalization appears to be the key to
  robust authorship attribution"). Keep Classic Delta for literature
  comparability. Decomposes fine (per-dim products + global normalizer).
  (https://aclanthology.org/W15-0709.pdf,
  https://academic.oup.com/dsh/article/32/suppl_2/ii4/3865676)
- **Fightin' Words variance**: IDEA.md has the paper's own Eq. 20 approximation
  (drops the 1/(n−y+α) terms; assumes n ≫ y ≫ α). Implement full Eq. 19 (two
  extra reciprocals; Eq. 20 inflates |z| by 1/√(1−p) — ~3.7% for "the"-class
  words). The Python `fightin-words` reference impl uses Eq. 20, so the
  cross-check acceptance test must pin the variant.
- **Char n-gram typing (Sapkota et al. 2015, NAACL)**: type 3-grams by position
  (prefix/suffix/space-prefix/space-suffix/mid-word/beg-/mid-/end-punct).
  Affix+punct-only matches or beats untyped everything at ~⅔ the features, with
  the biggest wins cross-domain — and typed dims make explanations meaningful
  (`sti` as word-prefix is authorial; mid-word it's junk).
  (https://aclanthology.org/N15-1010.pdf)
- **Topic-robustness claim is backwards** (IDEA.md limitations bullet): char
  n-grams transfer *better* than MFW cross-topic/cross-genre (Stamatatos 2013,
  https://brooklynworks.brooklaw.edu/jlp/vol21/iss2/7/). MFW's stopword-only
  mode is the topic-robust setting; document that instead.
- **Richness**: MATTR@100 is undefined for docs < window — define the fallback
  explicitly (flagged value, never silent whole-doc TTR). Add **MTLD** — the
  only index McCarthy & Jarvis 2010 found length-invariant
  (https://link.springer.com/article/10.3758/BRM.42.2.381). Yule's K/Simpson's D
  are theoretically length-invariant but empirically drift in real prose
  (Tweedie & Baayen 1998,
  https://quantling.org/~hbaayen/publications/TweedieBaayen1998.pdf) — carry
  length warnings.
- **Eder's Delta**: stylo's shipped code ≠ its documented formula (weight
  off-by-one, missing /n) — rank-equivalent, but "order match, not exact values"
  is the only viable acceptance criterion.
- **Length floors, graded** (Eder 2015/2017): <2,000 words unreliable;
  2,000–5,000 usable with clear signal; ≥5,000 literature-supported. Encode as
  tiers in `Profile::warnings()`, not one cliff.
- `compare(p,p) ≈ 0` is a vacuous test; assert cross-author-near-uniform on
  real (topic-confounded) data, not just synthetic.
- Punctuation-first is vindicated: Grieve 2007 found frequent words +
  punctuation the best single combo; punct/affix n-grams carry most char-n-gram
  power; emoji-only attribution reaches 83% on chat messages.

---

## 2. What actually marks AI text (feature targets)

### Empirically documented

- **Excess vocabulary** (Kobak et al., Science Advances 2025): "delve" +~1,500%
  in PubMed abstracts 2022→2024; style words (verbs/adjectives), not content
  nouns. **Ships 900 annotated excess words + full yearly count matrix**:
  https://github.com/berenslab/llm-excess-vocab (updated Jul 2025, monthly
  resolution). Best empirical seed list.
- **Liang et al. ICML 2024** (peer reviews): "commendable" 9.8×, "meticulous"
  34.7×, "intricate" 11.2×; top-100 adjectives + adverbs in Supp. Table 2.
  (https://arxiv.org/abs/2403.07183)
- **RLHF is the causal source** of the lexicon (Juzek & Ward, COLING 2025,
  https://arxiv.org/abs/2412.11385) — so every model/version has its own list.
  Base (non-RLHF) models largely evade detectors
  (https://arxiv.org/html/2605.19516v1): the detectable style is a post-training
  artifact.
- **Markers decay and are generation-specific**: "delve" *declined* after public
  mockery in early 2024 (Geng & Trotta, https://arxiv.org/abs/2502.09606);
  em-dash overuse is GPT-4-era (GPT-3.5: ~0). **Word lists must be versioned
  data (dated, per-model), never code.** AI-isms also leak into genuine human
  usage (spoken language shift: https://arxiv.org/abs/2409.01754) → false-positive
  risk on recent human text; cut human reference corpora at ~Nov 2022.
- **Structural, quantified**: lower sentence-length variance ("burstiness" —
  direction documented, vendor thresholds folklore; implement stddev/variance +
  Fano factor of per-sentence token counts), reduced lexical diversity
  (https://arxiv.org/pdf/2502.11266), markdown/bullet/bold affinity tied to
  markdown-heavy training (https://arxiv.org/pdf/2603.27006).
- **Community catalog**: Wikipedia "Signs of AI writing"
  (https://en.wikipedia.org/wiki/Wikipedia:WikiProject_AI_Cleanup/Guide):
  significance inflation ("stands as a testament", "pivotal role"), copula
  avoidance ("serves as/boasts/features" for "is"), negative parallelism ("not
  just X, but Y"), rule of three, elegant variation, curly-quote/Title-Case/
  bold-header-bullet formatting tells, vague attribution ("Experts argue"),
  formulaic conclusions. Machine-readable encodings: `no-slop` (13 rules,
  40+ words: https://github.com/Byk3y/no-slop), `awesome-slop` index
  (https://github.com/hwajongpark/awesome-slop).
- **Per-model profiles work**: 97.1% five-way ChatGPT/Claude/Grok/Gemini/DeepSeek
  identification, driven by word-level distributions, surviving rewrite/
  translation (Sun et al. ICML 2025, https://arxiv.org/abs/2502.12150); family
  fingerprints persist under style prompts (https://arxiv.org/abs/2503.01659);
  `slop-forensics` (https://github.com/sam-paech/slop-forensics) rebuilds model
  family trees from over-represented word/n-gram profiles alone — our exact
  feature class carries the signal. Per-model "this reads like GPT-4o" claims
  are realistic at family granularity; profiles need dating/versioning.

### Design notes

- **Unicode-normalize before scoring; flag homoglyphs** as artifacts (SilverSpeak
  homoglyph attacks collapse detectors but aren't humanization:
  https://aclanthology.org/2025.genaidetect-1.1/). Also flag chatbot artifacts
  (`oaicite`, `turn0search`, `[cite: 1]`).
- Detector context: supervised classifiers (Pangram: near-zero FPR/FNR on
  medium+ texts per NBER eval, https://www.nber.org/papers/w34223) currently
  beat zero-shot statistical methods (Binoculars, Fast-DetectGPT) but are
  opaque — interpretable feedback is the gap handprint fills, not raw accuracy.
  Known detector failure modes to design around: non-native speakers (>50% FPR
  on real TOEFL essays, https://arxiv.org/pdf/2304.02819), short texts,
  hybrid/edited text.
- **Score-guided rewriting is proven**: detector-guided adversarial paraphrasing
  cuts TPR@1%FPR by ~88% avg (https://arxiv.org/abs/2506.07001); CAMOUFLAGE
  implements the exact two-agent critique loop (https://arxiv.org/pdf/2505.01900).
  **No published system uses interpretable stylometric feature deltas as the
  loop signal — that is handprint's open niche.**
- **Style imitation needs explicit rules + exemplars**: few-shot yields up to
  23.5× better style matching than zero-shot (https://arxiv.org/abs/2509.24930),
  but LLMs still fail on informal idiosyncratic style
  (https://arxiv.org/abs/2509.14543) → corpus-mimic mode should emit extracted
  style rules + representative exemplars, not just scores.

---

## 3. Datasets

### Per-model AI style references (start with these)

| Dataset | Why | Freshness | License |
|---|---|---|---|
| `allenai/WildChat-4.8M` (HF) | 4.7M real ChatGPT convs, exact model label; 2.5M gpt-4o, o1, gpt-4.1-mini | through Jul 2025 | ODC-BY (cleanest big fresh corpus; add NOTICE attribution) |
| `lmarena-ai/arena-human-preference-140k` (HF) | only public multi-vendor 2025 set: Gemini 2.5, Claude 3.5/3.7, o3-mini, Qwen…; per-response labels; same-prompt battles = controlled contrasts | Apr–Jul 2025 | prompts CC-BY-4.0; outputs provider-ToS — profiles OK, no text redistribution |
| `liamdugan/raid` (HF) | 6M+ gens, 11 models × 4 decodings × 11 adversarial attacks, human-parallel by domain; the pipeline-validation benchmark | GPT-4-era (stale) | MIT |
| `browndw/human-ai-parallel-corpus` (HAP-E) | gold-standard parallel design (human vs model continuations of same seed); companion PNAS paper: Biber features separate 6 LLMs+humans 66% 7-way — a published prior for our approach | gpt-4o era | MIT |
| Self-generation | GPT-5.x / Claude 4.5+/Fable / Gemini 3 have no clean public corpus. ~$10–70 per model per 1M words (batch APIs halve); fixed prompt battery (reuse WildChat/arena prompts) = controlled register | current | record provenance; ToS fine for detection profiles |

Also useful: M4GT-Bench (canonical multi-way model-attribution benchmark,
https://github.com/mbzuai-nlp/M4GT-Bench), MAGE (27 older LLMs, Apache-2.0),
Beemo (expert-edited AI text — the hybrid-text adversarial case,
https://github.com/Toloka/beemo). Skip: HC3 (stale + CC-BY-SA), ShareGPT
(no provenance), LMSYS-Chat-1M (2023-era, no redistribution).

### Human style corpora

- **HN**: BigQuery `bigquery-public-data.hacker_news` (verify freshness:
  `SELECT MAX(timestamp)` — updates were broken 2022–24, then restored) + Algolia
  API `hn.algolia.com/api/v1` (free, no key, ~10k req/hr) for per-user pulls.
  **Cut human reference at ~Nov 2022** (contamination). Right register.
- **Reddit**: Pushshift is mod-only since 2023; use Academic Torrents monthly
  dumps (through 2025-06+) or Arctic Shift; pre-2023 slices; keep raw text
  internal (2023+ Reddit ToS).
- **Blog Authorship Corpus** (19k bloggers, 140M words, 2004 = AI-free) —
  non-commercial license → eval-only. **Enron** (~500k msgs, unrestricted).
  **PAN** verification sets (fanfiction PAN20/21; cross-discourse PAN22 incl.
  text messages) — research-only, no redistribution → benchmark-only.
  PAN'25/26 has LLMs-mimicking-specific-authors data (our threat model) but
  hard no-redistribution.

### Licensing rule

Fitted numeric profiles are low-risk everywhere; the landmines are all about
redistributing raw text (LMSYS/PAN forbid; HC3 share-alike; arena outputs
provider-ToS; Reddit/HN bulk text grey). **Ship profiles + provenance manifest,
never corpora.**

---

## 4. Agent-critic mode — design decisions from research

1. **JSON diagnostic contract, Vale/textlint-shaped** (`handprint critique
   --json`): per finding — namespaced rule ID (`punct.em_dash_rate`), severity,
   byte spans, observed vs target band, z-score, direction ("reduce ~60%"),
   optional fix commands for lexicon hits; top-level pass/fail vs a declared
   threshold profile; Vale-style exit codes (0 clean / 1 findings / 2 error).
   Precedents: Vale (https://vale.sh, incl. its `metric` rule type = our rate
   thresholds), textlint fix-command format, proselint's versioned wire schema.
2. **Vale-style Packages**: versioned downloadable packs — AI-ism lexicon
   (dated, per-model), fitted references (e.g. "HN 2015–2022"). Matches the
   markers-decay finding: lists are data with vintages.
3. **Goodhart resistance**: score all features, *report* only top-k offenders
   per iteration; hold out a canary family (reported-features improve while
   canary worsens ⇒ flag overfitting); joint gate = geometric mean over style
   score + SBERT meaning preservation (+ optional external style embedding);
   cap per-dim credit. STRAP showed single automatic metrics get gamed
   (https://arxiv.org/abs/2010.05700) — never emit one scalar as the claim.
4. **Stopping criterion** = the same-author/one-class calibration from H4:
   "draft is within the corpus's internal variation" (or GI verify says
   target-set wins with score above p2).
5. **Corpus-mimic output** should include extracted style rules + exemplar
   snippets (few-shot beats zero-shot 23.5×), not scores alone.
6. **New feature family — corpus-LM surprisal**: fit a char-trigram/word-bigram
   LM on the reference during `fit` (data-only, serializable); features = mean
   cross-entropy, per-token surprisal variance (burstiness), surprisal
   autocorrelation. Decomposes to spans; hard to fake by word-stuffing; the
   GLTR/Binoculars insight without an LLM dependency.
7. **Rolling/windowed profiling** (stylo rolling.delta precedent) to localize
   the most AI-ish region of a long draft.
8. **Non-circular eval harness as a deliverable**: away/towards scores in LUAR
   + Wegmann/StyleDistance embedding spaces, SBERT meaning preservation,
   Binoculars as external detector; report TPR at fixed low FPR on RAID splits;
   for corpus-mimic, rewrite must land in the same-author percentile band.
   The judge must not be our own features. (STYLL away/towards:
   https://arxiv.org/html/2212.08986v3; embeddings:
   https://huggingface.co/AnnaWegmann/Style-Embedding,
   https://huggingface.co/StyleDistance/styledistance)

### Closest prior art (citations that validate the approach)

- **StyleRemix** (EMNLP 2024, https://arxiv.org/abs/2408.15666): obfuscation via
  7 interpretable style axes — proof interpretable-axis steering works.
- **STYLL** (https://arxiv.org/abs/2212.08986): imitates a Reddit author from
  ~16 posts via textual style descriptors.
- **AuthorMist** (https://arxiv.org/abs/2503.08716): RL against detector APIs.
- Iterative critique-refine personalization (https://arxiv.org/abs/2510.24469):
  the loop, but LLM-judged rather than measured.
- None ship measured, corpus-calibrated per-feature deltas as a reusable CLI.

---

## 5. Landscape & naming

- **`handprint` is AVAILABLE on crates.io** (as of 2026-07-31). `stylo` is taken
  (Servo CSS engine). Nearest Rust competitors: `stylometry-cli` 0.2.0
  (Jul 2026, 21 downloads, Delta-only), `stylometry` (abandoned 2022
  placeholder), `phantomdev` suite (code-humanizer, adjacent). Niche empty but
  stirring — ship sooner.
- Existing AI-ness tools are word-list linters (`no-slop`, `ai-slop-detector`,
  slop-gate) or black-box MCP humanizer APIs (Text2Go, WriteHuman…). **No
  corpus-calibrated, span-attributed, interpretable critic exists.** No open
  CLI says "em-dash rate 6.2/1k vs corpus 1.1/1k (z=+3.4)".
- Feature-set gaps vs Writeprints-static/JGAAP worth cherry-picking: letter/
  digit unigram frequencies, word-length distribution, curated function-word
  list option, misspelling/typo rate (a *human* marker), discourse-marker/
  sentence-opener rates, readability grades. POS tags: skip or feature-gate
  (function words proxy most of it).
- R stylo remains the cross-check target (Delta ranking agreement, imposters,
  rolling delta). Python `faststylometry` calibrates Delta→probability with
  logistic regression — cite as prior art for our calibration.
