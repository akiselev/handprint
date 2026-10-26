# Docs 2 — gap analysis: what DOCS.md missed (2026-10-26)

Second research round on `DOCS.md`, in the shape of `WIT2.md`. Six parallel passes
looked for known methods and algorithms that DOCS.md lacks: (A) documentation quality
and smells, (B) keeping docs true to the code, (C) factual consistency and LLM judges,
(D) agents as readers, (E) GUI and CAD documentation, (F) terminology, controlled
language and information architecture. The full reports, with every URL, are in
`research/docs/`. Everything below passed a redundancy check against DOCS.md, WIT2.md
and the trees; items found to be covered already are listed at the end. Item numbers
continue DOCS.md's (2a.10, 2b.14, …) so the two documents can be merged later.

## Verdict

Five structural findings, each from at least two passes:

1. **DOCS.md checks names, not statements.** The resolver (2b.10) proves that
   `sketch.constrain.coincident` exists. A page that says it "makes two curves
   tangent", or that a default is 1e-6 when it is 1e-9, passes. The question bank
   (2c.15) runs in the *recall* direction only (questions from the registry). Every
   method in the factual-consistency and doc–code literature also runs the
   *precision* direction: each claim the page makes, checked against the source
   (B, C). The precise systems all restrict themselves to narrow, typed claims:
   Jdoctor 92% precision, DocTer 85%, MPDetector 93%. DocPrism found that an
   unrestricted LLM flags 98% of functions.
2. **The task agent, as written, can pass an empty page and tests the wrong
   route.** An agent with MCP gets every command id and schema from
   `commands.list`. AppAgent agents succeeded 48.9% of the time with no doc at all
   (D). And a scenario is command-level NDJSON, so a GUI how-to whose menu path
   does not exist still replays (E). The user ruled on 2026-10-26 that UI QC uses
   "proper GUI interaction", not RPC or MCP (`cadatomic/tools/gui-qc/README.md`).
3. **The defects practitioners rank worst are content, not style.** Ambiguity,
   incompleteness and incorrectness lead Uddin & Robillard's ranking (323
   developers). In Aghajani's survey (146 practitioners), 88% rate clarity
   important and only 33% format. The next "how" issues are superfluous content
   (55%) and duplicated content (46%), and DOCS.md has no detector for either. A
   many-agent doc set produces duplicates first (A).
4. **Much of what DOCS.md planned to build already exists in the trees.**
   cadatomic's tour design has a doc mode (TU8: pages with annotated screenshots,
   SVG callouts from captured rects, perceptual diffs, approvals), deferred in
   batch 9 (E). `docs/behavior/observed/` holds digest-pinned outcome sentences per
   trigger (E). `ui/tours/concepts.ts` is a prerequisite graph of 101 concepts
   (F). `en.ftl` has 56 `*-synonyms` search aliases (F). rustdoc JSON is the
   registry DOCS.md said cadabra3 lacks (B). Each command-log record carries a
   `CommandSource` (`Gui`, `Rpc`, …), which makes surface checks free (D).
5. **LLMs are weak at every fine-grained judgement the gate needs.**
   - smell detection: macro-F1 0.41, against 0.75–0.97 for a fine-tuned BERT (A);
   - picking the step that caused a failure: 14.2% (D);
   - deciding whether a GUI workflow was completed: F1 below 0.3 (WONDERBREAD, E);
   - fact checking: near chance on SummEdits (C);
   - findability: ChatGPT-4 passed 10 of 10 tree-test tasks where humans averaged
     about 51% (F).

   **Rules and execution decide; LLMs propose.** That is the design rule this round
   adds to DOCS.md.

## 1. Truth: from names to statements (2b, gate)

