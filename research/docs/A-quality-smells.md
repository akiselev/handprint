# A — Documentation quality and documentation smells: what DOCS.md misses (2026-10-26)

Research pass A of six for `DOCS.md`. Scope: what software-engineering and
technical-communication research says makes documentation good or bad, and which
of those properties have published detectors. Everything below was checked
against `DOCS.md`, `WIT2.md`, `FINDINGS.md` #14 and #23–#31, and the as-built packs
(`doc_style.rs` holds Vale-derived weasel, wordy, cliché, opener and jargon rules;
`readability.rs` holds five readability formulas and a passive-voice proxy).
Items that are already covered are listed in §9 and are not proposed again. I opened
every source cited here. The two marked [unverified] I saw only as abstracts in
search results.

## Verdict

1. **The defects the literature ranks highest are content defects, and the style
   layer targets the category practitioners rank lowest.** Uddin & Robillard
   surveyed 323 professionals. The three most severe of the ten problems they rank
   are ambiguity, incompleteness and incorrectness, and six of the ten were
   "blockers" that made developers switch APIs
   ([IEEE Software 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)).
   Aghajani et al. mined 878 artifacts into a taxonomy of 162 issue types: 55% of
   the artifacts concern *what* is written, 29% *how* it is written, 15% tools and 9%
   process. In the follow-up survey of 146 practitioners, only 33% rated
   format/presentation issues important, while 88% said clarity was important
   (dissertation that holds both the ICSE 2019 and ICSE 2020 papers:
   [Aghajani 2020](https://folia.unifr.ch/documents/319169/files/2020INFO014.pdf),
   §5.3 and §6.3). DOCS.md's verifiers (2b) cover *incorrect* and *outdated*. Only the
   question bank (2c.15) covers *incomplete* and *ambiguous*, and nothing covers the
   two "how" issues practitioners rank highest after clarity: **superfluous content
   (55%) and duplicated content (46%)**. This supports the pilot's prediction that
   arm (b) carries most of the value.
2. **API-reference smells have cheap, validated detectors, and DOCS.md uses rustdoc
   only as corpus (1a) and as doctests (2b.11).** Rustdoc is the prose that agents
   will write in the largest volume: cadabra3's `missing_docs`, skelter's public
   crates, Copperfield's facade. Two rule-based detectors with a single threshold
   score F1 ≥ 0.90 on a benchmark of 1,000 units: the *lazy* smell by edit distance
   between doc and signature, and the *bloated* smell by length above the 90th
   percentile ([Khan et al., SANER 2021](https://arxiv.org/pdf/2102.08486), Table II).
   Clippy and rustdoc already ship lints for the Rust API Guidelines'
   documentation rules. Neither appears in DOCS.md.
3. **The structural evidence favours minimalism and task-shaped topics, not
   Diátaxis as navigation.** I found no empirical evaluation of Diátaxis, only
   adoption reports ([Diátaxis](https://diataxis.fr/)). Minimalism has controlled
   experiments that were replicated ([Carroll et al. 1987](https://pure.psu.edu/en/publications/the-minimal-manual/);
   [Lazonder & van der Meij 1993](https://research.utwente.nl/en/publications/f811580e-953a-4312-a1ee-9916b3f98e05)).
   The one observation study of developers using API docs recommends organising
   content *by functionality*, not by information type ("Samples", "Concepts", "API
   reference", "Recipes"), and folding concepts into the tasks that need them
   ([Meng, Steinhardt & Schubert 2019](https://mangold-international.com/resources/meng-et-al-how-developers-use-api-documentation-2019.pdf)).
   DOCS.md is right to use the Diátaxis types to fit *rates* (1a). It should not use
   them as the site's top-level split.

A fourth result shapes where detectors go. **LLMs are poor fine-grained smell
classifiers.** On inline comment smells, the best LLM reached median macro-F1 0.41
([Çağlar et al., ASE 2025](https://conf.researchr.org/details/ase-2025/ase-2025-papers/36/Automated-Inline-Comment-Smell-Detection-and-Repair-with-Large-Language-Models)),
and GPT-4 reached 34–55% accuracy against 69% for a random forest
([Oztas et al. 2025](https://arxiv.org/abs/2504.18956)). A fine-tuned BERT on API-doc
smells reaches F1 0.75–0.97 ([Khan et al.](https://arxiv.org/pdf/2102.08486)). So
smell detection belongs in 2a/2b as rules or trained classifiers, and the 2c reader
critics should keep the jobs that need a reader.

## 1. Taxonomies of documentation issues (planning labels, 2c failure vocabulary)

| Source | What it gives | Algorithmic? | Slot |
|---|---|---|---|
| Aghajani et al. ICSE 2019 / 2020 ([dissertation](https://folia.unifr.ch/documents/319169/files/2020INFO014.pdf)) | 162 issue types in 4 categories. Practitioner ranking: faulty tutorial and bad installation instructions (65% important), missing docs for a new feature (69%), code–doc inconsistency (59%), erroneous code examples (59%), superfluous content (55%), duplicates (46%), clarity (88%), findability (65%) | No; a label set | **2c**: the judge and the task agent report failures in these labels, so failures can be counted by type. **2d**: weight the coverage queue by them (install pages and new-feature pages first) |
| Uddin & Robillard 2015 ([PDF](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)) | 10 problems: content (incompleteness, ambiguity, unexplained examples, obsoleteness, inconsistency, incorrectness) and presentation (bloat, fragmentation, excess structural info, tangled) | No; Khan et al. made five of them detectable (§2) | 2a/2b via §2 |
| Treude, Middleton & Atapattu, FSE 2020 ([arXiv](https://arxiv.org/pdf/2007.10744)) | 10 quality dimensions, each phrased as one question (quality, appeal, readability, understandability, structure, cohesion, conciseness, effectiveness, consistency, clarity), applied by 4 technical editors to 41 documents | No; a rubric | **2c.16**: a cheap per-dimension rubric for the pairwise judge, so a verdict names a dimension. Small sample; use it as a rubric, not as norms |
| Robillard & DeLine, EMSE 2011 ([PDF](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)) | 440+ Microsoft developers. Five documentation factors: documentation of intent, code examples, matching APIs with scenarios, penetrability, format and presentation. Boilerplate that "merely rehashes the name of an API method" is named as a frustration | No | §2 (non-information), §4 (examples) |

## 2. API-reference smells and their detectors (rustdoc; 2a and 2b)

**Khan, Khondaker, Uddin & Iqbal, SANER 2021** ([arXiv 2102.08486](https://arxiv.org/pdf/2102.08486);
data and code at [disa-lab/SANER2021-DocSmell](https://github.com/disa-lab/SANER2021-DocSmell),
no licence shown). 1,000 Java SE units, Cohen's κ 0.83, and a survey of 21
developers in which 95.2% said the smells hurt productivity.

| Smell | Rule (from the paper) | Rule F1 | Best model F1 |
|---|---|---|---|
| Lazy (says almost nothing) | Levenshtein distance between description and method prototype below a percentile | **0.95** (25th pct) | BERT ~0.97 |
| Bloated (too long) | Length above the 90th percentile | **0.90** | OVR-SVM 0.88 |
| Excess structural info | Count of class, method and package names | 0.42 | BERT, lower |
| Tangled | Flesch score plus acronym count | 0.41 | BERT, lower |
| Fragmented | URL count | 0.47 | BERT, lower |

Shallow and BERT figures are from Tables III and V of the paper. Khan's later
thesis reports that the models degraded on wider data and needed retraining
(RoBERTa F1 0.71–0.93), and that practitioners judged *lazy* the most impactful
smell to fix ([Khan, Calgary thesis](https://prism.ucalgary.ca/items/0aa2081a-75d2-4590-a92a-29ec484e61ac)).

**Slot:** two of these are deterministic and belong in **2a as a Rustdoc subfamily**.
*Lazy* is the normalised edit distance or token overlap between a `///` summary and
the item's split identifier plus signature ("Returns the foo" on `fn foo`). *Bloated*
is length against a band fitted per item kind (fn, struct, trait, module) on the
`api-reference` sub-register DOCS.md already proposes, which makes the band two-sided.
The weak three are not worth building as rules.

**Non-information** (Maalej & Robillard, TSE 2013,
[PDF](https://www.cs.mcgill.ca/~martin/papers/tse2013a.pdf)). Of 12 knowledge types,
non-information is "common", and it is "overwhelmingly present in members", where
the method's name is repeated. A classifier exists: SVM and deep models on the CADo
dataset of 5,574 labelled Java and .NET units reach AUPRC up to 87%, and the
non-information type is one where SVM beats the deep models
([Fucci et al. 2019](https://arxiv.org/pdf/1907.09807); data on
[Zenodo](https://zenodo.org/badge/latestdoi/194706952), licence unverified). The
lazy rule above is the cheap version. A trained classifier is a later option.

**Buried directives** (Monperrus et al., EMSE 2012, [arXiv 1205.6363](https://arxiv.org/pdf/1205.6363)).
This paper gives a taxonomy of 23 directive kinds and the lexical patterns that find
them (`must`, `require*`, `never`, `only`, `before`, `after`, `thread*`, `lock*`,
`overrid*`, `warn*`, `note*`, Table 2). Robillard & Chhetri's Krec used such word
patterns to recover "indispensable" fragments at about 90% precision and 69% recall
([EMSE 2015](https://www.cs.mcgill.ca/~martin/papers/cr2014a.pdf)). In Rust these
sentences belong under `# Panics`, `# Errors` and `# Safety`
([Rust API Guidelines C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html)).
**Detector (2a):** a directive-pattern sentence outside its section is a *buried
directive* finding, with a span and a fix of the form "move to `# Panics`". The same
rule works on prose pages, where the destination is an admonition. The pattern list
is a closed list, so it ships as a pack.

**Doc–code inconsistency.** Panthaplackel et al. detect `@return`/`@param`/summary
comments made stale by a code change, on 40,688 Java examples. The just-in-time
models score F1 around 0.80 on the full test set
([AAAI 2021](https://arxiv.org/pdf/2010.01625), Table 2). DRONE detects defective
parameter-constraint directives at 81.6% precision and 82.0% recall on JDK 1.8
([Zhou et al., ICSE 2017](https://repository.mdx.ac.uk/item/87050)). DOCS.md's
registry staleness (2b.13) covers pages that depend on ids. These cover *doc
comments on changed code*, which have no registry entry. Slot **2b.13**, as a later
extension: on a consolidation, flag `///` blocks whose item body changed while the
comment did not. A diff check is the cheap version, and these models are the
precise one.

**Already-built Rust tooling, not mentioned in DOCS.md (2b, almost free).** These are
in Clippy's current lint list ([clippy](https://clippy.rust-lang.org/master/index.html)):
`missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`,
`unnecessary_safety_doc`, `too_long_first_doc_paragraph`, `doc_markdown` (unticked
identifiers), `empty_docs`, `doc_broken_link`, `doc_lazy_continuation` and
`needless_doctest_main`. Rustdoc adds `broken_intra_doc_links`,
`missing_crate_level_docs` and the nightly-only `missing_doc_code_examples`
([rustdoc lints](https://doc.rust-lang.org/rustdoc/lints.html)). Together they
mechanise C-EXAMPLE, C-FAILURE and C-LINK. Turn them on as **deny in the docs
lane's gate** for the public crates.

## 3. Duplicate and superfluous content (2b; the largest gap)

Practitioners rank superfluous content (55%) and duplicate content (46%) as the top
maintainability issues, and the Aghajani survey explicitly calls for clone detection
on documentation ([dissertation §6.3.2](https://folia.unifr.ch/documents/319169/files/2020INFO014.pdf)).
Luciv et al. adapted the Clone Miner code-clone detector to text and found exact and
near duplicates in all 19 projects they tested, which covered specifications,
guides, API docs and manuals ([arXiv 1711.04705](https://arxiv.org/abs/1711.04705)).
An earlier n-gram version had 39% false positives at sentence level
([Kanteev et al. 2017](https://m.mathnet.ru/eng/tisp252)) [unverified: abstract only].

A many-agent doc set will produce this defect first, because every agent
re-explains the same prerequisite. **Detector (2b):** shingled MinHash or
suffix-array near-duplicate detection, run across all prose pages *and* against the
generated reference pages. A duplicate of a generated fact is a page restating the
registry instead of linking to it, which DOCS.md's own rule ("facts are generated")
forbids. Report pairs over a Jaccard threshold, with spans. This is the cross-page
counterpart of 2a.2 restatement, which only looks inside a page. It also feeds
2c.17: two near-duplicates that disagree are the likeliest contradictions.

## 4. Code examples (2a rates, 2b checks)

What the evidence says:

- Code examples are the most sought resource. Developers "eagerly seek examples in
  documentation pages, skipping over explanatory text". Snippets that exercise only
  one method "were of little value", and the ideal example shows one pattern, "a
  number of related API functions together"
  ([Robillard & DeLine 2011 §5.2](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)).
- The explanation around an example is as important as the example itself
  ([Nasehi et al., ICSM 2012](https://www.ucalgary.ca/news/most-influential-paper-be-awarded-icsme-2022)).
  "Unexplained examples" is one of Uddin & Robillard's six content problems.
- In a controlled study with 26 engineers, documentation without usage examples
  caused errors in data types, formats, headers and request bodies
  ([Sohan et al., VL/HCC 2017](https://www.cs.mcgill.ca/~martin/papers/vlhcc2017.pdf)).
- Of 14,876 code blocks in 600+ tutorials, 26% executed. Even with human annotation,
  no tutorial ran end to end. The causes were environment assumptions, missing
  steps, file-content blocks with no stated action (replace, append or create),
  `...` elisions, and unflagged placeholders such as `your-server-ip`
  ([Docable, FSE 2020](https://par.nsf.gov/servlets/purl/10196169)).
- In Meng et al.'s observation study, participants complained about code examples
  that could not be copied and reused
  ([Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)).

DOCS.md 2b.11 ("every snippet runs") and 2a.4 (colon→code adjacency) cover
*executability* and one direction of adjacency. These are missing:

| Detector | Measures | Kind | Slot |
|---|---|---|---|
| **Explained-example ratio** | Prose tokens in the paragraph before and after each code block. Flags a block with no explanation (Nasehi; Uddin & Robillard), and also a block buried in a paragraph that is too long | Count, two-sided band per page type | 2a |
| **Example scope** | API ids called per example, resolved through the Layer 0 tables. 1 is "too simple", and a high count is tangled | Count after resolution, band | 2a (needs 2b's resolver) |
| **Block-action marker** | Every file-content block names its path and its action (create, replace, append). No `...` or `[...]` elision without a marker, and every placeholder is declared | Regex over code fences | 2b |
| **Copyability** | Shell blocks mix `$` prompts with output, or mix commands for several platforms in one block | Regex | 2b |
| **Clean-environment replay** | Tutorials and install pages replayed in a fresh container, not the dev one. Installation instructions are practitioners' most important correctness issue (Aghajani), and environment assumptions are Docable's first failure theme | Execution | 2b.11, made explicit |

## 5. Minimalism, procedures, DITA: testable structure (2a and 2b)

**Minimalism.** The Minimal Manual is briefer, supports error recognition and
recovery, and helps the reader split attention between system and manual. In two
experiments it gave more efficient learning than a commercial manual
([Carroll et al. 1987](https://pure.psu.edu/en/publications/the-minimal-manual/)).
A replication with 64 students found minimal-manual users "learned faster and
better" ([Lazonder & van der Meij 1993](https://research.utwente.nl/en/publications/f811580e-953a-4312-a1ee-9916b3f98e05)).
A manual rich in error information made learners finish faster and with better
corrective skill, at the same constructive skill
([Lazonder & van der Meij 1995](https://research.utwente.nl/en/publications/7e445104-88ad-481f-87cf-0539f55ae327)).
The commonly cited heuristics are to start on real tasks immediately, minimise
passive reading, include error recognition and recovery, and make sections
self-contained ([EduTechWiki summary](https://EduTechWiki.unige.ch/en/Minimalist_instruction)).
The 1995 van der Meij & Carroll paper itself is [unverified]: I could not open it.
Testable forms:

- **Front matter before the first action** (how-to and tutorial): tokens before the
  first imperative step, as a band. Farkas's streamlined-step model also puts
  "relatively little" information before the steps and the notes after them
  ([Farkas 1999](https://faculty.washington.edu/farkas/dfpubs/Farkas-ConstructionOfProceduralDiscourse.pdf)). → 2a.
- **Error-recovery rate**: share of procedures that contain a recognise-and-recover
  move ("If the panel shows X, …"). This is a move in DOCS.md's 2d.19 index, but as a
  required *rate* per procedure it is new, and it has the strongest experimental
  backing of any item here. → 2a, and the 2d page plan.
- **Self-containment**: backward-reference phrases ("as described above", "see the
  previous section") and the count of prerequisite links. Docable's informants
  blamed tutorials that chain "multiple levels deep". → 2a (phrase pack plus a link
  count).

**Procedures.** Farkas's streamlined step: each step is one brief action statement
"built around an imperative verb" that "begins the main clause", optionally followed
by a feedback (result) statement. DOCS.md 2a.4 covers actions per step and the
result rate. The **imperative-first position** is not covered, and it is cheap: a
step that opens with a descriptive clause breaks the model. Goal-first
subordinates ("To open X, click Y") are fine.

**DITA task schema** ([OASIS DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part2-tech-content/archSpec/technicalContent/dita-task-topic.html)).
The strict taskbody orders its parts prereq → context → steps → result → example →
postreq, and every `<step>` needs exactly one `<cmd>`, with optional `<stepresult>`
and `<steptroubleshooting>`. This is a free, open, deterministic **page schema for
how-tos (2b)**. Map the headings and list items of a Markdown how-to onto it, and
fail a page that puts prerequisites after the steps or a step with no action. The
schema matches a task bank entry's shape (prereq = initial state, result = expected
end state), so it also links the page to its task (2c.14).

**Information Mapping** has thin peer-reviewed evidence: equal learning, faster
retrieval ([Jonassen 1981](https://journals.uc.edu/index.php/vl/article/view/5314)).
Headings speed search, but adding more headings did not
([Kools et al. 2008](https://cris.maastrichtuniversity.nl/en/publications/the-effects-of-headings-in-information-mapping-on-search-speed-an)).
That supports 2a.1's two-sided heading band. Nothing new to build.

**Simplified Technical English** (ASD-STE100). The specification is free on request
but "all rights reserved" ([asd-ste100.org](https://www.asd-ste100.org/)), so do not
ship its dictionary. Its sentence-length split (20 words for procedural sentences,
25 for descriptive) and its "one instruction per sentence" rule can be restated as
two *separate* sentence-length bands, one for procedural and one for descriptive
text. Evidence: STE improved comprehension of complex procedures in Shubert et al.
1995 [unverified: abstract only]. Low priority.

## 6. What readers actually read (2a front-loading, 2d information architecture)

- Engineers *said* they spent about 40% of their time reading docs; observation
  logged 3% of events (Lethbridge et al. 2003, quoted in
  [Meng et al. 2019](https://mangold-international.com/resources/meng-et-al-how-developers-use-api-documentation-2019.pdf)).
- On the web, users read about 20% of a page's words (at most 28%), and they read
  half the text only on pages of 111 words or fewer
  ([Nielsen 2008](https://www.nngroup.com/articles/how-little-do-users-read/)).
- People rarely read manuals and struggle to find search terms in help
  ([Novick & Ward, SIGDOC 2006](https://scholarworks.utep.edu/cs_papers/15)).
- Meng et al. (n=11, with eye tracking) found an opportunistic group and a
  systematic group of readers. Both used the reference and the examples most and the
  conceptual overview least consistently. The authors recommend organising content
  by API functionality and integrating concepts into tasks.

**Proposals.**

- **2d, information architecture:** the site's navigation is by domain (sketches,
  constraints, nets, routing), with Diátaxis types *inside* each domain. Concepts
  are linked from the task pages that need them, not only collected under
  `concepts/`. This changes DOCS.md Layer 3 step 2 (plan), not the corpus split.
- **2a, front-loading:** the first sentence of a page or section contains a fact-sheet
  id or a glossary term. The summary line has a length band (clippy's
  `too_long_first_doc_paragraph` is the rustdoc form). Headings carry a domain term
  in their first two words. 2a.1 covers template words in headings, but not their
  position or the first sentence.

## 7. Diagnostics text, critiqued at the source (2a and 2b on `codes.rs` and the `ecad.*` catalog)

DOCS.md generates diagnostic pages and says source text is critiqued "at the
source", but it gives no criteria for that. Becker et al.'s survey of research on
programming error messages lists ten guidelines: readability, low cognitive load,
context, positive tone, examples, solutions or hints, interaction, scaffolding,
logical argument, timing ([ITiCSE-WGR 2019](https://neverworkintheory.org/2021/09/02/compiler-error-messages-considered-unhelpful.html)).
A later test of these guidelines on SQL messages found mixed effects, so treat them as
hypotheses ([Taipalus et al. 2024](https://jyx.jyu.fi/bitstreams/713f45b9-b3ca-4c86-ad8f-46064a12ff8e/download)).
Deterministic checks (2b): every code has a fix or hint field (svelters already has
fix-its; Copperfield's string-literal codes do not, which is one more reason for
DOCS.md's registry request), plus a message-length band, and a blame or harsh-word
pack ("illegal", "invalid", "fatal", "you must") as a 2a lexicon.

## 8. LLM-written documentation, 2023–2026

- In checklist-scored comparisons, GPT-3.5, GPT-4 and Bard beat the original
  documentation, and file-level docs were the weakest level
  ([Dvivedi et al., AIware 2024](https://arxiv.org/abs/2312.10349)). For DOCS.md this
  means a checklist favours the machine, the same inversion as `p_same_author` in
  #23. It strengthens the case for the executed reader critic as the gate.
- Ad-hoc prompts gave documentation that was less readable, less concise and less
  helpful than a prepared few-shot prompt
  ([Kruse, Puhlfürß & Maalej, ICSME 2024](https://arxiv.org/abs/2408.00686)). The
  `doc-writer` skill's prompts are a controlled variable and should be versioned.
- For README generation, developer-supplied plans gave the best quality, and
  "autonomous planning is the main bottleneck"
  ([Saleh et al., ICSME 2026](https://arxiv.org/abs/2606.30524)). This supports
  DOCS.md's fact sheet and page plan (Layer 3 steps 2–3). It argues for a human or
  coordinator sign-off on page plans, not on finished pages.
- LLM smell detection is weak (macro-F1 0.41; GPT-4 accuracy 34–55%) and repair can
  introduce new smells ([Çağlar et al. 2025](https://conf.researchr.org/details/ase-2025/ase-2025-papers/36/Automated-Inline-Comment-Smell-Detection-and-Repair-with-Large-Language-Models);
  [Oztas et al. 2025](https://arxiv.org/abs/2504.18956)). This is the published form
  of FINDINGS #24 (each pass introduces new findings), and it supports the churn
  guard (2a.8).
- Code2Doc's curation pipeline keeps 25.6% of 52,069 candidate docstrings and flags
  2.9% as likely AI-generated with rule-based signals
  ([Karaman & Akarsu 2025](https://arxiv.org/abs/2512.18748)). The pipeline is
  released and could filter any post-2022 rustdoc that enters Layer 1, but DOCS.md's
  pre-2022 cutoff makes this a fallback only.
- A rule taxonomy for code comments also exists: 11 inline-comment smell types,
  from 2,447 labelled comments, with the dataset shared
  ([Jabrayilzade et al., EMSE 2024](https://repository.bilkent.edu.tr/items/30c855e3-0307-4d9a-a16b-3ed7529b349e)).
  It concerns `//` comments, not public docs, so it is out of scope unless the
  doc lane takes on code comments.

## Datasets

| Dataset | Size | Labels | Licence | Use |
|---|---|---|---|---|
| SANER2021-DocSmell ([GitHub](https://github.com/disa-lab/SANER2021-DocSmell)) | 1,000 Javadoc units | 5 smells | none shown | Check that the method transfers; fit our own percentiles on our rustdoc corpus |
| CADo ([Fucci et al.](https://arxiv.org/pdf/1907.09807)) | 5,574 Java/.NET units, plus a Python set | 12 knowledge types | unverified (Zenodo) | Optional non-information classifier |
| README sections ([Prana et al., EMSE 2019](https://arxiv.org/abs/1802.06997)) | 4,226 sections, 393 repositories | 8 categories (what, why, how, …); classifier F1 0.746 | code licence unverified | Seed labels for the 2d.19 move index on README-like pages |
| Deep JIT inconsistency ([Panthaplackel et al.](https://arxiv.org/pdf/2010.01625)) | 40,688 comment/code pairs (Java) | stale or not | repo licence unverified | Later 2b.13 extension |
| Docable ([FSE 2020](https://par.nsf.gov/servlets/purl/10196169)) | 600+ tutorials, 14,876 blocks | executability, failure themes | — | Failure taxonomy for 2b.11 reports |

## 9. Already covered: redundant, do not re-propose

- Readability formulas. They are already in the crate as shape statistics, and the
  code says they are not readability measures. The literature agrees: Redish
  argues that they ignore readers, layout and retrieval aids
  ([JCD 2000](https://ftp.math.utah.edu/pub/tex/bib/idx/jcd/24/3/132_137.html)), and
  Flesch was a weak feature for *tangled* in Khan et al. (F1 0.41). Add nothing.
- Weasel, wordy and cliché rules (Vale packs in `doc_style.rs`); passive voice
  (`readability.rs`); opener diversity (WIT2 §1).
- Heading specificity (2a.1); restatement and empty closers (2a.2); actions per
  step and the result rate (2a.4); terminology (2a.5); definition before use
  (2a.6); apparatus rates (2a.7); the churn guard (2a.8).
- Reference resolution (2b.10; DOCER); snippet execution (2b.11); coverage and
  staleness (2b.12–13); the task agent, the question bank, the pairwise judge and
  contradiction checks (2c); DocAgent's ordering (2d.18; its truthfulness check is
  2b.10's resolver).

## Top 5 to adopt (leverage per effort)

1. **Rust doc lints and the two Khan rules for rustdoc (2b and 2a).** Turn on
   `missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`,
   `too_long_first_doc_paragraph`, `doc_markdown`, `empty_docs`,
   `broken_intra_doc_links` and `missing_crate_level_docs` as deny in the docs gate.
   Add *lazy* (doc ≈ name plus signature) and *bloated* (length band per item kind)
   as a Rustdoc subfamily. The lints are configuration only, and the two rules are
   under a day of work with F1 ≥ 0.90 as published. They cover the largest volume
   of agent-written prose.
2. **Near-duplicate detection across pages and against generated reference pages
   (2b).** Shingling or MinHash with spans. It targets the second-ranked "how" issue
   (46%), the defect a many-agent doc set produces first, and restated registry facts.
   It also feeds 2c.17.
3. **Code-example checks (2a and 2b):** the explained-example ratio, API calls per
   example, block-action and elision markers, copyability, and clean-container
   replay for install and tutorial pages. These are regex and count work on top of
   the 2b.10 resolver, and the evidence behind them is the strongest in this report
   (Robillard, Nasehi, Sohan, Docable, Aghajani).
4. **Buried-directive pack and DITA task schema (2a and 2b).** Directive-pattern
   sentences outside `# Panics`, `# Errors`, `# Safety` or admonitions; a how-to
   mapped onto prereq → steps (one action, imperative-first) → result, with an
   error-recovery move required per procedure (minimalism's best-supported
   principle). A closed pattern list and a heading parser.
5. **Information architecture by domain plus front-loading (2d and 2a).** Navigate
   by domain, with Diátaxis types inside each domain and concepts linked from tasks
   (Meng et al.). The first sentence carries an id or glossary term, and headings
   put a domain term first. The IA part is a plan decision that costs nothing now
   and a migration later. The front-loading part is two small 2a dimensions.

Also adopt one design rule: **route fine-grained smell detection to rules or trained
classifiers, never to the LLM judge** (macro-F1 0.41 against BERT's 0.75–0.97).
