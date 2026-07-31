# Wit 2 — gap analysis: what WIT.md missed (2026-07-31)

Second research round, run while WIT-plan.md is being implemented. Five parallel
research passes hunted for metrics/algorithms/features/methods absent from
`WIT.md` and `WIT-plan.md`: (1) stylometry & authorship attribution, (2)
computational humor & figurative language, (3) discourse/cohesion/coherence,
(4) AI-text detection, (5) prose prosody & psycholinguistic norms. Everything
below passed a redundancy check against WIT.md, WIT-plan.md, and the as-built
`feature/` code; things verified as already covered are listed at the end.
Nothing here changes WIT-plan.md — this is a candidate pool for a future
plan revision (WIT-plan Wx phases are referenced where a candidate would slot
in cleanly).

## Verdict

Three structural findings dominate:

1. **Dialogue is a hole.** Nothing in any doc or the code measures quoted
   speech — no dialogue/narration ratio, no quote-length distribution, no
   speech-verb ("said-bookism") profile. Two research passes independently
   flagged it, and it directly hits the W10 capstone: Wodehouse, Hemingway,
   and Twain are all dialogue-heavy authors with opposite tag styles.
2. **The W5 `NormDensity` mechanism is under-fed.** At least eight published
   norm datasets (humor, iconicity, AoA, prevalence, socialness, imageability,
   valence/dominance, frequency bands) are pure `NormPack` payloads for
   machinery the plan already specifies — data curation, zero new feature
   code, each a distinct voice axis.
3. **Everything current is order-blind.** MFW, richness, lexicon packs, and
   norm densities are bags of words; punct and sentence stats are independent
   rates. The strongest uncovered stylometry (punctuation sequences,
   function-word adjacency, word dispersion, opener diversity, surprisal-shape
   statistics) is all *sequential/positional* — and sequence-shape uniformity
   is simultaneously the best-validated new AI tell (DivEye, GPT-who).

## 1. Sequence & position stylometry (parser-free, no new data)

