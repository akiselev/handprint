# D. Agents as readers and testers of documentation, and docs written for agents

Research for `DOCS.md` (handprint, 2026-10-26). Scope: tool-documentation research, benchmarks where
agents act from docs, simulated users, failure localisation, held-out test design, llms.txt and
docs-for-agents conventions, and how to weight agent pass rates against human readability. Every
source below was opened unless marked [unverified]. "Slot" names the place in `DOCS.md`: 2c.14–17
(reader critics), the Layer 3 loop (step 5.3), the gate (step 6), or a new item.

## Bottom line: what DOCS.md misses

DOCS.md's task agent (2c.14) is the right core, and the literature backs it: DRAFT, PLAY2PROMPT,
Anthropic's tool-writing guide and AgentTrek all run "agent reads doc, acts, outcome is checked". The
design is missing eight things the same literature shows matter:

1. **No-doc control.** An agent with MCP access can do many tasks with no page at all. In cadatomic,
   `commands.list` returns the ids and JSON Schemas of every command. AppAgent reached 48.9% success
   with no document and 73.3–95.6% with one ([AppAgent, Table 1](https://arxiv.org/html/2312.13771v2)).
   A page's score has to be its *lift* over a null page, or an empty page passes.
2. **Surface binding.** The task agent should act through the surface the reader of the page uses.
   A GUI how-to should be run through `ui.click`/`ui.drag`-level actions (the tour layer), with command
   introspection switched off. The RPC scenario runner tests a different contract. Without this the
   agent can swap the documented procedure for a direct command, a shortcut ImpossibleBench shows agents
   take ([Zhong, Raghunathan, Carlini 2025](https://arxiv.org/abs/2510.20270)).
3. **A weak reader as the main reader.** Strong models fill gaps from what they already know, and
   they can't be prompted into playing a weaker reader
   ([Senthil Kumar et al. 2025](https://arxiv.org/abs/2504.06460)).
4. **A train/test split of the task bank.** DOCS.md hides the bank from the writer but uses the same
   bank for feedback and for the gate. Each failure message ("task 7 failed at step 3") leaks a little
   of the bank. DRAFT measured a rise-then-fall over iterations, which it attributes to overfitting
   ([DRAFT](https://arxiv.org/html/2410.08197v2)), and Anthropic gates on held-out tasks
   ([Anthropic 2025](https://www.anthropic.com/engineering/writing-tools-for-agents)).
5. **Efficiency as well as pass/fail.** Making MCP descriptions fuller raised median success by
   5.85 pp, but it also raised execution steps by 67% and made 16.7% of cases worse
   ([Hasan et al. 2026](https://arxiv.org/abs/2602.14878)). A page that passes at three times the
   steps is a worse page.
6. **Repeats and reliability.** One run is noise. τ-bench found pass^8 below 25% where single-run
   success was near 50% ([Yao et al. 2024](https://arxiv.org/abs/2406.12045)).
7. **Failure localisation is weak as specified.** Asking an LLM which step caused a failure is
   unreliable: 14.2% step accuracy on Who&When
   ([Zhang et al. 2025](https://arxiv.org/abs/2505.00212)). DOCS.md should localise
   deterministically: the first scenario step whose app state diverges, then sentence ablation.
8. **The agent-facing docs are not tested by agents.** The command catalog text and the MCP tool
   descriptions are documentation that agents read. DOCS.md generates them from the registries but
   never runs an agent against them. The MCP-smell studies show this is where most of the defects are
   ([Hasan et al.](https://arxiv.org/abs/2602.14878);
   [Wang et al. 2026](https://arxiv.org/abs/2602.18914)).

## 1. Tool-documentation research (agents learning tools from docs)

| Work | Method | Evidence | Cost / code / licence | Slot; redundancy |
|---|---|---|---|---|
| Hsieh et al. 2023, *Tool documentation enables zero-shot tool-usage* ([abs](https://arxiv.org/abs/2308.00675), [html](https://arxiv.org/html/2308.00675)) | Replace few-shot demos with tool docs | On a 200-tool "LLM Cloud CLI" set, F1 went 0.13 → 0.45 for gpt-3.5 (no docs → docs). Adding 5–15 demos on top of the docs scored no higher (0.42–0.44). Success rose with doc length to about 600 words and fell after that | Prompting only; no code link found | Evidence for 2c.14 and for a per-section **length band** in 2a.7. Its no-docs arm is the control in gap 1 |
| EASYTOOL ([abs](https://arxiv.org/abs/2401.06201), [html](https://arxiv.org/html/2401.06201v3)) | Rewrite long, inconsistent docs into a fixed short instruction: a description plus per-function usage scenarios with example parameters | ToolBench docs cut from 2,530 to 748 tokens. ChatGPT+DFSDT success rate 15.0 → 52.8 | Code was "to come" at microsoft/JARVIS [unverified]. Cheap: one LLM pass per tool | A **template** for generated reference entries (Layer 0). Not a critic |
| DRAFT, *From Exploration to Mastery*, ICLR 2025 oral ([abs](https://arxiv.org/abs/2410.08197), [html](https://arxiv.org/html/2410.08197v2)) | An Explorer writes queries that probe the tool, an Analyzer compares doc against outcome and writes suggestions, a Rewriter edits the doc and picks the next direction. New queries must have cosine similarity below 0.9 to earlier ones. The loop stops when consecutive versions are near-identical (mean of BLEU and embedding similarity above 0.75), at most 5 rounds | GPT-4o correct-path rate 71 → 88 (TMDB) and 28 → 70 (Spotify). Docs refined with GPT-4o also helped GPT-4o-mini and Llama-3-70B. Human raters preferred the revised docs for accuracy (56% vs 0% on ToolBench), κ 0.69–0.77. Performance falls again after the peak | [github.com/quchangle1/DRAFT](https://github.com/quchangle1/DRAFT), MIT. Cost not reported | **The closest existing system to the Layer 3 loop.** Adopt two pieces: the diversity rule (for task generation) and the convergence stop (alongside the churn guard 2a.8). Note that DRAFT's writer sees whole trajectories; DOCS.md's blind writer is stricter, which is fine |
| PLAY2PROMPT, ACL Findings 2025 ([abs](https://arxiv.org/abs/2503.14432), [html](https://arxiv.org/html/2503.14432)) | Two beam searches. (a) Generate tool-use examples by trial and error, rewarding *difficulty*: how often the task model fails them using the current doc. (b) Rewrite the doc to maximise accuracy on those examples | BFCL: Llama-8B 85.9 → 92.9. StableToolBench: 55.8 → 62.2. The generator was Llama-3.1-8B | [github.com/wfangtw/play2prompt](https://github.com/wfangtw/play2prompt), **no licence file** (all rights reserved by default). Beam width 3–10 with 3 proposals each, so roughly 10–30 evaluations per round | **Adversarial task-bank growth** for 1d: reward tasks the current page fails. It sets a test generator against the writer, which is the anti-Goodhart structure DOCS.md wants |
| Trace-Free+, *Learning to Rewrite Tool Descriptions* (Feb 2026) ([abs](https://arxiv.org/abs/2602.20426)) | A trained rewriter: supervised on agent traces, moved by curriculum to a setting with no traces | StableToolBench with 150+ tools: success +60.89%, degradation −29.23% | No code. Needs training | Not for now; it shows the writer can work without traces. Record in REVISIT |
| Hasan et al., *MCP Tool Descriptions Are Smelly!* (2026) ([abs](https://arxiv.org/abs/2602.14878)) | Rubric of six components from the literature, an LLM scanner, then augmentation and ablation | 856 tools: 97.1% have a smell, 56% have no clear purpose. Augmenting everything gave +5.85 pp median success but +67% steps and regressions in 16.7% of cases. Compact combinations kept most of the gain | No code found | **New 2b/2c item** for MCP tool and command descriptions. It is also the evidence for an **efficiency band** at the gate |
| Wang et al., *From Docs to Descriptions* (2026) ([abs](https://arxiv.org/abs/2602.18914)) | Four dimensions (accuracy, functionality, completeness, conciseness), 18 smells, a mutation study | Over 10,831 servers, compliant descriptions were selected 72% of the time against 20% for alternatives. Functionality and accuracy mattered most (+11.6% and +8.8%) | Dataset said to be released; no link found | Same slot. Makes **tool selection** (does the agent pick the right command?) a cheap test, run before full tasks |
| Anthropic, *Writing effective tools for agents* (2025) ([post](https://www.anthropic.com/engineering/writing-tools-for-agents)) | Generate realistic multi-call tasks, run a simple agent loop, track accuracy, tool calls, tokens and errors, have Claude read the transcripts and refactor descriptions, gate on held-out tasks | Qualitative; reports that description edits helped reach SOTA on SWE-bench Verified | No code | Supports gaps 4 and 5. Its advice to ask the agent for reasoning and feedback blocks gives a **structured complaint channel** (see §5) |
| GEPA, ICLR 2026 oral ([abs](https://arxiv.org/abs/2507.19457)) | Reflect on trajectories in natural language, mutate the prompt, keep a Pareto front across task instances, separate train and validation sets | Beats GRPO by 6% on average with up to 35× fewer rollouts | [gepa-ai/gepa](https://github.com/gepa-ai/gepa), MIT | **Pareto selection over the task bank:** keep the revisions that win on *different* tasks rather than one scalar. This addresses #24's report-chasing directly |

## 2. Benchmarks where agents must read docs to act

- **DocPrompting** ([Zhou et al., ICLR 2023](https://arxiv.org/abs/2207.05987)): retrieving the docs
  for a request improves NL→Bash and NL→Python generation (CoNaLa pass@1 +2.85 pts, a 52% relative
  gain). Code at [shuyanzhou/docprompting](https://github.com/shuyanzhou/docprompting). Its `tldr`
  set is a model for **CLI question banks** (Copperfield `USAGE`, qc's clap help). Slot 2c.15;
  partly redundant with it.
- **CodeUpdateArena** ([Liu et al. 2024](https://arxiv.org/abs/2407.06249)): putting an API update's
  docs in front of open code models did *not* make them use the changed behaviour. The lesson for a
  fast-changing codebase: an agent's prior beats a page that contradicts it. The question bank should
  include **"changed behaviour" questions**, where the registry's current answer differs from the
  obvious guess. Slot 1d.
- **Chen et al. 2025, *When LLMs Meet API Documentation*** ([abs](https://arxiv.org/abs/2503.15231)):
  over 1,017 APIs from less common libraries, retrieval-augmented generation improved results by
  83–220%. **Example code helped most**, more than descriptions or parameter lists. Models tolerated
  mild noise such as typos and wrong parameters. So agents are *less* sensitive than humans to small
  errors (see §7), and the 2a.7 code-block band carries agent value too.
- **CodeRAG-Bench** ([Wang et al.](https://arxiv.org/abs/2406.14497)): retrievers fail when the query
  and the doc share few words, and generators don't combine several contexts reliably. For cadatomic
  this means a **retrieval test**: give the reader the whole site, not just the target page, and
  check it finds the page. Docs written in canonical catalog vocabulary (2a.5) also need the user's
  synonyms somewhere, or lexical retrieval misses them.
- **SPRING** ([Wu et al. 2023](https://arxiv.org/abs/2305.15486)): GPT-4 reading the Crafter paper's
  LaTeX beat RL baselines trained for 1M steps. It shows "agents read the manual" works. Background
  only.
- **AppAgent** ([abs](https://arxiv.org/abs/2312.13771), [Table 1](https://arxiv.org/html/2312.13771v2),
  [repo, MIT](https://github.com/mnotgod96/AppAgent)): an agent writes docs for UI elements while it
  explores. Success was 48.9% with no doc, 73.3% with exploration docs and 95.6% with hand-written
  docs. Two lessons: the **no-doc baseline is high** (gap 1), and docs written from exploration lose
  to curated ones. That supports DOCS.md's Layer 0 (facts from registries, not from an agent poking
  the app).
- **AgentTrek**, ICLR 2025 spotlight ([abs](https://arxiv.org/abs/2412.09605)): web tutorials are
  turned into task specs, a VLM agent replays them in real environments, and a VLM evaluator filters
  the trajectories. About $0.55 per trajectory kept. This is 2c.14 run in reverse (tutorials testing
  agents), and an existence proof that **tutorial replay** is cheap at scale. Redundant with 2c.14; it
  gives a cost anchor. No code licence found (the paper is CC BY-NC-ND).
- **DocOS** ([Liu et al. 2026](https://arxiv.org/abs/2605.18048)): GUI agents must find official
  documentation and ground its steps into GUI actions. They fail at both, and grounding in particular.
  A leaderboard summary says the correct manual helped weak agents by up to 10.8% and strong ones only
  slightly [unverified, secondhand: [benchmarklist](https://benchmarklist.com/benchmarks/docos_towards_proactive_document_guided_actions_in_gui_agents/)].
  If that holds, it argues for gap 3: weak readers are more sensitive to doc quality.
- **Evaluating AGENTS.md** ([Gloaguen et al. 2026](https://arxiv.org/abs/2602.11988), ICLR 2026):
  repository context files did not generally raise SWE-bench success and raised cost by more than 20%.
  Repository overviews "are not helpful". Context files help when they state non-standard practices.
  This applies to any docs DOCS.md writes *for agents*: minimal, only the non-obvious, and
  evaluated before rollout.
- **Compact Documentation for Coding Agents** ([Arman & Molybog, Sep 2026](https://arxiv.org/abs/2609.31587)):
  a "roundtrip" benchmark (regenerate code from its description; does it pass the original tests?) was
  optimised to full fidelity, and the gain **did not transfer** to resolving real issues. A positive
  control showed the evaluation could detect real gains. **This is the strongest warning against
  trusting a single executable reader signal:** a page can be optimised against one reader task and
  help nothing else. It supports the ensemble gate, and argues for gating on tasks *unlike* the ones
  used for feedback.

## 3. Simulated users and persona testing

- **UXAgent** ([2502.12561](https://arxiv.org/abs/2502.12561); system paper
  [2504.09407](https://arxiv.org/abs/2504.09407)): a persona generator, an LLM agent and a browser
  connector run thousands of simulated users against a site, with interviews and replays. UX
  researchers found the output useful, but by a secondary review not fully realistic. Slot: optional
  **persona variants** of the task agent (a "Blender user", a "KiCad user", a "new to CAD" user), each
  bringing different priors.
- **Personas can't play down** ([Senthil Kumar et al.](https://arxiv.org/abs/2504.06460)): models,
  o1 included, fail to simulate lower-performing personas. A "pretend you're a novice" prompt won't
  produce a novice. Use **real limits** instead: a smaller model, a restricted toolset, no
  introspection, a token budget. Slot 2c.14, reader ladder.
- **Doc Detective** ([npm](https://npmjs.com/package/doc-detective),
  [GitHub](https://github.com/doc-detective/doc-detective), MIT per third-party listings
  [licence unverified at source]): docs-as-tests that pull testable actions out of doc text and run
  them in a browser. **Redundant** with 2b.11 (scenarios that replay), which is stronger because
  cadatomic has a typed command log. Not worth adopting. Its "inline test spec in the page" idea
  matches binding a page to a hidden scenario id.

## 4. Failure localisation: which step and which sentence

- **Who&When** ([Zhang et al. 2025](https://arxiv.org/abs/2505.00212),
  [code](https://github.com/mingyin1/Agents_Failure_Attribution)): LLM judges found the responsible
  agent 53.5% of the time but the decisive step only 14.2%. Reasoning models didn't help. Attribution
  was better **aggregated over many logs** than per instance. Later methods report 29–52% step accuracy
  [unverified, search snippets only].
- **AgentDebug** ([Zhu et al. 2025](https://arxiv.org/abs/2509.25370)): an error taxonomy (memory,
  reflection, planning, action, system), root-cause step localisation, and corrective feedback. Step
  accuracy +17% over baselines. Code was promised at ulab-uiuc/AgentDebug [unverified availability].
- **ContextCite** ([Cohen-Wang et al. 2024](https://arxiv.org/abs/2409.00729),
  [MadryLab/context-cite](https://github.com/MadryLab/context-cite), MIT): attributes a model's output
  to parts of its context by ablating them. For docs: **drop or mask one sentence or section, re-run
  the reader, and see whether the failure moves**. This is the honest answer to "which sentence caused
  it", and its cost grows with the number of sections, not with the bank.

**What this means for DOCS.md.** cadatomic already has ground truth for steps. A scenario is a list
of typed steps, each with an `expect` and a golden `tree`
(`/home/user/cadatomic/scenarios/a8a-smoke.ndjson`). So localise in three tiers:

1. **Deterministic:** the first task-bank checkpoint whose app state differs from the expected
   state, which the end-state check already computes.
2. **Statistical:** aggregate across repeats and readers (Who&When's statistical-level result):
   "4 of 6 runs diverge at checkpoint 3".
3. **Causal, on demand:** section ablation à la ContextCite, run only for pages that fail the gate.

Hand the writer a bank-free summary: page section, failure class from a small taxonomy (missing
prerequisite, ambiguous referent, wrong or non-canonical name, missing expected result, step order),
and the count. Never the task text.

## 5. Held-out design: keeping the writer from overfitting the tester

The sources agree on these mechanisms, each with a slot:

- **Split the bank.** A *feedback split* the writer gets failure summaries from, and a *gate split*
  that only produces pass/fail at step 6 and is never fed back
  ([Anthropic](https://www.anthropic.com/engineering/writing-tools-for-agents); GEPA's
  trainset/valset). Rotate the gate split per batch. Slots 1d and the gate.
- **Keep generating hard tasks** (PLAY2PROMPT's difficulty reward; DRAFT's diversity threshold of
  0.9): new tasks are drafted from the registry and kept when the *current* page fails them. Slot 1d.
- **Stop on convergence and keep the best version, not the last.** DRAFT's curve rises and then
  falls. Slot: loop step 5.3, next to the churn guard 2a.8.
- **Readers from a different family than the writer**, since LLM judges recognise and favour their
  own outputs ([Panickssery et al. 2024](https://arxiv.org/abs/2404.13076)). This mostly matters for
  2c.15 and 2c.16, where an LLM judges the answer. For 2c.14 the app's state is the judge, so the bias
  is smaller, but a same-family reader shares the writer's priors and may fill the same gaps.
- **Check the trajectory as well as the end state.** ImpossibleBench shows agents take spec-violating
  shortcuts ([2510.20270](https://arxiv.org/abs/2510.20270)). For a GUI how-to the shortcut is calling
  the command directly. Require that the command log contains the invocation records the page's steps
  imply. The command layer already logs one record per action ("GUI, shortcuts, palette, RPC, MCP …
  produce the same invocation record", cadatomic CLAUDE.md), so this check is nearly free: each log record carries `source: CommandSource`
  (`Gui`, `ActionBar`, `RepeatLast`, `Rpc`, `Cli`, `Script`, `Replay`;
  `/home/user/cadatomic/crates/cadatomic-core/src/log/mod.rs:62`). A GUI how-to passes only if its
  invocations are `Gui`/`ActionBar`, not `Rpc`.
- **Transfer test** (Arman & Molybog): at the pilot, score the pipeline's pages on a *different*
  reader task family, such as questions when they were tuned on tasks, or the reverse. If the gain
  doesn't transfer, the loop is overfitting.

## 6. llms.txt and docs-for-agents conventions

- **llms.txt** ([llmstxt.org](https://llmstxt.org/), Jeremy Howard, 3 Sep 2024): a Markdown index
  with an H1, a blockquote summary, H2 link lists and an "Optional" section, plus `.md` mirrors of
  every page. The site gives adoption but no controlled measurements. A vendor benchmark (Mintlify,
  which sells the product) reports gains for agent-readable docs over a no-docs baseline: 64% more
  precise answers and about 50% fewer tokens
  ([Mintlify](https://www.mintlify.com/blog/structured-docs-coding-agents); not independent, suite
  unreleased). **Adopt as generated output** of Layer 0 (cheap, no LLM). Don't claim effectiveness;
  test it with the retrieval test in §2. Slot: Layer 0 and step 7, maintain.
- **AGENTS.md / CLAUDE.md** ([Gloaguen et al.](https://arxiv.org/abs/2602.11988)): keep context files
  for agents minimal and limited to non-standard rules. Overviews don't help and add cost. This applies
  to an "agent guide" for the MCP surface.
- **MCP tool descriptions** (the two smell studies above): purpose, when to use, parameters with
  meaning and units, return shape, limits, one example, kept compact. cadatomic's MCP tools are built
  from `commands.list`/`methods.list`, so the descriptions come from the command registry and
  `en.ftl`. **Apply the rubric at the source** (DOCS.md already says source text is critiqued at the
  source; this gives that critique its content), and test with a tool-selection check: given a task,
  does the agent pick the right command id from the catalog?

## 7. Weighting agent pass rates against human readability

Agents and humans fail differently:

| | Humans | Agents |
|---|---|---|
| Reading | 79% scan, 16% read word by word ([Nielsen 1997](https://www.nngroup.com/articles/how-users-read-on-the-web/)) | Read everything, but miss content in the middle of long contexts ([Liu et al., TACL](https://arxiv.org/abs/2307.03172)); success falls past about 600 words (Hsieh) |
| Small errors | Ambiguity, incompleteness and incorrectness are the worst problems; 6 of 10 problem types are "blockers" ([Uddin & Robillard 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)) | Tolerate typos and wrong parameters (Chen 2025) |
| Gaps | A novice is stuck | Fill gaps from priors or by introspecting the app (AppAgent no-doc 48.9%) |
| Shared | Examples help most (Chen 2025; Robillard) | Same |

So the agent signal is **necessary for correctness and completeness and blind to presentation**. A
page can pass every agent run and still be a wall of text a human won't scan. Recommendation:

- **Lexicographic gate, not a weighted sum.** First: verifiers (2b) at 100% and task-agent *lift*
  above threshold on the gate split, at both rungs of the reader ladder. Then: the readability
  ensemble (2a doc gate, 2c.16 pairwise against an anchor). A weighted sum lets a high agent score
  buy off a wall of text, which is the Goodhart route.
- **A skim reader** (new, cheap): a reader agent given only the headings, first sentences, code
  blocks and figures (a mechanical scan view) must find the step or section that answers a question.
  It is a machine proxy for the 79% who scan, and it uses the agent infrastructure to test human
  presentation.
- **Weak-model sensitivity as a proxy for novices.** Gate on the weak reader. Report the strong
  reader's result as informational: it upper-bounds what the page gives.
- **Human calibration** is outside what lanes may ask for. Raise it as an open question for the
  user: a few pages per batch read by a person, to check that the agent ensemble agrees with people.

## 8. Concrete changes to DOCS.md, by slot

| Slot | Change | Redundant with DOCS.md? |
|---|---|---|
| 2c.14 | Score = pass(page) − pass(null page), with n ≥ 3 repeats, reported as pass^k; also record steps and tokens against the reference trajectory | Extends |
| 2c.14 | Bind the surface: GUI how-tos run through `ui.*` actions with `commands.list` off; API pages through the scenario runner; CLI pages through a shell | New |
| 2c.14 | Trajectory check: the command log's invocation records match the page's surface and its implied commands | New |
| 2c.14 | Reader ladder: a weak model (gate) and a strong model (info), different family from the writer; optional persona variants with real limits, not role-play | New |
| 2c.15 | Add "changed behaviour" and "not covered, say so" questions; a retrieval variant across the whole site | Extends |
| 2c (new 18) | Tool-selection and description check for MCP/command catalog text: the smell rubric plus "does the agent pick the right id" | New |
| 2c (new 19) | Skim reader over a scan view | New |
| 1d | Feedback split vs gate split; rotate per batch; adversarial growth (PLAY2PROMPT); diversity threshold (DRAFT) | Extends ("held out" is there, the split is not) |
| Loop 5.3 | Bank-free failure summary: section, failure class, count; ablation only on gate failures; DRAFT's convergence stop; keep the best version | Extends ("which task failed and at which step" is there) |
| Gate 6 | Lexicographic order; efficiency band (steps and tokens ≤ k × reference); transfer check at the pilot | Extends |
| Layer 0 / 7 | Generate llms.txt and `.md` mirrors; an AGENTS-style agent guide kept minimal | New, cheap |
| Pilot | Add arm (d), a null page, and a weak/strong reader split; check transfer across task and question families | Extends |

Already in DOCS.md and confirmed by the literature, no change: the executed task agent with an
end-state judge (τ-bench, AgentTrek, DRAFT), facts from registries rather than exploration (AppAgent),
an ensemble gate (Compact Docs non-transfer), and the churn guard (DRAFT's rise and fall).

## Top 5 to adopt (leverage per effort)

1. **Null-page control and lift scoring in 2c.14** (plus n ≥ 3 repeats and pass^k). Small effort:
   one extra run condition. Without it the task agent can't tell a good page from an empty one,
   because MCP introspection solves many tasks unaided (AppAgent 48.9% with no doc).
2. **Surface binding plus a trajectory check from the command log.** Small effort: cadatomic already
   records one invocation record per action and has `ui.*` actions. It stops the main shortcut (the
   agent bypasses the GUI procedure) and makes the end-state judge mean what the page claims.
3. **Feedback/gate split of the task bank, rotation, and best-version stopping.** Small effort, mostly
   bookkeeping. It is the documented defence against overfitting the tester (Anthropic, GEPA, DRAFT's
   curve, Compact Docs' non-transfer).
4. **Deterministic failure localisation:** the first diverging checkpoint, aggregated across runs,
   with section ablation only for gate failures, returned as a bank-free failure class. Medium effort.
   It replaces LLM step attribution, which is 14% accurate.
5. **An agent check for the agent-facing docs:** the MCP smell rubric and a tool-selection test on the
   command catalog text, plus generated llms.txt. Medium-small effort. It covers the one doc surface
   agents read most, the evidence is direct (97% of MCP descriptions have smells; compliant ones are
   picked 72% vs 20%), and the cost stays bounded because compact fixes keep most of the gain.

Runners-up: the weak-model reader ladder (cheap, but the evidence for it is partly secondhand), the
skim reader (cheap, untested), and PLAY2PROMPT-style adversarial task growth (more effort; worth it
once the bank exists).
