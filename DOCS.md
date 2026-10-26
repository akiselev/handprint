# Docs — handprint as a documentation workflow (2026-10-26)

How to turn handprint into the critic for an agent workflow that writes the
documentation for cadatomic, svelters (skelter), cadabra3, Copperfield
(ecadabra) and qc. Companion to `WIT.md`: that document asks how to make an
agent funny, this one asks how to make an agent useful, and most of the
machinery carries over. Nothing here changes `WIT-plan.md`; it proposes a doc
family for the crate, a doc corpus, and an agent pipeline that lives outside
the crate (a `doc-writer` skill, like `.claude/skills/voice-transfer`).

## Verdict

1. **handprint has already run the documentation experiment, and the lessons
   are the design.** `eval/experiments/FINDINGS.md` #23–#31 fitted a
   `software-docs` reference on 1,077 pages from 48 projects and pointed the
   critic at AI drafts. Three results decide this plan:
   - `p_same_author` ranks the cold AI draft *above* 24 of 30 real human pages
     and scores the revision a blind judge could not tell from a human page as
     a regression (#23, #27). **The scalar cannot be the doc gate.**
   - The count of High `lex.doc-style.*` findings separates cleanly: human
     median 1 (max 5), cold AI 6, revised AI 1 (#31). **The findings are the
     signal; lead with them.**
   - There is no single AI signature: asked for docs, the model strips the
     parentheticals it floods chat with (#25). **The critic must be matched to
     the page type**, not fitted on chat and applied to everything.
2. **Docs differ from comedy in the one way that matters: the hard part can be
   executed.** For comedy, the top layer is an LLM judge because nobody can run
   a joke. For docs, the equivalent of wit is "a reader can do the task and
   every claim is true", and both are checkable: a fresh agent given only the
   page must perform the task against the real app, and the app's end state
   says whether it worked. That critic is stronger than anything the comedy
   pipeline has, and it is the gate.
3. **Most of the scope should not be written by an agent at all.** The
   reference layer (every command, panel, RPC method, MCP tool, CLI flag and
   diagnostic code) is a projection of registries the code already has.
   Generate it. Agents write the parts no registry can: how-tos, tutorials and
   explanations, each bound to the ids it talks about.

## The mapping from the comedy pipeline

| Comedy (`voice-transfer`) | Docs (`doc-writer`) |
|---|---|
| Plain retelling, the drift baseline | **Fact sheet**: the claims the page will make, each bound to a registry id or a source file |
| Device plan at the author's empirical rate | **Page plan**: Diátaxis type, the doc moves per section, at the type's empirical rates |
| Move-indexed exemplars (by mechanism, not topic) | **Doc-move-indexed exemplars**: "show 3 human pages stating a prerequisite", not "pages about sketches" |
| Corpus-mimic critic (author reference) | **Register critic**: one handprint reference per page type, plus de-AI contrast against register-matched AI drafts |
| Wit critic: pairwise, anchored LLM judge | **Reader critics**: task agent (executed), question bank, pairwise judge against human anchor pages |
| Drift guard (`content_overlap`) | **Fact guard**: every identifier resolves, every snippet runs, every procedure replays |
| Canary family (scored, never shown) | **Held-out reader checks**: the writer never sees the question bank or the task agent's transcript, only pass/fail |
| Two-sided bands (anti-caricature) | Two-sided bands per page type: a how-to with no code or no links is as wrong as one drowning in bold |

## Layer 0 — the facts, extracted, never written

**This builds on two plans that already exist.** svelters'
`plans/research/DOCS-PLAN.md` sets the rules for its docs: "facts are
generated; prose explains intent", `status:`/`verified_at:`/`sources:` front
matter on every page, every code block compiled in CI, no lane voice. Its
pipeline runs (metadata → IR → Starlight, link checks in CI, ~880 generated API
pages, `codes.rs` → `diagnostics.md`). cadatomic's teaching layer
(`docs/research/2026-10-24-teaching-layer.md`) adds "shown, not stated" and a
fact-check pass for lessons. This document adopts both and adds the critic.
Surveyed 2026-10-26 on the checked-out trees:

| Repo | Sources of truth a page can cite | Gap |
|---|---|---|
| cadatomic | Command registry (`crates/cadatomic-core/src/command/`, `inventory`, JSON Schemas for args and output); display text in `crates/cadatomic-ui/i18n/en.ftl` (726 `cmd-*` entries); diagnostics in `crates/cadatomic-core/src/diagnostics/catalog.rs`; **`docs/behavior/inventory.toml`** (182 commands, 113 key bindings, 39 panels, menus, ribbon, context menus: the best single index); JSON-RPC in `crates/cadatomic-cli/src/protocol.rs`, MCP tools built from `commands.list`/`methods.list`; 103 tours in `ui/tours/**`, `ui/tours/concepts.ts`, `scenarios/*.ndjson`; screenshots from `tools/gui-qc` (`gq shot`, `tour-shots`, `CADATOMIC_SHOTS`) | No README, no user docs; about 450 internal `.md` files |
| svelters | `crates/svelters-compiler/src/codes.rs` (172 codes with fix-its), `metadata/semantics/*.json`, `metadata/style/*.yaml`, theme tokens; the IR pipeline | `docs/skelter/` has only `icons.md`; skelter is undocumented |
| cadabra3 | Public rustdoc (`missing_docs = "warn"`, `cargo doc -D warnings` and doctests in `scripts/ci`) | No registry; `docs/walkthrough/` is architecture, not user docs |
| Copperfield | CLI `USAGE` (`crates/cli/src/main.rs`), file schema (`crates/model/src/schema.rs`), ERC catalog (generated) | **Diagnostic codes are string literals** (~63 `ecad.*` plus `drc.*`) with no registry; a request to add one, like svelters' `codes.rs`, is the precondition for generated diagnostic pages |
| qc | clap help, `schemas/run/*.v1.json`, D1 migrations, `worker/src/api.ts` | Small; generate almost all of it |

Every table above becomes three things at once: generated reference pages, the
resolution table for the verifier (2b.10), and the source of the question bank
(1d). A generated page is never critiqued for style. Its source text (a
tooltip, a code summary, a doc comment) is, at the source.

## Layer 1 — the corpus

Three corpora with three jobs, plus two banks for the reader critics.

### 1a. The style corpus: what good documentation sounds like

**Keep the cutoff.** Every page is taken at its project's last commit before
2022-01-01 (`eval/registers/fetch_docs.py`, verified in epoch seconds, #29).
After 2022 every large docs site is partly machine-written, and a reference
fitted on that learns the thing it is meant to detect.

| Source | Licence (docs) | Why | Disposition |
|---|---|---|---|
| The 48 `fetch_docs.py` projects (Rust book, Cargo, rust-reference, Svelte, Django, Kubernetes, LLVM, …) | MIT / Apache / BSD / CC-BY, per project | The base register; already built and fitted | Keep |
| **rustdoc comments** from widely used crates (`///` and `//!` extracted from serde, tokio, regex, clap, wgpu, std) | MIT OR Apache-2.0 | The API-reference register for cadabra3, Copperfield and skelter's public crates. No current corpus has it | Add, as its own `api-reference` sub-register |
| **KiCad docs** (schematic, PCB, DRC, footprints) | Dual: GPL-3+ **or** CC-BY-3+ ([docs.kicad.org](https://docs.kicad.org/)) | The closest domain match for Copperfield; take the CC-BY option. Confirm against the official `kicad-doc` repo's LICENSE before fitting | Add |
| **Blender manual** | CC-BY-SA 4.0 ([copyright page](https://docs.blender.org/manual/uk/5.0/copyright.html)) | The best model of a large docked-panel GUI manual: editors, tools, shortcuts, properties. Exactly cadatomic's shape | Add as a **separate share-alike pack file** (WIT-plan rule 0.4), or local fitting only |
| **FreeCAD manual** | "Creative Commons 4.0", variant unconfirmed ([manual intro](https://wiki.freecad.org/Manual:Introduction)) | MCAD vocabulary: sketches, constraints, Part/PartDesign, the closest match for cadatomic + cadabra3 | Verify the variant first; wiki pages separately |
| CadQuery, build123d, Open CASCADE docs | unverified | Kernel-API register for cadabra3 | Verify, then add if permissive |
| Onshape, SolidWorks, Fusion, Altium docs | proprietary | Tempting; not ours to scrape | Leave out |

**Split by page type before fitting.** Diátaxis's four types (tutorial,
how-to, reference, explanation) have opposite rates on the dimensions that
matter (imperatives, code density, second person, hedging). One `software-docs`
reference averages them into a register no real page has. Label pages by path
first (`tutorial/`, `guides/`, `reference/`, `concepts/` are common), then an
LLM labelling pass for the rest (same shape as `eval/agents/move-index`), and
fit one reference per type. **Keep each type above ~300 documents**: thin
references say yes to everything (#11) and lose discrimination (#22: recipe and
legal-terms, 141 and 145 documents, were the two worst).

**Normalise the draft exactly as the corpus was normalised** (#26). The
reference should record its normaliser (`strip_code` version) and `critique`
should refuse a draft that wasn't run through it. Today nothing enforces it.

### 1b. The contrast corpus: what to move away from

The de-AI side must be register-matched (#25). Generate it: cold drafts of
pages for **our own** features, written by the current models from the fact
sheets alone, with no critic. That is exactly the distribution the pipeline
starts from, and Fightin' Words over (human type-matched pages vs these
drafts) gives the per-type slop lexicon, as WIT2 §5 proposes for per-model
packs. Version it by model and date. The existing agent-prose corpus
(`handprint-data` transcripts) stays as a second, chat-register contrast.

**A third contrast is already on disk: lane voice.** The five repositories hold
over 1,500 internal agent notes (batch briefs, lane reports, STATUS blocks).
DOCS-PLAN found svelters' rustdoc accurate but "written in lane voice"
(milestone ids, `§` references, "Writer lane"). Fightin' Words over (internal
notes vs human docs) derives a `lane-voice` pack automatically, and every
public doc comment and page can be critiqued against it. It is the cheapest
item in this document with a known defect to catch.

### 1c. Anchors for the pairwise judge

Gold human pages, per type, topic-matched where possible: a KiCad page on
routing a track against our routing how-to, a Blender page on a docked editor
against our panel page, a Rust-book chapter against a skelter tutorial. The
judge compares against anchors, never scores absolutely (TTCW; `judge/README.md`).

### 1d. The reader banks (held out from the writer)

- **Task bank**: one task per how-to and tutorial, each with a machine-checked
  end state. cadatomic's scenarios and tours are the source; see Layer 0.
- **Question bank**: generated from the registries, not from the page
  ("which command makes a coincident constraint?", "what does
  `ecad.busentry.member` mean and how is it fixed?"). Because it comes from
  the code, it measures coverage as well as clarity.

## Layer 2 — algorithms

### 2a. In the crate: a `Doc` family (no LLM, decomposable, spans)

These are the doc analogs of the wit family: rates and patterns, fitted per
page type, two-sided bands. Each closes a gap a blind judge named and no
dimension caught (#28).

1. **Heading specificity.** Share of heading tokens that are template words
   (Overview, Introduction, Prerequisites, Best Practices, Troubleshooting,
   Conclusion, Summary, Next Steps) vs terms from the page's own fact sheet or
   the project glossary. The judge's tell #2, unmeasured today.
2. **Restatement / empty closer.** Novelty ratio of a sentence's content words
   against the preceding window (the list above it, the section heading).
   Flag closers with novelty near zero ("Each of these stages plays a crucial
   role…"). This is the same algorithm as comedy's `explained` (#14: flagged on
   8 of 9 passages, no dimension) — one implementation serves both.
3. **Conclusion-moral.** A final section with a template heading whose content
   words overlap the title and introduce nothing new. Tell #5.
4. **Instruction shape** (how-to and tutorial): actions per numbered step
   (imperative verbs per item; >1 is the classic defect), steps per list,
   colon→code adjacency (an instruction ending in a colon must be followed by
   a code block or a figure; #27 found stripped corpora full of dangling
   colons, so this needs the code-aware text mode), and result-statement rate
   ("The panel now shows…") after actions.
5. **Terminology consistency against the catalog.** cadatomic's rule "ids,
   never labels; display text from the catalog" gives every concept one
   canonical name. Fit a glossary pack from the catalog plus a synonym table
   (sketch/drawing/profile, net/signal, pad/land) and report non-canonical
   variants with spans and a `consider_replace` fix. The most mechanical win
   on the list, and the defect readers notice most in large doc sets.
6. **Definition before use.** The first mention of a glossary term on a page
   carries a link or a defining clause. Per-page rate, two-sided (a reference
   page that re-defines everything is also wrong).
7. **Apparatus balance per type**: code-block, inline-code, link, bold, list
   and table rates. Already measured (`md:*`); the change is fitting them per
   type. #25's separators (bold 26.8 vs 0.56, em-dash 18.8 vs 0.0, links 0.0
   vs 6.9) all live here.
8. **Findings-churn guard.** Flag a revision whose findings barely overlap the
   previous pass's (#24: acting on all eight findings produced fifteen new,
   disjoint ones). Cheap, and it catches report-chasing that `canary_ok` misses.
9. **A doc gate mode** for `critique`: pass is decided by High/Medium findings
   per family against the type's human distribution (doc-style HIGH ≤ the
   human 90th percentile, zero High in the Doc family), and `p_same_author`
   is reported as informational only (#31).

### 2b. Outside the crate: deterministic verifiers (the hard gate)

10. **Reference resolution.** Every inline-code token, identifier, menu path,
    command id, flag, config key and diagnostic code on a page must resolve
    against the Layer 0 tables. DOCER found outdated code-element references
    in 28.9% of the top 1,000 GitHub projects by regex-matching the source
    ([Tan, Wagner, Treude](https://arxiv.org/pdf/2212.01479)); we can do
    better because we have registries, not guesses. An unresolved reference
    fails the page.
11. **Every snippet runs.** Rust examples as doctests; CLI examples as
    transcript tests (the command runs, the shown output is compared, in the
    style of `trycmd`); GUI procedures as scenarios that replay. Screenshots
    come from those replays, never by hand, so a UI change regenerates them.
12. **Coverage matrix.** Registry ids × pages. Every command, panel,
    diagnostic and MCP tool has a reference entry (generated) and every
    user-facing one appears in at least one how-to. Uncovered ids are the work
    queue, which is how a huge scope becomes a finite list.
13. **Staleness.** Each page records the ids it depends on and a digest of
    their registry entries. The consolidation diffs the tables and marks
    pages stale; a stale page is regenerated (reference) or queued (prose).

### 2c. Reader critics (LLM, held out from the writer)

14. **Task agent.** A fresh agent with only the page and the app (MCP or the
    headless scenario runner) performs the task; success is the app's end
    state against the task bank's expected state, not the agent's claim. Pass
    rate per page, and the transcript shows where the reader got lost. This
    is the doc analog of the wit critic, with ground truth.
15. **Question bank.** A reader agent answers the registry-generated
    questions from the page alone. Unanswerable means a coverage gap; answered
    wrongly means the page misleads, which is worse.
16. **Pairwise judge against an anchor.** `judge/readable.md` adapted: "you
    need to do X; which page would you use?", decided before analysis, then
    the tells (template scaffolding, empty closers, filler intensifiers,
    conclusion-moral, `explained`) and the `sense` axis, which caught the
    continuity error nothing else did (#17).
17. **Cross-page contradiction.** Pairs of claims that share an id, across
    pages and against `DECISIONS.md`, checked for contradiction. Large doc
    sets written by many agents drift apart; this is where they do it.

### 2d. Planning

18. **Topological order.** Write pages in dependency order of the concepts
    they use, so a page can link to definitions that exist. DocAgent's
    ablation found processing order was vital for docstring quality
    ([DocAgent, ACL 2025](https://arxiv.org/pdf/2504.08725)).
19. **Doc-move index.** Label corpus chunks by move: states the purpose,
    states a prerequisite, gives a step, shows the expected result, warns
    about a pitfall, gives a worked example, names an error and its fix,
    defines a concept, gives the reason for a design. Retrieve exemplars by
    move, as the comic-move index does.

## Layer 3 — the pipeline (`doc-writer` skill)

1. **Extract** the Layer 0 tables at the commit being documented.
2. **Plan**: page inventory from the coverage matrix; Diátaxis type per page;
   dependency order; per section, the moves, at the type's rates.
3. **Fact sheet**: the claims, each bound to an id or a source path. This is
   the content baseline, as the plain retelling is for voice transfer.
4. **Render** with 2–4 move-matched exemplars per section.
5. **Loop, in order** (mechanics before wit, as in `voice-transfer`):
   1. verifiers (2b) until clean; they are cheap and the rest is noise while
      a page cites a command that doesn't exist;
   2. register critic (2a) for up to ~4 passes, watching the churn guard;
   3. reader critics (2c); the writer sees only which task failed and at
      which step, and which questions went unanswered, never the bank.
6. **Gate**, an ensemble (each part alone is Goodhart-able):
   resolution and snippets 100%; task agent passes; question bank above
   threshold; doc gate mode passes; pairwise judge ties or beats the anchor.
7. **Maintain**: on each consolidation, regenerate reference pages, diff the
   tables, queue stale and uncovered pages for the next batch's docs lane.

## Pilot (cheap, falsifies early)

Ten cadatomic pages across the four types. Arms: (a) cold draft from the fact
sheet; (b) + verifiers; (c) the full pipeline. Measure the task-agent pass
rate, question-bank score, doc-style High count, and blind pairwise verdicts
against anchors. Prediction from #23–#31: (a) reads as AI and fails tasks on
invented details; (b) fixes the facts but still reads as AI; (c) is the
thesis. If (b) and (c) tie on tasks and the judge, the register critic is
decoration for docs and the money goes into verifiers.

## Priority order

1. Layer 0 extraction, generated reference pages, resolution checker,
   coverage matrix. No LLM, the largest share of the scope, and it makes
   every later critic possible. cadatomic's `inventory.toml` and svelters'
   IR pipeline are most of it already; Copperfield needs a diagnostic
   registry first (a request, since this repository can't edit it). The
   `lane-voice` pack (1b) rides along: one `contrast` run.
2. Refit `software-docs` by page type; add rustdoc, KiCad, Blender (own
   pack), FreeCAD after licence checks; the doc gate mode (2a.9).
3. The Doc family in the crate (2a.1–2a.8). Restatement serves the wit family
   too.
4. Task agent and question bank (2c.14–15), on cadatomic's scenarios.
5. Move index and the `doc-writer` skill.
6. The pilot. Run it as soon as 1 and 4 exist; it decides how much of 2 and 3
   is worth building.

## Open questions for the user

- **Audience order.** End users of the GUI, developers against the kernel
  APIs, or agents using the MCP/RPC surface first? They need different page
  types and different corpora. Recommendation: GUI how-tos first, because the
  task agent can check them and they are the largest unwritten area.
- **Where the doc site lives** (a `docs/book/` in cadatomic, or its own
  repository) and its generator.
- **Share-alike corpora.** Fitting on the Blender manual means a share-alike
  pack file. Acceptable, or local-only?
- **Cost of the task agent.** Per-page runs in every batch, or nightly in the
  daily check?

## Sources

- handprint's own results: `eval/experiments/FINDINGS.md` #11, #14, #17, #22–#31; `eval/registers/README.md`; `eval/judge/readable.md`.
- svelters `plans/research/DOCS-PLAN.md`; cadatomic `docs/research/2026-10-24-teaching-layer.md`, `docs/behavior/inventory.toml`.
- Tan, Wagner, Treude, *Detecting outdated code element references in software repository documentation*, EMSE 29(1), 2024: https://arxiv.org/pdf/2212.01479; the CI tool: https://arxiv.org/pdf/2307.04291
- Yang et al., *DocAgent*, ACL 2025 demos: https://arxiv.org/pdf/2504.08725
- KiCad docs licence: https://docs.kicad.org/ (text: CC-BY 3.0+ or GPL 3+)
- Blender manual licence: https://docs.blender.org/manual/uk/5.0/copyright.html (CC-BY-SA 4.0)
- FreeCAD manual: https://wiki.freecad.org/Manual:Introduction ("Creative Commons 4.0", variant to verify)
- Diátaxis (the four page types): https://diataxis.fr/
