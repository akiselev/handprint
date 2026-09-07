# Handprint screenplay experiments: implementation plan

Date: 2026-09-07
Status: proposed, not implemented or experimentally validated.
Repository inspected: `akiselev/handprint` at `bc7653c425a02afb6f2558bc4c8ff29b7880698f`.

## Decision

Build an experiment harness for scene revision before building a general screenplay-quality model. Start with MovieSum as the structured reference corpus, small human-checked scene datasets as the validity anchor, and independent reader judgments as the outcome. Treat style compatibility, story preservation, and reader preference as separate outputs. Scale to sequences and complete screenplays only after scene-level feedback beats an equally funded revision baseline.

All numerical thresholds below are proposed engineering or experiment criteria, not findings from the repository or literature. Freeze them after a development pilot and before a confirmatory run. Do not repeatedly tune them on the final test set.

## Questions and falsifiers

1. Can the parser expose dialogue and action reliably? Falsifier: apparent style differences disappear after correcting segmentation or source-format errors.
2. Do proposed diagnostics identify meaningful problems? Falsifier: they flag valid monologues, accusations, rituals, or deliberate repetition as often as unmotivated exposition.
3. Does feedback improve revision beyond an equal-budget prompt-only baseline? Falsifier: internal scores rise but independent reader preference and preservation do not.
4. Does mechanism-matched retrieval help beyond topical retrieval? Falsifier: matched examples have no advantage or mainly increase copied language.
5. Can we change a declared style without breaking the specified story? Falsifier: style gain depends on content loss, changed motivation, or premature revelation.
6. Does local feedback compose into better long-form work? Falsifier: scenes become individually polished while the complete narrative becomes repetitive, incoherent, or less distinctive.

A negative result is a completed experiment. Prefer a smaller effective subsystem over a large unvalidated critic.

## Existing integration points

The existing Python `eval/harness/loop.py` defines an injected `Rewriter` and calls the CLI. When the iteration budget is exhausted, the last rewrite can be returned without a corresponding report: it scores the current text, rewrites it, then exits the loop. Add a regression test and fix this before collecting any results.

The current `eval/judge/run.py` supplies an injected judge, randomizes A/B placement, and pairs paragraphs with anchors. Keep injection and blinding, but add explicit matched-scene comparisons. Arbitrary anchor paragraphs are not a controlled comparison of two revisions of the same scene.

Do not reuse `p_same_author`, author-distance calibration, or the forensic pass gate as evidence of dramatic quality. Keep deterministic extraction in Rust; annotation and subjective judgment remain external. Code and model adapters are separate from fitted reference artifacts.

Inspected files:
- https://github.com/akiselev/handprint/blob/bc7653c425a02afb6f2558bc4c8ff29b7880698f/eval/harness/loop.py
- https://github.com/akiselev/handprint/blob/bc7653c425a02afb6f2558bc4c8ff29b7880698f/eval/judge/run.py
- https://github.com/akiselev/handprint/blob/bc7653c425a02afb6f2558bc4c8ff29b7880698f/crates/handprint-cli/tests/loop.rs

## Proposed layout

```text
crates/handprint-data/src/screenplay/
  mod.rs
  manifest.rs             # source records, hashes, rights and provenance
  moviesum.rs             # released XML-in-JSON -> ScriptIR
  cornell.rs              # separate dialogue-corpus adapter
  fountain.rs             # declared subset first, complete syntax later
  dedupe.rs               # exact/near duplicate and work grouping

crates/handprint-script/   # new, only once the representation is stable
  src/ir.rs               # scene/block/turn identities and source mapping
  src/profile.rs          # wraps existing core features on typed partitions
  src/dialogue.rs         # sequence and turn geometry
  src/reference.rs        # context-conditioned reference distributions
  src/contracts.rs        # style/content/change-scope contracts
  src/report.rs           # measurements vs hypotheses vs validations
  src/diff.rs             # explicit cross-version alignment

crates/handprint-cli/src/
  ...                     # proposed `script` subcommands, not existing commands

eval/screenplay/
  schemas/
  fixtures/               # original authored fixtures, not scraped scripts
  corpora/                # manifests/adapters, no restricted source text
  annotations/
  experiments/
  judges/
  reports/
```

Initially keep adapters and orchestration in `handprint-data` and `eval/screenplay`; do not force a new crate before the IR is exercised. The existing core must not gain model/API dependencies. Fitted state stays serializable and feature extraction reproducible.