- **2b.14 Typed claims.** The fact sheet stores claims as `(id, attribute, value)`
  with a type, and a deterministic check per type:

  | Type | Example sentence | Checked against |
  |---|---|---|
  | `default` | "Snap defaults to on" | Arg schema `default`, config tables, clap `default_value` (R's `codoc` does this for function defaults) |
  | `unit` | "Clearance is in millimetres" | Copperfield `i64` nm, cadabra3 metres, display units from the catalog |
  | `range` / `enum` | "Accepts 1–64 layers" | JSON Schema `minimum`/`maximum`/`enum` (DocTer's constraint classes) |
  | `requires` / `order` | "Run DRC before export" | Command preconditions, tour order (iComment's rule class) |
  | `shortcut`, `menu-path` | "Press Ctrl+K" | `inventory.toml` key bindings and menus |
  | `error` | "Fails with `ecad.busentry.member` when…" | Diagnostic catalog and its trigger test |
  | `count` | "There are four sketch tools" | Registry query |

  Most cross-page contradictions (2c.17) then become equality checks on the same
  `(id, attribute)`: free and deterministic. NLI handles free-text pairs that share
  an id, and an LLM only the residue.
- **2b.15 Sentence→claim alignment.** Every sentence that carries a number, unit,
  quoted value, key, id or ordering word must map to a fact-sheet claim. An
  unbacked sentence is a finding with a span; a contradicting one fails the page.
  Because our fact sheet is structured, this is alignment, not the open-domain
  decomposition FActScore and SAFE need. Score only verifiable sentences
  (VeriScore): orientation text ("This page shows how…") neither passes nor fails.
  Pin and version the decomposer, as #26 says for the normaliser.
- **2b.16 Entailment check.** A small local verifier confirms each aligned sentence
  against its registry entry and source snippet:
  - MiniCheck-Flan-T5-Large (770M; code Apache-2.0, weights MIT) reports GPT-4-level
    accuracy on LLM-AggreFact at about 400× lower cost;
  - AlignScore (355M, MIT) runs on CPU.

  Both were trained on news and Wikipedia. Calibrate them on our own mutants (§5)
  before trusting a threshold. Do not use Bespoke-MiniCheck-7B (CC BY-NC).
- **Sentence anchors and citation scoring.** Carry the fact-sheet binding into the
  rendered page as hidden `<!-- ids: … -->` anchors per sentence, the per-sentence
  form of DOCS-PLAN's `sources:` front matter. The gate computes ALCE-style:
  - **citation recall:** every verifiable sentence cites at least one id, and the
    cited entries entail it;
  - **citation precision:** no decorative ids.

  ALCE's automatic version agrees with humans at κ 0.70 for recall. The anchors also
  make staleness per sentence instead of per page.
- **2c.18 Behavioural claims, test-confirmed.** For claims no registry holds
  (cadabra3's "returns a closed shell when…", tolerance behaviour), a held-out agent
  writes a test from the sentence alone and runs it against the real crate. This is
  CASCADE's design (FSE 2026, run on Rust repositories among others). @tComment and
  METAMON confirm the same way. Only reproduced failures count, and a reproduced
  failure is a hard finding, because the test decides it, not the LLM. For cadabra3,
  a differential test against Parasolid is also available. Why it matters:
  - even GPT-4 wrote demonstrably inaccurate statements in about a fifth of 540
    checked comments, and nine existing detectors showed no relationship with
    accuracy (arXiv 2406.14836);
  - wrong docs hurt LLM code understanding while missing docs do not
    (arXiv 2404.03114). That matters most for the MCP/RPC surface, whose readers
    are agents. Where unverified prose fails, ship the generated reference without
    prose.

## 2. The task agent, rebuilt (2c.14)

DOCS.md's central critic stays, with seven changes:

1. **Three arms, not one.**
   - *Page:* the agent reads the page.
   - *Null:* the agent gets no page. The page's score is its **lift**,
     pass(page) − pass(null).
   - *Oracle:* the agent gets a page generated mechanically from the tour's own
     `hint` lines. If the agent fails the oracle, the failure belongs to the agent or
     the app's accessibility, not the page.

   Report three failure classes: page defect, accessibility defect (filed to qc), and
   agent grounding failure.
2. **GUI pages run through `gq` only.** Use the AT-SPI tree filtered to the active
   window, plus a screenshot, with no `command_run` and no MCP. OSWorld agents did
   best with the accessibility tree (read through pyatspi, as `gq tree` is), and
   ScreenSpot-Pro's CAD category was among the weakest for pixel grounding. So target
   controls by role and name through gq's hit-checked clicks, never by coordinates.
   API pages run through the scenario runner, CLI pages through a shell.
3. **Check the trajectory, not only the end state.** A GUI how-to passes only if the
   command log's invocation records come from `Gui`/`ActionBar`, not `Rpc`
   (`cadatomic-core/src/log/mod.rs:62`), and match the commands its steps imply.
   ImpossibleBench documents exactly this kind of shortcut.
4. **Repeats.** At least three runs per page, reported as pass^k. On τ-bench, pass^8
   fell below 25% where single-run success was near 50%.
5. **A reader ladder.** A weak model is the gating reader and a strong one is
   informational. Use a different family from the writer. Strong models fill gaps
   from prior knowledge, and models cannot be prompted into playing a weaker reader
   (arXiv 2504.06460), so give the reader real limits instead: a smaller model, a
   token budget, no introspection.
6. **Cost.** Record steps and tokens against the tour's reference trajectory, and
   fail outside a band. Fuller MCP descriptions raised success by 5.85 points but
   steps by 67%, and made 16.7% of cases worse (arXiv 2602.14878).
7. **Two navigation modes.** Run once with search and free browsing (does the page
   work?) and once with only the navigation tree (can it be reached?). The second is
   a scent signal, never a claim about human findability.

**Localise failures deterministically.** Work in three tiers:

1. the first tour checkpoint whose app state diverges;
2. aggregated over repeats and readers ("4 of 6 runs diverge at step 3");
3. on demand, ContextCite-style section ablation, only for pages that fail the gate.

The writer receives a bank-free summary: the section, a failure class and a count.
Use Aghajani's issue labels plus missing prerequisite, ambiguous referent,
non-canonical name, missing expected result and step order. Never the task text.
Keep each run as a chaptered recording for reviewers in qc (AXNav's output shape).

**Grow the bank adversarially.** PLAY2PROMPT rewards generating tasks the current page
fails. DRAFT keeps new probes below 0.9 cosine similarity to earlier ones. Add
"changed behaviour" questions, where the registry's answer differs from the obvious
guess: CodeUpdateArena found that putting an update's docs in the prompt did not get
models to use the changed behaviour, so priors beat a contradicting page. Write
questions in the user's goal vocabulary, seeded from tour `keywords` and the palette
aliases. Tag each with one of Grossman et al.'s five learnability categories (task
flow, awareness, locating, understanding, transition), and cover all five per area.

## 3. GUI pages come from tours (Layer 0, 2b.11)

- **Replace 2b.11's GUI clause with: "GUI how-tos are tours; their pages come from
  TU8."** One tour script already renders as test, guided, demo, doc and video
  (`cadatomic/docs/research/2026-10-23-tours-tests-tutorials-videos.md` §0.1). Doc
  comments on `tour`/`chapter`/`step` are the text. Test mode proves the taught route
  on every build. This is "record, then write": the generated tutorials that beat
  hand-made ones (Grabler et al., 20–44% faster with 60–95% fewer errors) started
  from a recorded demonstration with application state. TU8 becomes a priority-1
  dependency.
- **DITA's task schema maps one to one onto a tour:**

  | DITA | Tour |
  |---|---|
  | `prereq` | `setup` |
  | `context` | chapter prose |
  | `cmd` | `do` |
  | `stepresult` | `expect` |
  | `result` | the final checkpoint |

  Use these slots as the doc-move labels in 2d.19, so every move has an industry
  meaning and a predicate behind it. Check the order deterministically (2b.17): the
  prerequisites come before the steps, each step has exactly one action, and the
  action verb opens the clause. A goal-first clause ("To open X, click Y") is fine.
- **Feedback where it is needed, not at a rate.** Farkas's step model says a feedback
  sentence is needed only when the system's response is non-routine or hard to
  notice. Replace 2a.4's fitted result-statement rate with a lint over `expect`:
  - require a feedback sentence where `expect` names state the reader cannot see at
    the target (a DRC marker elsewhere, a lineage change);
  - flag one where `expect` is only `visible(next field)`.
- **Error recovery is required, and generated.** Manuals carrying detection,
  diagnosis and correction information made learners finish faster with better
  corrective skill (Lazonder & van der Meij 1995). Minimalism is the best-replicated
  result in this round. Commands return a typed `Refusal { reason, slot, message }`
  and diagnostics have a catalog, so a step whose command can refuse gets a generated
  `steptroubleshooting` line ("Accept stays grey until all three sizes are
  positive"). Every procedure carries at least one recovery move.
- **Figures.**
  - Full-window shots with a numbered callout and a zoomed inset for small targets,
    never a tight crop alone. Full captures beat partial ones for learning
    (Gellevij & van der Meij).
  - Short clips only for drag and gesture steps such as routing and sketch drags
    (MixT).
  - **Figure–text agreement (2b.18):** each figure keeps the AT-SPI snapshot taken
    with it, and every UI label the surrounding text names must appear in that
    snapshot. Regenerated screenshots move the staleness risk into the prose around
    them. DOSUD found 17 outdated screenshots in VS Code's docs, and VS Code counts
    more than 210 issues about them.
- **Generated reference boxes, Blender style, with two-way coverage.** Each command
  page opens with mode, menu and ribbon paths, shortcut, arguments, refusal reasons,
  "what happens" (from the behaviour catalog's `outcome`) and "learn this" links to
  the tours that route through it.
  - **Coverage in both directions.** Blender's `bl_rna_manual_reference.py` checks
    that every operator has a manual entry, that every entry maps to a real operator,
    and that every URL resolves. Read it for the idea only (GPL). DOCS.md 2b.12
    states one direction.
  - **Panel pages** get an anatomy figure numbered from the panel's AT-SPI subtree.
    An unnamed control fails generation, which doubles as an accessibility check.
- **One source, several outputs.** The same Layer 0 data generates tooltips,
  extended tooltips and "learn this" links. ToolClips (tooltips extended with short
  clips) got 7× as many unfamiliar tasks done as a commercial help system, with
  retention a week later. LemonAid's help keyed by UI element answered 90% of
  requests, and cadatomic has the anchors LemonAid had to infer.
- **Behaviour catalog as a Layer 0 source.** `docs/behavior/observed/*.toml`
  outcomes are pinned to digests, so they give staleness for free. An outcome tied to
  an open `finding` is a bug, and no page may document it as intended behaviour.

## 4. Content defects (2a, 2b)

- **2b.19 Near-duplicate detection.** Run shingled MinHash or suffix arrays across all
  prose pages and against the generated reference pages, and report pairs above a
  Jaccard threshold with spans. Luciv et al. found exact and near duplicates in all
  19 projects they tested. A prose duplicate of a generated fact breaks "facts are
  generated". Near-duplicates that disagree are the likeliest contradictions, so feed
  them to 2c.17. This is the cross-page counterpart of 2a.2.
- **2a.10 Rustdoc smells.** Rustdoc is the prose agents will write in the largest
  volume. Khan et al. (SANER 2021) scored single-threshold rules on 1,000 labelled
  units:
  - *Lazy:* the doc is nearly the item name plus its signature ("Returns the foo" on
    `fn foo`). Levenshtein rule F1 0.95, and practitioners call it the most damaging.
    It is 2a.2's novelty ratio applied to `///`.
  - *Bloated:* longer than a band fitted per item kind on the `api-reference`
    sub-register. F1 0.90 with the 90th percentile as the bound; the fitted band makes
    it two-sided.
  - *Buried directives:* sentences matching Monperrus et al.'s directive patterns
    (must, never, only, before, thread, lock, …) outside `# Panics`, `# Errors` or
    `# Safety`, with a "move to section" fix. Krec recovered such fragments at about
    90% precision. The pattern list ships as a pack.
- **2b.20 Rust doc lints as deny in the docs gate.** Configuration only:
  - clippy: `missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`,
    `too_long_first_doc_paragraph`, `doc_markdown`, `empty_docs`;
  - rustdoc: `broken_intra_doc_links`, `missing_crate_level_docs`.

  Together they mechanise the Rust API Guidelines' C-EXAMPLE, C-FAILURE and C-LINK
  rules.
- **2a.11 Code examples.** Code examples are the resource developers seek most
  (Robillard & DeLine, 440+ developers), and the explanation around an example matters
  as much as the example (Nasehi et al.). Five checks:
  - **explained-example ratio:** prose around each block, two-sided;
  - **example scope:** API ids per example, resolved through Layer 0. One is "of
    little value"; many is tangled;
  - **block-action markers:** every file block names its path and its action
    (create, replace, append); no `...` without a marker; placeholders declared;
  - **copyability:** no prompts mixed with output, and no multi-platform blocks;
  - **clean-container replay** for install and tutorial pages. Docable ran 26% of
    14,876 tutorial blocks, and no tutorial ran end to end; environment assumptions
    and missing steps were the main causes.
- **2a.12 Front-loading and self-containment.**
  - The first sentence of a page or section carries a fact-sheet id or a glossary
    term, and headings carry a domain term in their first two words. Users read about
    20% of a page's words (Nielsen).
  - A band on the tokens before the first action in a how-to.
  - A pack of backward-reference phrases ("as described above").
- **Diagnostics text at the source.** DOCS.md says diagnostics are critiqued at the
  source but gives no criteria. Every code has a fix or hint field: svelters has
  fix-its, and Copperfield's string literals don't, which is one more reason for the
  registry request. Add a message-length band and a blame-word pack ("illegal",
  "fatal", "you must"). Becker et al.'s ten error-message guidelines are hypotheses,
  not established effects.

## 5. Critic hygiene and anti-Goodhart (gate)

- **2c.19 Mutation-tested critics.** Inject typed mistakes from the registries into
  pages that already passed:
  - swap a command id for a sibling;
  - change a default, a key binding or a fix;
  - drop a prerequisite step;
  - reorder two steps;
  - use a deprecated term.

  Every critic gets a detection rate per mutation class. The gate's sensitivity
  becomes a number, redundant critics show themselves (keep the cheaper one), and the
  mutants are the calibration set for the entailment threshold (2b.16). SummEdits
  built its benchmark the same way by hand; ours comes free from typed registries.
  This is the doc analogue of the canary family.
- **Gate on yes/no rubric leaves; keep pairwise for information.** Tripathi et al.
  (COLM 2025) found pairwise verdicts flip in about 35% of cases with distractor
  features embedded, against about 9% for absolute scores. This narrows the "pairwise,
  never absolute" rule DOCS.md took from TTCW: pairwise remains right for creative
  quality, but a gate facing a writer that iterates needs binary leaves. Generate the
  leaves from the coverage matrix (each required section, id, refusal and example),
  in CodeWikiBench's shape but without its generated rubrics, which agreed across
  models only about 74% of the time. Keep pairwise-against-anchor as informational.
- **Judge rules:**
  - The judge is a different family from the writer. Self-preference scales with
    self-recognition (Panickssery et al.). Prometheus 2 (Apache-2.0, open weights) is
    a local option.
  - Judge both orders, and count a flip as a tie.
  - Record the length ratio, and control for it. A length-controlled GLM raised
    AlpacaEval's Spearman with Chatbot Arena from 0.94 to 0.98.
  - Loop judge ≠ gate judge, and rationales are hidden from the writer. In-context
    reward hacking gets worse when generator and evaluator share a model (Pan et al.
    2024).
  - Log loop-judge vs gate-judge divergence per page, as the hacking signal.
  - Calibrate every LLM critic on null pages (template only, truncated, an anchor
    with its facts swapped). A constant null output reached an 86.5% win rate on
    AlpacaEval 2.0.
- **Split the banks.** A feedback split produces failure summaries; a gate split
  produces only pass/fail and is rotated per batch. Anthropic's tool-writing guide and
  GEPA both gate on held-out tasks.
  - Stop on convergence and **keep the best version, not the last**: DRAFT's
    doc-refinement curve rises, then falls.
  - Use GEPA-style Pareto selection, which keeps revisions that win on different
    tasks rather than one scalar, and is a direct answer to #24's report-chasing.
  - At the pilot, test transfer: pages tuned on tasks are scored on questions, and
    the reverse. Arman & Molybog (2026) optimised a round-trip doc benchmark to full
    fidelity and the gain did not transfer to real issues.
- **Lexicographic gate, not a weighted sum.** First correctness: 2b at 100%, the
  entailment check, and task-agent lift on the gate split at both reader rungs. Then
  readability: the doc gate mode and the rubric leaves. A weighted sum lets a high
  agent score buy off a wall of text.
- **A skim reader.** A reader agent sees only headings, first sentences, code blocks
  and figures, and must find the section that answers a question. It is a cheap
  machine proxy for the 79% of people who scan (Nielsen). Untested; worth a pilot arm.

## 6. Terminology and structure (2a.5, 2d)

- **Replace 2a.5's synonym table with a concept-oriented, domain-scoped termbase.**
  The risk is polysemy across repositories, not synonymy. In `en.ftl` today:
  - "component" is a Svelte component (`cmd-designer-insert-tooltip`) and an ECAD
    part (`ecad-kind-ecad-component`);
  - "part" is an MCAD document (`cmd-file-new = New part`).

  One record per concept, in SKOS or TBX shape:
  - `id` and `subject_field` (`mcad`, `ecad`, `ui`, `kernel-api`, `cli`, `tracker`);
  - `pref`: a catalog message id;
  - `admitted[]` and `deprecated[]`, with a replacement;
  - `hidden[]`: the 56 existing `*-synonyms` aliases, which search accepts and pages
    must not use;
  - `definition` and `prerequisites`, the latter from `concepts.ts`.

  The checker resolves each match by the page's subject field, so it can tell
  wrong-domain use from a variant. Furnas et al. are the reason for keeping two
  lists: two people chose the same term with probability below 0.20. So writing needs
  one preferred label and finding needs many aliases.
- **UI labels as message-id shortcodes** (`{{ui:cmd-file-new}}` renders "New part").
  Nearly free, and it turns label resolution, terminology, label staleness and
  translated docs into exact lookups against `en.ftl`. Pass F's highest-leverage item.
- **Concept-split detection at consolidation.** Many parallel writers each pick a name
  for the same concept. Flag two distributionally similar terms that appear on
  disjoint sets of pages or writer runs (a term × writer contingency table). No LLM;
  the terminology analogue of the churn guard.
- **Term extraction for words no registry holds** (fillet, courtyard, annular ring):
  - C-value/NC-value: nested-term correction, parser-free;
  - TermSuite's variant-rule shapes (port the idea, not the Java and TreeTagger);
  - Schwartz–Hearst for acronyms: 96% precision, 82% recall, no training. This makes
    2a.6 exact for DRC, ERC, SSI and MCP.

  The output is a glossary work queue, not truth.
- **Order pages by `concepts.ts` (changes 2d.18).** Extend it to MCAD, the kernels and
  skelter, since "sketch" does not appear in the catalog yet. Have an LLM propose
  edges and the coordinator approve them.
  - **Prerequisite-before-use check (2b):** a page that teaches X links to, or comes
    after on a tutorial path, the page defining each prerequisite of X.
  - **RefD audit over the built site's links** (Liang et al., EMNLP 2015): a high-RefD
    pair with no declared edge is a missing prerequisite, and a declared edge with
    negative RefD means the pages teach in the opposite order.
- **Navigate by domain; use the four page types inside it.** No empirical evaluation
  of Diátaxis was found, only adoption reports. The one observation study of
  developers using API docs (Meng et al.) recommends organising content by function,
  not by information type, and folding concepts into the tasks that need them. Keep
  the types for fitting rates (DOCS.md 1a) and for a per-type section schema, which
  is a deterministic pass/fail.
- **Mixed-mode detection.** Classify each section with the per-type handprint
  references; character n-grams classify genre best (Sharoff et al.). An explanation
  section inside a how-to fails, with the fix "move it to the concept page and link
  it". The same nearest-reference test rejects a draft whose declared type does not
  match. This adds a mode to `critique` and costs nothing beyond the refit.
- **Findability without trusting an LLM.** A deterministic scent check (Cognitive
  Walkthrough for the Web): for each task-bank goal, rank navigation labels and
  headings by similarity to the goal. Flag weak scent (the correct heading is not the
  top match) and competing headings (a sibling within ε). In Blackmon's data,
  competing headings alone explained 44% of the variance in time to find a page. Add
  site-graph metrics: orphan pages, click depth, link-text vs target-title agreement,
  and reference pages with no inbound how-to link. Later, zero-result palette queries
  become `hidden` labels and work items.
- **A global-English pack**, small on purpose:
  - noun stacks over three words;
  - omitted *that*;
  - ambiguous *-ing*/*-ed*;
  - phrasal verbs where one verb exists;
  - coordination chains;
  - per-step length (20 words procedural, 25 descriptive).

  Simplified English helped most on hard documents and for non-native readers
  (Chervak et al., 175 technicians). But eight corporate rule sets shared only one
  rule (O'Brien), and with neural MT explicitness beat brevity. Mine it from Vale's
  MIT Microsoft and Google packs, and describe the STE rules in our own words: ASD
  holds the copyright, so ship nothing of STE.

## 7. Keeping it current (Layer 3.7)

- **rustdoc JSON as the kernel registry.** `rustdoc --output-format json` emits every
  item with docs, resolved links and `deprecation`. `cargo public-api` (MIT) diffs the
  public API across refs. That gives cadabra3 and Copperfield's facade the Layer 0
  table cadatomic gets from `inventory`, and the per-consolidation diff is the
  deleted/renamed list below. It needs nightly rustdoc.
- **Diff-triggered staleness.** On every lane diff, flag sentences and doc comments
  that mention an identifier the diff deleted or renamed, in prose as well as code
  spans. This one rule scored F1 68 in Panthaplackel et al. (AAAI 2021), against
  77–81 for the deep models, and just-in-time detection beat post-hoc detection by
  over ten points. It catches names that still resolve because something else took
  them, and names written as words.
- **Behavioural digest: "bless → stale".** A page depends on the goldens, tour end
  states and trycmd outputs it shows, not only on registry entries. When a lane
  re-blesses one (CLAUDE.md already requires lanes to list them), every page bound to
  it is queued for prose review. Pixel diffs above a threshold queue the prose around
  the figure; Mesa-only diffs are filtered, following cadatomic's existing rule.
- **Derive the owner map.** svelters' `OWNERS.toml` + `check_doc_touch.py` is a
  hand-written version. Derive it instead from each page's claim bindings, plus
  AdDoc-style co-change mining over git history. AdDoc found at least half of all doc
  changes in four Java projects followed existing documentation patterns.
- **Deprecation entries never vanish.** A renamed command id, flag, RPC method or code
  keeps `deprecated {since, replacement}`, so the resolver reports a `consider_replace`
  instead of an unresolved name. A deprecation without a replacement is itself a
  registry finding. A median of 67% (Java) and 78% (C#) of deprecated elements carry a
  replacement message (Brito et al.).
- **Execute the page itself.**
  - CLI pages for Copperfield and qc: trycmd takes the page's own `.md` as the test
    case, and `TRYCMD=overwrite` regenerates the shown output, so the agent writes the
    command and the prose, never the output.
  - RPC and MCP: every JSON example tagged with a method validates against its JSON
    Schema, and Elixir-style `→ request` / `← response` transcripts replay against
    `cadatomic-headless`.
  - Doctest hygiene: no `ignore` without a reason, and no high share of hidden `#`
    lines.
- **Stable heading ids and offline fragment checks.** 2a.1 rewards renaming
  "Overview", and every rename changes the slug and silently breaks inbound
  `#fragment` links. Give headings explicit ids (ids, never labels) and run lychee
  (MIT/Apache) in offline fragment mode in the lane gate. External links run nightly
  with the cache.

## 8. Agent-facing documentation (new surface)

The MCP tool descriptions and command catalog text are docs that agents read, and
DOCS.md never runs an agent against them. Two studies:

- Hasan et al. (2026) found 97.1% of 856 MCP tool descriptions have a defect, and 56%
  have no clear purpose.
- Wang et al. (2026) found descriptions that followed their guidelines were selected
  72% of the time against 20% for alternatives. Functionality and accuracy mattered
  most.

Apply a description rubric at the source (purpose, when to use, parameters with
meaning and units, return shape, limits, one example, kept compact). Then run a
**tool-selection test**: given a task, does the agent pick the right command id from
the catalog? It is cheap, and it runs before full tasks.

Generate `llms.txt` and `.md` mirrors from the page inventory. They cost nothing, but
the only gains measured come from a vendor benchmark, so claim nothing until our own
retrieval test runs. Keep any agent guide minimal: AGENTS.md-style context files did
not raise SWE-bench success and added more than 20% cost; they help only when they
state non-standard practice (Gloaguen et al., ICLR 2026).

## 9. Priority order (revises DOCS.md's)

1. **Free and deterministic, mostly already in the trees:**
   - the Rust doc lints;
   - UI-label shortcodes;
   - stable heading ids with lychee;
   - rustdoc JSON with `cargo public-api` as the kernel registry;
   - the deprecation-entry rule;
   - diff-triggered staleness and "bless → stale";
   - JSON Schema validation of RPC/MCP examples;
   - `llms.txt`.
2. **TU8 doc mode, with tours as the source of GUI how-tos.** Add the generated
   reference boxes with two-way coverage, the figure–text agreement check, and step
   lints over `expect` (feedback where hidden, generated troubleshooting).
3. **Typed claims, sentence alignment and the entailment check**, then registry
   mutants to measure them and every later critic.
4. **The rebuilt task agent:**
   - null and oracle arms;
   - gq only, with the `CommandSource` check;
   - at least three repeats;
   - weak and strong readers;
   - split banks;
   - deterministic localisation.
5. **The termbase** (with `concepts.ts` ordering and the prerequisite-before-use
   check), near-duplicate detection, and the rustdoc smells.
6. **The rest of the in-crate Doc family:** code-example checks, front-loading, the
   global-English pack, mixed-mode classification, the scent check and site-graph
   metrics.
7. **Then the pilot,** with two added arms: (d) the null page and (e) the weak/strong
   reader split, and a transfer check across task and question families.

## Open questions for the user (added to DOCS.md's)

- **TU8 was deferred in batch 9.** This plan makes it the GUI docs pipeline.
  Recommendation: schedule it with the first docs lane.
- **Human calibration.** Should a few pages per batch be read by a person, to check
  that the agent ensemble agrees with people? Lanes cannot arrange it.
- **Nightly rustdoc JSON** for the kernel registry (`cargo public-api` needs it), when
  the projects pin rustc 1.97.
- **The Copperfield diagnostic registry request**, now also needed for fix-it text.

## Checked and already covered (do not re-add)

- **Resolution and staleness:** DOCER/DocRef-style resolution (2b.10); doctests and
  snippet compilation (2b.11, DOCS-PLAN §6.3); screenshots from replays (2b.11);
  registry digests (2b.13).
- **Reader critics:** the executed task agent with an end-state judge (τ-bench,
  AgentTrek and DRAFT all confirm it); facts from registries, not exploration (AppAgent:
  docs written from exploration lost to curated ones, 73.3% vs 95.6%); the ensemble
  gate; the churn guard (DRAFT's rise and fall is the published form of #24);
  cross-page contradiction (2c.17).
- **Planning and style:** topological order (2d.18; DocAgent); DocAgent's Existence
  Ratio (a weaker 2b.10); RepoAgent's incremental updates (2b.13). Readability formulas,
  weasel, wordy and cliché rules, and passive voice (in the crate). Heading specificity,
  restatement, actions per step, definition before use, apparatus rates (2a.1–2a.7).
- **Rejected:**
  - BLEU, METEOR, SIDE and CODERPE as gates;
  - SelfCheckGPT except for drafting explanation pages, which rest on `DECISIONS.md`
    prose;
  - Doc Detective (AGPL; our replay is stronger);
  - the STE dictionary (copyright);
  - CommunityCommands (needs telemetry, which is undecided);
  - DEFT and ACTER in any shipped pack (CC BY-NC-SA, evaluation only).
