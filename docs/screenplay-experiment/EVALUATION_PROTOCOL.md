# Evaluation protocol: screenplay craft, transfer, and generation

Date: 2026-09-07. Proposed experiment design; no results are implied.

## 1. Evaluate three systems separately

**Measurement validity:** Did parsing and annotation recover what is on the page? Do diagnostics identify the proposed construct rather than a proxy such as formal vocabulary?

**Predictive validity:** Does a measurement or comparative judge predict independent human judgments on held-out works? Use grouped validation, report ties/unknowns, and calibrate confidence. Prediction does not establish that changing a feature improves the scene.

**Intervention utility:** Does giving this feedback to a writing agent improve its revisions beyond an equally funded baseline? This is the primary product question and needs randomized feedback conditions.

A universal screenplay-quality score is not required for any of these tests.

## 2. Three evidence sets

### Parser gold

Approximately 200 scenes across at least 40 films with scene/block/speaker labels and source spans. Include front matter and difficult formatting. Evaluate boundary F1, block-type macro-F1, attributable-turn speaker accuracy, unknown rate, and source coverage. Report per-source errors. This set must not silently double as quality training data.

### Diagnostic validation set

Use permitted original scenes and controlled variants. Candidate mutations include:
- A character acts on information not yet available to that character.
- A known event changes actor, polarity, quantity, place, or timing.
- A response becomes detached from the previous move.
- A line adds an explicit explanation of an already inferable emotion.
- Distinct speaker tactics are flattened while plot content stays fixed.
- A purely surface change adds contractions/slang without improving motivation.

Also include legitimate explicit confessions, ceremonial speech, repeated accusations, reminders, lies, misunderstandings, and intentionally unresolved exchanges. An explicit statement or repeated fact is not automatically wrong. Have readers judge the intervention; the mutation label alone is not a preference label.

Test natural failures as well as synthetic ones. Use multiple generation sources and human-authored variants so a detector cannot succeed merely by recognizing one model's output style. Keep independent held-out interventions, works, and writers where feasible.

### Preference set

Every case includes original draft, audience-visible preceding context, scene/genre/style brief, protected invariants, allowed changes, candidate revisions, provenance, and independent judgments. Collect preferences over variants of the same task. Comparisons between unrelated scenes cannot isolate revision quality.

## 3. Pilot: four equal-budget feedback arms

Use 48 scene cases spread over 48 distinct work/brief groups where feasible. These are development cases, not a confirmatory result. Include naturalistic, formal/theatrical, comic, and emotionally explicit registers; balance functions such as confrontation, information exchange, reconciliation, and concealed emotion. Several cases should be newly authored and unpublished to reduce recognition confounds.

Generate or choose one starting draft per case. Every arm receives the same draft, scene brief, model, decoding settings, and two rewrite opportunities.

| Arm | Feedback available |
|---|---|
| A | Strong general rubric and self-revision; no Handprint diagnostics or exemplars |
| B | A plus grounded Handprint measurements/hypotheses |
| C | A plus mechanism-matched exemplars; no Handprint diagnostics |
| D | A plus both diagnostics and mechanism exemplars |

Enforce the same total token/cost envelope, not merely the same number of calls. Retrieval and critique consume that envelope in the treatment arms; baseline may spend equivalent budget on self-critique/candidate exploration. Report all usage and quality–cost curves. Use the same preregistered candidate-selection policy in every arm and keep selection distinct from held-out evaluation.

The simple design produces 48 × 4 × 2 = 384 rewrite calls, excluding initial drafts, annotation, selection, and evaluator calls. Comparing B, C, and D with A produces 144 final-output pairs. Three independent ratings per pair means 432 pair ratings, not 432 independent scenes. Raters should not repeatedly see all versions of the same scene unless repetition is explicitly counterbalanced.

A separate equal-budget best-of-N baseline and topical/random retrieval controls are useful subsequent ablations. Do not dilute the first small pilot into dozens of inconclusive arms.

