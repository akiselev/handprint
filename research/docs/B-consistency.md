# B. Keeping docs true to the code: gap analysis for DOCS.md (2026-10-26)

Research pass B of six. Area: documentation/code consistency, staleness, executable documentation. Every
claim cites a URL I opened; [unverified] marks the few I could only see through a search snippet. Items
already in `handprint/DOCS.md` or in svelters' `DOCS-PLAN.md` §6 are marked **redundant** and listed at the
end; everything else is a candidate for a DOCS.md revision. Nothing here was built or run.

## Verdict

1. **DOCS.md checks that a page's *names* are true, but not its *values*.** 2b.10 resolves identifiers and
   2b.13 digests registry entries, so a page that says `--tolerance` "defaults to 1 µm" passes whenever the
   flag exists, whatever its default. The literature that turns sentences into checkable claims (iComment,
   @tComment, Jdoctor, DocTer, R's `codoc`, MPDetector) all reach 85–93% precision on *narrow, typed* claims
   and much less on open-ended ones. Our writer is our own agent, so we can do better than extracting
   claims after the fact: make it emit them, and check that every sentence carrying a number, unit, default,
   range, order or id is backed by a fact-sheet claim that itself resolves. This is the largest gap.
2. **Staleness is triggered by registry digests only, so behaviour changes that keep the same id are
   invisible.** Just-in-time detection (triggered by the code diff) beats post-hoc detection by a wide
   margin in the one controlled comparison (Panthaplackel et al. 2021), and a one-line rule ("the comment
   mentions a token this diff deleted") already gets F1 68–75. DOCS.md needs diff-triggered staleness and a
   *behavioural* digest (the goldens and scenario end states a page depends on), not just a schema digest.
3. **LLM consistency checkers are now usable as advisors, never as gates.** Plain prompting flags 98% of
   functions; with careful restriction the best tools reach precision 0.47–0.72 on real code (DocPrism,
   METAMON) and use executed tests to suppress false positives (CASCADE, METAMON). They belong in 2c for
   claims no registry covers (cadabra3 semantics), with a test-backed confirmation step.

## 1. Code–comment and code–doc inconsistency detection

| Work | Detects | Algorithm | Published result | Slot | Status |
|---|---|---|---|---|---|
| iComment (Tan et al., SOSP 2007) [link](https://www.eecg.utoronto.ca/~yuan/papers/icomment.html) | Comment rules about locks/calls ("must hold X before calling") violated by code | NLP rule extraction + static checking | 1,832 rules extracted at 90.8–100% accuracy; 60 inconsistencies (33 bugs, 27 bad comments) in Linux, Mozilla, Wine, Apache | 2b, as a pattern: typed claim templates | New |
| @tComment (Tan, Marinov, Tan, Leavens, ICST 2012) [link](https://arxiv.org/abs/1201.6078) | Javadoc null/exception claims contradicted by behaviour | Pattern extraction of properties, then random tests (Randoop) check them | 29 inconsistencies in 7 projects (paper; slides say 28) | 2b: doc claim → generated test | New |
| Jdoctor (Blasi et al., ISSTA 2018) [link](https://homes.cs.washington.edu/~mernst/pubs/comments-specs-issta2018-abstract.html) | Javadoc → executable pre/postconditions | Pattern, lexical and semantic matching | Precision 92%, recall 83% | 2b claim grammar | New |
| DocTer (Xie et al., ISSTA 2022) [link](https://arxiv.org/abs/2109.01002) | Parameter constraints (dtype, shape, range) in API docs | Rules learned from dependency-parse patterns | 85.4% extraction precision; 94 bugs in 174 functions | 2b claim grammar (ranges, enums) | New |
| DocRef (Zhong & Su, OOPSLA 2013) [link](https://ir.iscas.ac.cn/handle/311060/16909) | Code names in docs that don't exist; obsolete samples | NL + code analysis | >1,000 new documentation errors in 5 libraries | 2b.10 | **Redundant** (DOCS.md's resolver is the same idea with registries) |
| Panthaplackel et al., AAAI 2021 [link](https://arxiv.org/abs/2010.01625) | A comment made stale by *this* code change | GRU/GGNN over comment + code edit, with features | Just-in-time F1 77–81 (full test set), post-hoc 66–67, rule `OVERLAP(C, deleted)` F1 68.0 (Table 2 of the [PDF](https://arxiv.org/pdf/2010.01625)); `@param` best (F1 ~92), summaries worst (~70) | Layer 3 maintenance | New |
| Comment update (Panthaplackel et al., ACL 2020) [link](https://arxiv.org/abs/2004.12169) | Rewrites the stale comment | Edit-sequence generation from code edits | Beats baselines; task "remains difficult" | Layer 3 (writer, not critic) | Partly redundant (our writer is an LLM) |
| CoCC (Huang et al., 2024) [link](https://arxiv.org/abs/2403.00251) | Outdated comments | Learned, features before/after change | Precision "over 90%"; 15 causal factors | Layer 3 | New, preprint only |
| CCISolver (2025) [link](https://arxiv.org/abs/2506.20558) | Detect + repair method-level inconsistency | LLM framework | Detection F1 89.54%; finds many existing benchmark labels wrong (CCIBench) | 2c | New |
| JIT CodeT5+ (SANER 2026) [link](https://conf.researchr.org/details/saner-2026/saner-2026-short-papers-and-posters-track/16/Larger-Is-Not-Always-Better-Leveraging-Code-Evolution-for-Comment-Inconsistency-Dete) | Same, from ordered edit sequences | Small model on edit ops | +13.54% F1 over SOTA; +4–11% over fine-tuned LLMs | Layer 3 | New: the edit diff matters more than model size |
| DocChecker (EACL 2024 demo) [link](https://arxiv.org/abs/2306.06347) | Inconsistent code–comment pairs | Fine-tuned code LLM | 72.3% accuracy on ICCD | 2c | New, weak |
| DocPrism (Xu, Wahab, Holmes, Lemieux; ISSTA 2026) [link](https://arxiv.org/abs/2511.00215) | *Incorrectness* (doc contradicts code), not incompleteness | Off-the-shelf LLM, "local categorization, external filtering" | Plain prompting flags 98% of functions; LCEF flags 14–17%; precision 0.63 across 4 languages; 0.47–0.67 vs SOTA's 0.05–0.14 on real Java; ~11% of pairs inconsistent | 2c | New |
| METAMON (Lee, An, Yoo; LLM4Code 2025) [link](https://arxiv.org/abs/2502.02794) | Doc vs actual behaviour | EvoSuite regression tests capture behaviour; LLM judges each oracle against the doc, metamorphic + self-consistency | P 0.72, R 0.48 on 9,482 Defects4J pairs | 2c | New |
| CASCADE (Kiecker et al., FSE 2026) [link](https://arxiv.org/abs/2604.19400) | Doc vs code, low false positives | LLM writes tests *from the doc*; also writes code from the doc; reports only when real code fails and doc-derived code passes | 13 unknown inconsistencies found, 10 fixed; run on Java, C# **and Rust** repos; code on GitHub (licence not stated) | 2c, confirmation step | New |
| MPDetector (Xu et al., 2024) [link](https://arxiv.org/abs/2411.11410) | Multi-parameter constraints ("if a then b must be None") | Symbolic execution on code + LLM on docs + fuzzy constraint logic | Precision 92.8%; 11 of 14 reports confirmed | 2b/2c hybrid | New |
| Radmanesh et al., 2024 [link](https://arxiv.org/abs/2409.10781) | Effect of inconsistency | GPT-3.5 detector + SZZ | Inconsistent changes ~1.5× likelier to introduce a bug; risk highest right after | Motivation | New |

What this means for DOCS.md:

- **The cheap rule is most of the value.** The `OVERLAP(C, deleted)` rule scored F1 68 against 77–81 for
  the deep models. For us that rule is: on every lane diff, extract identifiers deleted or renamed (from
  rustdoc JSON, the registries, and a tree-sitter token diff), and flag every page sentence and doc comment
  that mentions one, *including in plain prose*, not only in inline code. 2b.10 catches code-formatted
  tokens that no longer resolve; this catches the ones that still resolve because something else has the
  name, and the ones written as words ("the boolean driver").
- **Narrow typed claims are precise; open claims are not.** Every high-precision system above (iComment,
  Jdoctor, DocTer, MPDetector) restricts itself to a small claim grammar. DocPrism's main finding is that an
  unrestricted LLM flags almost everything. So: a fixed claim grammar for the 2b verifier, and an LLM only
  for what falls outside it, restricted to incorrectness.
- **Execution is the false-positive filter.** @tComment, METAMON and CASCADE all confirm a suspected
  inconsistency by running a test. We have the strongest harness of any of them (scenarios, tours, the
  headless app, Parasolid as an oracle for cadabra3). An LLM-raised inconsistency becomes a finding only
  when a generated test, scenario or doctest reproduces it.

## 2. Claim-level verification of prose (the gap)

DOCS.md's fact sheet (Layer 3.3) binds claims to ids, but nothing checks that the *rendered prose* stays
inside the fact sheet, and nothing checks values. Proposal, built from the work above:

**2b.14 Typed claims.** The fact sheet holds typed claims, each checkable against a registry or source:

| Claim type | Example sentence | Checked against |
|---|---|---|
| `default` | "Snap defaults to on" | Registry arg schema `default`, `config` tables, clap `default_value` (R's `codoc` does exactly this for function defaults: [link](https://web.mit.edu/r/current/lib/R/library/tools/html/codoc.html)) |
| `unit` | "Clearance is in millimetres" | Copperfield stores `i64` nm and millidegrees (EQ4); cadabra3 metres; display units from the catalog |
| `range` / `enum` | "Accepts 1–64 layers" | JSON Schema `minimum`/`maximum`/`enum` (DocTer's constraint classes) |
| `requires` / `order` | "Run DRC before export" | Command preconditions; scenario order (iComment's rule class) |
| `shortcut`, `menu-path` | "Press Ctrl+K" | `docs/behavior/inventory.toml` key bindings and menus |
| `returns` / `error` | "Fails with `ecad.busentry.member` when…" | Diagnostic catalog; a trigger test (svelters already has one per code) |
| `count` | "There are 4 sketch tools" | Registry query |

**2b.15 Sentence→claim alignment.** Split each page into atomic claims (FActScore's decomposition; its
automated estimator errs <2% on its own domain: [link](https://arxiv.org/abs/2305.14251)) and require
every sentence that contains a number, unit, quoted value, key, id or ordering word to entail some
fact-sheet claim. An unbacked sentence is a finding with its span; a sentence that contradicts its claim
fails the page. This is the docs analog of the comedy pipeline's drift guard and closes the hole between
"the fact sheet is true" and "the page is true". The decomposition is an LLM step, but the check against
the typed claim is deterministic.

**2c.18 Behavioural claims (advisory).** For claims no registry holds (cadabra3's "returns a closed shell
when…", tolerance behaviour), run a CASCADE-style loop: the critic writes a test from the sentence alone,
runs it against the real crate, and reports only failures that reproduce. METAMON's precision (0.72) and
DocPrism's (0.63) say this must stay advisory; a reproduced failure is still a hard finding because the
test, not the LLM, decides it. For cadabra3, a differential test against Parasolid is available too.

## 3. Co-evolution: which code changes should trigger which doc updates

- Fluri, Würsch and Gall found 97% of comment changes land in the same revision as the code change, but
  that **API changes are typically re-documented in a later revision** and new code barely gets commented
  (WCRE 2007, SQJ 2009) [unverified: both primary pages were unreachable; figures from search snippets and
  citing papers]. That is our batch shape: lanes change APIs, docs lag a consolidation.
- Wen et al. mined 1.3 billion AST-level changes across 1,500 systems to see which change types trigger
  comment updates (ICPC 2019) [unverified: PDF returned 403; abstract via the ICPC program and a search
  snippet].
- AdDoc (Dagenais & Robillard, TSE) mines *documentation patterns*, sets of code elements documented
  together, and flags changes that break them; at least 50% of all documentation changes in four Java
  projects related to existing patterns. It also counted, in one Hibernate release, 5,661 new code
  elements with 6 documented and 26 references to deprecated elements with no mention of the replacement
  ([PDF](https://www.cs.mcgill.ca/~martin/papers/tse2014.pdf)).
- Swimm's commercial "Auto-sync" auto-updates trivially changed snippets and fails the PR when a change is
  "impactful enough" ([docs](https://docs.swimm.io/features/keep-docs-updated-with-auto-sync/)); the
  criterion is patented and unpublished. Proprietary, ideas only.

**For DOCS.md (Layer 3.7):** svelters' `OWNERS.toml` + `check_doc_touch.py` (DOCS-PLAN §6.4) is a
hand-written version of this and is **redundant for svelters**, but DOCS.md does not generalise it. Two
additions: (a) derive the owner map instead of hand-writing it, from each page's claim bindings (every
claim names an id or path) plus AdDoc-style co-change mining over git history to propose missing edges;
(b) a **behavioural digest**: a page depends not only on the registry entries it cites but on the goldens,
scenario end states and trycmd outputs it embeds or describes. When a lane re-blesses a golden (the
CLAUDE.md rule already requires lanes to list them), every page bound to it is queued for prose review.
"Bless → stale" is the one-line rule.

## 4. API migration and deprecation in docs

- Deprecations often ship without a replacement: a median of 66.7% (Java) and 77.8% (C#) of deprecated
  elements per project carry a replacement message (Brito et al., JSS 2018:
  [link](https://bia.unibz.it/esploro/outputs/journalArticle/On-the-use-of-replacement-messages/991005773308101241)).
- **rustdoc JSON is the missing cadabra3 registry.** DOCS.md says cadabra3 has "no registry". It has one
  for free: `rustdoc --output-format json` (unstable, [link](https://doc.rust-lang.org/rustdoc/unstable-features.html))
  emits every item with `docs`, resolved intra-doc `links`, `span`, and a `deprecation` field
  ([rustdoc-types `Item`](https://docs.rs/rustdoc-types/latest/rustdoc_types/struct.Item.html)).
  `cargo public-api` (MIT) lists the public API one item per line and diffs it across git refs
  (`diff ref1..ref2`), usable as a library and as a snapshot test
  ([link](https://github.com/cargo-public-api/cargo-public-api)); `cargo-semver-checks` (MIT/Apache) runs
  Trustfall queries over the same JSON to name removed and changed items
  ([link](https://github.com/obi1kenobi/cargo-semver-checks)). Both need nightly rustdoc JSON. This gives
  cadabra3 and Copperfield's facade the same Layer 0 table cadatomic gets from `inventory`, and its
  per-consolidation diff is exactly the "deleted/renamed identifiers" list §1 needs.
- **Rule for every registry:** a deprecated or renamed command id, flag, RPC method or diagnostic code keeps
  a `deprecated {since, replacement}` entry instead of vanishing. The resolver then reports a reference to
  it as a finding whose fix is the replacement (a `consider_replace`, as 2a.5 does for terms), rather than
  an unresolved name. A deprecation without a replacement is itself a registry finding.

## 5. Executable and literate documentation

| Practice | What it gives | Licence | Slot | Status |
|---|---|---|---|---|
| Rust doctests: `no_run`, `compile_fail`, `should_panic`, hidden `#` lines, README tested via `#[doc = include_str!] #[cfg(doctest)]` ([link](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)) | Rust snippets compile/run | — | 2b.11 | **Redundant** (DOCS.md 2b.11, DOCS-PLAN §6.3) |
| `rustdoc --show-coverage` (unstable): per file, documented items *and items with examples*, also as JSON ([link](https://doc.rust-lang.org/rustdoc/unstable-features.html)); `missing_doc_code_examples` lint is nightly-only, allow-by-default ([link](https://doc.rust-lang.org/rustdoc/lints.html)) | Example coverage for cadabra3 | — | 2b.12 coverage matrix | New column |
| `mdbook test`: runs Rust blocks in a book through rustdoc, `-L target/debug/deps` for the crate ([link](https://rust-lang.github.io/mdBook/cli/test.html)) | Book pages as doctests | MPL-2.0 | 2b.11 | New if the site is mdBook |
| trycmd: `.md` files *are* test cases (`$` command, expected output, `? failed` status), `[..]`/`...` elision, `[ROOT]`/`[CWD]`, `TRYCMD=overwrite` ([link](https://docs.rs/trycmd/latest/trycmd/)) | CLI pages tested in place | MIT/Apache | 2b.11 | Partly redundant; see below |
| Go testable examples: `// Output:` compared to stdout; `ExampleFoo_bar` naming binds the example to the identifier; no output comment = compiled, not run ([link](https://go.dev/blog/examples)) | Example ↔ id binding by name | — | 2b.12 | New idea |
| Elixir doctests: `iex>` transcripts, `** (Error)` for exceptions; discouraged for side effects ([link](https://ex-unit.hexdocs.pm/ExUnit.DocTest.html)) | The transcript format for RPC/MCP | — | 2b.11 | New idea |
| nbval: executes notebooks, compares stored outputs, `--nbval-lax` only checks marked cells ([link](https://pypi.org/project/nbval/)) | Notebook docs | BSD | — | Not needed (no notebooks) |
| Cucumber/Gherkin "living documentation": the spec file is both doc and test, stays true only where CI runs it (vendor/tutorial sources only, [unverified: search snippets of vendor pages only]) | Procedure ↔ test as one artifact | — | 2b.11 | Pattern |
| Doc Detective: parses docs for testable actions and runs them in a browser, takes screenshots ([link](https://github.com/doc-detective/doc-detective)) | Docs-as-tests runner | **AGPL-3.0** | — | Ideas only |
| insta / approval testing: snapshot, review, re-approve ([insta](https://insta.rs/docs/), [ApprovalTests](https://approvaltests.com/)) | The bless workflow | — | Layer 3 | Redundant in mechanism (cadatomic has goldens) |

Docable studied 600+ tutorials with ~15,000 code blocks: a naive executor ran 26% of them, human
annotation roughly doubled that, and **no tutorial could run every step**; causes ranged from interactive
prompts to missing steps and unreachable resources ([link](https://par.nsf.gov/biblio/10196169)). That is
the empirical case for 2b.11 and 2c.14, and it says missing steps are a main failure mode, which only the
task agent catches.

Gaps against DOCS.md:

- **Test the page, not a copy.** DOCS.md says "in the style of `trycmd`". trycmd can take the page's own
  `.md` as the case, so Copperfield's and qc's CLI pages are their tests, and `TRYCMD=overwrite` regenerates
  the shown output: the agent writes the command and the prose, never the output. Same for svelters'
  snippet markers (already in DOCS-PLAN §6.3, **redundant there**).
- **RPC and MCP examples have no executor in DOCS.md.** cadatomic's JSON-RPC and MCP surfaces are the one
  reference area with examples but no runner. Two checks: every JSON block tagged with a method or tool id
  validates against that entry's JSON Schema (cheap, no app), and an Elixir-style transcript (`→ request`,
  `← response` with `[..]` elision) replays against `cadatomic-headless`. qc's `schemas/run/*.v1.json` get
  the first check for free.
- **Bind the procedure to the scenario in one direction.** DOCS.md says GUI procedures replay as
  scenarios, but a page whose numbered steps differ from its scenario passes. Either render the step list
  from the scenario (step = command id → catalog label, as Gherkin binds step text to step definitions) and
  let the writer write only between steps, or check alignment: each numbered step maps, in order, to a
  scenario event, and each command-invoking event maps to a step.
- **Doctest hygiene findings.** `ignore` without a reason, `text` fences that contain Rust, and a high
  ratio of `#`-hidden lines (setup the reader can't see, then can't reproduce) are cheap, mechanical
  findings for 2a or 2b.
- **Example coverage.** Add an "has a runnable example" column to the coverage matrix, from
  `--show-coverage`'s examples count for crates and from Go-style naming or `api:`/id front matter for pages.

## 6. Screenshots

DOSUD (Tang et al., TOSEM) is the first tool for outdated GUI screenshots in documentation; it found 20
outdated screenshots across 50 Android app sites and 17 in VS Code's docs, all fixed by VS Code
([PDF](https://www.cs.sjtu.edu.cn/~zhonghao/paper/DOSUD2025.pdf)). We don't need detection: DOCS.md already
regenerates screenshots from replays (**redundant**). What's missing is the consequence: a regenerated
screenshot whose pixel diff exceeds a threshold (Playwright's pixelmatch default per-pixel `threshold` is
0.2, plus `maxDiffPixels`: [link](https://playwright.dev/docs/test-snapshots)) means the *prose around it*
may be stale ("the panel shows three tabs"), so the page is queued. A diff caused only by a Mesa change is
filtered out, following cadatomic's existing rule that Mesa-only diffs go to the coordinator.

## 7. Links and anchors

Almost 10% of links in source comments are dead, and link targets change while the links don't (Hata,
Treude, Kula, Ishio, ICSE 2019: [link](https://arxiv.org/abs/1901.07440)). lychee (MIT/Apache) checks
Markdown and HTML, checks fragments (`--include-fragments`), has `--offline` for local-only runs and a
response cache ([link](https://github.com/lycheeverse/lychee)). svelters' site already link-checks, and
cadabra3 already denies broken intra-doc links through `cargo doc -D warnings` (**redundant** there).

New, and specific to this pipeline: **the heading critic will break anchors.** 2a.1 rewards replacing
"Overview" with a specific heading; every such rewrite changes the generated slug and silently breaks
inbound `#fragment` links across pages. Give headings explicit stable ids (`{#id}`, ids never labels, as
cadatomic's rule says) and run lychee in offline fragment mode in the lane gate; external links run in the
nightly check with the cache, since lanes should not depend on the network.

## Already covered (redundant)

DocRef/DOCER-style resolution (2b.10); doctests and snippet compilation (2b.11, DOCS-PLAN §6.3.1–3);
screenshots from replays (2b.11); registry digests (2b.13); cross-page contradiction (2c.17); svelters'
symbol check `doc_api.rs` (§6.3.4), owner map and doc-touch check (§6.4), `verified_at` freshness warning
(§6.3.6, the same idea as Google's freshness dates and "Last reviewed by" bylines:
[link](https://abseil.io/resources/swe-book/html/ch10.html)); intra-doc link denial in cadabra3 and svelters;
svelters' diagnostic trigger tests (§6.2).

## Top 5 to adopt (leverage per effort)

1. **rustdoc JSON as a Layer 0 registry, with a per-consolidation API diff** (`cargo public-api diff` or
   direct rustdoc-types). Low effort, no LLM. It gives cadabra3 and Copperfield the table DOCS.md says they
   lack, feeds the resolver, carries `deprecation`, and produces the deleted/renamed list for item 2.
2. **Diff-triggered staleness with a behavioural digest** (Layer 3.7): flag sentences and doc comments that
   mention identifiers the diff deleted or renamed (the `OVERLAP(C, deleted)` rule, F1 68 alone), and queue
   pages whose bound goldens, scenario end states or trycmd outputs were re-blessed ("bless → stale").
   Low effort, deterministic, catches same-id behaviour changes that registry digests miss.
3. **Typed claims + sentence→claim alignment** (2b.14–15): defaults, units, ranges, enums, order, shortcuts,
   counts, checked against schemas and `inventory.toml`; every value-bearing sentence must map to a claim.
   Medium effort (one LLM decomposition step, deterministic checks). Closes the largest hole: names right,
   values wrong.
4. **Execute the page itself, including RPC/MCP**: trycmd on the `.md` pages for Copperfield and qc CLIs,
   JSON Schema validation of every RPC/MCP example plus transcript replay against `cadatomic-headless`, and
   scenario↔step alignment for GUI procedures. Low to medium effort; extends 2b.11 to the surfaces it skips.
5. **Stable heading ids + offline fragment link-checking in the lane gate** (lychee). Very low effort, and
   it prevents the register critic from silently breaking cross-page anchors on every heading rewrite.

Runner-up: a CASCADE-style, test-confirmed LLM checker (2c.18) for cadabra3 behavioural claims, advisory
only, once 1–4 exist.
