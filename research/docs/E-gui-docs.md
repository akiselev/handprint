# E. GUI documentation for CAD/EDA: generating, verifying and illustrating procedures

Research for `handprint/DOCS.md`, 2026-10-26. Scope: documentation for cadatomic's GUI (and by extension
skelter apps). Every outside claim carries a URL that was opened unless marked [unverified]. Internal
claims cite paths in the checked-out trees (`/home/user/cadatomic`, `/home/user/handprint`).

## 0. What DOCS.md misses (the short version)

1. **cadatomic already has a design for generating GUI how-to pages, and DOCS.md doesn't cite it.** The
   tour note (`cadatomic/docs/research/2026-10-23-tours-tests-tutorials-videos.md`) defines one tour script with
   five renderings: test, guided, demo, **doc** ("Markdown pages with annotated screenshots") and video (§0.1).
   Doc comments on `tour`/`chapter`/`step` *are* the text tutorial (§0.4, §2.1). §7.3–7.4 already specify
   perceptual screenshot diffs, an approval file, key-addressed images (`<TourShot id=…>`), SVG callouts drawn
   from captured rects, masks, light/dark and 1×/2×. Its precedent table (§10.3) already covers Doc Detective,
   GitLab's screenshot specs, CloudCannon DocShots, Scribe and Tango. The lane that builds it, **TU8, was
   deferred** in batch 9 (`docs/notes/batch9/BRIEF.md` line 60). DOCS.md 2b.11 ("GUI procedures as scenarios
   that replay. Screenshots come from those replays") is a weaker re-invention of TU8. **Redundant: replace
   2b.11's GUI clause with "GUI how-tos are tours; their pages come from TU8", and make TU8 a priority-1
   dependency.**
2. **A scenario doesn't test the taught route; a tour does.** cadatomic scenarios are command-level NDJSON
   (`scenarios/mcad-measure.ndjson`: `{"step":"command","id":"body.sphere","args":…}`). They prove the end
   state, but they would pass a page whose menu path doesn't exist. Tours run "UI acts on locators (click the
   Extrude button, type 10 mm), not `command.run`" (tour note §0.3), and `expect` doubles as the step's
   completion condition (§0.2). Bind each GUI how-to to a **tour**, not a scenario (Layer 0, 1d, 2b.11).