## PR 0 — Make experimental runs trustworthy

Dependencies: none.

Implement immutable candidate records and a run journal. Each candidate contains its text hash, parent candidate, scene/work ID, arm, iteration, generator identity, prompt hash, decoding settings, token usage, finish reason, and errors. Each evaluation references the candidate hash it actually scored.

Separate `original`, `current`, `best_feasible`, and `selected`. Keep the original as a valid no-change option. Returning the last iteration is not a selection strategy. Score the last generated candidate or return an earlier evaluated candidate; never attach an old score to new text.

Make baseline-relative checks independent of process lifetime: persist the original and relevant prior state, or explicitly retain one critic session. Verify this behavior rather than assuming a subprocess remembers it. Detect repeated hashes and oscillation; support cancellation/resume, schema validation, bounded retries, and cost limits. Count retries and failed paid requests in usage accounting.

Separate mechanical execution success, configured constraint status, and human quality judgments. Unknown is not pass. Do not stop a quality experiment because a stylometric gate passed. Use the same fixed revision budget in all arms, then choose among candidates using a preregistered development-time selector.

Acceptance tests:
- Every selected output has an evaluation bound to its exact hash.
- An exhausted one-iteration run cannot return an unscored rewrite.
- Resume does not duplicate successful requests or change recorded outcomes.
- The unchanged original is selectable.
- Failed or unparseable judge output is reported, not converted to a tie or loss.
- Any content-preservation baseline remains the original, not the immediately preceding draft.
- Blinded reports contain no arm labels or titles unless explicitly required.

## PR 1 — Corpus ingestion, provenance, and leakage control

Dependencies: PR 0 for run manifests; corpus adapter work can proceed independently.

Implement MovieSum first. Preserve source release/split, repository revision, source URL, download timestamp, raw content hash, parsed-document hash, work identity, document version identity, language, known translation/transcript status, and rights metadata. Keep source data immutable; derived artifacts are rebuildable.

Use local files plus SQLite metadata and JSONL or Parquet exports. No vector database, distributed crawler, or training infrastructure is necessary for this milestone. Use a network importer with explicit source configuration when implementing; do not execute remote dataset code by default.

Record separate permissions for acquisition, analysis, sending text to external annotators, fitting reference artifacts, retrieval/display, model training, and redistribution. An upstream dataset label does not automatically resolve underlying screenplay rights. Unknown permission stays unknown and blocks the corresponding operation until the intended use is reviewed. Keep approved research and redistributable/product corpora separate. See DATASETS.md.

Deduplicate exact raw and normalized forms, then identify near-duplicate scenes and scripts using token shingles. Confirm film identity with identifiers and source evidence rather than fuzzy title alone. Keep genuinely different drafts as versions within the same work group. Store uncertain matching decisions for review.

MovieSum includes ScriptBase-J; CML-Bench derives from MovieSum. Counts are not additive. Before fitting or retrieval, make a cross-source work/version map. Hold out entire works and all versions/derivatives. For broader generalization, add writer and franchise-disjoint evaluations; report the counts remaining when overlap makes these restrictive. Preserve upstream benchmark splits for external benchmark reproduction, but do not assume those splits are sufficient for our evaluation.

Outputs: locked corpus manifest, rights-use decisions, coverage census, duplicate/version map, data-quality report, frozen splits.

Acceptance tests:
- All imported records have source and content hashes.
- Every record either parses with documented coverage or is quarantined with a reason.
- No verified duplicate work crosses a split.
- Corpus census distinguishes scripts, drafts, conversations, transcripts, and summaries.
- Re-running ingestion is idempotent.
- Retrieval indexes contain only eligible training-side material.

## PR 2 — ScriptIR and parser validation

Dependencies: PR 1.

Represent scenes, ordered blocks, speaker turns, actions, parentheticals, transitions, and unresolved blocks. Preserve original text and source spans. A normalized dialogue view must have a map back to source spans; do not pretend offsets in a concatenated dialogue string are offsets in the input file.

Use stable within-version IDs plus explicit alignment tables across revisions. Text hashes identify content, not durable logical identity: an edited scene has a new hash but can align with its predecessor. Support scene splits/merges and uncertain alignments.

