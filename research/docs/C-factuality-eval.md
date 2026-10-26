# C. Factual consistency, automatic evaluation, and LLM doc-generation systems

Research for `DOCS.md` (handprint, 2026-10-26). I opened every source cited below unless it is marked [unverified]. Most of them I read only at abstract level. Numbers I took from the full text say so. A licence marked "LICENSE file" is one I read from the repository's `LICENSE` on `raw.githubusercontent.com`. GitHub's API was not open to this session, so star counts and activity are not given.

## 1. What DOCS.md already has, and the gap this area finds

DOCS.md already has these parts:

- a fact sheet of claims bound to ids (Layer 3.3);
- a resolution checker, where every identifier resolves (2b.10);
- executed snippets and procedures (2b.11, 2c.14);
- a registry-generated question bank answered from the page (2c.15);
- a pairwise judge against anchors (2c.16);
- cross-page contradiction checks (2c.17).

Measured against the factual-consistency literature, one hole stands out. **Nothing checks that the sentences the page actually contains are entailed by the fact sheet or the sources.**

- The fact sheet is what the writer *meant* to say.
- The resolution checker only confirms that `sketch.constrain.coincident` exists. A page that says it "makes two curves tangent" passes, and so does a page that says the default tolerance is 1e-6 when it is 1e-9.
- The question bank runs in the *recall* direction: questions come from the registry, answers from the page. It finds omissions and some wrong statements, but only where the bank happens to ask.

Every factual-consistency method below also runs the *precision* direction: every claim the text makes is checked against the source. That is the main thing to add. The other gaps:

- judge-bias controls;
- a calibration set for the critics themselves;
- behavioural testing of API prose;
- a few Goodhart guards.

## 2. Factual-consistency metrics

### 2.1 QA-based: QAGS, QuestEval, QAFactEval

