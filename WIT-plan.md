# `handprint` — WIT implementation plan

Implementation plan for everything in `WIT.md` (voice/wit feature families, register families, data packs, the author & register batteries, and the Gutenberg voice-transfer capstone). Companion to `IDEA.md` — phases here are numbered **W1–W10** and assume IDEA phases 1–9 as built. Where `IDEA.md` and the shipped code disagree, **the as-built code wins**; this plan is written against the actual `Feature`/`FittedFeature` traits (`crates/handprint-core/src/feature/mod.rs`), the `feature_kinds!` registry (mod.rs:375), the as-built critique contract (`critique.rs`, `CONTRACT_VERSION = "1"`), and the as-built text layer (`src/text/`).

Priority order is `WIT.md` §"Priority order" verbatim: register packs first (they serve every mode at lexicon-lookup cost), punchline surprisal second, band/contract hardening third, then frames/clash, device+marketing, syntax texture, verse, and the out-of-crate agent pipeline last. The capstone experiment (W10) is the plan's exit criterion.

Because parallel work is in flight, every phase carries a **Touches existing files** line. The one serialization hotspot is `feature/mod.rs`: W1, W2, W4, W6, and W7 each add a `Family` (W7 also a `Unit`) variant plus `feature_kinds!` registration — those edits must land sequentially. The CLI hotspot (`config.rs`/`main.rs`/`commands.rs`) is shared by W1, W3, and W10. Genuinely add-only modules: `biber.rs`, `readability.rs`, `frames.rs`, `formality.rs`, `device.rs`, `norms.rs`, `syntax.rs`, `verse.rs`, `gutenberg.rs`, and everything under `eval/`.

---

## 0. Ground rules — deltas to IDEA.md §0

All IDEA.md invariants stand (decomposable scores, data-only fitted state, corpus-relative logic only in `fit`, versioned data packs, no LLM in core). New invariants forced by the as-built code:

1. **New-family checklist is mandatory** (from the architecture map): new module in `feature/`, spec + fitted structs (fitted: serde, data-only, `pub const FAMILY: Family`), register in `feature_kinds!`, add `Family` variant + `as_str()` + `token_floors()`, choose `Unit`s (the unit's `noise_floor()` becomes the z-scale floor — pick deliberately), intern dim names in `fit` in a stable order, `mark_missing` never placeholder zeros, `note_span`/`add_at` for anything a finding should point at, `.rollup()` on aggregates, re-export from `feature/mod.rs`, `FeatureArg` + builder wiring in `handprint-cli` (`main.rs` + `commands.rs` ~line 89).
2. **Canary safety.** `DimInfo.family` is per-dim; the critique canary skips findings by dim family. Any new dim intended to be *reported* must never carry `Family::Surprisal` (the default canary) — see W2, where punch dims emitted by `FittedSurprisal` are deliberately tagged with a different family. (Verified against as-built `critique.rs`: skip/trip logic keys off per-`DimInfo.family`, so mixed-family dims inside one fitted feature work.)
3. **Serde compatibility for extended fitted types.** New dims on existing fitted structs (`FittedSurprisal`, `FittedSentence`) are gated by spec flags with `#[serde(default)]` so old `Reference` JSON still loads; the new dims appear only after a refit. Pre-1.0 we do not migrate artifacts, we refit. Where old data would otherwise be *silently reinterpreted* rather than merely lack dims, the fitted struct carries a serde-defaulted version field validated at load — see W3.2 and W7.2.
4. **License rollup on `Reference`.** A reference fitted with a loader-only (non-redistributable) norm pack embeds that pack's table in its fitted state and is itself non-redistributable. `Provenance` gains `licenses: Vec<String>` (rollup of every pack consumed at fit) and the CLI warns on `fit` when any entry is non-redistributable. Share-alike (CC-BY-SA) derived packs live in their **own** pack files so the obligation never infects sibling packs.
5. **Build-independent transform.** `transform` output for a given fitted reference must not depend on which cargo features compiled the binary (IDEA invariant 3: pure and deterministic given the fitted state). Anything computed differently under a feature gate (the W7 CMU syllable dictionary) must be recorded in the fitted state, and a build that cannot honor the recorded method must **fail loudly**, never silently fall back — see W7.2.
6. **Three pack formats, all data-only JSON.** Term/phrase packs use the existing `LexiconPack` serde format unchanged. Weighted lexicons (formality scores, concreteness, arousal, sensorimotor, VADER boosters) get a new `NormPack` mirroring `LexiconPack`'s provenance envelope (see W1). Background frequency tables (novel-bigram backgrounds, rare-word tails) get a `CountPack` with the same envelope and `entries: Vec<(String, u64)>` (see W5). Grammatical closed-class lists that do not decay (Biber verb/modal/conjunct classes, subordinators, deixis lists) follow the as-built `FUNCTION_WORDS` precedent: consts in code with a `*_VERSION` string.

---

## Phase W1 — Hyland metadiscourse, Biber Tier-1, doc-style/readability packs (+ pack plumbing, contract open-set note)

The highest-leverage phase: ready-made lists, no new machinery beyond two small feature modules, and most dims double as de-AI features with published LLM-vs-human deltas (PNAS 2025 Biber signature; hedging gap).

**Touches existing files:** `feature/mod.rs` (`Family::Register` + registry), `lexicon.rs` (`category_findings`), `reference/mod.rs` (`Provenance.licenses`), `config.rs`/`main.rs`/`commands.rs` (pack plumbing), `critique.rs` module docs (open-set note). Add-only: `biber.rs`, `readability.rs`, pack JSON files.

**Deliverables**

1. **Contract documentation change (no shape change): open value sets.** Because this phase ships the first new family strings, the note must land here, first. Document in the contract (critique.rs module docs + book) that `Finding.family` and the keys of `doc.confidence` are an **open set** — consumers must tolerate unknown family strings. `Family` is `#[non_exhaustive]` so this is Rust-source-compatible and JSON-additive; the only breakage risk is an external strict deserializer, which the contract now explicitly disclaims. Same statement for `Fix.kind` (already a free `String`). `CONTRACT_VERSION` stays `"1"`.
2. **CLI pack plumbing (prerequisite, small).** `handprint.toml` `[[pack]]` entries are currently parsed but never applied (`config.rs` declaration only). Wire them: `fit` loads each declared pack (path → `LexiconPack` via serde), constructs one `LexiconFeature::new(pack)` per entry, appends to the pipeline. Add `fit --pack <path>` (repeatable) as the flag-form equivalent. Version pinning: bail if the loaded pack's `qualified_name()` ≠ the declared `name@version`.
3. **`LexiconFeature` category-findings flag.** Hyland's finding-worthy dims are the *category* rates (hedges, boosters…), not per-term rates. Add `category_findings: bool` (serde default `false`) to `LexiconFeature`: when set, `lex:{pack}:cat:{category}` dims are **not** marked `.rollup()` (so they can become findings) and `per_term` defaults off. Existing packs unaffected.
4. **`hyland` pack** — re-curated Hyland 2005 taxonomy (~300–500 items; the items are unprotectable words, the appendix compilation is what's copyrighted). Categories: `hedges, boosters, attitude, engagement, self_mention, transitions, frame_markers, endophorics, evidentials, code_glosses`. Phrase entries use the existing pattern language (`{a|b}` alternatives, `*` gaps). Fit as `LexiconFeature { category_findings: true }` → dims `lex:hyland:cat:hedges` etc., `Unit::PerThousandTokens`.
5. **`BiberTier1` feature family** — new module `feature/biber.rs`, no tagger (Tier 2 explicitly deferred per WIT.md):
   - `pub struct BiberTier1 { participial_proxy: bool }` / `pub struct FittedBiber { dims: Vec<DimInfo>, ... }` with `pub const FAMILY: Family = Family::Register` (new variant, see below). Trivial fit (fixed dim set, closed-class consts `BIBER_LISTS_VERSION = "2026.08"`, **re-curated from published feature definitions — never vendored from GPL biberpy**).
   - Dims (all `PerThousandTokens` unless noted): `biber:private_verb_rate`, `biber:public_verb_rate`, `biber:suasive_verb_rate`, `biber:modal:{possibility,necessity,prediction}`, `biber:conjunct_rate`, `biber:amplifier_rate`, `biber:emphatic_rate`, `biber:downtoner_rate`, `biber:discourse_particle_rate`, `biber:pron:{p1,p2,p3,it,demonstrative,indefinite}`, `biber:analytic_negation_rate`, `biber:synthetic_negation_rate`, `biber:nominalization_rate` (suffix rules `-tion/-ment/-ness/-ity` with a short stoplist), `biber:participial_clause_rate` (the PNAS regex proxy: sentence-final `, VERBing …` clauses), `biber:mean_word_len` (`Unit::Index`), `biber:ttr_proxy` — omitted (Richness owns it), `biber:contraction_rate` — omitted (punct owns `punct:contraction_rate`; no double-weighting). Composites `biber:d1_involved`, `biber:d4_persuasion` as `Unit::Index` `.rollup()` dims (informational, excluded from findings).
   - Spans: `add_at` per closed-class hit so findings point at words.
6. **`Readability` feature family** — new module `feature/readability.rs`: `Readability { min_sentences: usize = 4 }` / `FittedReadability`, `FAMILY = Family::Register`. Dims: `read:flesch_kincaid`, `read:gunning_fog`, `read:smog`, `read:coleman_liau`, `read:ari`, `read:syllables_per_word` (all `Index`; syllables via vowel-group counting — the W7 dict upgrade is opt-in at refit and recorded in fitted state, see W7.2), `read:passive_rate` (`PerHundredSentences`, be-form + participle regex heuristic — document the adjectival-participle false-positive bias as consistent), `read:acronym_rate` (`PerThousandTokens`). Below `min_sentences` → `mark_missing`.
7. **`doc-style` pack** — mined from Vale's MIT packs (Microsoft, Google, write-good, proselint): weasel words, wordy phrases (`utilize` → `alternatives: ["use"]` — maps directly onto `Term.alternatives` and the existing `Fix { kind: "consider_replace" }`), clichés, sentence-initial "So". One pack, MIT attribution in `sources`.
8. **`tech-voice` pack** — the phrase-shaped tech-blogger markers WIT.md names: discourse-pivot openers ("Anyway,", "OK, so", "Look,", "Here's the thing"), hypothetical-dialogue frames ("you might say", "you might be wondering"), analogy frames ("it's like", "think of X as", "imagine"). Original curation, bundle, `category_findings: true`. (The code-side tech-blogger dims — link/footnote density, rhetorical questions — land in W2 with the other `sentence.rs` work.)
9. **`Family::Register` variant**: add to `Family` (`#[non_exhaustive]`, additive), `as_str() = "register"`, `token_floors()` = `(150, 600)` (closed-class rates stabilize faster than MFW but slower than punct).
10. **`NormPack` format** (used from W4 on, defined now): `NormPack { name, version, date, description, license, sources: Vec<PackSource>, entries: Vec<NormEntry { term: String, value: f64 }> }` — same provenance envelope as `LexiconPack`, round-trip serde JSON, `qualified_name()` identical semantics.

**Acceptance**

- House-pattern inline tests: seeded `compose` fixtures salted with hedges/boosters at known rates → `lex:hyland:cat:*` dims within 1e-9 of hand-computed per-1k values; nominalization suffix rule against a fixture with stoplist edge cases (`nation`, `station` handling documented); readability grades vs hand-computed values on a fixed paragraph; passive-proxy precision spot-check fixture.
- Contract: new `"register"` family string appears in `doc.confidence` and `Finding.family` — assert `CritiqueReport` round-trips and `contract == "1"`; the open-set note (W1.1) ships in the same change.
- CLI: `loop.rs`-style integration — temp `handprint.toml` declaring the hyland pack, `fit` + `critique` on a hedge-stripped draft yields an `Increase` finding on `lex.hyland.cat.hedges`.

## Phase W2 — Punchline surprisal, punch rhythm, markdown/discourse extensions

Extends three existing families; highest-validated wit signal (Xie/Li/Pu ACL 2021), lowest cost. No new fitted models.

**Touches existing files:** `surprisal.rs`, `sentence.rs`, `feature/mod.rs` (`Family::Rhythm` — sequence after W1's `mod.rs` edit).

**Deliverables**

1. **`SurprisalLm` punch dims** — `surprisal.rs`: add spec flag `punchline: bool` (`#[serde(default)]`, off ⇒ old artifacts load unchanged, dims absent). Requires `word_bigrams: true` (validation error otherwise: `Error::InvalidConfig`). Implementation reuses the existing `word_surprisal(analysis) -> Vec<(Span, f64)>` plus `structure().sentences` to split each sentence into body vs final clause (final clause = tokens after the last mid-sentence `,;:—` clause boundary, min body length 4 tokens):
   - `surp:punch_ratio_mean` (`Index`) — mean over sentences of final-clause mean surprisal ÷ body mean surprisal;
   - `surp:punch_spike_rate` (`Fraction`) — share of sentences whose ratio exceeds 1.5 (constant, documented);
   - `surp:setup_entropy_mean` (`Bits`, non-actionable — informational, feeds interpretability tests).
   - **Canary safety (invariant 0.2):** these dims are emitted by `FittedSurprisal` but their `DimInfo.family` is **`Family::Rhythm`** (new variant, `as_str() = "rhythm"`, floors `(200, 800)`), *not* `Family::Surprisal` — otherwise the default canary would silently make them unreportable, and reporting them would contaminate the canary. Critique skip/trip logic keys off per-dim family (`DimInfo`/`Contribution.family`), so this works as-built (verified). Spike sentences get `note_span` on the final clause.
2. **`SentenceStats` punch-rhythm + aside dims** — `sentence.rs`, spec flag `rhythm: bool` (`#[serde(default)]`):
   - `sent:punch_short_rate` (`PerHundredSentences`) — short sentence (≤ 0.5× rolling mean) immediately following a long one (≥ 1.5×);
   - `sent:final_clause_len_ratio` (`Index`) — mean final-clause length ÷ mean sentence length;
   - `sent:aside_rate` (`PerHundredSentences`), `sent:aside_len_mean` (`Tokens`), `sent:aside_share` (`Fraction`) — parenthetical/dash-delimited asides (paired `(…)`, ` — … — `, footnote markers), spans via `add_at`. These stay `Family::Sentence` (never a canary candidate).
3. **Markdown + discourse extensions** — `sentence.rs`, same spec-flag pattern (`md_extended: bool`, `#[serde(default)]`): the WIT.md tech-blogger dims the register battery (W8.3) asserts on. `md:link_rate` (`PerThousandTokens`, markdown links + bare URLs), `md:footnote_rate` (`PerThousandTokens`, `[^n]`-style markers), `sent:rhetorical_question_rate` (`PerHundredSentences`, question-terminated sentence not preceded by an interrogative addressee pattern — documented heuristic). Completes the as-built `md:*` set (`heading,bullet,numbered,quote,code_fence,inline_code,bold,italic,table_row`) with the two densities WIT.md names explicitly (Luu-vs-Gwern apparatus contrast).

**Acceptance**

- Seeded fixture with constructed windup-spike sentences (rare word planted in final clause) vs flat control: `surp:punch_ratio_mean` fixture > control; `punch_spike_rate` exact against hand count; spans land on the final clauses byte-exactly.
- Canary test: with default config (canary = Surprisal), a punch-dim deviation **does** produce a finding (family `"rhythm"`), and the canary trip test from `critique.rs` still passes with punch dims present.
- Markdown fixture: link/footnote-salted document → exact per-1k rates; rhetorical-question fixture with hand-counted rate.
- Old-artifact test: a `Reference` JSON serialized without the new flags deserializes and profiles identically (structural assertion, no golden file, per house pattern).

## Phase W3 — Two-sided bands, contract hardening, and closing the contrast loop

**Status of the WIT.md item: the band mechanics are already built.** As-built `critique.rs` fires both arms — `observed > hi → Direction::Reduce`, `observed < lo → Direction::Increase`, inside band → no finding — `Finding.direction` serializes `"reduce"`/`"increase"`, `target_band: [f64; 2]` is already an interval, and `a_slopped_draft_produces_actionable_findings` asserts both arms. **Therefore: no field or semantics of the JSON contract changes; `CONTRACT_VERSION` stays `"1"`; nothing here is semver-major.** (The contract open-set documentation note originally drafted here moved to W1.1, where it must precede the first new family string.) What remains is hardening plus the two genuine gaps the as-built map identifies:

**Touches existing files:** `vocab.rs`, `critique.rs`, `config.rs`/`main.rs`/`commands.rs` (`--vocab` plumbing).

**Deliverables**

1. **Contrast-vocab actionability.** `FittedContrastVocab` dims are `Unit::RelativeFrequency`, for which `is_actionable() == false` — contrast dims can never become findings today. Change the unit to **`Unit::PerThousandTokens`** (value = universe-term count ÷ lexical tokens × 1000): honest semantics ("'atavistic' at 0.0/1k vs band 0.8–2.4 — increase"), already actionable, sane `noise_floor`. Behavior-visible, JSON-shape-unchanged; requires refit of any contrast-bearing reference (pre-1.0, acceptable). **Mechanism for loud failure** (an old artifact would otherwise deserialize cleanly and its dims silently stay `RelativeFrequency`/non-actionable — exactly the silent reinterpretation we must forbid): add `version: u32` to `FittedContrastVocab` with `#[serde(default)]` (old artifacts read as `0`), current fit writes `1`; validation at reference load / `Critic` construction rejects `version < 1` with a "refit required" error. Rejected alternative: flipping `RelativeFrequency` to actionable globally would flood findings with `mfw:*`/`c3:*` dims, contra IDEA.md phase 5.
2. **`fit` consumes saved vocabularies.** New `fit --vocab <path>` (repeatable) + `handprint.toml` `[[vocab]] path` — deserialize `ContrastVocab` JSON (the `contrast --out` artifact), push as a `FeatureSpec`. This closes discover → fit → critique: **author signature lexicons are auto-derived via Fightin' Words, with hand-curated lists demoted to golden tests** (WIT.md, resolved).
3. **Anti-caricature severity audit.** Confirm `severity_of` on `Increase`-direction findings escalates via z/deviation tiers (it does; the lexicon-declared-severity escalation staying `Reduce`-only is correct — under-using a slop word must stay Low). Add the over-firing test against a dim that exists at this phase: a `lex:` category dim (or `punct:` dim) pushed above the band's hi edge yields a `Reduce` finding whose message renders both band edges. (The device-dim variant of this test — "more Adams than Adams" on `dev:*`/`frame:*` — lands in W5 acceptance, where those dims first exist.)

**Acceptance**

- Contract test extension: a fixture reference with one contrast vocab → over- and under-firing drafts produce `Reduce`/`Increase` findings on `contrast.{name}.w1.{term}` ids with byte spans (spans already flow via `note_span` in `vocab.rs`).
- `loop.rs` extension: `contrast a b --out vocab.json` → `fit --vocab vocab.json` → `critique` end-to-end in a temp dir.
- Round-trip: a `FittedContrastVocab` JSON without the `version` field (hand-built fixture representing a pre-change artifact) loads but `Critic` construction fails with the "refit required" error — assert the error path (now writable, per the W3.1 version mechanism).

## Phase W4 — Comparison frames + register clash (formality variance)

The fiction-signature detectors; the formality lexicon doubles for the NN/g tone axes (marketing) and the Latinate↔profane Thompson pole.

**Touches existing files:** `feature/mod.rs` (`Family::Device`). Add-only: `frames.rs`, `formality.rs`, pack files.

**Deliverables**

1. **`Family::Device`** variant (`as_str() = "device"`, floors `(300, 1200)` — pattern rates need length) — shared by W4 frames and W5 device rates.
2. **`ComparisonFrames`** — new module `feature/frames.rs`: `ComparisonFrames { frozen_pack: Option<LexiconPack> }` / `FittedFrames { dims, frozen: HashSet<...> }`, `FAMILY = Family::Device`. Frame extraction is pattern-grade over normalized lexical forms (Niculae & DNM as the template): `as X as Y`, `like a/the Y`, `much the same way (that)`, `about as X as Y`. Dims (`PerThousandTokens` unless noted): `frame:simile_rate`, `frame:negated_vehicle_rate` (negator inside the vehicle clause window), `frame:ironic_hedge_rate` (`about as` hedge, Veale & Hao), `frame:frozen_share` (`Fraction`, vs the `cliche-similes` pack), `frame:vehicle_len_mean` (`Tokens`). Every matched frame gets `add_at` spans.
3. **`RegisterClash`** — new module `feature/formality.rs`: `RegisterClash { pack_path→NormPack, clash_window: usize = 12 }` / `FittedFormality { dims, norms: Vec<(String, f64)> table, ... }`, `FAMILY = Family::Register`. Fitted state embeds the norm table (data-only, serde; triggers the invariant-0.4 license rollup). Dims: `form:mean`, `form:variance`, `form:bimodality` (all `Index`), `form:clash_rate` (`PerHundredSentences` — sentence contains both a top-quartile and bottom-quartile formality word within `clash_window` tokens; spans on the clashing pair), `form:latinate_rate` (`PerThousandTokens`, suffix heuristic), `form:latinate_profanity_clash_rate` (`PerHundredSentences`, needs the W6 profanity pack; `mark_missing` if absent).
4. **`formality` NormPack** from Pavlick & Tetreault scores (HF `osyvokon/pavlick-formality-scores`) — **license unverified → loader-only until verified** (open decision O5); fallback: Heylighen–Dewaele-style closed-class proxy (pronouns/particles vs nouns/prepositions ratio) shipped as the bundled default so the feature works without the pack.
5. **`cliche-similes` pack** — curated from the frozen-simile literature (arxiv 1511.01756), original curation, bundle.

**Acceptance**

- Frame extraction fixture: a paragraph of hand-placed frames (incl. the negated-vehicle Adams construction "…in much the same way that bricks don't") → exact counts, byte-exact spans, negated-vehicle correctly distinguished from plain similes.
- Bimodality fixture: interleaved formal/profane synthetic text scores higher `form:variance` and `form:clash_rate` than either pure register; uniform text near zero (the Bukowski ○-cell test — two-sided band catches drift *up*).

## Phase W5 — Device-rate lexicons + marketing families

Same pattern+lexicon shape throughout; splits cleanly into packs (data) vs one code module (patterns that need token context).

**Touches existing files:** `lexicon.rs` (`#` wildcard), `feature/mod.rs` (registration only — `Family::Device` landed in W4). Add-only: `device.rs`, `norms.rs`, pack files.

**Deliverables**

1. **Pattern-language extension**: add `#` = single Number-token wildcard to `parse_pattern` (`lexicon.rs`) so pressure phrasings like `only # left` are expressible in pack data. Additive; existing packs unaffected.
2. **`CountPack` format** — background frequency tables: same provenance envelope as `NormPack`, `entries: Vec<(String, u64)>` + `total: u64`. Consumed by `dev:novel_adj_noun_rate` (below) and W6's `syn:rare_word_tail_rate`. This is the fix for a real grounding gap: `Feature::fit(&self, ctx: &FitContext<'_>, interner)` sees only the reference corpus — there is **no** background-corpus input on the trait (only `ContrastModel::fit` takes one, outside the `Feature` trait). Backgrounds therefore enter as versioned **data inputs on the spec**, produced offline (a small `handprint-data` builder or eval-side script), never magically at fit time.
3. **`DeviceRates`** — new module `feature/device.rs`: `DeviceRates { antonyms: bool, novelty: bool, background: Option<CountPack>, ... }` / `FittedDevices`, `FAMILY = Family::Device`. Dims (`PerThousandTokens` unless noted; all span-attributed):
   - `dev:litotes_rate` ("not entirely", "not un-", "hardly", "rather" + minimizer-adjacent-extreme), `dev:absurd_precision_rate` (precise numeral adjacent to abstract/cosmic noun list), `dev:transferred_epithet_rate` (emotion-lexicon adjective + concrete noun via the `concreteness` NormPack; `mark_missing` without it), `dev:alliteration_rate` (initial-phoneme runs ≥ 3, letter-based; dict-upgradeable per W7.2), `dev:antonym_pair_rate` (WordNet antonym pack, within-sentence), `dev:ambiguity_density` (`Index`, senses-per-word table), `dev:novel_adj_noun_rate` — **the one fit-stateful dim**: `fit` builds the reference's adj-noun bigram set (suffix-heuristic adjective detection, no tagger); fitted stores it plus, when `background` is supplied, the background bigram counts from the `CountPack`; transform scores unseen-bigram rate weighted against background frequency. Without a background pack, novelty is defined purely against the reference corpus's own bigram set (documented as the weaker variant). Either way the fingerprint reading holds: the Dark & Stormy over-firing signal — its *upper* band is the caricature guard.
   - Marketing-structural: `dev:imperative_rate` (`PerHundredSentences`, sentence-initial base-verb closed list, negated-imperative subtracted per Leech), `dev:headline:{number_initial,listicle,question,demonstrative}` (`PerHundredSentences`), `dev:triplet_rate`, `dev:negative_parallelism_rate` ("Not X. Not Y. Just Z.").
4. **Packs** (all `LexiconPack` JSON unless noted, `category_findings` per pack):
   - `hyperbole` (Troiano & Strapparava features; re-curated, bundle);
   - `marketing-eval` (Leech ad-adjectives, universal quantifiers, benefit connectives "so you can/which means", risk reversal, social proof `trusted by #`, urgency/scarcity with `#` slots from the Mathur dark-patterns vetted list, **clickbait curiosity phrases** — Blom & Hansen forward-reference deixis: "This is why", "What happened next", "You won't believe" — re-curated, bundle);
   - `marketing-slop` (consensus core: unlock/elevate/seamless/…, structural tells as phrases; original curation, bundle);
   - `wiki-ai-signs` (**separate pack**, CC-BY-SA share-alike, attribution in `sources` — invariant 0.4);
   - `epistemic-certainty` (~40 gonzo paranoid-certainty phrases, original, bundle);
   - `drug-lexicon` (small, original, bundle);
   - `wordnet-antonyms` + `senses-per-word` (Princeton WordNet license — permissive, bundle with attribution) as compiled tables consumed by `DeviceRates`, versioned like packs;
   - `vader-boosters` NormPack (MIT, bundle) — consumed in W6;
   - `concreteness` NormPack (Brysbaert/Warriner/Kuperman norms, ~37k lemmas — **likely CC-BY-NC per the WIT.md ledger → loader-only pending verification**, added to O5 and the W8 license-hygiene grep) — consumed by `dev:transferred_epithet_rate` here and `syn:concreteness_mean` in W6;
   - `arousal`/`sensorimotor` NormPacks (Warriner VAD likely CC-BY-NC → **loader-only**; Lancaster sensorimotor CC-BY pending OSF verification → bundle w/ attribution) driving generic norm-density dims via **`feature/norms.rs`**: `NormDensity { pack }` / `FittedNorms` (`FAMILY = Family::Register`), dims `norm:{pack}:mean`, `norm:{pack}:p90`, `norm:{pack}:high_share` — one generic feature covers concreteness, arousal, and sensory density.
5. **Kobak-method marketing slop discovery** (novel per WIT.md): eval-side recipe (W8) running phase-6 Fightin' Words over a marketing corpus vs background — the derived pack supersedes the hand list; hand list becomes the golden test (same rule as author signature lexis, resolved in W3).

**Acceptance**

- Per-detector fixtures with hand-counted rates and spans (house seeded-compose pattern), incl. a litotes/hyperbole-salted paragraph and a "Not X. Not Y. Just Z." block.
- Novel-bigram determinism: same corpus + seed → identical fitted bigram set (sorted, like MFW's freq-desc-then-alpha rule); with vs without a background `CountPack` → documented, distinct, both deterministic.
- Pack round-trip tests for every new pack (serde JSON, `qualified_name`), plus a `#`-wildcard pattern unit test and a `CountPack` round-trip test.
- Anti-caricature tests (the device-dim half of W3.3, landing here where the dims exist): the "more Adams than Adams" test — a `dev:*`/`frame:*` dim above the band's hi edge yields a `Reduce` finding rendering both band edges; and an over-fired marketing fixture (booster-stuffed, triplet-heavy) trips upper bands with `Reduce` findings — the register-battery version of the W8 fixture.

## Phase W6 — Syntax-texture family

Shared across Bukowski/Thompson/marketing; mostly closed-class counting and window detectors. Everything statistical-in-Rust per WIT.md; digression/satire stay LLM-judge tier (W9).

**Touches existing files:** `feature/mod.rs` (`Family::Syntax`). Add-only: `syntax.rs`, profanity pack.

**Deliverables**

1. **`Family::Syntax`** variant (`as_str() = "syntax"`, floors `(150, 600)`).
2. **`SyntaxTexture`** — new module `feature/syntax.rs`: `SyntaxTexture { booster_pack: Option<NormPack>, concreteness: Option<NormPack>, background: Option<CountPack> }` / `FittedSyntax`, `FAMILY = Family::Syntax`. Dims:
   - `syn:fragment_rate` (`PerHundredSentences`; no-finite-verb heuristic via modal/aux closed list + inflection rules — documented as noisy per-sentence, reliable as a rate);
   - `syn:sub_coord_ratio` (`Index`, subordinator:coordinator closed-class counts), `syn:initial_conj_rate` (`PerHundredSentences`, sentence-initial And/But);
   - `syn:ly_adverb_rate` (`PerThousandTokens`, with false-positive stoplist: `only, early, family, …`) — the *scarcity* is the Bukowski signature; the low band does the work;
   - `syn:intensifier_rate`, `syn:booster_chain_mean`, `syn:booster_chain_max` (`Index`; consecutive-booster runs via the VADER booster NormPack);
   - `syn:interjection_rate` (`PerThousandTokens`, small closed interjection list, `INTERJECTIONS_VERSION` const — the Thompson marker WIT.md names alongside ALL-CAPS/`!`);
   - `syn:allcaps_exclaim_rate` (`PerThousandTokens`, acronym-excluded — reuses punct's ALLCAPS machinery but scoped to exclamatory context; dedupe note vs `punct:allcaps_rate` documented);
   - `syn:polysyndeton_rate`, `syn:asyndeton_rate` (`PerHundredSentences`, in-sentence window detectors — validated ~100% P/R per the rhetorical-figures survey), `syn:comma_chain_mean` (`Index`);
   - `syn:person:{p1s,p2,p1p}` (`PerThousandTokens`, LIWC-style closed lists — dedupe vs `biber:pron:*`: enforcing "omit when `BiberTier1` is in the same pipeline" at fit is not practical; the doc rule is "pick one family for pronouns per pipeline" and `short_text_defaults` never includes both);
   - `syn:concreteness_mean` (`Index`, via the W5 `concreteness` NormPack; `mark_missing` without it);
   - `syn:rare_word_tail_rate` (`PerThousandTokens` — the Bukowski **anti-allusion** axis WIT.md names: rate of tokens below a background-frequency threshold, via the W5 `CountPack` mechanism; `mark_missing` without a background pack; the *low* band is the signature);
   - `syn:syllable_slope`, `syn:syllable_autocorr`, `syn:syllable_peak_ratio` (`Index`; per-sentence syllable-count sequence stats — the Wills escalation-rhythm axis; vowel-group syllables, dict-upgradeable per W7.2).
3. **`profanity` pack** — LDNOOBW (permissive, bundle) merged with dsojevic severity tags (license verify; fall back to LDNOOBW-only) as a `LexiconPack` with `Severity` per term → dims `lex:profanity:*` + `cat` rollups; feeds `form:latinate_profanity_clash_rate` (W4).

**Acceptance**

- Fragment-rate fixture: verbless fragments vs finite controls, hand-counted rate ±0 (heuristic exactness on the fixture, documented recall limits in module docs).
- The Bukowski/Thompson contrast-pair test in miniature: two seeded synthetic corpora (comma-scarce/adverb-free vs em-dash/booster/ALL-CAPS-heavy) fit into one background → `compare` separates them and each corpus's top-|z| dims match the designed axes (this is the unit-scale prototype of the W8 battery golden test).
- Booster-chain and polysyndeton window detectors: exact counts + spans on hand-built sentences; rare-word-tail fixture against a toy `CountPack`.

## Phase W7 — Verse pack + line-preserving text mode

Feature-gated (cargo feature `verse` — the CMU dict is ~134k entries; gate keeps core light per IDEA invariant #7).

**Touches existing files:** `segment.rs` + `text/mod.rs` (`Structure.lines`), `feature/mod.rs` (`Family::Verse`, `Unit::PerHundredLines`), `readability.rs`/`syntax.rs`/`device.rs` (syllable-method plumbing + rhyme dim). Add-only: `verse.rs`, `verse-dict` pack.

**Deliverables**

1. **Line-preserving text mode — specified against the as-built text layer.** WIT.md calls this "a tokenizer mode", but as-built the right layer is **segmentation, not tokenization**: source text is never mutated (`ScoringText` only folds confusables with an `OffsetMap` back to source bytes; normalization is per-token), so line structure is *not* destroyed — it is merely not exposed. `segment.rs` already computes `line_offsets` for block classification and then discards them. Change: `Structure` gains `lines: Vec<Line>` with `Line { span: Span /* source bytes, half-open, \r\n-trimmed */, tokens: Range<usize>, blank_before: bool }`, populated from the existing `line_offsets` pass with an `attach_tokens`-style token-range mapping (same mechanism as `Sentence.tokens`). No `Tokenizer` enum change, no serde impact (`Analysis` is recomputed, never serialized), no cost when unused beyond the vec. Verse/line-geometry features read `analysis.structure().lines`; `blank_before` runs delimit stanzas. Prose pipelines ignore it.
2. **Syllable method is fitted state, never build state (invariant 0.5).** The CMU dict lives behind the `verse` cargo feature, but `read:*` (W1), `syn:syllable_*` (W6), `dev:alliteration_rate`/`dev:rhyme_chain_rate` (W5/here) can benefit from dict syllables/phonemes. If the upgrade were implicit, the same `Reference` JSON would profile differently on a `verse` build vs a non-verse build — violating transform purity and silently breaking calibration. Mechanism: fitted structs that touch syllables record `syllable_method: SyllableMethod` (`enum { VowelGroup, Dict { dict_version: String } }`, `#[serde(default)] = VowelGroup`). `fit` selects `Dict` only when the build has `verse` **and** the spec opts in (`--syllables dict`); `transform` on a build that cannot honor a recorded `Dict` method **fails loudly** ("reference requires verse build / dict {version}"), never silently falls back. Dict-first with vowel-group fallback applies per-OOV-word *within* the `Dict` method (deterministic given the versioned dict), not across builds.
3. **`verse-dict` data pack**: CMU pronouncing dictionary (BSD-like/public domain, bundle) compiled to a compact serialized table + suffix-based stress guesser for OOV + finite early-modern elision rules ("heaven" = 1 syl, syllabic `-èd`, `o'er`), versioned `cmudict@0.7b-2026.08`.
4. **`dev:rhyme_chain_rate`** (`PerThousandTokens`, `device.rs`) — completes the Mihalcea–Strapparava trio (W5 shipped alliteration/antonymy/ambiguity; rhyme was deferred to here because it is only honest with phoneme data): line-final and in-sentence rhyme chains via the CMU dict. `Dict`-method-only (per W7.2); absent on `VowelGroup` references (`mark_missing`, never a letter-based fake).
5. **`Family::Verse`** variant (`as_str() = "verse"`, floors `(200, 800)`), **`Unit::PerHundredLines`** variant (actionable, `noise_floor` aligned with `PerHundredSentences`) — `Unit` is exhaustive, so this is source-breaking for exhaustive matches inside the crate only (pre-1.0, fine; external JSON is additive).
6. **`VersePack`** — new module `feature/verse.rs`: `VersePack { min_verse_share: f64 = 0.5 }` / `FittedVerse`, `FAMILY = Family::Verse`. Verse-vs-prose detection: from markup when present (Folger TEI is pre-processed to md with verse markers in the battery pipeline, W8), else heuristic (short-line share, right-margin raggedness) with a confidence note; below `min_verse_share` → all dims `mark_missing` (never scan prose as verse). Dims:
   - `verse:feminine_ending_rate` (`PerHundredLines`) — the single best-attested discriminator (Spedding 1850; Plecháč 2019);
   - `verse:stress_pos:{1..10}` (`Fraction`, stress frequency per metrical position — the Plecháč stress-profile vector), `verse:iambic_conformity` (`Fraction`), `verse:midline_pause_rate`, `verse:enjambment_rate`, `verse:end_stop_rate` (`PerHundredLines`; punctuation-position histograms + line-final function-word proxy);
   - line geometry (the Bukowski poetry axes): `verse:line_len_mean` (`Tokens`), `verse:line_len_stddev` (`Index`), `verse:one_word_line_rate`, `verse:lowercase_initial_rate` (`PerHundredLines`);
   - archaism morphology: `verse:thou_you_ratio` (`Index`), `verse:eth_rate`, `verse:est_rate` (`PerThousandTokens`);
   - `verse:vocative_rate` (`PerHundredLines`, the "O + NP" pattern rule WIT.md names for vocative/apostrophe);
   - figure rates: `verse:figure:{anaphora,epistrophe,chiasmus,antithesis}` (`PerHundredLines`; Dubremetz-style pattern features; antithesis uses the W5 antonym table; per-instance F1 caveats documented — rates wash out), `verse:doublet_lowpmi_rate` (the honest hendiadys proxy: Noun-and-Noun coordinations with low corpus co-occurrence — fit-time PMI table, and **no overclaiming**: named "doublet", not "hendiadys").
   - Skipped in core, per WIT.md: pun detection, functional-shift rate (POS tagger) — LLM-judge tier.

**Acceptance**

- Scansion: fixture couplets with known syllable counts and stress patterns → per-syllable accuracy ≥ 85% on the fixture, feminine-ending detection exact on hand-marked lines (aggregate-washes-out argument documented, per Plecháč).
- Syllable-method round-trip: a `Dict`-method reference fails loudly on a non-`verse` test build (compile-gated test); a `VowelGroup` reference profiles identically on both builds; a pre-W7 reference (no `syllable_method` field) deserializes as `VowelGroup` and profiles unchanged.
- Line table: property test — every `Line.span` slices source to the line's text sans terminator; `blank_before` marks stanza breaks in a fixture poem; prose fixture yields lines but `VersePack` marks all dims missing (`min_verse_share` guard).
- The Spedding split as a golden test: two synthetic verse corpora built with 30% vs 65% feminine-ending rates → `verse:feminine_ending_rate` bands don't overlap and `compare` separates them.

## Phase W8 — Eval batteries: author battery + register battery (`eval/`)

Instantiates IDEA phase 10 (the `eval/` dir is currently empty) with the WIT.md battery acceptance criteria as concrete tests. Python harness, non-circular judges, real corpora — **in-copyright corpora are local-only, never committed, never shipped** (fixtures in the repo are synthetic or PD only).

**Touches existing files:** none in `crates/` — everything under `eval/` (add-only).

**Deliverables**

1. **Corpus acquisition pipeline** (`eval/corpora/`, gitignored for in-copyright content; scripts committed):
   - **epub → markdown via pandoc** (`pandoc book.epub -t gfm -o book.md`) — chapter headings survive as `#` sections; a small splitter chapterizes to `{author}/{book}-ch{NN}.md` per-chapter docs (within-author variance for two-sided bands and same-author calibration, per WIT.md pilot design).
   - **Dual ingestion, shared corpus directory, no shared code**: `codeindex add` on the same directory (retrieval-grade chunking + ONNX embeddings + hybrid search, for exemplar retrieval in W9/W10); `handprint fit` on the same normalized text via the as-built `load_corpus` directory layout (author-per-subdir). The two tools share bytes, not abstractions.
   - Authors: Adams (5 HHGG + 2 Dirk Gently, ≈600k+ words), Pratchett (subset), Bukowski (prose + poems; poems as per-poem docs exercising W7 line geometry), Thompson (Fear and Loathing, Hell's Angels, columns), Shakespeare (Folger TEI → md with verse markers — **non-commercial license: local battery only**; MIT/Moby as PD fallback; EEBO-TCP as period background for `coinage/OOV` if attempted).
2. **Battery pipeline** (`eval/battery/`): fit all five references (full pipeline: W1–W7 families + auto-derived signature vocab via `contrast ... --out` + `fit --vocab`), calibrate each (`calibrate --metric burrows`), then three test suites per WIT.md's acceptance criteria:
   - **(a) Separation** — cross-author attribution sanity via existing `compare`/`rank` + GI `verify` (impostor pool = the other authors): held-out chapters rank their own author first; GI returns `TargetFavoured` with abstention rate reported.
   - **(b) Interpretability golden tests** — per author (all five), assert the top-|z| dims (vs a pooled background reference) intersect the critic-named trait set from the WIT.md coverage matrix: Adams ⊇ {`frame:negated_vehicle_rate`, `form:clash_rate`, `surp:punch_spike_rate`, `dev:litotes_rate`}; **Pratchett ⊇ {`surp:punch_spike_rate`, `form:clash_rate`, `dev:litotes_rate`/`dev:absurd_precision_rate`, `sent:aside_rate` (the footnote/aside digression axis — `md:footnote_rate` where the source formatting preserves footnotes)}**; Bukowski ⊇ {`syn:fragment_rate`, `syn:ly_adverb_rate` (low), `syn:sub_coord_ratio` (low), `lex:profanity:*`, `syn:concreteness_mean`}; Thompson ⊇ {`syn:booster_chain_*`, `syn:allcaps_exclaim_rate`, `punct:ellipsis*`/em-dash, `contrast:hst:*` signature lexis}; Shakespeare ⊇ {`verse:feminine_ending_rate`, `verse:stress_pos:*`, `verse:thou_you_ratio`}. The ○ cells are asserted too: Bukowski's intensifier/adverb dims must sit in *low* bands — absence is fingerprint.
   - **(c) Anti-caricature fixtures** — deliberately over-fired pastiches (booster-stuffed fake Thompson; adverb-free-but-intensifier-laden fake Bukowski; negated-simile-saturated fake Adams), generated by seeded scripts (committed — they're original text): each must trip **upper**-band `Reduce` findings on the relevant device/syntax dims at Medium+ severity, and must *fail* the same-author gate.
3. **Register battery** (`eval/registers/`): tech-blog corpus vs marketing corpus vs agent-prose (from the as-built `handprint-data` extractors) — assert the register coverage matrix: blogs high on `sent:aside_rate`/`md:link_rate`/`md:footnote_rate`/`md:*` apparatus/`sent:rhetorical_question_rate`/`lex:tech-voice:*` pivots/`lex:hyland:cat:hedges` (all delivered by W1–W2); marketing high on `dev:imperative_rate`/`lex:marketing-eval:*`/booster:hedge ratio; agent prose reproduces the PNAS LLM signature directions (`biber:nominalization_rate` ↑, `biber:participial_clause_rate` ↑, `punct:contraction_rate` ↓, `biber:pron:p1` ↓). Webis-Clickbait-17 (free, graded) fits the headline-slop dimension. The Kobak-method marketing-slop discovery run (W5.5) lives here.
4. Carried over from IDEA phase 10 (unchanged, now unblocked): LUAR/Style-Embedding away/toward scores, SBERT meaning gate, Binoculars external detector, the HN forensic eval.

**Acceptance**

- The battery suites (a)–(c) pass and are re-runnable from clean scripts (`make battery` in `eval/`); results table checked into `eval/RESULTS.md` (metrics only, no corpus text).
- License hygiene test: CI greps shipped packs and fixtures for the loader-only/local-only resources (Folger, NRC, Warriner VAD, **Brysbaert concreteness**, Pavlick–Tetreault pending verification) — zero hits.

## Phase W9 — Out-of-crate deliverables: move index, agent pipeline, wit critic

None of this goes in the `handprint` crates — no LLM in core is a standing invariant. Repo boundaries stated per deliverable.

**Touches existing files:** none in `crates/` — `eval/agents/`, `.claude/skills/`, corpus-side artifacts.

**Deliverables**

1. **Comic-move index** (lives with the corpus + `codeindex`, not handprint): a one-time LLM labeling pass over ingested chunks tagging (a) comic device(s) from the corpus-derived inventory (subverted simile, register clash, absurd precision, digression, deadpan escalation…), (b) scene function (character intro, technology going wrong, narratorial aside…). Stored as markdown-annotation sidecars re-ingested into `codeindex` as a parallel corpus, so retrieval is **by mechanism, not topic** ("3 passages where the author introduces a character"). Labeling script + prompt live in `eval/agents/move-index/`; the index artifact lives beside the corpus. Sidecars contain quoted passages, so they follow the corpus's own distribution rule: gitignored for in-copyright corpora, committed only where the underlying text is committed (the worldwide-PD W10 corpora — see W10.4's jurisdiction rule). Guard: retrieved passages get parroted — the pipeline instructs mechanism-imitation, and handprint's char-ngram overlap (drift fingerprint machinery) catches verbatim leakage.
2. **Agent pipeline skill** (Claude Code skill, project-scoped: `.claude/skills/voice-transfer/`): the four-stage loop from WIT.md Layer 3 — (1) plain content-preserving retelling; (2) comic/device plan per beat at **corpus-calibrated rates** (the skill reads the target `Reference` JSON dim stats / critique `target_band`s to place devices at the author's empirical density, not maximum density), with a CLoT-style remote-association step and punchlines drafted as one-liners before prose; (3) render with 2–4 move-matched exemplars per scene retrieved via `codeindex search` over the move index; (4) polish loop against `handprint critique --json` (Vale exit codes drive iteration) sequenced so mechanics converge early and later iterations are wit-critic-driven.
3. **Wit-critic judge protocol** (`eval/judge/`): GTVH-structured per-paragraph rubric (intended mechanism present? delivery — punchline position, escalation, conciseness? failure flags — cliché, overexplanation, both-meanings-stated?), judged **pairwise against gold author anchors, never absolute** (TTCW: absolute LLM creativity judgments don't track experts; pairwise+anchored is the stable protocol). Prompt spec + harness script; judge model pinned per run in the results manifest.
4. **Ensemble acceptance gate** documented in the skill: handprint `p_same_author` + wit-critic pairwise win-rate + (optional) phase-10 external style embeddings — each alone is Goodhart-able; the ensemble is the gate.

**Acceptance**

- Move-index labeling reproducibility spot-check: two runs over a 50-chunk sample, device-label agreement **≥ 0.7 Cohen's κ (and ≥ 80% raw agreement)**; disagreements reviewed and the prompt tightened once before accepting.
- Skill dry-run against the W8 synthetic mini-corpora: the loop terminates, drift guard never trips on faithful rewrites, and mechanism-retrieval returns device-matched (not topic-matched) exemplars on a labeled fixture.

## Phase W10 — EXPERIMENTATION capstone: Gutenberg cross-author voice transfer

The plan's exit criterion, and deliberately double-duty: because every corpus here is **public domain in the US** (the 95-year publication rule), the experiment produces handprint's shippable demo packs and README headline numbers, which the in-copyright battery authors (Adams, Pratchett, Bukowski, Thompson) can never provide. `wodehouse@2026.x`, `twain@2026.x`, `poe@2026.x`, `hemingway-early@2026.x` become the crate's first published reference packs.

**Jurisdiction rule (governs every shippable below).** US PD ≠ worldwide PD: Wodehouse (d. 1975) and Hemingway (d. 1961) remain in copyright in life+70 jurisdictions until 2046 and 2032. So shippables split by whether they **carry text**:

- **Data-only artifacts** — fitted references, calibration data (dim stats, band edges; the embedded word lists are individual words/short patterns, noted in provenance) — ship for all four authors, with an explicit `pd_basis: "US (pre-1930 publication)"` provenance field and a README jurisdiction note.
- **Text-bearing artifacts** — move-index sidecars (quoted passages) and full transfer transcripts (which embed the source stories wholesale) — are **committed/published only for worldwide-PD authors** (Twain d. 1910, Poe d. 1849, Austen d. 1817). Wodehouse/Hemingway sidecars and the T1/T2 transcripts stay local artifacts; what publishes from T1/T2 is metrics, finding excerpts, and short quotations within ordinary quotation limits.

**Touches existing files:** `handprint-cli` `main.rs`/`commands.rs` (`extract gutenberg` dispatch). Add-only: `crates/handprint-data/src/gutenberg.rs`, `eval/experiments/voice-transfer/`.

**Author selection** (3–5 distinctive PD voices spanning the WIT.md battery axes):

| Author | PD basis | Pole / axes exercised |
|---|---|---|
| P. G. Wodehouse | US: pre-1930 works (in copyright in life+70 jurisdictions until 2046) | comic-device pole: transferred epithet, simile frames, slang-in-formal-frame register clash → W4/W5, punch rhythm → W2 |
| early Hemingway | US: *In Our Time* 1925, *Men Without Women* 1927, *A Farewell to Arms* 1929 (life+70 until 2032) | minimalism pole: fragments, parataxis, adverb scarcity, low device rates → W6 and the ○-cell/low-band machinery |
| Mark Twain | worldwide (d. 1910) | vernacular + comic digression: dialect orthography (punct family), 1st-person deixis, aside rate → W2/W6 |
| E. A. Poe | worldwide (d. 1849) | gothic maximalism: adjective/arousal density, hypotaxis, em-dash/semicolon exuberance — Hemingway's opposite on nearly every syntax dim |
| (stretch) Jane Austen | worldwide (d. 1817) | irony + hypotaxis, litotes pole → `dev:litotes_rate` |

Wodehouse↔Hemingway is the designed-in headline pair: maximum axis separation (device-rate pole vs absence pole), both directions test different machinery. Twain/Poe/(Austen) are the worldwide-PD trio that carries the committed text-bearing artifacts.

**Deliverables**

1. **Corpus prep pipeline** — new module `crates/handprint-data/src/gutenberg.rs` (this part *is* in-crate; it's corpus extraction, handprint-data's job): `strip_pg_boilerplate` (the `*** START/END OF THE PROJECT GUTENBERG EBOOK` markers + license block removal, with a fixture-tested fallback scanner for older header variants) and `chapterize` (split on `CHAPTER`/roman-numeral/heading patterns; for epub sources, pandoc → md first so headings are `#` and the splitter is trivial). CLI: `handprint extract gutenberg --in raw/ --out corpus/{author}/`. Target ≥ 10 books/author where available, per-chapter docs (~2–5k words each — above the reliability floor per doc after aggregation, and enough docs for calibration's same-author pairs). Dual ingestion as in W8: `handprint fit` + `calibrate` per author; `codeindex add` + W9 move-index labeling pass (sidecars committed per the jurisdiction rule: Twain/Poe/Austen yes, Wodehouse/Hemingway local).
2. **Transfer protocol** — at least one comic→minimal and one minimal→comic:
   - **T1 (comic → minimal):** Wodehouse, "Extricating Young Gussie" (1915) → Hemingway's voice. Tests the *absence* bands: device/frame/adverb/subordination dims must descend **into** Hemingway's low bands — a "write minimally" prompt gives an agent no target rate; the calibrated critic does.
   - **T2 (minimal → comic):** Hemingway, "Cat in the Rain" (*In Our Time*, 1925) → Wodehouse's voice. Tests calibrated device *injection*: the documented failure mode is over-firing (Dark & Stormy), so this is the live anti-caricature trial — upper-band `Reduce` findings must steer the agent back down.
   - **T3 (stretch, vernacular → maximal):** Twain, "The Celebrated Jumping Frog of Calaveras County" (1865) → Poe's voice — the largest orthography/punctuation shift, and (both authors worldwide-PD) the transfer whose full transcript is committable, making it the published-transcript exemplar even if T1/T2 keep the README headline numbers.
   - Three arms per transfer (the WIT.md pilot design): (a) zero-shot "rewrite in B's voice"; (b) + handprint critique loop, mechanics only; (c) full W9 pipeline — plain retelling → device plan at B-calibrated rates → render with move-matched exemplars → dual-critic polish loop until the pass gate (exit 0) or iteration cap (12). The plain retelling is the draft the `Critic` is seeded with, so `content_overlap` is measured against it.
3. **Measurable success criteria** (per transfer, arm (c); arms (a)/(b) reported as baselines):
   - **Gate:** `p_same_author` vs reference B ≥ the Balanced threshold (0.10; report Strict 0.25 too) — the calibrated same-author band, i.e. the draft lands inside B's own within-author variation.
   - **Away:** BurrowsDelta distance to reference A's exemplars strictly increases vs the plain-retelling baseline (measured via `compare -r refA`), and GI `verify` with impostor pool = the other Gutenberg authors returns `TargetFavoured` for B and not for A.
   - **Anti-caricature:** zero Medium+ findings **in either direction** on Device/Frames/Syntax/Rhythm dims in the final report — inside the two-sided band, not merely below the ceiling: over-firing (`Reduce`) is caricature, under-firing (`Increase`) is a mechanics-only pastiche that never acquired the voice, and the ○-cells rule (WIT.md) makes both failures.
   - **Content:** `guards.drift_ok == true` (`content_overlap ≥ 0.5` vs the plain retelling) and `canary_ok == true` throughout.
   - **External judge:** blind pairwise LLM-judge (W9 protocol) of final draft vs gold B anchors — arm (c) win-rate > arm (a) and arm (b) with the judge blinded to arm; absolute win-rate vs real B reported honestly (expected < 0.5; the claim is the arm ordering, per the WIT.md prediction).
4. **Shippables** (split per the jurisdiction rule): the four fitted+calibrated reference packs with full provenance manifests (Gutenberg source IDs, extraction tool version, tokenizer policy, `pd_basis` per author); the labeled move indexes and full transfer transcripts **for the worldwide-PD authors**; metrics + results manifests for all transfers; and a README section with the T1/T2 numbers plus the jurisdiction note.

**Acceptance**

- `gutenberg.rs` golden tests on committed PD fixture files (header variants, a chapterization fixture — fixtures drawn from worldwide-PD texts) — house pattern, and for once golden text fixtures are legal to commit.
- End-to-end reproducibility: `eval/experiments/voice-transfer/run.sh` regenerates corpora → references → transfers from Gutenberg IDs + seeds; results manifest records model IDs, seeds, pack versions.
- T1 and T2 arm (c) meet all five criteria; any failure is written up as a falsification finding, not massaged (this experiment is allowed to fail — that is its job, per IDEA.md's M3 philosophy).

---

## Milestones

| Milestone | Contents | Exit criterion |
|---|---|---|
| WM1 | W1 (packs + Register family + plumbing + contract open-set note) | hyland/doc-style/tech-voice packs fit via `handprint.toml`; PNAS-signature dims live; open-set note shipped with the first new family string |
| WM2 | W2 + W3 | punch dims reportable under default canary; md/link/footnote + rhetorical-question dims live; `contrast → fit --vocab → critique` loop closed with the version-gated refit error; anti-caricature severity test green on existing dims |
| WM3 | W4 + W5 | frames/clash/device/marketing families + all W4/W5 packs incl. `concreteness` (loader-only) + `CountPack`; over-fired marketing fixture and device-dim "more Adams than Adams" test trip upper bands |
| WM4 | W6 + W7 | syntax + verse families; line table in `Structure`; syllable-method round-trip green; Bukowski/Thompson mini-contrast and Spedding-split tests green |
| WM5 | W8 | author battery (a)(b)(c) — five authors incl. Pratchett — + register battery pass; `eval/` harness real; license-hygiene grep green |
| WM6 | W9 | move index (κ ≥ 0.7 spot-check) + voice-transfer skill + wit-critic protocol dry-run green |
| WM7 | W10 | T1 + T2 criteria met (or falsified and written up); PD demo packs published under the jurisdiction rule; README numbers |

WM2 is the earliest falsification point for the wit thesis (punch dims + closed contrast loop on synthetic fixtures); WM7 is the thesis test. Reprioritize W4–W7 scope after WM2 if punch/contrast signals underperform on fixtures.

## Resolved decisions (from WIT.md / IDEA.md — not relitigated here)

- Priority order: register packs → punch surprisal → band/contract → frames/clash → device+marketing → syntax → verse → out-of-crate (WIT.md §Priority).
- Two-sided bands are mandatory anti-caricature machinery; **as-built they already exist** — W3 is hardening + the contrast-actionability gap, not a contract change. `CONTRACT_VERSION` stays `"1"`. Every anti-caricature acceptance criterion (W5, W8, W10) counts findings in **both** directions — under-firing is as much a failure as over-firing.
- Author signature lexicons are auto-derived (Fightin' Words); hand lists are golden tests only.
- Background corpora enter `fit` only as versioned data (`CountPack`) on the spec — the `Feature` trait sees the reference corpus alone, and that stays true.
- Syllable/phoneme method is recorded in fitted state and honored or refused loudly — transform output never depends on the build's cargo features.
- GPL word lists (biberpy) and book compilations (Hyland, Leech): re-curate, never vendor. NC/no-redistribution resources (NRC, likely Warriner, likely Brysbaert concreteness): loader-only, never bundled; CC-BY-SA derivations isolated in their own pack.
- LLM judgment (wit critic, move labeling, tone axes funny/irreverent, pun/functional-shift detection) stays out of core — agent-side/eval-side only; judging is pairwise vs gold anchors, never absolute.
- In-copyright corpora (Adams, Pratchett, Bukowski, Thompson, Folger Shakespeare): local fitting only, never shipped. Shippable packs come from the W10 corpora under the split jurisdiction rule: data-only artifacts for US-PD authors with a stated `pd_basis`; text-bearing artifacts only for worldwide-PD authors.
- Hendiadys: ship the low-PMI doublet proxy under an honest name; no overclaiming. Verse-vs-prose comes from markup when available, heuristic + confidence flag otherwise.
- Biber Tier-2 (POS-tagged features): deferred; not needed for v1 (WIT.md).
- ContrastVocab actionability: unit change to `PerThousandTokens` + fitted `version` field (W3.1), not a global `RelativeFrequency` flip.
- Callback/running-gag profile (WIT.md Tier B): **deferred** — WIT.md itself scopes it as "mostly a corpus descriptor + drift check rather than a per-draft gate"; document-level rare-n-gram recurrence has no per-draft actionable band at typical draft lengths. Revisit post-WM5 as an eval-side corpus descriptor if the battery shows a use.
- Mihalcea trio: alliteration/antonymy/ambiguity ship in W5; rhyme-chain ships in W7 where the CMU dict makes it honest (`dev:rhyme_chain_rate`, dict-method-only).
- Coinage/OOV-vs-background (Shakespeare): attempted only if EEBO-TCP background lands in W8, per WIT.md's own caveats (period lexicon + normalized spelling required); not a core deliverable.

## Open decisions (fine to defer)

1. **Family granularity**: `Register` currently covers Biber + readability + norms + formality-clash; if `doc.confidence` proves too coarse for agents (readability trustworthy at lengths where clash isn't), split `Readability` out into its own family before 1.0 — additive either way.
2. **`Unit::PerHundredLines`** vs reusing `PerHundredSentences` for verse dims — plan says new variant (honest unit strings in findings); revisit if the `Unit` churn bites.
3. **Pronoun-dim dedupe enforcement** (`biber:pron:*` vs `syn:person:*`): doc-rule now; consider a fit-time overlap warning in `PipelineBuilder` later.
4. **Norm-pack license enforcement**: `Provenance.licenses` rollup + CLI warning now; whether `save_reference` should hard-refuse to write a reference fitted with a non-redistributable pack absent `--local-only` ack.
5. **Data licenses to verify before promoting from loader-only to bundle**: Pavlick–Tetreault formality scores, **Brysbaert concreteness norms**, Warriner VAD, dsojevic profanity severity tags, Lancaster sensorimotor OSF terms. Heylighen-style proxy is the bundled formality fallback meanwhile; LDNOOBW-only is the profanity fallback.
6. **Fifth capstone author** (Austen vs Jerome K. Jerome as a second comic voice) and whether T3 (Twain→Poe) makes the README or stays the published-transcript appendix.
7. **Judge model + anchor count** for the wit critic (pinned per run regardless); whether phase-10 style embeddings join the W10 gate ensemble or stay report-only.
8. **Move-index storage**: markdown-annotation sidecars re-ingested into codeindex (current plan) vs a codeindex-native tagging feature — depends on codeindex roadmap; keep sidecars until that repo grows tags.