Keep presentation order, inferred story chronology, and unknown chronology distinct. Store simultaneous/dual dialogue explicitly or mark it unsupported. Speaker aliases and stage directions such as voice-over must not silently create different people. Do not collapse unknown speaker into a guessed character.

Begin with MovieSum's structured records and a declared Fountain subset. Add FDX next if needed, then heterogeneous HTML and text-based PDFs. Broad PDF/OCR ingestion is not on the critical path. Fountain reference: https://fountain.io/syntax/ .

Create a parser gold set of approximately 200 scenes across at least 40 films, stratified by source quality and layout. This set tests parsing; it is not a quality benchmark. Include title pages, credits, repeated headers, interruptions, parentheticals, and missing cues. The MovieSum preview itself exposes some title-page material assigned as character/dialogue, so structured data is not guaranteed gold.

Suggested acceptance targets after a development audit: at least 0.95 scene-boundary F1 and 0.98 speaker attribution accuracy on attributable turns for supported clean inputs. Report unknown rates, block-type macro-F1, unsupported cases, and subgroup results; never hide unsupported documents inside an overall average. These targets are project choices, not literature guarantees.

## PR 3 — Descriptive measurements and conditional reference profiles

Dependencies: PR 2.

Reuse existing core features separately for dialogue, action, and sufficiently long speaker collections. Add turn-length distributions, long-turn share, speaker share, alternation patterns, question/response cues, action-between-turn geometry, short-response runs, and explicit repetition measures. Do not infer dramatic quality from these counts alone.

Represent sequence features with transition counts or short motifs rather than bags of words only. Use opportunity-based denominators: transitions for transition rates, eligible exchanges for response patterns, and token counts only where appropriate. Missing opportunity is not zero. Estimate reliability at scene, speaker, and script scale separately.

Condition comparisons on declared medium, register, genre, scene purpose, language, translation status, and text length when sample sizes permit. Use hierarchical pooling or broad strata before sparse fine-grained combinations. Report sample counts and uncertainty. Retain distinct successful modes; do not move every script to one centroid.

For speaker differentiation, mask names and obvious topical entities, balance lengths, hold out scenes, and compare a modest style-feature classifier against shuffled-speaker and topic-only baselines. Do not maximize speaker-classification accuracy: catchphrases and caricatures can make the task easier while worsening the writing.

Outputs: deterministic profile artifacts, evidence spans, context-conditioned reference cards, length/missingness reports. No global `quality_score`.

## PR 4 — Scene annotations and mechanism retrieval

Dependencies: PR 2; uses PR 3 as available.

Use external, versioned annotation jobs. Start with a small ontology: requests, refusals, challenges, concessions, evasions, corrections, topic changes, declared/possible goals, evidence-backed facts, and audience-visible reveals. Keep literal claims, character beliefs, and story facts distinct; a character's lie is not automatically a continuity error.

Every label has evidence spans, annotator/model and prompt versions, document hash, and uncertainty/abstention. Changes invalidate dependent annotations. Do not accept a label solely because a model gives a confident explanation. Measure inter-annotator agreement and validate a human-checked subset. Low-agreement labels remain hypotheses or are simplified.

Generalize the comic-move index into problem/constraint/mechanism/effect cases. Retrieve solutions to a dramatic problem, not merely scenes about the same subject. Preserve contextual exceptions and source permissions. Compare mechanism retrieval with topical retrieval and random-within-register retrieval, matching exemplar count and length. Enforce title/version exclusions and copied-language checks.

Start by annotating a stratified sample rather than every scene in all 2,200 scripts. Cache by source hash + schema + prompt + model version. Expand with high-disagreement and high-impact samples, while retaining a random audit set to avoid active-learning selection bias.

## PR 5 — Independent validity and revision experiments

Dependencies: PR 0–4, but a human-annotated manual pilot can start before all automated components exist.

Build three distinct datasets:
- Parsing gold: structural labels only.
- Diagnostic validation: matched scene variants with specific controlled changes and human-adjudicated effects.
- Preference outcomes: actual candidate revisions from multiple generators plus human-authored/edited material where permission permits.

Mutation operators should include speaker/knowledge errors, explicit restatement, topic-disconnected responses, personality flattening, and trivial lexical/register changes. Also include valid repetition, emotionally necessary confessions, deliberately formal scenes, and unchanged controls. A mutation is an intervention, not automatically a negative label. Later real-world drafts are not automatically better than earlier ones.