3. **The task agent (2c.14) must not act through MCP.** DOCS.md lets it use "MCP or the headless scenario
   runner". The user ruled on 2026-10-26: "we won't want to use RPC, MCP, etc. we want proper GUI
   interaction" (`tools/gui-qc/README.md`, first section). An agent that reads "Sketch ▸ Extrude" and then calls
   `command_run body.extrude` checks nothing a reader does. The checker exists already: `gq` (XTEST input,
   AT-SPI tree, the renderer probe's hit-checked clicks).
4. **The behaviour catalog is a Layer 0 source DOCS.md omits.** `docs/behavior/observed/*.toml` holds
   generated records per trigger (`id`, `trigger`, `context`, an `outcome` sentence pinned to a `digest`,
   `tree_delta`, `trace`, `repro` script, `conventions` it departs from, `finding`). This is the "what happens
   when you click ×" layer that reference pages need. It is replayable (`tools/gui-qc/replay`), and the digest
   gives staleness for free. An outcome tied to an open `finding` is a bug: a page must not document it as
   intended behaviour.
5. **The question bank (1d) is in the system's vocabulary, which tests the wrong thing.** "Which command
   makes a coincident constraint?" already names the feature. Users search in goal language, and mapping that
   language to commands is its own research line (§5). Tours already carry user-vocabulary `keywords`
   (`ui/tours/mcad/first-box.tour.ts`: "cube", "block", "primitive"…).
6. **In-app help is a documentation output, not a separate system.** The evidence for contextual help in
   feature-rich software is stronger than for manuals (§5). DOCS.md generates pages only; the same Layer 0
   sources should also generate tooltips, extended tooltips and "Learn this" links. The tour note's "auto tips"
   (`tipsFor(commands)`, §3.2.3) is the consumer.

## 1. Tutorial generation from demonstrations and traces

| System | Method | Evidence | Code / licence | Slot; redundancy |
|---|---|---|---|---|
| Grabler et al., SIGGRAPH 2009 ([abstract](https://history.siggraph.org/?p=108824)) | An instrumented GIMP records interface and application state while an author demonstrates. The system generates a step tutorial of images, text and annotations, and uses image labelling for more precise step text | Against hand-designed tutorials and screen videos, users were 20–44% faster with 60–95% fewer errors | none | Layer 3 step 3: **demonstrate first, then write.** The fact sheet for a GUI how-to should be a passing tour or a gq session (`$GQ_DIR/actions.ndjson`), not prose |
| MixT, Chi et al., UIST 2012 ([PDF](https://people.eecs.berkeley.edu/~bjoern/papers/chi-mixt-uist2012.pdf)) | Segments screen video into steps using command and input logs; static steps plus a short clip per step | 12-participant study: video helps "brushing, control point manipulation, and adjustment of continuous parameters", and cuts repeated attempts. Generated tutorials were as effective as manual ones (informal) | none | 2a.4 / TU8: a step whose `do` holds a drag or gesture (routing, sketch drag) gets a clip ("Watch this step", tour note §7.4); click-and-type steps stay static. New rule, not in DOCS.md |
| DocWizards ([IBM](https://research.ibm.com/publications/docwizards-a-system-for-authoring-follow-me-documentation-wizards)); Pause-and-Play ([TAU](https://cris.tau.ac.il/en/publications/pause-and-play-automatically-linking-screencast-video-tutorials-w/)) | Follow-me guidance captured from expert demonstrations (Eclipse); a video that pauses until the user's own action in SketchUp or Photoshop matches it | Users preferred both to the plain format | none | Both are the tour note's guided mode. Redundant |
| Torta, Mysore & Guo, UIST 2017 ([PDF](https://pg.ucsd.edu/publications/Torta-generating-multi-application-software-tutorials_UIST-2017.pdf)) [unverified: host returned 503; abstract read in search results only] | OS-wide trace plus screencast becomes a mixed-media tutorial. The author adds **validation criteria** per step; the viewer checks step-level progress | 10 TAs preferred it to writing by hand | none | 2b: the per-step check is cadatomic's `expect`. Redundant in kind |
| Chronicle ([Autodesk](https://www.research.autodesk.com/?p=1833)); Sikuli and Sikuli Test ([2009](https://dspace.mit.edu/handle/1721.1/72686), [2010](https://dspace.mit.edu/handle/1721.1/72684)) | Workflow history linked to document regions (shipped as Autodesk Screencast); screenshot patterns as help queries and test scripts | Qualitative; screenshot search faster to specify than keywords | product; open source | Not needed: the command log, anchors and the probe already do this better |

**The lesson for DOCS.md.** The systems that beat hand-written tutorials started from a recorded demonstration
with application state. For GUI pages, Layer 3 should record first (a tour, or a gq session turned into one) and
then write the tour's doc comments, `hint` and `say`; handprint's critics run on the extracted Markdown.

## 2. LLM and GUI-agent era (2023–2026)

| Work | What it shows | Slot |
|---|---|---|
| WONDERBREAD, Wornow et al., NeurIPS 2024 ([arXiv](https://arxiv.org/abs/2406.13264)) | Models recall 88% of a demonstration's steps when writing workflow docs, but score F1 < 0.3 when they **validate whether a workflow was completed**. 2,928 demonstrations | 2c.14: never let a model judge task success. Use the app's end state (DOCS.md already says so; this is the evidence). Also: drafting from demonstrations works |
| AppAgent, Zhang et al. ([arXiv](https://arxiv.org/abs/2312.13771); MIT, [LICENSE](https://raw.githubusercontent.com/TencentQQGYLab/AppAgent/main/LICENSE)) | In an exploration phase the agent acts on UI elements, compares before and after screenshots, and writes per-element docs used later. 50 tasks across 10 apps | Layer 0: cadatomic's behaviour catalog is the same idea with replayable evidence and digests. Prefer it to agent-written element docs |
| AutoManual ([arXiv](https://arxiv.org/abs/2405.16247)); Synatra ([arXiv](https://arxiv.org/abs/2409.15637)) | AutoManual compiles rules learnt by interaction into a readable manual (97.4% on ALFWorld, GPT-4-turbo). Synatra turns human tutorials into 100k synthetic demonstrations at about 3% of the cost of human ones | Note only: tours are already direct demonstrations |
| DocOS, Liu et al., 2026 ([arXiv](https://arxiv.org/abs/2605.18048)) | Agents must find documentation and ground its steps into GUI actions. The bottlenecks are finding the page and **grounding instructions into precise actions** | 2c.14: the task agent's failures will mostly be grounding failures. Classify them (§6) |
| HelpViz, Zhong et al., UIST 2021 ([arXiv](https://arxiv.org/abs/2108.03356v1)) | Parses text instructions into actions, **executes them in emulators**, and builds a visual tutorial from the run's screenshots and tapped elements. Participants preferred it to text | The closest analog of "a page whose figures come from executing it". TU8 does this with anchors instead of parsing |
| AXNav, Taeb et al., CHI 2024 ([arXiv](https://arxiv.org/abs/2310.02424)) | An LLM agent executes a natural-language manual test and outputs a **chaptered, annotated video**; heuristics flag issues. 10 QA professionals found it useful | 2c.14 output: keep the task agent's run as a chaptered gq recording, one chapter per page step, for the reviewer in `qc` |
| dBlocks, Bhat et al., 2026 ([arXiv](https://arxiv.org/abs/2609.05563)) | Writers using LLMs needed help verifying output; inline execution made them verify while drafting (n=16) | Supports DOCS.md's verifiers-first loop. Redundant |

## 3. Screenshots: annotation, granularity of the image, staleness

- **Evidence on screenshots is weaker than practice assumes.** Gellevij and van der Meij compared a text-only
  manual with partial-capture and full-capture manuals. As a job aid there was no difference; for learning,
  text-only and full-screen beat partial captures; "the use of full-screen captures is preferable to partial
  ones" ([U. Twente, 1999](https://research.utwente.nl/en/publications/the-effects-of-screen-captures-in-manuals-a-textual-and-two-visua/)).
  A later study reports 25% less training time and 60% better retention for full-screen images in a two-column
  layout [unverified, search summary only]. **Rule for TU8:** the default figure is the whole window with a
  numbered callout on the target, plus a zoomed inset when the target is small. Never a tight crop alone.
  `tour-shots` already grabs the whole window (`tools/gui-qc/README.md`, "Tour shots").
- **Callouts from geometry, not pixels; masks; pinned volatile UI.** The probe's `locate --json` gives `rect`,
  `visible` and `occluded_by`, and tour note §7.3–7.4 already draws SVG callouts from captured rects and pins
  the clock (fastlane's 9:41). Redundant; DOCS.md needs only to point at it.
- **Stale screenshots are a measured problem.** DOSUD ([TOSEM PDF](https://www.cs.sjtu.edu.cn/~zhonghao/paper/DOSUD2025.pdf))
  found 17 outdated screenshots in VS Code's docs, and VS Code fixed all of them. It also counts more than 210
  VS Code issues about outdated screenshots. Regenerating every shot removes that failure mode
  (tour note §7.3), but it moves the risk into the **prose around the figure**: "the panel now shows three
  rows" can go stale against a correct image. **New deterministic check (2b, beside 2b.10):** each figure keeps
  the AT-SPI snapshot taken with it (`gq tree`, or the scene YAML in tour note §7.3). Every UI label the
  surrounding paragraph names in bold or code must appear in that snapshot. Cheap, no LLM, and it catches
  exactly what DOSUD finds.

## 4. Step granularity and "expected result"

- **Farkas's streamlined-step model** ([Technical Communication 1999](https://faculty.washington.edu/farkas/dfpubs/Farkas-ConstructionOfProceduralDiscourse.pdf)):
  each step is "a brief action statement (or perhaps two short, closely related action statements)" built on an
  imperative verb. A feedback statement describes "the system's response … and the new state". It is "only
  needed occasionally" in GUI documentation, because the GUI gives its own feedback, but it is useful "when the
  system's response is non-routine or hard to notice". **Change to DOCS.md 2a.4:** don't fit a result-statement
  *rate*. Require a feedback sentence where the step's `expect` names state the reader can't see at the target
  (a DRC marker elsewhere, a lineage change, a disabled button lighting up), and flag one where `expect` is only
  `visible(next field)`. The tour's `expect` predicate decides it mechanically.
- **DITA's task schema** ([OASIS DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part2-tech-content/archSpec/technicalContent/dita-task-topic.html)):
  `prereq`, `context`, `steps` (each `step` has a required `cmd` plus optional `info`, `stepresult`,
  `steptroubleshooting`), `result`, `example`, `postreq`. This maps one to one onto a tour: `setup` →
  prereq, chapter prose → context, `do` → cmd, `expect` → stepresult, the final checkpoint → result. DOCS.md
  2d.19's doc-move index should use these slots as its move labels, so the moves have an industry-standard
  meaning and a predicate behind each.
- **Error information helps.** Novices whose manual carried detection, diagnosis and correction information did
  better during and after practice and finished training faster ([Lazonder & van der Meij 1995](https://research.utwente.nl/en/publications/7e445104-88ad-481f-87cf-0539f55ae327)).
  The minimal manual replicated with 64 students: "minimalist users learned faster and better"
  ([Lazonder & van der Meij 1993](https://research.utwente.nl/en/publications/f811580e-953a-4312-a1ee-9916b3f98e05)).
  **Slot (Layer 0 → 2a):** cadatomic's commands return a typed `Refusal { reason, slot, message }`
  (`crates/cadatomic-core/src/command/def.rs`) and diagnostics have a catalog. A how-to step whose command can
  refuse should carry a `steptroubleshooting` line generated from those reasons ("Accept stays grey until all
  three sizes are positive"). The first-box tour already says this in prose; generate it instead.
- **Step labels, thumbnails, text first.** ToolScape's step descriptions and before/after thumbnails on the
  timeline improved performance and self-efficacy ([Kim et al., CHI 2014](https://www.eecs.harvard.edu/~kgajos/papers/2014/kim14crowdsourcing.shtml)):
  TU8's video gets a chapter per step with the step shot as its thumbnail. Palmiter, Elkerton and Baggett (1991)
  found demonstrations faster in training but worse a week later [unverified: search summary only; DeepBlue
  returned 403], so text stays primary.

## 5. In-app and contextual help, learnability

| Work | Finding | Slot |
|---|---|---|
| Grossman, Fitzmaurice, Attar, CHI 2009 ([PDF](https://research.autodesk.com/app/uploads/2023/03/a-survey-of-software.pdf_recqWPTEYd6nsP0r8.pdf)) | Studied in AutoCAD. Five issue categories: **Task Flow** (didn't know where to start), **Awareness** (didn't know the tool existed), **Locating** (knew it, couldn't find it), **Understanding** (found it, couldn't use it), **Transition** (didn't use the efficient way). Awareness dominated for novices; users search "low level tasks", not goals | 1d: tag every bank question with one of the five, and cover all five per area. Locating questions ("where is Extrude?") are answered by a generated reference box (§7). Task Flow questions are the how-to bank |
| ToolClips, Grossman & Fitzmaurice, CHI 2010 ([Autodesk](https://www.research.autodesk.com/?p=2111)); AutoCAD's tooltip tiers ([Options ▸ Display](https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-LT/files/GUID-F464608C-15BC-48E8-9484-D311053D9B73.htm)) | Tooltips extended with short video: "7 times as many unfamiliar tasks" completed vs a commercial online help system, with retention a week later. AutoCAD shows a basic tooltip, then an extended one after a delay | In-app output: the step clip TU8 makes for a command's first-use tour is that command's ToolClip; tooltip tiers come from `en.ftl`. One source, two places |
| CommunityCommands, Matejka et al., UIST 2009 ([Autodesk](https://www.research.autodesk.com/publications/communitycommands-command-recommendations-for-software-applications/)) | Collaborative filtering over command logs gave 2.1× as many good suggestions; tested for three months with professionals | Awareness. Needs telemetry, which is undecided (tour note Q9). Out of scope now |
| LemonAid, Chilana et al., CHI 2012 ([PDF](https://faculty.washington.edu/ajko/papers/Chilana2012LemonAid.pdf)) | Help retrieved by **selecting a UI element** instead of typing: a result for 90% of requests | In-app: help keyed by anchor and command id, which cadatomic has and LemonAid had to infer |
| Query-feature graphs, Fourney et al., UIST 2011 ([OpenAlex](https://api.openalex.org/works/doi:10.1145/2047196.2047224)); CommandSpace, Adar et al., UIST 2014 ([page](https://cond.org/commandspace.html)) | Map users' goal vocabulary ("make it black and white") to commands ("desaturate") from query logs and web text | 2a.5's synonym table and the bank's paraphrases. Seed both from tour `keywords` and the palette's search aliases; no web mining needed |
| Ambient Help ([Autodesk](https://www.research.autodesk.com/publications/ambient-help/)); Novick & Ward, SIGDOC 2006 ([UTEP](https://scholarworks.utep.edu/cs_papers/15)) | Context-relevant help on a second display found more useful information without lowering productivity. 25 interviewed users avoid manuals; their trouble is finding search terms | The Learn panel and palette search, in user vocabulary. Not a DOCS.md item beyond the bank |

## 6. GUI agents and testers as procedure checkers

- **OSWorld** ([arXiv](https://arxiv.org/abs/2404.07972); Apache-2.0, [LICENSE](https://raw.githubusercontent.com/xlang-ai/OSWorld/main/LICENSE)):
  369 tasks, each with an initial-state config and an execution-based evaluator (getters pull final state,
  evaluators compare it; 134 functions). Humans 72.36%, best agent 12.24% at launch. **The accessibility-tree
  input scored best** (GPT-4 with the a11y tree 12.24% vs GPT-4V screenshot-only 5.26%), and the tree comes
  from **pyatspi on Ubuntu**, which is what `gq tree` reads ([HTML](https://arxiv.org/html/2404.07972)). The raw
  tree "is large (usually over 1 million tokens in XML format)" and needs filtering. **Slot 2c.14:** give the
  task agent the gq tree filtered to visible nodes in the active window, plus a screenshot. Reuse OSWorld's
  getter/evaluator split for the task bank: the getter is the tour's final checkpoint or scenario state, the
  evaluator is the tour's `expect`.
- **ScreenSpot-Pro** ([arXiv HTML](https://arxiv.org/html/2504.07981); MIT, [repo](https://github.com/likaixin2000/ScreenSpot-Pro-GUI-Grounding)):
  23 professional apps, including AutoCAD, SolidWorks, Inventor and Vivado. The best grounding model at release
  scored 18.9% overall, and the CAD category was lowest for the best models (about 12%). Pixel grounding in CAD
  is the weakest skill agents have. **So the task agent targets by accessible role and name through gq's
  probe-resolved, hit-checked clicks, never by coordinates.**
- **What DOCS.md 2c.14 needs on top (new):**
  1. **GUI-only action space:** gq verbs, no `command_run`, no MCP (point 3 of §0).
  2. **An oracle control arm.** Run the same agent on a page generated mechanically from the tour's own
     `hint` lines. If the agent fails the control, the failure is the agent's or the app's accessibility
     (unnamed icon `label ""` nodes, no author ids in AT-SPI; `tools/gui-qc/README.md` "Known limits"), not the
     page's. Report three classes: page defect, a11y defect (file a QC finding), agent grounding failure.
     Without the control, the pass rate measures the agent model.
  3. **Several runs per page;** report a pass rate, because agents are stochastic.
  4. **Keep the run as a chaptered recording** (AXNav) for the reviewer: gq's `actions.ndjson` plus shots.
     The writer still sees only the failing step (DOCS.md's held-out rule).

## 7. Mapping a command registry and an accessibility tree into reference pages

Precedents, all opened:
- **Blender:** each tool page opens with a "Reference" box. The [Bevel page](https://docs.blender.org/manual/en/latest/modeling/meshes/editing/edge/bevel.html)
  reads "Mode: Edit Mode; Menu: Vertex ‣ Bevel Vertices, Edge ‣ Bevel Edges; Shortcut: Shift-Ctrl-B, Ctrl-B".
  The page anchor is the operator id (`bpy-ops-mesh-bevel`). A test, [`bl_rna_manual_reference.py`](https://github.com/blender/blender/blob/main/tests/python/bl_rna_manual_reference.py)
  (GPL-2.0-or-later: read for the idea, don't copy), checks both directions: every RNA path and operator has a
  manual URL ("RNA Patterns Unknown to the Manual"), every mapping is used, and every URL resolves.
- **FreeCAD:** each command declares `sMenuText`, `sToolTipText`, `sStatusTip`, `sWhatsThis`, `sPixmap`,
  `sAccel` ([Command.h](https://github.com/FreeCAD/FreeCAD/blob/main/src/Gui/Command.h), LGPL-2+), and
  `sWhatsThis` is the command id, which is also the wiki page name
  (`sWhatsThis = "Sketcher_ConstrainCoincident"` in [CommandConstraints.cpp](https://github.com/FreeCAD/FreeCAD/blob/main/src/Mod/Sketcher/Gui/CommandConstraints.cpp)).
  Pages are keyed by id, not label. (wiki.freecad.org blocked the fetch, so the page template is [unverified].)
- **KiCad:** one manual per editor, and the PCB Editor manual ends in an "Actions reference" chapter that also
  holds the default hotkeys ([PCB Editor](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html); manuals list at
  [docs.kicad.org](https://docs.kicad.org/)). Whether it is generated is [unverified].
- **Fusion** separates a task page ([Extrude a solid body](https://help.autodesk.com/cloudhelp/ENU/Fusion-Model/files/SLD-EXTRUDE-SOLID.htm):
  11 numbered steps starting "click Solid > Create > Extrude", then Tips) from a reference page
  ([Extrude reference](https://help.autodesk.com/cloudhelp/ENU/Fusion-Model/files/SLD-REF-EXTRUDE.htm)).
  **Onshape** combines both on one page ([Extrude](https://cad.onshape.com/help/Content/extrude.htm): shortcut
  in the header, "Steps for creating solids/surfaces/thin extrudes", then option and end-type sections).
  Fusion's split is the Diátaxis split; Onshape repeats steps per variant. Altium's docs blocked the fetch;
  not assessed.
- **Emacs**, the "self-documenting" editor ([AIM-519A](https://dspace.mit.edu/handle/1721.1/5736)), is the
  in-app half of the same idea.

**Proposal (Layer 0 + 2b.12):** generate, per command id, a reference entry with a Blender-style box. Mode and
context come from the command's availability, Menu/Ribbon/Context paths from `inventory.toml` `menus.*`, the
Shortcut from `key_bindings`, display text from `en.ftl`, arguments from the JSON Schema, refusal reasons from
the registry, "what happens" from the behaviour catalog's `outcome`, and "Learn this" links from the tours that
`routes(command(id))`. Add a Blender-style **bidirectional coverage test**: every registered command and panel
has an entry, every entry's id is registered, and every in-app help link resolves. DOCS.md 2b.12 states one
direction only. **Panel reference pages** get an anatomy figure. Take the panel's AT-SPI subtree (`gq tree`),
number the named controls, place callouts from `gq locate --json` rects, and list each control with its role,
accessible name and tooltip. An unnamed control fails generation. That doubles as an accessibility check,
since a control a screen reader can't name can't be documented either.

## 8. Slots in one line each

Layer 0: behaviour catalog, tours with `expect`, refusal reasons, tour `keywords`. 1d: tours not scenarios;
user vocabulary; Grossman's five categories. 2a.4: feedback where `expect` is hidden, not a rate. 2b.10–11:
menu paths against `inventory.toml` and the AT-SPI tree; pages from TU8; figure–text agreement. 2b.12:
bidirectional coverage. 2c.14: gq only, control arm, pass rate, recording, no model judge. 2d.19: DITA slots
as moves. Layer 3: record, then write. Priority 1: add TU8.

## 9. Top 5 to adopt (leverage per effort)

1. **Make the tour the single source of every GUI how-to, and build TU8.** The design exists (tour note §7),
   the runner and `tour-shots` exist, and 103 tours are already half-written pages. The doc-writer writes doc
   comments; test mode proves the taught route; doc mode makes the page and figures. Evidence: generated from
   demonstration beats hand-designed (Grabler: 20–44% faster, 60–95% fewer errors). Effort: TU8 as the tour
   note scopes it (3–4 in its lane table). Removes DOCS.md's separate GUI pipeline.
2. **Generated reference boxes plus a bidirectional coverage test** (Blender, FreeCAD id-keyed pages), built
   from `inventory.toml`, `en.ftl`, the keymap, refusal reasons and behaviour-catalog outcomes. The same data
   feeds tooltips and "Learn this". No LLM; it answers every Locating question; small effort, because the
   inventory generator exists.
3. **Task agent through gq only, with an oracle control arm and end-state scoring.** OSWorld shows a11y-tree
   agents do best and AT-SPI is what gq reads; WONDERBREAD shows models can't judge completion; ScreenSpot-Pro
   shows CAD pixel grounding is weakest. The control separates page defects from accessibility and agent
   defects. Effort: a prompt and a harness over existing gq verbs.
4. **The figure–text agreement check and the behaviour-catalog digest as staleness inputs.** Deterministic and
   cheap; it covers the failure DOSUD measured in VS Code's docs, which regeneration alone moves rather than
   removes.
5. **Step rules from the procedure literature, mechanised through `expect`:** one action per step; feedback
   only where `expect` is non-obvious (Farkas); generated troubleshooting from refusals (Lazonder & van der
   Meij); full-window figures with callouts, not partial crops (Gellevij & van der Meij); clips only for drag
   and gesture steps (MixT). Each rule becomes a lint over tour data, not a fitted rate.