## 4. Independent judging

Use human readers as the primary outcome anchor. Include craft-trained readers and target-audience readers; report panels separately when they disagree. Automated judges can triage, diagnose, and scale development, but agreement with each other is not ground truth.

Randomize A/B placement, blind titles/authors/arms/model names, and normalize presentation without erasing genuine stylistic differences. Provide enough prior context, but no information from later scenes. Allow A better, B better, tied, and insufficient-context outcomes. Parsing/API failures are separate errors.

Separate three judging tasks:

1. **Audience inference:** Reader sees only the scene and audience-visible prefix. Ask what they infer and request evidence before revealing private intent. Do not tell this reader the hidden emotion the writer was instructed to convey.
2. **Constraint verification:** A different verifier may see the full preservation contract and inspect whether required events, reveal timing, and knowledge states survive.
3. **Preference/style fit:** Judge sees the declared target register and scene brief, but not the optimizer's scores or preferred answer. Ask which candidate better fulfills the brief and why.

Use evidence spans rather than ungrounded numerical ratings alone. Distinguish textual stage directions from filmable/audible cues: a parenthetical saying a character secretly feels guilt is not sufficient evidence that a viewer could infer guilt. Unknown inference is a valid answer. Intentional ambiguity should have a set or distribution of acceptable readings rather than one compulsory answer.

Validate automated judges on a held-out human-labeled subset, including A/B swaps, verbosity controls, positive formal scenes, and negation/event-preservation traps. Pin model versions and prompts. Avoid evaluating with the same prompt/rubric that generated the feedback. A different model is useful but does not eliminate shared biases or training contamination.

## 5. Outcomes and statistical analysis

Preregister one primary comparison before confirmation, normally combined feedback D versus strong baseline A. Use a scene-level score that gives a clear win 1, genuine tie 0.5, and loss 0; report wins, losses, ties separately. Insufficient-context and technical-error outcomes are not ties; report their rate and conduct sensitivity analysis rather than dropping them silently.

Use overall preference under the task brief as the primary endpoint with separate preservation and diversity guardrails. Also report the rate of feasible outputs and a composite success analysis so that high invalid-output rates cannot be hidden by judging only survivors. Failed generations count as operational failures under the preregistered end-to-end comparison policy; semantic uncertainty remains uncertainty.

Estimate uncertainty by resampling work/brief groups with paired arms intact. If multiple scenes share a movie, cluster at movie rather than line/turn. Reader effects may require a mixed model or multiway resampling. Do not treat three ratings of a scene as three independently generated scenes.

For feature modeling, use a small regularized pairwise/logistic or additive model, with context effects and partial pooling as data permits. Handle genuine ties explicitly. Use source-grouped cross-validation; never tune and report on the same preferences. Correct or label exploratory multiple comparisons.

The 48-case pilot checks instrumentation, false positives, judge agreement, and plausible effect size. It is not sufficient to establish small gains. Under an idealized independent, binary, tie-free preference model, detecting a true 60% preference against 50% with two-sided 5% significance and 80% power requires roughly 200 cases; detecting 55% needs roughly 800. These are normal-approximation planning values, not a final sample-size prescription. Clustering, ties, missingness, and multiple comparisons change requirements. Simulate power using pilot estimates before locking the confirmatory sample.

Use a fresh approximately 200-case confirmatory set only if pilot-derived power supports it; increase it for smaller expected gains. Never repeatedly peek and stop on significance without a preregistered sequential design.

Proposed promotion criteria: a positive primary effect with a confidence interval excluding no benefit, acceptable predefined content-preservation noninferiority margin, and no material deterioration in target-style fit or diversity. An example preservation margin is five percentage points, to be reviewed and frozen before confirmation. Hard factual violations can have a stricter per-output rule. These are choices, not established universal cutoffs.

## 6. Style-transfer experiment