Run the 48-case four-arm revision pilot specified in EVALUATION_PROTOCOL.md. Evaluate human preference, required-event preservation, reveal timing, voice, and style fit independently. Log quality-cost curves and rejection rates. Keep the final test panel and its judgments outside the agent loop.

Only after useful signal appears, fit a small regularized pairwise model on human judgments. Use low-dimensional interpretable features, partial pooling for context, grouped cross-validation, calibration, and abstention. Model coefficients explain a predictor, not the causal effect of an edit. Retain intervention experiments as the causal test of feedback utility. Do not begin with reward-model fine-tuning or reinforcement learning.

Go/no-go: promote only if the chosen system outperforms the equal-budget baseline in an untouched confirmatory set without unacceptable loss of required content, style diversity, or character distinction. If retrieval alone accounts for the gain, ship retrieval first. If improvement is uncertain, report uncertainty and improve measurements rather than expanding scope.

## PR 6 — Style-transfer validation

Dependencies: PR 5.

Implement separate `StyleSpec`, `PreservationSpec`, and `ChangeScope` contracts. Style-only transfer freezes plot events, relevant beliefs, promises, relationships, audience reveal order, and prohibited additions. More extensive changes require an adaptation contract. Some changes in tone legitimately alter emphasis; declare those allowances before evaluating.

Fit style references on eligible works distinct from the test scripts. Use styles defined by traits and reference sets, not just famous names. Separate spoken dialogue from action prose and preserve differences among characters. A character's private and public register can vary intentionally.

Generate a candidate using the same base writer as the prompt-only baseline; critique; make bounded revisions; validate against the original immutable contract. Select the best feasible evaluated candidate, not the last candidate and not the one with minimum stylometric distance.

Measure target-style recognition independently, substantive preservation, reader preference under the target brief, character distinction, and leakage from exemplars. Include identity transfer, reverse transfer, same-content/different-style controls, same-style/different-content controls, and a range of transfer intensities. Cycle consistency is a diagnostic, not proof of preservation.

First transfer benchmark: 24 source scenes × three target style specifications. Hold event contracts fixed. Expand to three-to-five-scene sequences, then a complete short script. Validate the entire output, not just changed lines, because a changed commitment can affect later scenes.

## PR 7 — New writing and long-form composition

Dependencies: PR 6 for validated local diagnostics, although a baseline generator can be built earlier.

Use a staged generation system: original brief -> multiple outline candidates -> selected outline -> scene contracts -> drafts -> bounded scene revisions -> global reconciliation. The authoring agent may see the complete plan, while simulated audience readers and characters only see checkpoint-valid information. Keep hidden evaluator questions out of the generator's context.

Test two generation tasks separately. With a fixed outline, evaluate scene realization and dialogue while holding narrative content stable. With only a shared premise, evaluate planning, character arcs, pacing, and ending as additional outcomes. Do not label reconstruction from a known film summary as evidence of original writing.

Maintain a source-grounded ledger of events, physical states, commitments, beliefs, knowledge access, and setup/payoff obligations. Allow unknown and changed states. Re-run affected downstream checks when a revision changes dependencies. Never silently truncate long scripts for the global evaluator: report coverage and provide full-script human reading for final trials.

Begin with original short-script briefs, then longer sequences and full features. Use fixed word/page budgets consistently; page counts require a controlled renderer. Evaluate final reading experience in addition to local metrics. Same-voice, counterbalanced read-aloud trials can be secondary evidence; performer choices are a confound, so text-only evaluation remains distinct.

## Deferred until evidence warrants them

General feature-film quality prediction; ratings- or box-office-trained reward models; large-scale fine-tuning; comprehensive mind-state graphs; automatic cross-cultural quality judgments; audiovisual judging; scraping every available archive; arbitrary PDF recovery; a proprietary vector service; and product claims based only on automated judges.

## First runnable milestone

A command imports eligible MovieSum records, parses and profiles a selected scene set, runs a four-arm equal-budget revision experiment, writes exact candidate/judge provenance, and exports a blind comparison packet plus an evaluation report. Its claim is limited to the experiment actually run. The corpus and evaluator code must be independently reusable even if the critic hypothesis fails.

See DATASETS.md for sources and access caveats, EVALUATION_PROTOCOL.md for measurement, and experiment.example.json for a non-executable draft contract requiring implementation and a locked data/model manifest.
