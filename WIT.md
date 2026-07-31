# Wit — extending corpus-mimic mode to an author's humor (2026-07-31)

Research synthesis for the question: can a Claude Code agent rewrite a story in
Douglas Adams' voice using handprint, and how do we cover his *wit*, not just
his sentence mechanics? Three research passes: computational-humor literature,
LLM author-imitation literature, and the codeindex `prose` crate as corpus
substrate. Companion to `RESEARCH.md`; nothing here changes the phase plan in
`IDEA.md` — it adds a feature family (phase 3 candidate ~9), a data pathway,
and an agent-side pipeline that lives *outside* the crate.

## Verdict

Feasible, in three layers. The evidence splits cleanly:

1. **Mechanics converge, wit doesn't.** GPT-4o imitating literary authors with
   15k-word in-context excerpts matches surface stylometry (sentence length)
   but stays near its generic cluster on deeper features — it *overshoots*
   lexical diversity vs. real Hemingway (47.8 vs 34.6)
   (https://academic.oup.com/dsh/article/40/2/587/8118784). Few-shot imitation
   works for structured genres, fails for voice-heavy prose ("Catch Me If You
   Can", EMNLP 2025 Findings, https://arxiv.org/abs/2509.14543). Expert writers
   found LLM stories fail hardest on originality/surprise (TTCW,
   https://arxiv.org/abs/2309.14556); 20 professional comedians found LLM humor
   "cruise-ship comedy from the 1950s" and kept models only for structure
   (DeepMind FAccT 2024, https://dl.acm.org/doi/fullHtml/10.1145/3630106.3658993).
   The user's hypothesis — agents can't handle wit alone — is well documented.
2. **But wit has measurable, corpus-fittable correlates.** Most of Adams' named
   techniques reduce to *rates of detectable patterns*, and the key incongruity
   signal (punchline surprisal spike) is validated in the literature with
   exactly the LM machinery handprint already plans.
3. **The authorial fingerprint is device-rate calibration.** "Dark & Stormy"
   (Bulwer-Lytton comic fiction, https://arxiv.org/abs/2510.24538) found LLMs
   imitating comic prose **over-fire literary devices** (excess novel
   adjective–noun bigrams, metaphor density) — humans calibrate rates, models
   don't. A corpus-calibrated rate critic is precisely the missing controller.
   No published work statistically humor-fingerprints a specific prose author;
   Pratchett scholarship hand-counts device rates in theses
   (https://researchportal.tuni.fi/en/publications/the-witcraft-of-seeing-things-differently-hyperdetermined-humor-u)
   — precedent that device rates characterize an author, never implemented.

## Layer 1 — `WitProfile` feature family in handprint-core

All of the below satisfy the design invariants (decomposable, fit-only
corpus-relative state, serde-able, no LLM in core). Lexicons are versioned data
packs like the AI-ism lists.

**Tier A — direct extensions of planned features (cheap, validated):**

- **Punchline surprisal ratio + setup entropy.** Xie, Li & Pu (ACL 2021,
  https://arxiv.org/abs/2012.12007): jokes = high entropy during the setup +
  high surprisal at the punchline. handprint's `SurprisalLM` (phase 3 feat 7)
  already computes per-token surprisal; add per-sentence features: final-clause
  surprisal ÷ sentence-body surprisal, per-position entropy, and the *rate of
  sentences with a final spike*. Word-bigram LM (already an option) works
  better than char trigrams for this. Adams' rhythm — long even windup, spike
  at the end — is this feature's exact shape. Caveat: LM surprisal detects
  lexical twists, not syntactic garden paths (Huang et al. 2024,
  https://www.sciencedirect.com/science/article/abs/pii/S0749596X24000135).
- **Punch-delay rhythm proxy**: clause count before a short final clause;
  long-sentence→short-sentence contrast rate (the flat punch after the windup).
  Pure sentence-stats extension.
- **Parenthetical/aside rate + length distribution** (authorial digression;
  Pratchett footnotes). Trivial.

**Tier B — new pattern+lexicon features (statistical-in-Rust, some novel):**

- **Comparison-frame family** — the Adams signature. Extract frames
  ("as X as Y", "like a/the Y", "much the same way that…"), then sub-features:
  frame rate; **negated-vehicle rate** ("…in much the same way that bricks
  *don't*" — negation inside the vehicle clause); **ironic-simile markers**
  (Veale & Hao: the hedge "about as X as Y" flags irony,
  https://link.springer.com/article/10.1007/s11023-010-9211-1); **frozen vs
  creative ratio** (cliché-simile lexicon; novel vehicle = creative,
  https://arxiv.org/pdf/1511.01756). Frame extraction is regex-grade;
  Niculae & DNM (EMNLP 2014, https://aclanthology.org/D14-1215.pdf) is the
  template for the figurative-comparison classifier.
- **Register clash within sentence** (bureaucratic language for cosmic events;
  Wodehouse slang-in-formal-frame). Ship a per-word formality lexicon (from
  Pavlick & Tetreault 2016 data,
  https://huggingface.co/datasets/osyvokon/pavlick-formality-scores, or
  Heylighen–Dewaele's POS-proportion F-score); feature = within-sentence
  formality variance/bimodality, span-attributed to the clashing words. No one
  has published this as a humor feature — ingredients validated, combination
  novel.
- **Understatement/litotes rate**: negation patterns ("not entirely", "not
  un-", "hardly", "rather") + minimizer-next-to-extreme-magnitude ("Space is
  big"). Partial recall, fine for rates.
- **Absurd numeric-precision rate**: precise numerals/over-precise
  measurements adjacent to abstract or cosmic nouns ("roughly once every ten
  million years"). No prior art; easy.
- **Hyperbole lexicon rate** (Troiano & Strapparava EMNLP 2018 features,
  https://aclanthology.org/D18-1367/).
- **Transferred-epithet rate** (Wodehouse: "a moody forkful"): mental-state
  adjective + concrete-inanimate noun, via free emotion-lexicon + concreteness
  norms. Novel feature, noisy but rate-usable.
- **Mihalcea–Strapparava trio** (validated one-liner discriminators,
  https://www.semanticscholar.org/paper/d14e27b336bd2aad2839342a7a9f4067d89b55af):
  alliteration/rhyme-chain rate, antonymy-within-sentence rate (WordNet antonym
  table as data), word-sense **ambiguity density** (senses-per-word table).
- **Novel adjective–noun bigram rate** (the Dark & Stormy over-firing
  fingerprint): reference-corpus bigram table vs background counts.
- **Callback/running-gag profile**: rare-n-gram recurrence at long distance
  across the document. Measurable; document-level, so mostly a corpus
  descriptor + drift check rather than a per-draft gate.

**Tier C — out of core:** simile tenor↔vehicle semantic distance needs
embeddings (feature-gated static vectors keep the data-only constraint, or
corpus PMI as a weak proxy); true bathos/paraprosdokian, non-sequitur, and
garden-path detection need an LLM judge → belongs in the agent-side wit critic
(Layer 3), not the crate. THInC (https://arxiv.org/abs/2409.01232) validates
the per-mechanism interpretable-classifier architecture generally.

**Two-sided bands are mandatory (anti-caricature).** Parody drift is
documented: models *amplify* salient persona markers (CoMPosT's
individuation-vs-exaggeration axes, https://arxiv.org/abs/2310.11501;
https://arxiv.org/pdf/2507.00657). handprint's `target_band` is already an
interval — make wit-family findings fire in **both directions**, including
"more Adams than Adams: negated-simile rate above the 95th percentile of
per-chapter corpus values — reduce". This falls out of the existing critique
contract nearly for free and is the direct defense against the over-firing
failure mode.

## Layer 2 — Corpus ingestion & move-indexed retrieval (codeindex-prose)

`codeindex-prose` (~630 lines, landed days ago, well-tested) is a
retrieval-grade chunker/ingester for **md/txt only** — recursive
paragraph→sentence→word chunking (default 1,200 bytes, 150 overlap), SHA-256 +
whitespace-normalized identity, `FullSource`/`Body` representations, local ONNX
embeddings into SQLite, hybrid semantic+lexical search via `codeindex search`.
It does **zero** linguistic/stylometric analysis, and chapters only exist if
the source is markdown with headings. Division of labor:

- **Ingestion**: epub → markdown upstream (pandoc; chapter headings survive as
  section scope), then `codeindex add`. handprint fits its `Reference` on the
  same normalized text independently — the two tools share a corpus directory,
  not code.
- **Retrieval**: the agent's exemplar source. The mimicry research says
  demonstration *selection* dominates count, structural/rhetorical similarity
  beats topical similarity (https://arxiv.org/html/2601.20803v1,
  "Writing Like the Best" https://arxiv.org/pdf/2505.18859), and random
  few-shot conveys register but not move-level technique.
- **The novel piece — a comic-move index**: one-time LLM labeling pass over
  the ingested chunks tagging each with (a) comic device(s) from the
  Adams inventory (subverted simile, bureaucratic-register clash, absurd
  precision, personification, digression, deadpan escalation…), (b) scene
  function (character intro, technology going wrong, narratorial aside…).
  Store as a sidecar (or markdown annotations re-ingested as a parallel
  corpus) so the agent retrieves exemplars **by mechanism, not topic**:
  "give me 3 passages where Adams introduces a character" rather than
  "passages about spaceships". Nothing published does move-indexed pastiche
  retrieval — defensible and novel. Guard: retrieved passages get parroted —
  instruct mechanism-imitation and let handprint's n-gram overlap catch
  verbatim leakage.

## Layer 3 — Agent pipeline (two critics, staged rewrite)

Evidence: decomposition beats end-to-end for humor (CLoT CVPR 2024
https://github.com/sail-sg/CLoT; CLoST ICLR 2025
https://arxiv.org/abs/2410.10370; the pun-pipeline lineage), and comedians
validated LLMs as structure-then-inject tools.

1. **Plain draft** — content-preserving retelling, no style pressure.
2. **Comic plan per beat** — pick a device from the corpus-derived inventory
   (the `WitProfile` fit tells you the *empirical rates* to hit, so the plan
   places devices at Adams-calibrated density, not maximum density); a
   CLoT-style remote-association step ("what distant bureaucratic/domestic
   concept can this cosmic event be yoked to?"); planned punchline written as
   a one-liner before prose.
3. **Render** with 2–4 move-matched exemplars per scene from Layer 2.
4. **Polish loop against two critics**:
   - **handprint** (`critique --json`, corpus-mimic mode + wit family):
     mechanics and device-rate calibration, two-sided bands, canary/drift
     guards as designed.
   - **Wit critic** (LLM judge, agent-side — not in the crate): GTVH-structured
     per HumorRank (https://arxiv.org/html/2604.19786v1) — per paragraph:
     intended mechanism present? delivery (punchline position, escalation,
     conciseness)? failure flags (cliché, overexplanation, both-meanings-stated
     "lazy generation")? Judge **pairwise against gold Adams anchors**, never
     absolute — TTCW showed LLM judges don't track experts on creativity;
     pairwise+anchored is the stable protocol (cross-judge τ=0.889 in
     HumorRank).
5. **Acceptance gate is an ensemble** (each alone is Goodhart-able):
   handprint `p_same_author` + wit-critic pairwise win-rate vs anchors +
   optionally the phase-10 external style embeddings. Sequence the loop so
   mechanics converge early and later iterations are wit-critic-driven —
   otherwise the agent polishes rhythm forever while the jokes stay flat.

## Pilot experiment (cheap, falsifies early)

Corpus: 5 Hitchhiker's books + 2 Dirk Gently ≈ 600k+ words — far above the 5k
reliability floor; per-chapter docs give the within-author variance needed for
two-sided bands and same-author calibration. Personal-use fitting is fine;
never ship the fitted Adams pack (in-copyright text; profiles-not-corpora rule
still applies but keep this one local).

Arms: (a) zero-shot "rewrite in Adams' voice"; (b) + handprint mechanics loop;
(c) + wit family + move-indexed exemplars + comic plan + wit critic. Measure:
handprint distance, device-rate deltas vs corpus bands (over/under-firing),
blinded pairwise LLM judge vs gold anchors, and a human read. Prediction from
the literature: (a) reads as generic pastiche with over-fired tics; (b) fixes
rhythm, not funny; (c) is the test of the thesis. This slots in as a variant of
the phase-10 headline experiment.

## Author battery — widening beyond Adams

Second research round (same date): Pratchett, Bukowski, Hunter S. Thompson,
Shakespeare as maximally diverse stress tests, so the feature family is
voice-agnostic rather than tuned to British comic SF. Key structural finding:
**Bukowski and Thompson are a designed-in contrast pair** — both paratactic,
profane, first-person, low-subordination writers, yet opposite poles on
sentence-length variance, intensifier density, punctuation exuberance
(CB: comma/semicolon scarcity; HST: em-dash/ellipsis/ALL-CAPS), and formal
vocabulary. The battery can't cheat with a single "informal register" axis.
No published stylometric profile exists for either author — novel ground again.

### Bukowski / dirty realism (all statistical-in-Rust)

Critic-named markers (Hakobyan 2014,
https://journals.ysu.am/foreign-lang/en/article/view/vol18_no2_2014_pp162_170;
https://literariness.org/2020/07/11/analysis-of-charles-bukowskis-poems/;
dirty-realism register: short declaratives, adverb scarcity, unadorned lexicon):

- **Fragment rate** — no-finite-verb heuristic (closed-class modal/aux list +
  inflection rules; ACL 2015 taxonomy https://aclanthology.org/P15-2099.pdf).
  Noisy per-sentence, reliable as a corpus-level rate — which is all a profile
  needs.
- **Parataxis**: subordinator:coordinator ratio (closed-class counting),
  sentence-initial And/But rate.
- **-ly adverb rate** (with small false-positive stoplist) — the scarcity is
  the signature.
- **Profanity rate + severity mix** — LDNOOBW list
  (https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words),
  dsojevic/profanity-list (severity-tagged).
- **Concreteness density** — Brysbaert/Warriner/Kuperman norms, ~37k lemmas,
  1–5 scale (https://link.springer.com/article/10.3758/s13428-013-0403-5;
  cleanest packaging via NoRaRe/CLDF). Bukowski: concrete ≫ abstract.
- **Anti-allusion vocabulary**: rare/"literary" word-tail rate vs background.
- **Person deixis**: 1sg/2nd/1pl per 1k (LIWC-style closed list).
- **Line geometry (poetry mode)**: line-length variance, 1-word-line rate,
  end-stop vs enjambed rate, lowercase-line-initial rate. Requires a
  **line-preserving text mode** in the tokenizer — prose normalization
  destroys this signal (architectural note for phase 1).

### Thompson / gonzo (all statistical-in-Rust except digression/satire)

Named by the stylistics literature (Mosser, "What's Gonzo about Gonzo
Journalism?", https://ialjs.org/wp-content/uploads/2012/06/085-090_WhatsGonzoMosser.pdf;
Wills, https://huntersthompson.substack.com/p/what-is-gonzo and *High White
Notes*; Auburn thesis on outlaw rhetoric):

- **Signature-lexis rate** ("swine", "atavistic", "doomed", "savage"…) — and
  note: author signature lexicons should be **auto-derived via the phase-6
  Fightin' Words contrast** (high log-odds tokens vs background), with the
  hand-curated list as a golden test, not the mechanism.
- **Intensifier rate + booster-chain length** (consecutive-booster runs) —
  VADER `BOOSTER_DICT` (~70 entries, MIT,
  https://github.com/cjhutto/vaderSentiment); taxonomy per Taboada et al.
  https://aclanthology.org/J11-2001.pdf.
- **ALL-CAPS exclamation rate** (excluding acronyms), `!` rate, interjection
  lexicon; ellipsis and em-dash per 1k (already in `PunctTypography`).
- **Paranoid-certainty phrase rate** — small custom epistemic pack ("of
  course", "no doubt", "sure as hell"…), ~40 phrases.
- **Catalog rhythm**: polysyndeton/asyndeton window detectors (validated,
  ~100% P/R in-sentence per the rhetorical-figures survey
  https://arxiv.org/pdf/2406.16674), comma-chain length distribution.
- **Escalation rhythm**: per-sentence syllable-count sequence stats (slope,
  variance, peak-to-mean, autocorrelation) — directly motivated by Wills'
  wave-cadence analysis of Thompson imitating Gatsby's rhythm.
- **Register clash, lexical proxy**: Latinate-suffix vocabulary × profanity
  co-occurrence in a clause window ("atavistic swine") — same formality-clash
  feature as Adams, different pole.
- Drug lexicon (small custom pack). Digression/satire/fact-blending →
  LLM-judge tier only.

### Shakespeare / verse & rhetorical figures

One shared **verse data pack** (CMU pronouncing dict ~134k entries, public
domain + suffix-based stress guesser for OOV + finite early-modern elision
rules: "heaven"=1 syl, syllabic "-èd", "o'er") unlocks the four highest-value
features. Dictionary scansion is 85–90% per-syllable accurate (Prosodic,
https://github.com/quadrismegistus/prosodic, claims 97.5% foot parsing;
ZeuScansion https://aclanthology.org/W13-1803.pdf 86.8%) — per-line errors
wash out in aggregates, which is exactly how Plecháč attributes authorship
from stress profiles (https://arxiv.org/pdf/1911.05652,
https://karolinum.cz/en/books/plechac-versification-and-authorship-attribution-25198).

- **Feminine-ending rate** — the single best-attested verse discriminator in
  the field: Spedding 1850 split Henry VIII on it (Shakespeare 28–40% vs
  Fletcher 50–77%, zero overlap; http://gabrielegan.com/publications/Egan2018c.htm),
  confirmed by Plecháč 2019 with MFW + rhythmic patterns.
- **Stress-profile vector** (stress frequency per metrical position),
  **iambic-conformity / constraint-violation rate**, mid-line pause and
  enjambment proxies (punctuation-position histograms, line-final function
  words).
- **Verse-vs-prose segmentation**: take it from markup (Folger TEI marks it
  explicitly), heuristic fallback with a confidence flag — lineation in
  original playbooks is genuinely ambiguous, don't solve it algorithmically.
- **Rhetorical figure rates**: chiasmus (Dubremetz & Nivre, F1≈0.78 with pure
  pattern features,
  https://www.frontiersin.org/journals/digital-humanities/articles/10.3389/fdigh.2018.00010/full),
  anaphora/epistrophe (F1≈0.5 per-instance — fine as rate features),
  polysyndeton/asyndeton (trivial), antithesis (needs the WordNet antonym pack
  already planned for the Mihalcea trio; noisy but usable). Hendiadys has
  **no published detector** — ship the honest proxy ("low-PMI doublet rate":
  Noun-and-Noun coordinations with low corpus co-occurrence), don't overclaim.
  Vocative/apostrophe via "O + NP" pattern rules.
- **Archaism morphology**: thou/thee/thy vs you/ye ratio, -eth vs -es
  (doth/hath vs does/has), -est rates — closed-list counting, well-studied
  sociolinguistic variables (Busse; Walker).
- **Coinage/OOV-vs-background rate** — only meaningful against a *period*
  background lexicon (EEBO-TCP phase 1, public domain) and only on
  spelling-normalized text, else it measures orthography. The "Shakespeare
  coined 1,700 words" figure is an OED-citation-bias myth
  (https://oed.hertford.ox.ac.uk/wp-content/uploads/2019/07/Brewer_2012.pdf).
- **Skip in core**: pun detection (SemEval-2017 Task 7 location/interpretation
  scores collapse to F≈0.1, and early-modern homophones defeat CMU dict) and
  functional-shift rate (needs a POS tagger; risky on EME) — LLM-judge tier.
- **Corpora**: modernized-spelling editions sidestep normalization — Folger
  TEI XML (best structure: verse/prose markup, speakers; **non-commercial
  license** — fine locally, never a shipped pack), MIT/Moby (public domain,
  weaker structure), EEBO-TCP as period background. Original-spelling texts
  poison every lexicon feature without VARD-style normalization (~94% P / 74%
  R; architecture re-implementable as a variant table + rewrite rules if ever
  needed).

### Coverage matrix

| Feature family | Adams | Pratchett | Bukowski | Thompson | Shakespeare |
|---|---|---|---|---|---|
| Punchline surprisal + punch rhythm | ●● | ●● | ○ (flat deadpan = low band, still signal) | ● (escalation variant) | ○ |
| Comparison frames (negated/ironic/creative) | ●● | ● | ○ | ● | ● (conceits) |
| Register clash (formality variance) | ●● bureaucratic↔cosmic | ●● | ○ uniformly low — two-sided band catches drift *up* | ●● Latinate↔profane | ● high↔low by speaker |
| Device-rate lexicons (litotes, hyperbole, precision, epithet) | ●● | ●● | ○ | ●● hyperbole pole | ● |
| Parataxis / fragments / adverb scarcity | ○ | ○ | ●● | ● (long-breathless variant) | ○ |
| Profanity + concreteness + deixis | ○ | ○ | ●● | ●● | ○ |
| Intensifier chains / caps / catalogs | ○ | ● | ○ | ●● | ● (polysyndeton) |
| Verse pack (meter, stress, feminine endings, line geometry) | — | — | ● (line geometry, poetry) | — | ●● |
| Figure rates (anaphora, chiasmus, antithesis) | ○ | ● | ● (anaphora) | ○ | ●● |
| Archaism / period-OOV | — | — | — | — | ●● |

●● = signature axis, ● = secondary, ○ = present as a *low* band (absence is
also a fingerprint — that's what two-sided bands buy), — = N/A.

Every author's ○ cells matter as much as the ●● cells: Bukowski is
characterized by what he *doesn't* do (adverbs, subordination, intensifiers),
which only a calibrated-band critic can enforce — a "write minimally" prompt
gives an agent no target rate.

### Battery acceptance criteria

Fit all five references (per-chapter/per-poem docs for within-author
variance), then require: (a) **separation** — cross-author attribution sanity
via existing compare/rank; (b) **interpretability** — each author's top-|z|
dims match the critic-named traits above (golden test per author); (c)
**anti-caricature** — a deliberately over-fired pastiche fixture (booster-
stuffed fake Thompson, adverb-free-but-intensifier-laden fake Bukowski) trips
the two-sided upper bands. All corpora except Shakespeare are in-copyright:
local fitting only, never shipped packs (same rule as the Adams pilot).

## Nonfiction registers — technical writing, blogs, marketing

Third research round (same date). These registers close the loop back to
handprint's original use cases (HN voice, de-AI mode, agent prose) and, unlike
fiction, come with **validated academic frameworks that are already
lexicon/count-shaped** — the cheapest, highest-leverage additions in this
whole document, because most double as de-AI features.

### Biber multidimensional analysis (the register backbone)

Biber 1988: 67 lexico-grammatical features factor into interpretable
dimensions — D1 involved↔informational, D2 narrative, D3 explicit reference,
D4 overt persuasion, D5 abstract/impersonal, D6 on-line elaboration
(dimension/feature tables: https://corpus-stats.lancs.ac.uk/docs/multidimensional.pdf).
Feasibility splits into two tiers:

- **Tier 1, no tagger (~half the features)**: closed-class lists (private/
  public/suasive verbs, modals by class, conjuncts, hedges, amplifiers,
  emphatics, discourse particles, pronoun classes, contractions, negation),
  suffix-rule nominalizations (-tion/-ment/-ness/-ity), TTR, word/sentence
  length. Covers D1/D4 and much of D2/D3/D5. Word lists extractable from open
  implementations: MAT (https://sites.google.com/site/multidimensionaltagger/about),
  biberpy (https://github.com/ssharoff/biberpy — **GPL: re-curate the lists
  from published feature definitions, don't vendor the files**), biberplus
  (https://github.com/davidjurgens/biberplus, 96 features).
- **Tier 2, needs POS tags** (tense/aspect, passives, participial clauses,
  that-deletion, attributive adjectives): biberpy demonstrates a parser-free
  frequency-list-tagging approximation works; a feature-gated lightweight
  perceptron tagger is the clean path later. Not needed for v1.

**Why it matters for de-AI**: Reinhart et al., PNAS 2025
(https://www.pnas.org/doi/10.1073/pnas.2422455122, HAP-E corpus) found the
LLM signature *in Biber features, consistent across registers*: 2–5× more
present-participial clauses, 1.5–2× nominalizations, more phrasal
coordination and long words; ~half the agentless passives, far fewer
contractions, first-person pronouns, pro-verb *do*. Several are Tier-1
countable now (nominalization rate, contraction rate, 1sg rate), and the
participial-clause finding has a no-parser regex proxy (", VERBing …"
sentence-final analysis clauses).

### Hyland metadiscourse (ready-made interactional family)

Hyland 2005 taxonomy: interactive (transitions, frame markers, endophorics,
evidentials, code glosses "i.e./e.g./namely") + interactional (**hedges,
boosters**, attitude markers, engagement markers "note that/consider",
self-mentions). ~300–500 items, all word/phrase lookups — re-curate the list
(items are unprotectable words; the appendix compilation is what's copyrighted;
reprinted category tables:
https://www.researchgate.net/figure/Hylands-2005-taxonomy-of-metadiscourse_tbl1_322164943).
Polysemy noise (*may*, *about*) is systematic and cancels in comparisons.
**Double duty**: humans hedge ~40% more than LLMs in advice text
(https://arxiv.org/pdf/2604.22143) — hedge/booster ratio is both a voice axis
and an AI-tell.

### Tech-blogger voice markers (all statistical except quality judgments)

Named habits from commentary on Levine/Graham/Luu/Evans/Gwern/patio11 map to
counts: **parenthetical density** (chars-in-parens ratio, parens/sentence —
Levine's signature), **footnote density** (markers per 1k words — Levine,
Gwern), hypothetical-dialogue frames ("you might say…"), conversational
plainness (= Biber D1 composite: contractions + 1sg + short declaratives +
low Latinate rate — Graham), rhetorical-question and 2nd-person rates
(Evans, Spolsky), exclamation/enthusiasm lexicon (Evans), **discourse-pivot
openers** ("Anyway,", "OK, so", "Look,", "Here's the thing"), markdown
extensions to the existing structure family: **link density, inline-code-span
rate, heading/list rates** (Luu's minimal formatting vs Gwern's heavy
apparatus is a clean contrast pair). Anecdote/analogy/humor *quality* →
LLM-judge tier; their *rates* (1sg+past-tense openings, "it's like/think of
X as/imagine" frames) are countable.

### Doc-style + readability (Vale-derived, essentially free)

Vale's MIT-licensed style packs (Microsoft, Google, write-good, proselint —
https://github.com/errata-ai) are pure YAML regex/wordlist rules, directly
minable: weasel-word rate, wordy-phrase rate ("utilize→use" maps), cliché
lists, sentence-initial "So", adverb flags. Note the register signal in the
packs themselves: Microsoft *prefers* contractions. Add readability grades
(Flesch-Kincaid/Fog/SMOG via vowel-group syllables; Coleman-Liau/ARI are
character-only), **nominalization rate** (suffix rules — also a top LLM tell),
passive-voice regex heuristic (be-form + participle; known adjectival-
participle false positives are consistent bias, fine for per-author deltas —
PassivePy documents the tradeoff), acronym density.

### Marketing families

Grounded in Leech 1966 (*English in Advertising* — imperative rate >1-in-4
major clauses in ads, negated imperatives near-absent; the favorite-adjective
list *new/good/best/free/fresh/delicious/sure*; disjunctive verbless
"block language"), Koteyko's Biber-style MDA of press ads
(https://doi.org/10.1177/0075424215605566), the dark-patterns literature for
*vetted* urgency/scarcity phrasings (Mathur et al., 11k shopping sites,
https://arxiv.org/pdf/1907.07032), and the clickbait literature (Blom &
Hansen forward-reference deixis; Chakraborty "Stop Clickbait"; Potthast's
215-feature set; **Webis-Clickbait-17** is a free graded corpus to fit a
headline-slop dimension against, https://webis.de/data/webis-clickbait-17.html).

Eight families, all statistical: **directive** (imperative rate via
sentence-initial base-verb list, CTA phrases, 2nd-person density),
**evaluation** (ad-adjective density, comparative/superlative morphology,
booster:hedge ratio, universal quantifiers), **pressure** (urgency/scarcity
regex with numeric slots "only \d+ left", social proof "trusted by/join N+",
risk reversal "money-back/cancel anytime", benefit connectives "so you
can/which means"), **headline** (number-initial, listicle "N ways", question
form, demonstrative deixis "This is why", curiosity phrases, punctuation
excess), **affect** (arousal density — the marketing analog of concreteness
density; sensory-modality densities for product copy), **fragment/rhythm**
(Leech's disjunctive syntax = the fragment detector from the Bukowski round),
**AI-slop** (below), **tone axes** — with an honest boundary: of NN/g's four
tone dimensions, formal↔casual and enthusiastic↔matter-of-fact are computable
composites; funny↔serious and respectful↔irreverent stay LLM-judge tier, and
saying so is itself a defensible design statement.

**Marketing AI-slop**: consensus core across curated lists (*unlock, elevate,
seamless, game-changer, leverage, empower, transformative, "in today's
fast-paced world", "look no further"*) plus structural tells: emoji-prefixed
bullets, bold-lead-in bullets ("**Fast:** …"), "Not X. Not Y. Just Z.",
negative parallelism, triplet rate, colon-headline rate. No academic study of
marketing-specific slop exists — **applying the Kobak excess-vocabulary
method to a marketing corpus would be novel** and is exactly the phase-6
Fightin' Words machinery. Wikipedia's Signs-of-AI-writing page is CC-BY-SA —
a derived lexicon pack is legal with attribution + share-alike on that file.

### Licensing ledger (bundle vs load vs re-curate)

| Resource | License | Disposition |
|---|---|---|
| VADER lexicon + booster dict | MIT | bundle |
| Lancaster sensorimotor norms | CC-BY (verify on OSF) | bundle w/ attribution |
| Vale style packs | MIT | mine into packs |
| CMU pronouncing dict | BSD-like/public domain | bundle (verse+syllables) |
| Wikipedia signs-of-AI | CC-BY-SA | derived pack, share-alike |
| Kobak excess-vocab list | paper supplementary | seed lexicon, cite |
| LDNOOBW profanity | permissive | bundle |
| Brysbaert concreteness, Warriner VAD | BRM supplementary, likely CC-BY-NC | **verify**; NC conflicts with commercial use — plan loader fallback |
| NRC EmoLex / NRC-VAD | research-only, **no redistribution** | loader only, never bundle |
| Hyland appendix, Leech lists | book compilations | re-curate (items are unprotectable words) |
| biberpy word lists | GPL-3.0 | re-curate from published feature definitions, don't vendor |
| Folger Shakespeare TEI | non-commercial | local battery only |

### Register coverage matrix (companion to the author matrix)

| Family | Tech/blog voice | Marketing | de-AI mode |
|---|---|---|---|
| Biber Tier-1 dims (D1/D4 composites) | ●● plainness axis | ●● persuasion axis | ●● (PNAS LLM signature) |
| Hyland hedges/boosters/engagement | ●● | ●● booster-heavy pole | ●● (humans hedge more) |
| Doc-style (weasel/wordy/readability/nominalization) | ●● | ● | ●● (nominalization tell) |
| Markdown apparatus (links/code/footnotes/parentheticals) | ●● | ● (emoji bullets) | ●● (bullet/bold tells) |
| Directive/pressure/headline families | ○ | ●● | ● (formulaic CTA slop) |
| Affect (arousal/sensory density) | ○ | ●● | ● |
| Fragment + discourse pivots | ● | ●● | ● (LLMs under-fragment) |
| Fiction wit family (surprisal spikes, comparison frames) | ● (humor-in-tech-writing) | ○ | ● |

## Priority order (if adopted)

Reordered after the register round — the nonfiction families jump the queue
because they serve *every* mode (voice mimicry, de-AI, brand voice) at
lexicon-lookup cost:

1. **Hyland metadiscourse + Biber Tier-1 + doc-style/Vale-derived packs** —
   ready-made lists, no new machinery, double as de-AI features with
   published LLM-vs-human deltas (PNAS 2025, hedging studies).
2. Punchline-surprisal + punch-rhythm features (extends planned SurprisalLM —
   highest-validated wit signal, lowest cost).
3. **Two-sided band findings in the critique contract** (small change, kills
   the caricature failure mode for every register).
4. Comparison-frame family + register-clash/formality-variance (the
   fiction-signature detectors; formality lexicon doubles for NN/g tone axes).
5. Device-rate lexicon features (litotes, hyperbole, precision, epithet,
   alliteration/antonymy/ambiguity, adj-noun novelty) + marketing
   directive/pressure/headline/affect families (same pattern+lexicon shape).
6. Syntax-texture family (fragments, parataxis, deixis, catalogs, adverb
   scarcity, intensifier chains) — shared across Bukowski/Thompson/marketing.
7. Verse pack (CMU dict + elision rules: meter, stress profile, feminine
   endings, line geometry) — feature-gated; needs the line-preserving
   tokenizer mode.
8. Move-index tooling + agent pipeline prompts (outside the crate; can start
   as a skill/prompt protocol as soon as codeindex ingestion works).