- **Punctuation-sequence transitions.** Joint/conditional probabilities of
  successive punctuation marks + word-gap distributions between marks —
  authors are recognizable from punctuation *order* alone, with mark→mark
  transition probabilities carrying the most signal (Darmon et al., EJAM 2020,
  https://arxiv.org/abs/1901.00519). As-built `PunctTypography` has rates
  only. Dims: top-k `punctseq:{a}->{b}` transitions (fit-selected, MFW-style)
  + gap mean/CV. Closed mark alphabet, no pack.
- **Function-word adjacency (WAN).** Directed transition profile over the
  closed function-word list ("of→the", "and→I") — the best parser-free
  syntactic proxy in the attribution literature (Segarra et al., IEEE TSP
  2015, https://arxiv.org/abs/1406.4469). Relative entropy between WANs
  decomposes per-edge, satisfying the decomposability invariant. Reuses the
  as-built `FUNCTION_WORDS` const; fit selects top bigrams as dims.
- **Positional MFW: sentence-initial and sentence-final token distributions.**
  Where an author starts and lands sentences (Bukowski ends on concrete
  nouns; LLM prose ends on significance-inflation phrases). Fit-selected
  top-k per slot (`mfw_init:*`, `mfw_final:*`), reusing MFW + `structure()`
  machinery wholesale. Grieve 2007 evaluated positional variables; nothing
  in the plan generalizes beyond `syn:initial_conj_rate` and specific opener
  phrase packs.
- **Sentence-opener diversity** (de-AI overlap): fraction of sentences
  beginning with a unique 5-word prefix, entropy of the first-token
  distribution — formulaic openers are a documented LLM repetition tell
  (https://arxiv.org/pdf/2603.21228). Needs MATTR-style length windowing.
  Complements (doesn't duplicate) the positional-MFW dims: one measures
  *which* openers, the other *how varied*.
- **Word-dispersion / burstiness fingerprint.** DP (Gries 2008) or gap-CV
  intermittency of top function words across document segments — how
  *clumped* usage is, invisible to frequency features (Altmann et al. 2009,
  https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0007678;
  attribution via intermittency: https://arxiv.org/abs/1502.01245). Doubles
  as an AI tell (LLM text is temporally more uniform). Needs a documented
  min-length floor; the deferred callback/running-gag item is the only
  adjacent thing in the plan and this is computable per-draft.
- **Alternation-pair preference pack.** Within-pair ratios for doublets
  (while/whilst, toward/towards, -ise/-ize, first/firstly, email/e-mail…) —
  the Mosteller–Wallace discriminator class; the pair normalizes topic and
  length away. `verse:thou_you_ratio` is the same mechanism applied only to
  EME. ~40–60 pairs, original curation, `alt:{pair}` as `Fraction` with
  per-pair count floors → `mark_missing`.
- **Text-distortion preprocessing mode** (Stamatatos EACL 2017,
  https://aclanthology.org/E17-1107/): mask tokens outside a fitted top-k
  list before char n-gramming → topic-neutral `c3d:*` dims. Directly targets
  the topic-confound failure mode RESEARCH.md documents but doesn't mitigate.
  Architectural (a `distortion` option on `CharNgrams`; mask vocab is
  fit-time data-only state).
- **HD-D** — one more `Richness` dim. The repo's own cited authority
  (McCarthy & Jarvis 2010) recommends pairing MTLD with HD-D; as-built has
  MTLD/MATTR/Yule/Simpson but not HD-D. Closed-form from the frequency
  spectrum already computed. (vocd-D itself: don't add — HD-D supersedes it.)
- **Zeta (Craig/Burrows) as a second contrast engine.** Segment-presence
  dispersion instead of frequency log-odds — burst-robust complement to
  Fightin' Words for the auto-derived signature lexicons (Schöch et al., CHR
  2021, https://ceur-ws.org/Vol-2989/short_paper11.pdf). ~100 lines beside
  `ContrastModel`, flowing into the same `ContrastVocab` plumbing.
- **PPM-grade surprisal + eval-side compression baselines.** (a) Optional
  variable-order escape-smoothed backoff on `SurprisalLm` (spec flag, serde
  default) — strictly better LM, same fitted-counts shape, still decomposes
  per token. (b) NCD/OCCAV (Halvani et al.,
  https://arxiv.org/abs/1706.00516) as eval-harness baselines only — full
  NCD violates the decomposability invariant and stays out of core
  (Binoculars precedent).
- **Intra-document style-consistency dims.** Rolling-window profiling is
  planned only as *localization* (RESEARCH.md §4.7); the window-score
  *dispersion* itself is an unemitted trait — register-stable vs swingy
  authors, and style uniformity is a known LLM tell (PAN style-change task
  series). Architectural: sits above the feature layer (needs the fitted
  pipeline per window); flag for a design note, not a quick add.

## 2. Humor & figurative additions

- **Humor norms NormPack** — Engelthaler & Hills 2018, 4,997 words with
  funniness ratings (https://link.springer.com/article/10.3758/s13428-017-0930-6;
  NoRaRe packaging exists); Westbury & Hollis extend to 45k model-estimated
  words with published *form* predictors (length, letter-probability
  improbability) that are computable in Rust as an OOV fallback
  (https://gwern.net/doc/psychology/novelty/2019-westbury.pdf). The most
  obvious "humor as data" slot-in and it's absent. Rides `NormDensity`
  unchanged. License: BRM/OSF — verify; loader-only fallback per house rule.
- **Idiom-modification rate.** Near-miss matches to canonical idioms
  (one-slot substitution: "spilling the bag") vs exact-idiom rate — creative
  twisting (Pratchett/Adams) vs cliché as two dims. MAGPIE provides 1,756
  idiom types, **CC-BY-4.0, bundleable**
  (https://aclanthology.org/2020.lrec-1.35/). `frame:frozen_share` covers
  similes only; general idioms are a different device. Needs a small fuzzy
  "one substituted content-word slot" extension to the pattern matcher
  (cousin of the planned `#` wildcard).
- **Bathos/deadpan proxy: peak-end affect trajectory.** Final-clause valence/
  arousal minus sentence-body mean — the statistical proxy for what WIT.md
  punts entirely to the LLM judge. Same clause-split machinery as
  `surp:punch_ratio_mean`, over a norm table instead of LM surprisal; the
  2026 DPV stand-up study finds *peak* violation placement beats average
  incongruity (https://arxiv.org/html/2605.00143). VADER valence (MIT,
  bundle) as default; Warriner VAD loader-only upgrade.
- **Irony-marker battery** (Burgers et al. 2012 taxonomy,
  https://journals.sagepub.com/doi/abs/10.1177/0261927x12446596): scare-quote
  rate (short non-speech quoted spans — also a blogger-voice axis), echoic
  cue-phrase pack ("yeah right", "oh sure", "how convenient"),
  interjection-adjacent-exclamation and hyperbole+positive-word combos.
  WIT.md has exactly one irony marker (the "about as" hedge). Re-curate from
  published taxonomy tables (Hyland rule). Scare quotes fall out of the
  dialogue partition (§3).
- **Verb-based personification rate.** Animate-selecting verb (closed list:
  decide, refuse, sulk, lurk…) near an inanimate/abstract noun — the Adams
  "ships hung in the sky" pole. Transferred epithet covers the *adjective*
  side only; personification currently exists only as an LLM move-index
  label. Animacy via a small NormPack (VanArsdall & Blunt; verify license)
  with a WordNet hypernym fallback (license machinery already planned).
- **Homophone/phonological-ambiguity density.** Mean homophone-set size and
  within-sentence homophone-pair rate — pun *material* without pun
  *detection* (which WIT.md rightly skips). Homophone sets compile directly
  from the already-planned CMU dict (group by pronunciation) into a small
  derived table so the dims aren't verse-gated. No new license exposure.
- **Punchline semantic-cohesion drop (PMI tier).** Mean PMI of final-clause
  content words against sentence-body words, PMI table fitted from the
  reference corpus — catches *off-topic* twists that LM surprisal misses
  (WIT.md's own Huang-2024 caveat is the argument; Skalicky & Attardo
  validate the incongruity-as-distance construct,
  https://aclanthology.org/2025.chum-1.6/). Pure Rust, data-only fitted
  state; the embedding version stays Tier C.
- **Typographic pause-before-punch.** Rate of pause punctuation (em-dash,
  ellipsis, colon) immediately preceding the final clause of spike
  sentences — written comic timing; near-zero cost inside the W2 clause-split
  code path (DPV: pauses lengthen before high-surprise punchlines).
- **Iconicity + reduplication.** Winter et al. 2023 iconicity ratings, 14,776
  words, open access (https://link.springer.com/article/10.3758/s13428-023-02112-6)
  — sound-symbolic/playful diction as a `NormDensity` pack (subsumes a
  separate onomatopoeia lexicon); plus a trivial reduplication-rate dim
  ("okey-dokey", "higgledy-piggledy") and letter-bigram improbability
  (Westbury's form-funniness predictors) in code.
- **Eval-side humor ground truth.** SemEval-2021 HaHackathon (10k texts,
  graded funniness) to regress the wit dims against human ratings — the
  external anchor the W8 batteries lack (they validate separation and
  interpretability, never funniness). Twitter-sourced → local eval only.

## 3. Dialogue-mechanics family (two passes independently; merged)

The biggest single gap. Requires one piece of infrastructure: a
**dialogue/narration span partition in `Structure`** (paired-quote detection
over the existing quote-char machinery + paragraph structure; documented
heuristics for unpaired/nested) — the W7 `lines` table is the architectural
precedent. Then, all closed-list/regex grade:

- `dlg:dialogue_share` (quoted-speech token ratio), quoted-run length
  distribution (mean/var — Hemingway's clipped exchanges vs Wodehouse's
  speeches), dialogue-block burstiness;
- **said-bookism index**: share of attributions using *said/says* vs a
  colorful reporting-verb closed list (~150 verbs, original curation,
  inflection rules) — a strong era/author/genre marker
  (https://academic.oup.com/dsh/article/32/suppl_2/ii31/3978683 is the
  validated computational template; PDNC documents attribution structure);
- adverbed-attribution rate ("she said, icily" — reuses the `-ly` machinery);
  attribution position (pre/mid/post-quote);
- scare-quote rate (§2) and **eye-dialect split by partition** (§6) both
  consume the same spans.

New `Family::Dialogue` (or fold into Syntax); `mark_missing` on
dialogue-free docs.

## 4. Discourse, cohesion, and document shape

- **Categorized connective density with position conditioning.** Causal/
  temporal/adversative/additive (+ polarity) classes per 1k, **plus
  sentence-initial and paragraph-initial conditioning** and a connective
  type-token diversity dim. Hyland `transitions` is one undifferentiated
  category; the documented AI tell is distributional — formulaic
  paragraph-opening "Moreover" drumbeat up, connective *diversity* down
  (https://www.mdpi.com/3042-8130/2/1/2; https://arxiv.org/abs/2505.12218).
  Re-curate class lists from Coh-Metrix/PDTB tables (words unprotectable).
  Dedupe note vs `lex:hyland:cat:transitions` (pick one per pipeline).
- **Adjacent-sentence lexical overlap** (Coh-Metrix "argument overlap" /
  TAACO adjacent-overlap): share of adjacent sentence pairs sharing ≥1
  content lemma, mean overlap proportion, paragraph→paragraph variant.
  Content word = non-`FUNCTION_WORDS` + light suffix stripping; no parser.
  Nothing inter-sentence exists anywhere in the plan. De-AI pair: AI text
  shows conjunctive framing *up*, referential maintenance *down* — overlap
  vs connectives is a discriminating pair.
- **Paragraph-shape family.** Paragraph length distribution (mean/SD/CV in
  sentences and tokens), single-sentence-paragraph rate, paragraph-length
  autocorrelation, per-paragraph readability/sentence-length variance —
  the paragraph-level analog of sentence burstiness; uniform paragraphing is
  a replicated AI tell, and Gwern-vs-punchy-blogger is a clean two-sided
  axis. Trivial on existing `Structure`.
- **Reference & givenness dims.** Pronoun:content-word ratio,
  sentence-initial pronoun rate, **demonstrative-NP vs bare-demonstrative
  split** ("this + noun" shell-noun construction — known academic-cohesion
  and AI tell), definite:indefinite article ratio (articles appear nowhere
  in the plan), new-content-introduction rate. Extends `biber.rs`; dedupe
  note vs `biber:pron:*`.
- **Paragraph opener/closer conventions.** Distribution of paragraph-initial
  token *classes* (connective/pronoun/article/demonstrative/capital/number)
  — the general version of what `tech-voice` covers with specific phrases.
  Shares a module with the connective work.
- **Propositional idea-density proxy** (CPIDR-style): verbs + adjectives +
  adverbs + prepositions + conjunctions ÷ words, from closed lists + suffix
  heuristics, named `idea_density_proxy` with documented recall limits (the
  fragment-rate honesty pattern). Pairs with nominalization rate as a
  verbal-vs-nominal axis the plan half-has. CPIDR software is GPL —
  re-implement from the published rule description.
- **TextTiling-style lexical-cohesion curve.** Hearst block-comparison
  cosine over raw counts as a *feature source*: cohesion-curve mean/var,
  valley depth, topic-shift rate, valley↔paragraph-boundary alignment — a
  countable correlate of the digression axis currently shipped entirely to
  the LLM judge. High token floors (~800+); part corpus descriptor.
- **Entity grids: honest infeasibility.** Full Barzilay–Lapata needs parsing
  + coreference — document as out of scope (hendiadys precedent). The
  degraded parser-free form (content-lemma chain length/gap stats,
  orphan-sentence rate) is essentially the overlap family above plus chain
  statistics; ship those dims, skip the grid.

## 5. De-AI mode additions

- **Surprisal-shape statistics (UID family).** First/second-order deltas of
  the per-token surprisal sequence, longest maximally-flat run,
  extremal-window stats, cross-sentence surprisal Fano — human text is
  information-bursty, LLM text approximates Uniform Information Density too
  well (DivEye, https://arxiv.org/abs/2509.18880 — up to +33% over zero-shot
  detectors; GPT-who, https://arxiv.org/pdf/2310.06202). Pure extensions of
  the existing fitted n-gram LMs. **Honest flag:** published results use
  neural-LM surprisal; n-gram-grade power is a hypothesis for the W8 battery
  to test, not a published fact.
- **Per-model slop packs + model-family fingerprinting.** LLMs are
  distinguishable from word distributions alone at 97% five-way accuracy,
  robust to rewriting (Idiosyncrasies in LLMs, ICML 2025,
  https://arxiv.org/abs/2502.12150). Sam Paech's **slop-forensics is MIT
  incl. generated per-model slop lists** — bundleable, and its method is
  exactly phase-6 Fightin' Words run per model, so handprint can regenerate
  packs itself (https://github.com/sam-paech/slop-forensics). RLHF-drives-
  overuse and per-generation drift papers justify dated versioned per-model
  packs over one generic "AI" pack (https://arxiv.org/pdf/2508.01930).
- **Within-document compression ratio + self-repetition.** Gzip/LZMA CR of
  the doc against itself — strongest simple correlate of homogenization in
  Shaib's diversity-metric comparison (https://arxiv.org/html/2403.00553v2);
  plus 4-gram self-repetition rate. Richness measures *type* diversity, not
  sequence redundancy. Determinism: pin compressor version in fitted state
  (syllable-method invariant). Seeded-compression distance (ZipPy, MIT) is
  partially redundant with `SurprisalLm` — cross-check/eval tier only.
- **Zipf/Heaps/Gini dims in Richness.** Fitted Zipf exponent + mid-rank
  residual, Heaps exponent, Gini over word frequencies — LLM text deviates
  measurably (EMNLP 2025 Findings,
  https://aclanthology.org/2025.findings-emnlp.837/; multilevel-Zipf
  detector 2026). Corpus-level mainly; noisy under ~2k words — floors
  accordingly. (Note: one research pass recommended *against* Zipf/Heaps for
  *authorship*; the AI-detection evidence is newer and specific — adopt as
  de-AI/corpus dims, not author-fingerprint dims.)
- **Syntactic-template repetition — POS-lite, hypothesis-grade.** The
  strongest published templating tell (Shaib et al., EMNLP 2024,
  https://arxiv.org/abs/2407.00211) needs a tagger (Tier-2, deferred). A
  no-tagger skeleton proxy (function words as themselves + suffix-shape
  classes, repeated length-4–6 sequences) is buildable now but unvalidated —
  battery-gated, or wait for Tier 2. The CR/self-repetition dims above are
  the near-equivalent available today.
- **Humanizer-robustness annotations + TH-Bench eval arm.** Perplexity-family
  detectors collapse under paraphrase attacks (Binoculars 66→34 AUROC) while
  style-space features hold at 95–99; sentence-length SD is specifically
  robust (https://arxiv.org/html/2505.14608). Encode per-family
  robust-vs-fragile annotations in docs (surprisal dims are the fragile
  layer — consistent with their canary role) and add a TH-Bench/adversarial-
  paraphrase arm to W8 (https://arxiv.org/pdf/2503.08708).
- **Slop taxonomy + annotated corpus** (Shaib 2025, CC-BY-4.0,
  https://arxiv.org/pdf/2509.19163): first operationalized "slop" definition
  with annotator agreement — an eval/calibration target and a citable
  definition; the component metrics largely already exist in W1.
- **Published human-vs-LLM delta table as calibration data.** Muñoz-Ortiz
  et al. (https://arxiv.org/abs/2308.09067): narrower sentence-length
  distributions, more numbers/symbols/auxiliaries/pronouns, affect skew; the
  only quantified em-dash number found is a ratio (GPT-4.1 ≈ 3.28× human),
  so encode directions + effect sizes as golden-test assertions for the W8
  agent-prose battery — not absolute constants. Extends the PNAS-directions
  assertion already planned.

## 6. Prosody, phonaesthetics, and orthographic voice

- **Prose stress-rhythm family (incl. cadence/clausulae).** CMU-dict stress
  statistics on *prose*: stress density, inter-stress interval mean/var,
  binary-alternation (iambic-tendency) score, and a **sentence-final cadence
  histogram** over the last 5–7 syllables before terminal punctuation (cursus
  planus/tardus/velox bins) — rhythm-only features verify literary
  authorship at F≈0.78 (Lagutina et al.,
  https://fruct.org/publications/volume-28/fruct28/files/Lag.pdf); English
  prose-cadence attribution is essentially unpublished ground. W7 uses the
  dict only for verse (all dims `mark_missing` for prose) and prose syllable
  *counts*; stress *patterns* of prose are untouched. Dict-method-only per
  invariant 0.5; zero new data.
- **Assonance/consonance density + phoneme-class texture.** Windowed
  identical-vowel and non-initial-consonant repetition + plosive/fricative/
  liquid/nasal share (harsh-vs-liquid euphony) — WIT.md ships only
  alliteration and rhyme chains (RhymeDesign formalizes the full sonic-device
  set, https://aclanthology.org/W15-0702.pdf). Same window-detector shape as
  polysyndeton; dict-gated.
- **Eye-dialect / nonstandard-orthography pack.** Casual-speech spellings
  (gonna, kinda, ain't), apostrophe-elision patterns ('em, -in', o'), true
  respellings (wuz, sez) — the Twain vernacular axis W10 currently hand-waves
  at "punct family", which only counts apostrophes. ~150–300 items original
  curation + two pattern rules; split in-dialogue vs narration once §3
  lands. Also a blog-informality axis.
- **Phonological neighborhood density — derived, not licensed.** Don't ship
  CLEARPOND (terms unclear): PND is derivable offline from the CMU dict
  alone (edit-distance-1 over phoneme strings) into a BSD-clean pack via the
  planned `handprint-data` builder. Thinnest voice-evidence of the batch —
  eval-tier first.
- **Number/house-style dims.** Spelled-out vs digit ratio for small numbers
  (AP/Chicago axis), percent style, ordinal style, date formats, unit rate —
  brand-voice mode's most enforceable axes; as-built has only
  `punct:number_rate`/`number_grouped_share`. Closed lists + regex; some
  rules minable from Vale (MIT).
- **Proper-noun/name texture (exploratory).** Capitalized-non-initial rate,
  name-repetition burstiness, honorific/title rate (Mr./Dr./Lord — strong
  period marker: Wodehouse/Austen high). Thin published evidence — flag as
  exploratory; cheap.

## 7. Norm-pack shopping list (all ride W5 `NormDensity`, zero new code)

| Pack | Size | Axis served | License / disposition |
|---|---|---|---|
| Humor (Engelthaler–Hills; Westbury 45k ext.) | 5k / 45k | lexical funniness | BRM/OSF — verify; loader-only fallback |
| Iconicity (Winter 2023) | 14.8k | sound-symbolism/playfulness | open-access BRM — likely bundle w/ attribution, verify OSF |
| Age of Acquisition (Kuperman 2012) | 30k | plain vs erudite diction; LLM tell direction | Brysbaert-group — loader-only pending (add to O5) |
| Word prevalence (Brysbaert 2019) | 62k | psychological obscurity — honest anti-allusion axis (vs corpus-rarity CountPack) | OSF — loader-only pending (O5) |
| Socialness (Diveica 2023) | 8.4k | people- vs thing-focused prose | OSF — loader-only pending (O5) |
| Glasgow Norms: imageability, familiarity, semantic size | 5.5k | vivid diction; "cosmic-scale" nouns (Adams!) | open-access — verify; plausibly bundle |
| Valence + dominance (Warriner VAD — **same file already planned for arousal**) | 14k | affect trajectory (§2 bathos), gonzo valence bimodality | one-line spec addition; CC-BY-NC loader-only as already planned |
| Frequency-band CountPack (Google Books Ngram v3) | large | full Zipf-band profile, generalizes `syn:rare_word_tail_rate` | **CC-BY 3.0 — bundleable** (ready-made cleaned lists exist); SUBTLEX stays loader-only |
| Animacy (VanArsdall & Blunt) | 1.2k | personification detector (§2) | verify; WordNet-hypernym fallback bundleable |

Rejected from the ledger: MRC Psycholinguistic Database (superseded by
Glasgow; murky license), SemD (mostly redundant with senses-per-word),
CLEARPOND (derive PND from CMU instead), vocd-D (HD-D supersedes).

## 8. Eval-side additions (no core code)

- **mStyleDistance (2025), LISA, STEB** refresh the W8/phase-10 external-
  judge roster; LISA's 768 interpretable style dims can cross-check *which*
  attributes moved in the W10 gate without circularity
  (https://arxiv.org/abs/2305.12696).
- **TH-Bench / adversarial-paraphrase arm** (§5) for detector-robustness
  claims.
- **PAN multi-author style-change corpora** — free eval data for the
  intra-document consistency dims (§1); research-use, never shipped.
- **HaHackathon** humor-rating regression (§2); local only.
- **Shaib slop-annotation corpus** (§5) as a de-AI calibration target.

## 9. Explicitly considered and rejected/deferred

- **GLTR rank histograms, GECScore, dependency-distance metrics** — need a
  large neural LM / GEC model / parser; eval-side or never.
- **Watermark-artifact detection** — all practical detection is keyed;
  nothing usable without keys, and artifacts vanish under paraphrase.
- **Zipf/Heaps as authorship dims** — weak attribution evidence (Grieve
  2007); adopted only as de-AI/corpus dims (§5).
- **Full entity grids, TAACO's LSA/word2vec tier** — parser/embedding-bound;
  degraded parser-free forms adopted instead (§4).
- **Sycophantic-opener packs** — no licensed quantified curation exists;
  derive via Fightin' Words from the phase-9 chat-log corpora instead
  (machinery already planned).
- **vocd-D, MRC, SemD, CLEARPOND** — superseded (§7).

## 10. Priority order (if adopted)

Ranked by leverage-per-effort, respecting WIT-plan's in-flight phases (the
`feature/mod.rs` serialization hotspot means new families should batch with
already-planned Family edits where possible):

1. **Norm-pack shopping list** (§7) — zero new feature code, eight new voice
   axes; batch curation + license verification with the W5 pack work. The
   frequency-band CountPack (CC-BY) is the standout: it's the only
   *bundleable* background-frequency source found, upgrading two planned
   dims.
2. **Cohesion + paragraph-shape + reference/givenness module** (§4) — one
   new module, closed lists + existing `Structure`; the connective-position
   and demonstrative-NP dims are the best-evidenced *new* de-AI tells found
   in this round.
3. **Sequence stylometry batch** (§1: punct sequences, WAN, positional MFW,
   opener diversity, HD-D) — parser-free, no data packs, all reuse existing
   machinery; collectively closes the "everything is order-blind" hole.
4. **Dialogue partition + mechanics family** (§3) — one infrastructure piece
   unlocking dialogue share, said-bookisms, scare quotes, and eye-dialect
   splits; directly serves the W10 capstone authors.
5. **De-AI round 2** (§5: per-model slop packs via slop-forensics MIT lists,
   surprisal-shape/UID dims, CR/self-repetition, delta calibration table) —
   slop packs are pure data; UID dims are cheap but hypothesis-grade at
   n-gram scale.
6. **Humor round 2** (§2: bathos peak-end, pause-before-punch, irony
   battery, idiom modification, PMI cohesion drop) — mostly small extensions
   of W2/W4/W5 machinery; bathos and pause-before-punch piggyback on the W2
   clause splitter and should land with or just after it.
7. **Prose rhythm + phonaesthetics** (§6) — novel-ground upside, but
   dict-gated (W7 dependency) and unvalidated for English prose; after W7.
8. **Architectural items** (text distortion, intra-doc consistency, Zeta,
   PPM upgrade) — each needs a design note; none blocks the others.

## Checked and already covered (verified this round — do not re-add)

Richness: MTLD/MATTR/Yule/Simpson as-built; TTR/hapax deliberately excluded.
Burrows/Eder/Cosine Delta, Ruzicka, GI verification, dual calibration,
Fightin' Words (corrected math). Writeprints inventory absorbed (RESEARCH.md
§5). Sentence-length burstiness, surprisal mean/var/autocorr, punct rates and
typography style choices, contraction/ALLCAPS/emoji. All WIT-plan W1–W7
content: Biber Tier-1 + PNAS signature, Hyland, readability, passive/
nominalization/acronym, doc-style and tech-voice packs, punch surprisal +
rhythm + asides, md link/footnote/rhetorical-question, contrast-vocab
actionability, comparison frames, register clash, litotes/hyperbole/precision/
epithet, Mihalcea trio, novel adj-noun bigrams + CountPack, marketing
families, fragments/parataxis/deixis/catalogs/booster chains, verse pack +
line table + syllable-method invariant. Rolling stylometry as localization
(only the variance-as-fingerprint reading is new). LUAR/Wegmann/
StyleDistance/Binoculars/SBERT/RAID eval roster (only the 2025 additions are
new). Structural AI tells incl. "Not X. Not Y. Just Z." (the "it's not about
X, it's about Y" construction is the same detector — now with an RLHF
citation, https://arxiv.org/pdf/2508.01930).
