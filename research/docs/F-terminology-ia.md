# F — Controlled language, terminology management and information architecture

Research lane F for `DOCS.md` (2026-10-26). Scope: controlled natural languages, terminology
management and term extraction, prerequisite learning, information-architecture (IA) evaluation,
page-type classification, link-structure metrics and localisation readiness. Every claim has a URL.
I opened each source unless it is marked [unverified]. Repository facts come from the checked-out
trees and are labelled "(tree)".

## Verdict

1. **DOCS.md's terminology item (2a.5) has the right goal but the wrong data model.** A flat synonym
   table cannot handle the risk this project family actually has, which is **polysemy across
   repositories**, not synonymy. In cadatomic's own catalog, "component" already means a Svelte
   component (`cmd-designer-insert-tooltip`, `gallery-title = Component gallery`) and an ECAD part
   (`ecad-kind-ecad-component = Component`), and "part" means both an MCAD document (`cmd-file-new =
   New part`) and "a piece of" (tree: `crates/cadatomic-ui/i18n/en.ftl`). Terminology standards solve
   this with *concept-oriented* entries scoped to a subject field, each with designations that carry
   a status (preferred / admitted / deprecated / obsolete / superseded, the ISO 10241 scale quoted
   [here](https://lists.w3.org/Archives/Public/public-esw-thes/2010Jun/0017.html)). The machine
   formats already exist: SKOS (one `prefLabel` per language, plus `altLabel` and `hiddenLabel`,
   [W3C Rec 2009](https://www.w3.org/TR/skos-reference/)) and TBX
   ([ISO 30042:2019](https://en.wikipedia.org/wiki/TermBase_eXchange)).
2. **Two inputs DOCS.md treats as future work already exist in the tree.** The catalog has **56
   `*-synonyms` entries**, which are palette search aliases such as `cmd-edit-undo-synonyms = revert,
   take back, go back…` (tree; read by `cadatomic-core/src/palette.rs`). These are SKOS
   `hiddenLabel`s: words a search should accept and a page should not use. And
   `ui/tours/concepts.ts` is a hand-made prerequisite graph: **101 concepts with `prerequisites:`
   lists and 11 roots**, across shell, project, schematic, board and library (tree). The graph has
   **no MCAD concepts**, and the catalog contains "sketch" zero times, so DOCS.md's example
   synonym set "sketch/drawing/profile" has nothing to anchor to yet. DOCS.md 2d.18 should order
   pages with this graph and validate the graph, not infer a new one.
3. **Controlled-language evidence supports a small set of rules, not a whole standard.** Simplified
   English improved comprehension most for difficult documents and non-native readers
   ([Chervak, Drury, Ouellette 1996](https://researchconnect.buffalo.edu/en/publications/simplified-english-for-aircraft-workcards/)).
   But eight corporate rule sets shared only **one** rule: keep sentences short
   ([O'Brien 2003](https://aclanthology.org/2003.eamt-1.12.pdf)). With neural MT, making a sentence
   explicit mattered more than making it short
   ([Miyata & Fujita, EACL 2021](https://arxiv.org/abs/2102.02955)). Adopt the rules that are
   checkable and that also catch LLM drift. Do not adopt STE's dictionary: ASD holds its copyright.
4. **An LLM reader is a poor proxy for human findability.** In a tree test, ChatGPT-4 found the
   correct path in all ten tasks, while humans averaged about 51%
   ([MeasuringU](https://measuringu.com/chatgpt4-tree-test/)). DOCS.md's task agent (2c.14) proves
   a page *works*. It cannot prove that a human will *find* the page. For findability, use a
   deterministic scent model (CWW-style competing headings, below).

## 1. Controlled natural languages

**What exists.** ASD-STE100 Issue 9 (January 2025) has 53 writing rules and a dictionary of about
900 approved words. Its main rules: procedural sentences of at most 20 words, descriptive sentences
of at most 25, paragraphs of at most 6 sentences, noun clusters of at most 3 words, and one meaning
and one part of speech per word. The standard is free to download, but ASD keeps the copyright
([Wikipedia](https://en.wikipedia.org/wiki/Simplified_Technical_English); asd-ste100.org returned 403).
Third-party projects state that the rules and dictionary may not be reproduced and paraphrase them
instead ([amaruki/ste100](https://github.com/amaruki/ste100)). Caterpillar Technical English used a
lexicon of about 70,000 terms with a "narrow semantic scope", plus an authoring tool that enforced
compliance and disambiguated terms interactively. IBM's EasyEnglish scored each document with a
"clarity index" that had to pass a threshold
([Kuhn 2014, *Computational Linguistics*](https://arxiv.org/abs/1507.01701), appendix). Kuhn also
gives the classification frame, PENS (Precision, Expressiveness, Naturalness, Simplicity).
Documentation English belongs at P2 E5 N5 S1: only slightly more precise than full English, with no
loss of expressiveness.

**Evidence.**

- Comprehension, verified: 175 aircraft technicians, 16 workcards. Simplified English (SE)
  significantly improved comprehension, most for difficult cards and non-native speakers. Layout had
  no effect ([Chervak et al. 1996](https://researchconnect.buffalo.edu/en/publications/simplified-english-for-aircraft-workcards/)).
- Comprehension, abstract only: Shubert et al. (JTWC 25(4), 1995) report the same pattern for
  complex procedures, with equal completion time
  ([catalog record](https://search.fid-media.de/Record/ai-49-aHR0cDovL2R4LmRvaS5vcmcvMTAuMjE5MC93ZzY5LWQ3NGItNGRsbC0yd2Jr?lng=en)
  [unverified]).
- Machine translation: text that followed CL rules got significantly higher human scores for
  Chinese, French and Dutch with Microsoft's statistical MT system. "Formal style" and short or
  ambiguous sentences were the strongest rule categories. For Arabic, the BLEU difference was not
  significant ([Aikawa et al., MT Summit 2007](https://aclanthology.org/2007.mtsummit-papers.1.pdf)).
  With neural MT, explicitness beats brevity
  ([Miyata & Fujita 2021](https://arxiv.org/abs/2102.02955)).
- Rule sets: only one rule common to eight CLs, and seven rules shared by half of them
  ([O'Brien 2003](https://aclanthology.org/2003.eamt-1.12.pdf)).

**Which rules to adopt.** For each rule below: what it catches, whether it is new or overlaps
DOCS.md, and where it goes. All of them are parser-free or need only the heuristics already in
`readability.rs` and `biber.rs`.

| Rule (source) | Catches | Redundancy | Slot |
|---|---|---|---|
| ≤20 words per procedural step, ≤25 descriptive (STE) | Run-on steps | Partly redundant: sentence length exists; per-*step* length is new | 2a.4 instruction shape (add a dimension) |
| One instruction per step (STE; DITA `<cmd>`, below) | Two actions in one step | **Redundant** with 2a.4 "actions per numbered step" | — |
| Noun cluster ≤3 (STE); "avoid modifier stacks" ([Microsoft](https://learn.microsoft.com/en-us/style-guide/global-communications/writing-tips)) | "sketch constraint solver failure diagnostic panel" | New | 2a, new `cl.noun_stack` dimension with spans |
| Keep optional *that*/*who* and articles; watch ambiguous *-ing*/*-ed* (Microsoft) | Telegraphic, MT-hostile prose; LLM headline-ese | New | 2a pack (`cl.omitted_that`, `cl.ing_ambiguity`) |
| No phrasal verbs where one verb exists; avoid idioms and humour ([Google](https://developers.google.com/style/translation)) | "set up", "carry out", "kick off" | New. `doc_style.rs` has weasel, wordy, cliche, opener and jargon, but nothing for global English (tree) | Lexicon pack `global-english`, `consider_replace` fixes |
| ≤3 clauses joined by *and*/*or*/*but* (Microsoft) | Coordination chains | New | 2a dimension |
| "One word for a concept… don't use the same word for multiple concepts" (Microsoft) | Synonymy **and** polysemy | Extends 2a.5 | §2 below |
| Clarity index above a threshold (IBM EasyEnglish) | — | **Redundant** with 2a.9 doc gate mode | — |

Licences: Vale's Microsoft and Google packs are MIT, so `doc_style.rs`'s existing mining route
covers the new rows (the Google style guide text itself is CC-BY 4.0). The STE rules are
re-described in our own words, never quoted, and none of the dictionary is shipped. Cost: one pack
file plus about three small dimensions.

## 2. Terminology management

**Principles.** ISO 704:2022 sets out the relations between objects, concepts, definitions and
designations, and the rules for forming terms and writing definitions
([SCC listing](https://scc-ccn.ca/standardsdb/standards/8178944) [unverified, search snippet]). The standard text is sold, so this
lane saw only its scope. The practical content is a *concept-oriented* entry with acceptability
ratings on its designations (ISO 10241 scale, above). Furnas et al. found that two people chose the
same term with probability below 0.20, and that the remedy, "unlimited aliasing", improves access
several-fold ([CACM 1987](https://honnef.co/notes/references/furnasvocabularyproblemhumansystem1987)).
Taken together: **writing needs one preferred label per concept; finding needs many aliases.** A doc
set must keep the two lists apart.

**Proposed data model (replaces the synonym table in 2a.5).** One TOML or SKOS record per concept:

- `id`: the concept id, e.g. `ecad.component` or `ui.svelte-component`.
- `subject_field`: one of `mcad`, `ecad`, `ui`, `kernel-api`, `cli`, `tracker`.
- `pref`: the label, taken from the catalog message id.
- `admitted[]`, `deprecated[]` (with a `consider_replace` target), `hidden[]`: the existing
  `*-synonyms` entries.
- `definition`, `broader`, `prerequisites`: the last from `concepts.ts`.

The checker resolves each match by the page's subject field (from its front matter or its cited
ids), so it can tell **wrong-domain use** ("component" on an ECAD page meaning a Svelte component)
apart from **variant use** ("footprint pad" vs "land"). A plain lexicon pack cannot tell them
apart; this is the one place the 2a.5 design needs code, not data. Slot: 2a family plus Layer 0
(the termbase is a generated table that the 2b.10 resolver also reads).

**Automatic term extraction (ATE), for the terms no registry holds.** Layer 0 covers ids. ATE
covers domain words that appear in prose but have no catalog entry yet (fillet, courtyard, pour,
annular ring). Methods:

- **C-value / NC-value.** Termhood is log₂|a|·f(a), with a penalty for candidates nested inside
  longer terms. NC-value adds weights from context words
  ([Frantzi, Ananiadou & Mima, IJDL 2000](https://link.springer.com/article/10.1007/s007999900023)
  [unverified, paywalled]; formula from [Fahmi's seminar slides](https://www.let.rug.nl/nerbonne/teach/rema-stats-meth-seminar/presentations/fahmi-statistics-c-value.pdf)).
  Parser-free with a POS-pattern filter, cheap, implementable in Rust.
- **TermSuite.** Apache-2.0. Ranks candidates by weirdness ratio against a general corpus and groups
  syntactic and morphological variants with YAML rules ("turbine structure" ~ "turbine base
  structure"; coordination "onshore and offshore locations"). It relies on TreeTagger, which has its
  own non-free licence ([Cram & Daille, ACL 2016](https://aclanthology.org/P16-4003.pdf)). Port the
  *variant rule shapes*, not the Java.
- **Clustering glossary candidates.** Gains of 20% or more in recall over generic ATE tools on
  industrial requirements documents
  ([Arora et al., TSE 2017](https://research.monash.edu/en/publications/automated-extraction-and-clustering-of-requirements-glossary-term/)).
  These are the closest documents to ours.
- **Acronym definitions (Schwartz & Hearst).** 96% precision and 82% recall with no training data
  ([PSB 2003](https://biotext.ischool.berkeley.edu/papers/psb03.pdf)). This makes "define before
  use" (2a.6) exact for abbreviations (DRC, ERC, SSI, MCP) at almost no cost.
- **Term variation.** Variants are frequent in specialised corpora, and Daille's inventory covers
  five languages ([Daille 2017, Benjamins](https://lwc1.benjamins.com/catalog/tlrp.19.c1) [unverified]).

Redundancy: handprint's Fightin' Words contrast (DOCS.md 1b) already does the weirdness-ratio part
when run as our docs against `software-docs`. C-value adds the nesting correction for multi-word
terms, and the variant rules add grouping. Both are new.

**A drift metric DOCS.md lacks: concept split.** When many writer agents work in parallel, the
typical failure is that two agents each choose a different name for the same concept. Signature:
two candidate terms that are distributionally similar (nearby in context vectors and in the same
variant group), each used almost only on a disjoint set of pages or writer runs. Test the
term × writer contingency table, e.g. by χ² or by the share of pages that use both terms. Run it in
the consolidation (2b, cross-page), next to 2c.17 contradiction checking. It is the terminology
analog of the 2a.8 churn guard and needs no LLM.

**Glossary induction.** For definition-sentence detection, the DEFT corpus (textbook sentences from
cnx.org) is **CC BY-NC-SA 4.0**
([README](https://raw.githubusercontent.com/adobe-research/deft_corpus/master/README.md)). The
ACTER ATE benchmark has the same licence
([b2find](https://b2find.eudat.eu/dataset/a4baf9b0-a021-546f-9176-43f4cfa6e1a2) [unverified]). Use
either for evaluation only; neither may go into a shipped pack. For our own pages, an LLM labelling
pass (as in `eval/agents/move-index`) is cheaper and already planned. The "defines a concept" move
in 2d.19 *is* glossary induction, so mark that redundancy.

## 3. Prerequisite and concept-dependency learning

- **RefD (reference distance).** RefD(A,B) is the weighted share of A's related concepts that link
  to B, minus the reverse. If it is above θ, B is a prerequisite of A. On Wikipedia, with TF-IDF
  weights, it matched or beat a supervised MaxEnt classifier
  ([Liang et al., EMNLP 2015](https://aclanthology.org/D15-1193.pdf)). **Our doc set and registries
  form exactly such a link graph**: pages link to concept pages and cite ids. Use: compute RefD over
  the built site and compare it with `concepts.ts`. A high-RefD pair with no declared edge is a
  missing prerequisite. A declared edge with negative RefD means pages explain the concepts in the
  opposite order. Slot: 2b verifier at consolidation, also an input to 2d.18. Cost: linear in links.
- **Information-theoretic dependency** (cross-entropy and information flow over topics), with the
  first human evaluation of the inferred edges
  ([Gordon et al., ACL 2016](https://www.isi.edu/results/publications/13074/modeling-concept-dependencies-in-a-scientific-corpus/)).
  Heavier, and adds little once a declared graph exists.
- **LLMs.** Thirteen models were tested zero-shot on 3,196 ESCO skills. The best "closely align"
  with expert links, but the page gives no numbers
  ([Le & Abel 2025](https://arxiv.org/abs/2507.18479)). Use an LLM to *propose* edges for the
  missing MCAD and kernel areas of `concepts.ts`. A human or the coordinator approves them; they
  are data, not truth.
- **Datasets.** CrowdComp, used by RefD. LectureBank has 1,352 lecture files, 208 labelled topics
  and a 320-topic taxonomy ([arXiv](https://arxiv.org/abs/1811.12181) [unverified],
  [repo](https://github.com/Yale-LILY/LectureBank)). I found no licence in the repo; treat it as
  evaluation-only.

**Redundancy and change to DOCS.md.** 2d.18 assumes the concept order must be computed. It mostly
exists already. Change the item to: order pages by `concepts.ts`, extend that file to MCAD, kernel
and skelter, and check it against RefD. Also add a 2b check, **prerequisite-before-use**: a page
that teaches X must link, or come after in a tutorial path, every page that defines each
prerequisite of X. This is deterministic and complements 2a.6 (define before use).

## 4. Information architecture

**Topic-based authoring.** "Every Page is Page One" sets five properties: self-contained,
establishes context, one specific purpose, conforms to a type, stays on one level
([TechWhirl summary of Baker 2013](https://techwhirl.com/every-page-page-one-topic-based-authoring-tech-comm-web/)).
DITA's strict task gives a checkable shape: optional `prereq`, `context`, `steps`, `result`,
`example`, `postreq` in that order, and each step contains a `<cmd>`
([OASIS DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part2-tech-content/archSpec/technicalContent/dita-task-topic.html)).
Diátaxis says a how-to has "no digression, explanation, teaching" and should link out instead
([how-to guides](https://diataxis.fr/how-to-guides/)). Its compass applies at sentence level as
well as page level ([compass](https://diataxis.fr/compass/)).

→ **A per-type section schema** (2b verifier, deterministic). A how-to has, in order, a purpose
line, prerequisites (linked), steps, and a result. A reference page has no imperative steps. A
tutorial has a result after each step. Redundancy: 2a.4 measures rates; the schema is a structural
pass/fail. Both are needed, and the schema is cheaper. **Mixed-mode detection**: label each
section's Diátaxis quadrant with the per-type handprint references (next section). An explanation
section inside a how-to fails, with the fix "move it to the concept page and link it". That
puts EPPO's "stays on one level" into practice.

**Information scent.** Users follow the links whose labels best match their goal
([Card et al., CHI 2001](https://research.cs.vt.edu/ns/cs5724papers/5.usageenvir.adaptation.card.infoscent.pdf);
theory: [Pirolli & Card 1999](https://www.doi.org/10.1037/0033-295X.106.4.643) [unverified]).
Cognitive Walkthrough for the Web (CWW) automates this by scoring the semantic similarity between a
goal and each heading and link label. In Blackmon's data (428 tasks, 82 participants), **the
competing-headings variable alone explained 44% of the variance in time to find the page**.
Participants kept clicking under a heading that seemed close to their goal but did not lead to it
([Blackmon, CU statement](https://www.colorado.edu/ics/sites/default/files/attached-files/ics_affiliateposition2021brown1.pdf);
[CHI 2002](https://www.doi.org/10.1145/503376.503459) [unverified]).

→ **Scent check** (2b, deterministic apart from an embedding). For every task-bank goal (1d), rank
the navigation labels and headings by similarity to the goal. Flag a task when the correct path's
heading is not the top match (**weak scent**), or when a sibling heading scores within ε of it
(**competing heading**). Embeddings, or BM25 as a baseline, with no generation. The data is already
planned: task bank × site tree.

**Tree testing and card sorting with agents.** LLM card sorts agreed well with 28 human studies
(1,399 participants) by mutual information, but cannot replace them
([Kuric et al. 2025](https://arxiv.org/abs/2505.09478)). LLM tree tests are far too optimistic
(MeasuringU, above). Human tree-test success after a correct first click was 81%, against 14% after
a wrong one ([MeasuringU](https://measuringu.com/?p=36842)), so the first click predicts the
outcome. Use: an agent card sort to *propose* the navigation tree in 2d planning, and the scent
check to *gate* it. **Change 2c.14**: run the task agent once with search and free browsing (does
the page work?) and once with only the navigation tree and no search (can the page be reached?).
Count the second as a scent signal, never as human findability.

**Shape and links.** Greater depth hurt web search performance, and a medium breadth/depth
structure did best ([Larson & Czerwinski, CHI 98](https://www.microsoft.com/en-us/research/publication/web-page-design-implications-memory-structure-scent-information-retrieval/)).
Cheap graph metrics for the built site: orphan pages (in-degree 0 from navigation), click depth from
the index, pages more than k clicks from any tutorial path, link-text vs target-title agreement (the
scent of the link itself), and the share of reference pages with no inbound link from a how-to. The
coverage matrix (2b.12) covers ids × pages; these metrics cover pages × pages. They are new, and
link checking in svelters' CI catches only broken links.

**Search logs.** Zero-result queries are the standard way to find documentation gaps
([Document360](https://docs.document360.com/v3/docs/de/analytics-search) [unverified]). Some zero results come
from filters, not missing content ([Lucidworks](https://support.lucidworks.com/hc/en-us/articles/34023142085399-Understanding-zero-results-in-Fusion-Analytics-reports) [unverified]).
No site exists yet. When one does, log queries to qc. In the meantime, two in-house query logs
already exist: the command palette's searches (the `*-synonyms` aliases suggest users search there)
and agents' failed MCP or doc lookups. Zero-result palette queries become `hidden` labels and work
queue items (2d).

**Agent readers.** `llms.txt` (an H1, a summary blockquote, H2 link lists, an "Optional" section)
is the cheap index for MCP and RPC consumers ([llmstxt.org](https://llmstxt.org/)). Generate it
from the page inventory in Layer 3.

## 5. Page-type classification

DOCS.md plans an LLM labelling pass. Two deterministic aids:

- **Character n-grams classify genre best** on web genre corpora, but labels do not transfer between
  collections ([Sharoff, Wu & Markert, LREC 2010](https://aclanthology.org/L10-1011/)). handprint
  already computes character n-grams and fits references. Once the per-type references exist (DOCS.md
  1a), nearest-reference classification by z-distance is a free classifier. Use it to (a) check the
  LLM labels, (b) find mixed-mode sections, and (c) reject a draft whose declared type does not
  match its nearest reference. Slot: 2a, as a mode of `critique`. Cost: none beyond the refit.
  I found no peer-reviewed Diátaxis classifier, only agent skills.
- **API reference knowledge types.** Maalej and Robillard's 12 types (functionality, directives,
  purpose, quality, control-flow, structure, patterns, examples, environment, references,
  non-information, among others), with κ rising to 0.664 over three coding iterations. They found
  **non-information "overwhelmingly present" in member docs**
  ([TSE 2013](https://www.cs.mcgill.ca/~martin/papers/tse2013a.pdf)). For cadabra3's
  `missing_docs = "warn"` rustdoc this is the defect to expect: a doc comment that restates the
  signature. Detector: the novelty ratio of comment words against the item's identifier tokens.
  This is 2a.2 (restatement) applied to `///`; reuse it. Use the taxonomy as the move vocabulary
  for the `api-reference` sub-register in 2d.19.

## 6. Localisation readiness

The catalog is Fluent, and Mozilla's rules apply to it: no concatenation (use placeholders),
descriptive ids, a new id when the meaning changes, comments for localizers, and no reuse of one
string in different contexts
([Mozilla l10n](https://mozilla-l10n.github.io/documentation/localization/dev_best_practices.html);
also [Microsoft i18n checklist](https://learn.microsoft.com/en-ie/Windows/Win32/intl/internationalization-checklist)).
For docs:

1. **Reference UI labels by message id, not literal text**, for example `{{ui:cmd-file-new}}`
   rendered as "New part". This one convention makes the 2b.10 resolver exact (a label either
   resolves or does not), makes the 2a.5 terminology check trivial for UI strings, keeps docs right
   when a label is renamed (2b.13 staleness goes away for labels), and lets translated docs show
   the translated label. **Highest leverage per effort in this report.**
2. **Mark untranslatable spans** (code, ids, CLI flags) in the ITS 2.0 `translate` sense, and mark
   terms in the `terminology` sense with a link to the termbase entry
   ([W3C ITS 2.0](https://www.w3.org/TR/its20/)). In Markdown, inline code plus the id shortcode
   already does both. State it as the rule.
3. **No text-only information in images.** "Images don't get translated"
   ([Google](https://developers.google.com/style/translation)). Screenshots come from `gq shot`
   replays (2b.11), so regenerate them per locale and require alt text or a caption that carries
   the information.
4. **Catalog-side lint**, in cadatomic, not handprint: a message that is only a fragment, reuse of
   one id across contexts, missing localizer comments on messages with placeholders. This goes to
   cadatomic as a request; it improves every tooltip the reference pages quote.
5. The global-English pack (§1) is the prose half.

## 7. Summary table

| Candidate | Algorithm | Data / licence | Cost | Slot | vs DOCS.md |
|---|---|---|---|---|---|
| Concept-oriented termbase with status and subject field | SKOS/TBX record; domain-scoped matching | Our catalog, `concepts.ts`, `*-synonyms` | M (one code path in 2a) | Layer 0 + 2a.5 | Replaces the synonym table |
| UI-label shortcodes | Message-id references | Our catalog | S | Layer 3 rule + 2b.10 | New; makes 2a.5 and 2b.13 cheaper |
| Global-English / CL pack | Lexicon + 3 dims (noun stack, omitted *that*, coordination) | Vale MIT packs; STE paraphrased | S | 2a pack | New; one-step rule redundant with 2a.4 |
| Concept split (term × writer) | Variant grouping + contingency test | Our pages | S–M | 2b, consolidation | New |
| C-value + variant rules + Schwartz–Hearst | ATE | Our corpus; DEFT/ACTER eval only (NC-SA) | M | Layer 0 glossary queue, 2a.6 | Partly redundant with Fightin' Words |
| RefD vs `concepts.ts`; prerequisite-before-use | Link-graph asymmetry; DAG check | Our site graph | S | 2b + 2d.18 | Changes 2d.18 |
| Per-type section schema + mixed-mode | DITA-like order; nearest-reference per section | Per-type references (1a) | S | 2b + 2a | Complements 2a.4 and 2a.7 |
| Scent check (weak / competing headings) | Goal × label similarity | Task bank × nav tree | S–M | 2b | New; corrects 2c.14 |
| Nav-only task-agent run | Agent without search | Task bank | Run cost | 2c.14 | Modifies |
| Site-graph metrics | Orphans, depth, link-text/title agreement | Built site | S | 2b | New |
| Palette/agent query logs → hidden labels | Zero-result mining | qc | S, later | 2d | New |
| `llms.txt` | Generated index | Inventory | XS | Layer 3 | New |

## Top 5 to adopt (leverage per effort)

1. **UI-label shortcodes (`{{ui:<message-id>}}`) as a Layer 3 rule.** Nearly free. It turns
   terminology, resolution, staleness and localisation for UI strings into exact lookups against
   `en.ftl`.
2. **Upgrade 2a.5 to a concept-oriented, domain-scoped termbase** generated from the catalog, the
   56 `*-synonyms` entries (as `hidden`) and `concepts.ts`, with preferred, admitted and deprecated
   statuses. It catches the cross-repo polysemy ("component", "part", "layer") that a synonym table
   cannot. Medium effort, and the defect class with the most hits in a many-agent doc set.
3. **Use `concepts.ts` for 2d.18 and add the prerequisite-before-use check, with RefD as an
   audit.** The graph exists. Extend it to MCAD, kernel and skelter, then run deterministic checks.
4. **CWW-style scent check plus a nav-only run of the task agent.** Cheap with the task bank, and
   it fills the findability gap that an over-capable LLM reader hides.
5. **A global-English / controlled-language pack**: noun stacks, phrasal verbs, omitted *that*,
   ambiguous *-ing*/*-ed*, coordination chains, and per-step length. Mined from MIT Vale packs and
   paraphrased STE rules, so it is pack data plus three small dimensions. The evidence on
   comprehension and MT supports it; O'Brien's result argues against adopting more than this.

Runners-up: concept-split detection at consolidation (cheap, high value once more than ten writers
run); Schwartz–Hearst for acronym define-before-use.