Use three separate contracts:
- **StyleSpec:** traits/reference groups, strength, intended register, narrator/action versus character-voice scope.
- **PreservationSpec:** required events, identities, commitments, relations, beliefs, reveal timing, and forbidden additions.
- **ChangeScope:** lexical/syntactic changes only, dialogue-and-action realization, or adaptation with explicitly permitted structural changes.

Example: a mechanic refuses an unsafe launch; the captain still launches; concern must remain inferable; neither character acknowledges the personal relationship. A laconic version may change wording and filmable action, but cannot delete the safety warning, turn refusal into permission, or reveal the relationship early.

First task: 24 scenes × three target registers, then expansion after validity checks. For every transfer report:
- Independent target-style fit, not only Handprint distance.
- Required-content retention and contradiction checks, grounded in spans.
- Audience-inference and reveal-order preservation.
- Naturalness/dramatic effectiveness under the target brief.
- Character differentiation and exemplar overlap.

Do not maximize one aggregate style score. Stronger stylistic change can legitimately lower lexical similarity. Embedding similarity or BLEU/ROUGE cannot certify preservation of actor, negation, intention, or knowledge. The research in [Mind the Style Gap](https://arxiv.org/html/2502.15022v3) specifically cautions that test-data construction can make unsuitable preservation metrics look reliable.

Controls: identity transfer should not force changes; reverse transfer should work where the request is meaningful; same content in different styles should remain content-compatible; same style with altered facts should fail preservation. Use intensity sweeps. Round-trip transfer is useful but not proof, because both directions can omit the same fact.

Successful operation: the target style is recognizable, required content remains, the scene works, and no unauthorized copying is detected. Return qualified findings for uncertain constraints rather than declaring a universal pass.

## 7. New-writing experiment

Run two distinct tasks:

**Fixed-outline realization:** Same outline, scene goals, cast constraints, and length budget. Compare prompt-only scene writing with validated critic-assisted writing. This isolates realization from planning.

**Open-premise writing:** Same original premise and production constraints, but independently generated outlines. Evaluate plan quality, payoff, character development, scene necessity, pacing, and ending as well as dialogue. Do not compare these scores directly with fixed-outline results.

Progression: individual scenes -> three-to-five-scene sequences -> complete short scripts -> feature-length work. Carry forward source-grounded event and knowledge ledgers. Check both missed constraints and unjustified invented constraints. Keep planned canon and realized text separate; realized text can intentionally deviate only through an explicit update to the plan.

For global evaluation, read the complete work rather than only summaries. Evaluate setup/payoff timing, causal continuity, character change, repetition across scenes, and whether quieter scenes serve the work. A local average can miss a failed ending. Report evaluator context coverage and uncertainty; do not silently truncate.

Use held-out, newly authored briefs. Public-movie reconstruction risks memory leakage and should be labeled a reconstruction task. Renaming familiar characters is only a weak contamination control. Include a replication with a second writer model or human-edited drafts before claiming the method generalizes.

## 8. External resources as baselines

[ExpressivityBench](https://aclanthology.org/2026.findings-eacl.235/) motivates implicit-communication probes. [CML-Bench](https://arxiv.org/abs/2510.06231) supplies screenplay-oriented coherence/consistency/reasonableness baselines. [STAGE](https://arxiv.org/html/2601.08510v9) provides checkpointed state and knowledge tasks. [LitBench](https://arxiv.org/abs/2507.00769) informs creative-preference evaluation. None alone validates this tool's usefulness; each needs integration checks and overlap controls.

## 9. Results artifact

Every report should contain corpus and model locks; protocol hash; sample flow and exclusions; parsed input coverage; raw A/B judgments; reader/judge agreement; effect size and confidence interval; failures and abstentions; content-preservation outcomes; per-style performance; copied-language flags; iteration and cost curves; and examples with evidence where display rights permit.

Mark every number as observed, planned, or simulated. Do not publish plausible placeholders. Preserve the result even when no advantage is detected.