| Method | Algorithm | Human agreement | Code / licence |
|---|---|---|---|
| QAGS (Wang, Cho, Lewis, ACL 2020) | Generate questions **from the generated text**, answer them on both the source and the generated text, compare the answers. Disagreeing answers locate the inconsistent span. | Pearson **54.53** on CNN/DM vs 17.72 for ROUGE-2, and **17.49** on XSUM (full text, Table 1). [arXiv:2004.04228](https://arxiv.org/abs/2004.04228) | No official repo linked from the abstract page |
| QuestEval (Scialom et al., EMNLP 2021) | A unified QA framework that needs no reference. It asks questions in both directions (source→summary and summary→source), giving precision and recall. | Higher correlation with humans than prior QA metrics, ROUGE and BERTScore on consistency, coherence, fluency and relevance (abstract gives no figures). [arXiv:2103.12693](https://arxiv.org/abs/2103.12693) | [recitalAI/QuestEval](https://github.com/recitalAI/QuestEval), MIT (LICENSE file) |
| QAFactEval (Fabbri et al., NAACL 2022) | Studies the components of QA metrics. Question generation and **answerability classification** are what matter. | **14%** average improvement over earlier QA metrics on the SummaC benchmark, and better than the best entailment metric. QA and NLI signals are complementary and combine for a further gain. [arXiv:2112.08542](https://arxiv.org/abs/2112.08542) | salesforce/QAFactEval, BSD-3-Clause (LICENSE file) |

**What it catches.** Wrong entities, values and relations in the generated text. Unanswerable questions find content that is missing.

**Cost.** One QG pass and two QA passes per question. The models are small, but there are many questions per page.

**Where it slots in.** The QAGS/QuestEval *precision* direction is the missing half of 2c.15. Generate questions *from the page*, then answer them from the fact sheet plus the cited registry entries.

**Redundancy.** DOCS.md 2c.15 is the recall half already, and it is better than QA-metric recall because its questions come from ground-truth registries.

**Recommendation.** Don't adopt QA metrics as such. The atomic-claim methods below do the precision direction more directly, and they produce spans and per-claim verdicts the writer can act on. TRUE ([arXiv:2204.04991](https://arxiv.org/abs/2204.04991)), a meta-evaluation over 11 annotated datasets, found that large NLI and QG/QA approaches are the best, and that they are complementary.

### 2.2 NLI-based: SummaC, AlignScore, MiniCheck

| Method | Algorithm | Agreement | Code / licence | Cost |
|---|---|---|---|---|
| SummaC (Laban et al., TACL 2022) | Split the document and the summary into sentences, score NLI on each sentence pair, then aggregate (SummaC-Conv learns the aggregation). | 74.4% balanced accuracy on the six-dataset SummaC benchmark, about 5 points above prior work. [arXiv:2111.09525](https://arxiv.org/abs/2111.09525) | [tingofurro/summac](https://github.com/tingofurro/summac), Apache-2.0 (LICENSE file) | Cheap, NLI-sized model |
| AlignScore (Zha et al., ACL 2023) | One alignment function trained on 4.7M examples from seven tasks (NLI, QA, paraphrase, fact verification, IR, similarity, summarisation). It chunks the context and splits the claim into sentences. | Over 22 datasets (19 unseen), it "matches or even outperforms" metrics built on ChatGPT and GPT-4. [arXiv:2305.16739](https://arxiv.org/abs/2305.16739) | yuh-zha/AlignScore, MIT (LICENSE file); the `yzha/AlignScore` weights are MIT on Hugging Face | **355M** parameters; runs on CPU |
| MiniCheck (Tang, Laban, Durrett, EMNLP 2024) | A small checker trained on synthetic hard cases. Input is a grounding document and a claim; output is supported or not. | MiniCheck-FT5 (770M) "reaches GPT-4 accuracy" on LLM-AggreFact at a claimed **400× lower cost**. [arXiv:2404.10774](https://arxiv.org/abs/2404.10774) | Liyan06/MiniCheck, Apache-2.0 (LICENSE file); `lytang/MiniCheck-Flan-T5-Large` weights MIT. **Bespoke-MiniCheck-7B is CC BY-NC 4.0** ([card](https://huggingface.co/bespokelabs/Bespoke-MiniCheck-7B)), so it is unusable without a commercial licence. | 770M |

**What it catches.** A sentence that the evidence does not support, or that it contradicts.

**Where it slots in.** It is the verifier inside a **Layer 3 gate**: "each page claim entailed by its bound registry entry and source snippet". It is also the cheap pre-filter for 2c.17 contradiction pairs.

**Redundancy.** None. DOCS.md has no entailment step.

**Caveat.** All of these were trained on news, Wikipedia and dialogue. Registry entries (JSON Schema, Fluent strings, Rust signatures) are out of domain. They need a calibration run on our own mutants (§7) before anyone trusts a threshold.

### 2.3 Atomic-claim methods: FActScore, SAFE, VeriScore, claim decomposition

| Method | Algorithm | Agreement | Code / licence |
|---|---|---|---|
| FActScore (Min et al., EMNLP 2023) | Split the generation into atomic facts and report the share a knowledge source supports. The automatic version uses retrieval plus an LM. | Automatic estimate within **<2% error** of human. Human evaluation of 6,500 generations was estimated at **$26K**. [arXiv:2305.14251](https://arxiv.org/abs/2305.14251) | [shmsw25/FActScore](https://github.com/shmsw25/FActScore), MIT (LICENSE file) |
| SAFE / LongFact (Wei et al., 2024) | An LLM splits the response into facts, then runs multi-step Google searches per fact. The metric is F1@K: precision is the supported share, recall is facts provided relative to a preferred count K. | **72%** agreement with crowd annotators on ~16k facts. SAFE wins **76%** of 100 disagreements. **>20× cheaper** than crowd annotation. [arXiv:2403.18802](https://arxiv.org/abs/2403.18802) | [google-deepmind/long-form-factuality](https://github.com/google-deepmind/long-form-factuality), Apache-2.0 (vendored FActScore MIT) |
| VeriScore (Song, Kim, Iyyer, 2024) | Extracts and checks only **verifiable** claims. FActScore and SAFE assume every sentence is checkable. | Humans found its extracted claims more sensible than competitors' on eight long-form tasks. Scores on one task don't predict another. [arXiv:2406.19276](https://arxiv.org/abs/2406.19276) | Yixiao-Song/VeriScore, Apache-2.0 (LICENSE file) |
| Claim decomposition (Wanner et al., 2024) | Shows that FActScore "is sensitive to the decomposition method used". Introduces DecompScore to measure decomposition quality on its own. [arXiv:2403.11903](https://arxiv.org/abs/2403.11903) | n/a | n/a |
| Claimify (Metropolitansky & Larson, Microsoft, 2025) | Extracts a claim only when it is confident of the reading, and evaluates extraction for entailment, coverage and decontextualisation. [arXiv:2502.10855](https://arxiv.org/abs/2502.10855) | Beats prior extractors on its own framework | No code release found [unverified] |

**What it catches.** Every unsupported statement, with a span. That is exactly the "invented details" the pilot predicts for arm (a).

**Cost.** One decomposition call per page, plus one verifier call per claim. With MiniCheck or AlignScore as the verifier, the second part is nearly free.

**Where it slots in.** It is the **page→fact-sheet entailment gate** in Layer 3 step 6, placed after the 2b verifiers.

**Two lessons carry over directly.**

1. **Pin and version the decomposer**, as #26 says for the normaliser. A factuality score is partly a score of the decomposer.
2. **Score only verifiable claims** (VeriScore). Docs contain orientation and rhetoric ("This page shows how…") that should neither count as supported nor fail.

**Our advantage over all four.** The fact sheet is already structured. Claim extraction can therefore be *alignment*: map each page sentence to fact-sheet entries or to "no entry". Open-domain decomposition is not needed. A sentence that makes a verifiable claim and aligns to nothing is an unsupported claim, whatever the NLI model says.

### 2.4 Attribution and citation: AIS, ALCE, Attributed QA

- **AIS** (Rashkin et al., *Computational Linguistics* 2023) defines "attributable to identified sources" with a two-stage human annotation: is the output interpretable, and is it supported by the source. It was tested on conversational QA, summarisation and table-to-text ([arXiv:2112.12870](https://arxiv.org/abs/2112.12870)).
- **Attributed QA** (Bohnet et al., 2022) treats human AIS ratings as gold and finds an automatic metric that correlates with them ([arXiv:2212.08037](https://arxiv.org/abs/2212.08037)). I didn't open the AutoAIS details [unverified].
- **ALCE** (Gao et al., EMNLP 2023) is the operational version ([full text](https://arxiv.org/html/2305.14627)).
  - **Citation recall:** a statement scores 1 if it has a citation and the concatenated cited passages entail it, judged by TRUE (a T5-11B NLI model).
  - **Citation precision:** each citation must be needed.
  - **Agreement with humans:** Cohen's κ **0.698** for recall and **0.525** for precision; accuracy 85.1% and 77.6%.
  - **Headline finding:** even the best models lack complete citation support 50% of the time on ELI5.
  - Code: [princeton-nlp/ALCE](https://github.com/princeton-nlp/ALCE), MIT (LICENSE file).

**Where it slots in.** Layer 3 step 3 already binds fact-sheet claims to ids. Carry the binding into the rendered page: each sentence carries hidden `<!-- ids: … -->` anchors, the way DOCS-PLAN's `sources:` front matter does at page level. The gate then computes ALCE-style **citation recall** (every verifiable sentence cites ≥1 id, and the cited entries entail it) and **citation precision** (no decorative ids).

The anchors also give 2b.13 staleness *per sentence* instead of per page.

**Redundancy.** Partial. DOCS.md binds claims at the fact-sheet stage only.

### 2.5 Self-consistency: SelfCheckGPT and self-contradiction

- **SelfCheckGPT** (Manakul et al., EMNLP 2023) samples N responses and scores each sentence by its consistency with the samples. The variants are BERTScore, MQAG, n-gram, NLI and LLM-prompt. It is zero-resource and black-box ([arXiv:2303.08896](https://arxiv.org/abs/2303.08896)). The [repo README](https://github.com/potsawee/selfcheckgpt) (MIT) reports Prompt with gpt-3.5-turbo as best (NonFact AUC-PR 93.42) and NLI as the best open variant.
- **Self-contradiction** (Mündler et al., ICLR 2024): 17.7% of ChatGPT's sentences were self-contradictory. A prompting detector reaches about 80% F1, and 35.2% of the contradictions could not be verified against online text ([arXiv:2305.15852](https://arxiv.org/abs/2305.15852)).

**Where it slots in.** It is weak where we have ground truth, which covers most pages. It is useful in one place: **explanation pages and design rationale**, whose claims come from `DECISIONS.md` prose and not from a registry. When drafting the fact sheet for such a page, sample it three times and flag claims that appear in only one sample for human or coordinator review.

**Redundancy.** None, and it is low priority.

## 3. LLM-as-judge reliability for long-form technical text

| Bias / finding | Evidence | What it means for 2c.16 |
|---|---|---|
| GPT-4 agrees with humans at **>80%**, about human–human level; it has position, verbosity and self-enhancement biases | Zheng et al., MT-Bench ([arXiv:2306.05685](https://arxiv.org/abs/2306.05685)) | The baseline is usable, but the biases are documented and systematic |
| **Position bias:** with ChatGPT as judge, the order of answers let Vicuna-13B beat ChatGPT on 66 of 80 queries. Fixes: multiple-evidence calibration, *balanced position* (judge both orders and aggregate), human in the loop on high-entropy items | Wang et al. ([arXiv:2305.17926](https://arxiv.org/abs/2305.17926)) | `readable.md` randomises order once. **Judge both orders**; a verdict that flips counts as a tie |
| The size of position bias depends strongly on the quality gap; it is worst when the candidates are close. Measured over 15 judges and 150k evaluations | Shi et al. ([arXiv:2406.07791](https://arxiv.org/abs/2406.07791)) | "Ties or beats the anchor" is exactly the close-gap regime, so it needs both orders |
| **Verbosity:** AlpacaEval favours longer outputs. A GLM controlling for length difference raised Spearman with Chatbot Arena from 0.94 to **0.98** | Dubois et al. ([arXiv:2404.04475](https://arxiv.org/abs/2404.04475)) | Record the length ratio with each verdict. Fit the same GLM over the pilot's verdicts, or cap the length difference against the anchor |
| **Self-preference:** LLMs rate their own output higher than humans do, and the strength scales linearly with self-recognition ability | Panickssery et al., NeurIPS 2024 ([arXiv:2404.13076](https://arxiv.org/abs/2404.13076)) | **The judge must be a different model family from the writer.** `readable.md` pins the model but doesn't require this |
| G-Eval (CoT plus form-filling, probability-weighted): Spearman 0.514 on summarisation, with a warning that LLM evaluators may favour LLM-written text | Liu et al. ([arXiv:2303.16634](https://arxiv.org/abs/2303.16634)); [nlpyang/geval](https://github.com/nlpyang/geval) MIT | The bias runs against the human anchors. A pairwise "beats the human page" can be bias, not quality |
| **Pairwise is the more hackable protocol under optimisation:** with distractor features embedded, pairwise preferences flip in **~35%** of cases vs **~9%** for absolute scores | Tripathi et al., COLM 2025 ([arXiv:2504.14716](https://arxiv.org/abs/2504.14716)) | **This weakens DOCS.md's "pairwise, never absolute" (from TTCW) when the judge sits in a writer's optimisation loop.** Pairwise is right for creative quality; for a gate against a writer that iterates, use rubric checks with binary leaves |
| Rubric judges: Prometheus (13B) Pearson **0.897** with humans on 45 custom rubrics, vs GPT-4 0.882. Prometheus 2 does both absolute and pairwise grading, with open weights | Kim et al. ([arXiv:2310.08491](https://arxiv.org/abs/2310.08491), [arXiv:2405.01535](https://arxiv.org/abs/2405.01535)); prometheus-eval Apache-2.0 (LICENSE file), 7B/8x7B v2.0 weights Apache-2.0 on HF | An open, local, different-family judge for rubric leaves. It also removes self-preference with a Claude writer |
| **LLMs are poor fact checkers:** most LLMs score near chance on SummEdits; GPT-4 is about 8 points below estimated human performance | Laban et al. ([arXiv:2305.14540](https://arxiv.org/abs/2305.14540)) | Never let the readability judge stand in for truth. The `sense` axis is informational; truth comes from §2 and execution |
| Judges are misled by outputs that are more "engaging" but don't follow the instruction | Zeng et al., LLMBar ([arXiv:2310.07641](https://arxiv.org/abs/2310.07641)) | Same lesson: polish beats correctness in an unanchored judge |

**Where it slots in.** All of this is hygiene for 2c.16 and for any LLM-judged rubric leaf.

**Redundancy.** DOCS.md already pins the judge model, blinds it to the arm, and decides before analysis. It lacks:

- evaluation in both orders;
- a different family from the writer;
- length control;
- any protocol choice that reflects optimisation pressure.

## 4. Code summarisation and docstring evaluation

- **BLEU is not usable for this.** Roy, Fakhoury and Arnaoudova (FSE 2021, 226 annotators) found that metric gains under 2 points don't guarantee better summaries, that METEOR and chrF are reliable only for gaps within 5 points, and that corpus BLEU stays unreliable ([FSE listing](https://2021.esec-fse.org/details/fse-2021-papers/57/Reassessing-Automatic-Evaluation-Metrics-for-Code-Summarization-Tasks)).
- **SIDE** (Mastropaolo et al., ICSE 2024) is a reference-free, contrastively trained MPNet score (−1 to 1) of how well a summary matches the *code*. It was trained on Java methods ([arXiv:2312.15475](https://arxiv.org/abs/2312.15475); [repo](https://github.com/antonio-mastropaolo/code-summarization-metric), MIT LICENSE file). It would need retraining for Rust, and it scores alignment, not truth.
- **LLM raters:**
  - Sun et al. (2024) found GPT-4-based evaluation correlates best with humans for LLM-written summaries ([arXiv:2407.07959](https://arxiv.org/abs/2407.07959)).
  - CODERPE (role-played reviewer, author and editor raters) reaches **81.59%** Spearman, 17.27% above BERTScore ([arXiv:2412.01333](https://arxiv.org/abs/2412.01333)).
  - Robinson and Kummerfeld (2025) find a plain "rate it" LLM prompt, reference-free variant included, is as good as or better than prior metrics. They recommend pairing it with embedding metrics to limit LLM bias ([arXiv:2505.19392](https://arxiv.org/abs/2505.19392)).
- **Accuracy of LLM comments is the real problem.** In a manual check of 540 comments, even GPT-4 wrote demonstrably inaccurate statements in **about a fifth** of comments. Nine existing consistency and similarity detectors had *no significant relationship* with accuracy. **"Document testing"** worked: an LLM generates tests from the comment, the tests run, and pass/fail predicts accuracy ([arXiv:2406.14836](https://arxiv.org/abs/2406.14836)).
- **Wrong docs are worse than no docs for LLM readers.** Incorrect documentation "greatly hinders" LLM code understanding; missing documentation has no significant effect ([arXiv:2404.03114](https://arxiv.org/abs/2404.03114)). This matters for the MCP/RPC surface, whose readers are agents. When unverified prose fails, the fallback is the generated reference *without* prose.

**Where it slots in.** DOCS.md generates reference pages, but agents still write source text: rustdoc for cadabra3, Copperfield and skelter, tooltips, and `codes.rs` fix-its.

- Doc-comment truth for cadabra3/Copperfield needs **document testing** (§7 item 4), not SIDE or BLEU.
- Completeness should use clippy's `missing_errors_doc` and `missing_panics_doc` (pedantic) and `missing_safety_doc` (style) ([lint list](https://clippy.rust-lang.org/master/index.html)). That is DocAgent's AST completeness check, already built for Rust.

## 5. Documentation-generation systems

| System | What it does | How it was evaluated | Relation to DOCS.md |
|---|---|---|---|
| **DocAgent** (Yang et al., ACL 2025) | Reader, Searcher, Writer, Verifier and Orchestrator agents, working in topological order of dependencies | Completeness from the AST (required sections from signature and visibility); helpfulness from GPT-4o Likert scoring per docstring part, with a rubric and examples; truthfulness as **Existence Ratio** (repo entities named in the docstring that exist in the dependency graph): 95.74% DocAgent/GPT-4o-mini vs 61.10% plain chat (full text, Table 3; the raw counts as I read them don't reconcile with the ratios, so treat the figures as approximate). [arXiv html](https://arxiv.org/html/2504.08725); [facebookresearch/DocAgent](https://github.com/facebookresearch/DocAgent) MIT | **Redundant:** Existence Ratio is a weaker 2b.10 (LLM-extracted mentions vs our registries); topological order is 2d.18. DocAgent's Verifier judges "information value, detail level, completeness", not truth |
| **RepoAgent** (Luo et al., 2024) | Repo-level docs from a global structure tree and Jedi caller/callee references; a **git pre-commit hook** regenerates only affected objects | Human preference over human-written docs: **0.70** (Transformers) and **0.91** (LlamaIndex) on 150 pairs; reference recall; parameter accuracy, chosen because models hallucinate parameters. [arXiv html](https://arxiv.org/html/2402.16667v1); [OpenBMB/RepoAgent](https://github.com/OpenBMB/RepoAgent) Apache-2.0 | **Redundant** with 2b.13 staleness. The win rates are a warning (§3): preference over human docs at 0.91 is what verbosity and LLM-text bias would produce too, so a pairwise anchor win is not evidence of correctness |
| **CodeWiki / CodeWikiBench** (2025) | Hierarchical decomposition, recursive multi-agent generation, diagrams; 7 languages | **Rubrics generated from each project's official docs** by three model families and merged; judges score **binary leaf requirements**; three judges from different families are averaged (mean std ≈3.7%). Cross-model rubric similarity is only ~74% semantic. Human check: 3 people, 7 of 9 preferences, called preliminary. CodeWiki 68.79% vs DeepWiki 64.06%. [arXiv:2510.24428](https://arxiv.org/abs/2510.24428), [v5 html](https://arxiv.org/html/2510.24428v5); repo `FSoft-AI4Code/CodeWiki`, MIT (LICENSE file; the paper doesn't link it) | **New:** the binary-leaf rubric and the multi-family judge ensemble are the right gate shape (§3). Our leaves can come from the registry (coverage matrix 2b.12), not from LLM rubric generation. That removes the 74% rubric-agreement problem |
| **SWD-Bench** (2026) | Docs judged by whether an LLM can use them: functionality *detection*, *localisation* (finding the files) and *completion* (implementing it); 4,170 entries mined from PRs. Docs from the best method raised SWE-Agent issue resolution by 20% ([arXiv:2604.06793](https://arxiv.org/abs/2604.06793)) | | **Mostly redundant** with 2c.14/2c.15. Adds one question type for developer docs (cadabra3, Copperfield): *localisation* ("which crate/module implements X?"), answerable from the registry |
| DOCER (Tan, Wagner, Treude) | Already cited in DOCS.md | | Redundant |

**A README generator.** LARCH (Koreeda et al., 2023, [arXiv:2308.03099](https://arxiv.org/abs/2308.03099)) [unverified, not opened]. I found no README-specific evaluation benchmark.

## 6. Goodhart and reward hacking against LLM judges

- **In-context reward hacking in self-refinement.** When a generator revises against an evaluator's feedback and ratings, the ratings rise while human judgement does not. The effect emerges spontaneously in context, gets worse when generator and evaluator **share a model**, and depends on whether they share context (Pan, He, Bowman, Feng, 2024; [arXiv:2407.04549](https://arxiv.org/abs/2407.04549)). This is #24's churn, with a judge in place of the profile.
- **Persuasion without correctness.** RLHF made models better at convincing time-limited humans without making them more correct. Evaluator false positives rose **24.1%** on QA and **18.3%** on programming (Wen et al., [arXiv:2409.12822](https://arxiv.org/abs/2409.12822)). The same pressure on a doc writer yields pages that *sound* authoritative, which supports gating truth only on execution and entailment.
- **Judges are cheap to fool.**
  - A constant "null model" reached an 86.5% LC win rate on AlpacaEval 2.0, 83.0 on Arena-Hard and 9.55 on MT-Bench ([arXiv:2410.07137](https://arxiv.org/abs/2410.07137)).
  - Single tokens and openers such as ":" or "Thought process:" produce false positive rewards across judges, GPT-o1 and Claude-4 among them. The fix is adversarial negatives built from truncated outputs ([arXiv:2507.08794](https://arxiv.org/abs/2507.08794)).
- **Over-optimisation is a scaling law, not a bug.** Optimising a proxy reward eventually lowers the gold reward, with different curves for best-of-n and RL (Gao, Schulman, Hilton, [arXiv:2210.10760](https://arxiv.org/abs/2210.10760)).

**What DOCS.md already does right:**

- the writer sees only pass/fail, never the banks;
- the register loop is capped at ~4 passes;
- the gate is an ensemble.

**What it should add:**

1. **The loop judge and the gate judge are different models**, and both differ from the writer's family.
2. **Hide judge rationales from the writer**, not just the bank.
3. **Rotate the held-out question subset** each pass, so it can't be overfitted through the pass/fail channel.
4. **Log loop-judge vs gate-judge divergence per page**, as the hacking signal (the canary analogue).
5. **Calibrate every LLM critic on adversarial null pages** before trusting it: template-only pages, truncated pages, an anchor with the facts swapped (§7).

## 7. Calibration data the design lacks: mutation-tested critics

SummEdits builds inconsistency benchmarks by applying **minimal edits** to verified seed texts. It is about 20× cheaper per sample than earlier benchmarks, with inter-annotator agreement around 0.9 ([arXiv:2305.14540](https://arxiv.org/abs/2305.14540)).

We can do better than human editing, because the registries give typed mutations:

- swap a command id for a sibling;
- change a default value, a key binding, or a diagnostic's fix;
- drop a prerequisite step;
- reorder two steps;
- rename a concept to a non-canonical synonym.

Apply these to pages that already passed the gate. Every critic (resolution, entailment, question bank, task agent, judge, contradiction) gets a **detection rate per mutation class**. The gate's sensitivity becomes a measured number, not an assumption.

This is the doc analogue of the canary family, and it is Layer 1 data. It also tells us which critic is redundant: if the entailment gate and the question bank catch the same mutants, keep the cheaper one.

## 8. Cross-page contradiction (2c.17), done mostly without an LLM

ContraDoc (Li et al., NAACL 2024) found GPT-4 best at self-contradiction detection in long documents, but still unreliable on contradictions that need nuance ([arXiv:2311.09182](https://arxiv.org/abs/2311.09182)).

Because the fact sheet binds claims to ids, store them as `(id, attribute, value)` triples, for example `(cmd.sketch.coincident, shortcut, "C")`. Most cross-page contradictions then become **equality checks** on the same `(id, attribute)`: deterministic and free.

- NLI (AlignScore or MiniCheck) handles free-text claim pairs that share an id.
- An LLM handles only the residue NLI flags.
- Contradictions against `DECISIONS.md` stay LLM-checked, since that file is prose.

## 9. Slot map and redundancy

| Item | Slot | Redundancy with DOCS.md |
|---|---|---|
| Page→fact-sheet entailment (atomic claims + MiniCheck/AlignScore) | Layer 3 gate | **New**: the precision direction |
| Sentence-level id anchors, ALCE citation recall/precision | Layer 3 step 3–6, 2b.13 | Extends the fact sheet to page sentences |
| QAGS/QuestEval | 2c.15 precision half | Recall half exists; superseded by the row above |
| SelfCheckGPT | Fact-sheet drafting for explanation pages | New, low priority |
| Judge hygiene (both orders, other family, length GLM, rubric leaves) | 2c.16 | Partly (pinning, blinding) |
| Prometheus 2 as an open judge | 2c.16, rubric leaves | New |
| Document testing for rustdoc / API prose | 2c, next to the task agent | **New**; 2b.11 only runs snippets the writer wrote |
| Clippy doc-section lints | 2b | New, free |
| CodeWikiBench binary-leaf rubric, multi-family judges | Layer 3 gate | New shape; leaves from the registry |
| SWD-Bench localisation questions | 2c.15 | Mostly redundant; one new question type |
| DocAgent Existence Ratio, topological order | 2b.10, 2d.18 | **Redundant**; DOCS.md is stronger |
| RepoAgent incremental git-hook update | 2b.13 | **Redundant** |
| SIDE, BLEU, METEOR, CODERPE | — | Don't adopt for the gate. At most, CODERPE-style LLM rating as an informational helpfulness signal |
| Registry-typed mutants | Layer 1 data | **New** |
| Structured-triple contradiction | 2c.17 | Upgrades an existing item |
| Loop/gate judge separation, divergence log, rotated banks | Layer 3 step 5–6 | Partly (held-out banks) |

## Top 5 to adopt, ranked by leverage per effort

1. **A page→source entailment gate.**
   - What: align every verifiable page sentence to fact-sheet ids, with sentence anchors and ALCE citation recall/precision. Verify each against its registry entry and source snippet with MiniCheck-Flan-T5-Large (MIT, 770M) or AlignScore (MIT, 355M), both local. A verifiable sentence that aligns to nothing fails.
   - Why: it closes the one hole the literature shows in DOCS.md, true identifiers carrying false statements, at near-zero marginal cost. Pin the decomposer and its version.
2. **Registry-typed mutation tests for every critic** (SummEdits, automated).
   - What: inject a wrong default, a swapped id, a dropped step or a non-canonical term into gated pages, then measure each critic's catch rate per class.
   - Why: cheap to generate. It turns "the gate works" into a number, finds redundant critics, and doubles as the adversarial calibration set for LLM judges.
3. **Judge hygiene and protocol choice for 2c.16 and the gate.**
   - What: a judge from a different family than the writer (Prometheus 2, Apache-2.0, is a local option); both orders, with a flip counted as a tie; the length ratio recorded and controlled; and, for the *gate*, binary rubric leaves drawn from the coverage matrix (CodeWikiBench shape), with pairwise anchors kept as informational.
   - Why: under optimisation pressure, pairwise flips at about 35% vs 9% for absolute scoring. A small prompt and config change with a large bias reduction.
4. **Document testing for API prose** (cadabra3, Copperfield, skelter rustdoc and the MCP tool descriptions).
   - What: a held-out agent writes tests *from the doc text alone*, and the tests run against the real function or tool. Add the clippy `missing_errors_doc`/`missing_panics_doc`/`missing_safety_doc` lints for completeness.
   - Why: about a fifth of GPT-4 comments are inaccurate, and nine existing detectors don't catch it. Wrong docs hurt agent readers more than missing ones.
5. **Anti-Goodhart separation, plus structured contradiction.**
   - What: the loop judge differs from the gate judge, rationales stay hidden, the held-out question subset rotates each pass, and loop/gate divergence is logged per page. Store fact-sheet claims as `(id, attribute, value)` so most of 2c.17 is equality checks, with NLI then an LLM for the rest.
   - Why: both are design rules, not new systems. They target the documented failures (in-context reward hacking with a shared evaluator; persuasion-without-correctness at +18–24% false positives).
