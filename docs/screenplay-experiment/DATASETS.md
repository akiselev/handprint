# Screenplay data sources and acquisition policy

Verified against publisher/author pages on 2026-09-07. Availability and public metadata were checked; no full corpus download or corpus-wide quality audit was performed. Dataset sizes below are publisher claims, not a new count. Public accessibility is not a blanket permission for commercial use, model training, external processing, or redistribution.

## Recommended sources

| Source | Published scope | Useful role | Access and qualification |
|---|---|---|---|
| [MovieSum](https://huggingface.co/datasets/rohitsaxena/MovieSum) | 2,200 structured English screenplays, Wikipedia summaries, IMDb IDs; 1,800/200/200 source splits | Primary first corpus; dialogue/action partitions, complete-script context | XML strings inside dataset records. Published license: CC BY-NC 4.0. Download only for a permitted intended use; preserve provenance and underlying-rights uncertainty. |
| [ScriptBase](https://github.com/EdinburghNLP/scriptbase) | Alpha: 1,276 movies; J: 917 with cleaned text/XML and metadata | Alternative representations, provenance comparison, historical metadata | Archives are linked from the repository. J is a subset, not another 917 independent films. The reviewed README does not establish broad commercial rights. |
| [Cornell Movie-Dialogs](https://www.cs.cornell.edu/~cristian/Cornell_Movie-Dialogs_Corpus.html) | 617 films, 304,713 utterances, 220,579 exchanges | Speaker/turn experiments, conversational style coordination | Original project and ConvoKit distribution. Extracted conversations, not 617 complete screenplay documents. Review original terms; do not rely on third-party mirrors' license labels. |
| [TRIPOD](https://github.com/ppapalampidi/TRIPOD) | 99 films, plot synopses, scene-segmented screenplays, turning-point annotations | Structural/turning-point evaluation | Derived from ScriptBase. Inspect which annotation level exists for each split; not every scene is a human quality label. |
| [CML-Bench data](https://huggingface.co/datasets/songdj/CML-Bench) and [code](https://github.com/DuNGEOnmassster/CML-Bench) | MovieSum-derived script segments and associated evaluation artifacts | Reusable screenplay-specific evaluation baselines | Dataset card: CC BY-NC 4.0; code repository: MIT. These are separate licenses. The dataset viewer reports a mixed-schema cast failure; import explicitly selected data files/configurations rather than assuming all JSON files share one schema. |
| [STAGE current release](https://github.com/roytian1992/STAGE_v0) | Task assets for 151 films: 109 English, 42 Chinese; original and identity-anonymized conditions | Cross-scene state, knowledge-boundary and character consistency tests | The current repository explicitly does not redistribute full screenplay text. Metadata gives source URLs where available. Obtain scripts separately under applicable terms. Do not count STAGE as 151 downloadable full scripts. |
| [SummScreen](https://github.com/mingdachen/SummScreen) | About 26.9k TV transcript/recap pairs in the paper's corpus | Later television extension: episodic continuity and dialogue analysis | Community-contributed transcripts, not production screenplays. Preserve show/season/episode identity and source subset. Tokenized versus untokenized representations affect style analysis. |

### Why MovieSum first

Its structured representation reduces the first project's parsing burden and includes recent and older works. It is adequate to start testing statistical extraction without launching a crawler. The [paper](https://arxiv.org/html/2408.06281v1) states that it includes ScriptBase-J, so combining both does not produce 3,117 unique movies. The paper also documents source collection from Script Slug, IMSDb, and Daily Script, making overlap with those archives likely.

Do not treat formatted XML as audited ground truth. The published preview includes title-page text appearing under character/dialogue tags. Preserve header regions, audit samples, and quarantine uncertain records. Do not automatically count every `<dialogue>` span as spoken text.

The dataset card and paper use different average-length figures. Measure actual words and model tokens after importing the pinned release rather than estimating costs from an ambiguous published average.

### Extra scripts and alternate drafts

[IMSDb](https://imsdb.com/) provides HTML scripts. [SimplyScripts](https://www.simplyscripts.com/) explicitly includes screenplays, transcripts, TV, unproduced material, and other forms. [Script Slug](https://www.scriptslug.com/) and [Daily Script](https://www.dailyscript.com/) are useful targeted sources, including for versions and underrepresented reference styles.

Treat these as source discovery, not an entitlement to mass redistribution. Build per-source importers only when they fill a measured gap. Store access terms, fetching policy, draft/date evidence, completeness, and source type. Do not automatically merge a transcript with a screenplay or assume a later draft is artistically superior.

There was no large verified public corpus of human-preferred screenplay revision pairs established in this research pass. That is the important dataset we would build: permitted drafts, deliberate interventions, agent revisions, and independently judged preferences. Collections of finished scripts do not supply those labels.

### Subtitles are a separate data class

[OPUS OpenSubtitles](https://opus.nlpl.eu/datasets/OpenSubtitles) is useful for spoken-language and translation research, but subtitle lines should not be treated as screenplay turns or full narrative context.

The currently reviewed [Helsinki-NLP/OpenSubtitles2024](https://huggingface.co/datasets/Helsinki-NLP/OpenSubtitles2024) repository is specifically a held-out bilingual development/evaluation benchmark. Its card requires accepting access conditions and says it is not intended for model training. The repository name should not be mistaken for unrestricted access to the entire OpenSubtitles training collection. It is not on the first milestone's critical path.

## Rights-aware source manifest

For each source artifact record:

```text
source_id, source_uri, retrieved_at, upstream_revision
raw_hash, parser_version, normalized_hash
work_id, version_id, parent_version_id, duplicate_group
source_dataset, source_split
language, original_language, translation_status, source_kind
title, release_year, script_date, credited_writers, credit_provenance
completeness, parse_status, unknown_block_share
upstream_license_claim, underlying_rights_status
allowed_uses: acquire / analyze / external_processing / fit_reference /
              retrieve_display / train / redistribute
permission_evidence, reviewer, review_date
```

Use `unknown` where evidence is missing. Do not infer that an MIT code license covers screenplay text. [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/) restricts commercial use and notes that its permissions may not cover every right needed for a use. Local storage or calling a project research does not itself resolve permissions.

Keep raw scripts, extracted passages, and any sidecars reproducing text out of the public repository unless redistribution is established. Use original authored fixtures for continuous integration. Separately review whether a fitted artifact or snippet-rich report is distributable; avoid assuming all derived artifacts are unrestricted.

## Deduplication and split procedure

1. Freeze source releases and acquisition records.
2. Hash exact raw content and a documented normalized form.
3. Link works using stable IDs and verified title/year/source metadata.
4. Identify near duplicates using token-shingle similarity; review uncertain matches and preserve genuine draft variants.
5. Create work/version groups across all data sources, annotations, summaries, and benchmark derivatives.
6. Assign entire groups to fit, development, and final-test roles before extracting reference examples.
7. Keep external benchmarks' published splits for reproducing those benchmarks, with a separate cross-corpus exclusion map for Handprint experiments.
8. Test additional writer/franchise-disjoint and source-disjoint conditions where the corpus supports them.

No movie-level rating should be copied onto every scene as a quality label. IMDb ratings, awards, release dates, and production outcomes are optional descriptive/stratification variables, not a substitute for judgments about the screenplay text or a particular edit.

## Research methods to reuse, not quality ground truth

- [ExpressivityBench](https://aclanthology.org/2026.findings-eacl.235/): implicit communication of target properties without naming them; adapt to audience-inference probes.
- [Dramatic Conversation Disentanglement](https://aclanthology.org/2023.findings-acl.248/): annotated dialogue structure; useful for interaction extraction, not quality labels.
- [CML-Bench paper](https://arxiv.org/abs/2510.06231): dialogue coherence, character consistency, and plot reasonableness; use as an external baseline and validate against our readers.
- [STAGE current paper](https://arxiv.org/html/2601.08510v9): evolving character state and checkpoint-bounded knowledge; current scope differs from its original January 2026 version.
- [LitBench](https://arxiv.org/abs/2507.00769): creative-writing preference evaluation; not screenplay-specific.
- [Mind the Style Gap](https://arxiv.org/html/2502.15022v3): content-preservation evaluation can be distorted by style differences and test-set construction.
- [Evaluating Style Transfer for Text](https://aclanthology.org/N19-1049/): separate style strength, content preservation, and naturalness.
