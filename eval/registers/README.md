# Register corpora — what exists, what is missing, and where to get it

The author battery asks "does this sound like Adams". The register battery asks
"does this sound like a person writing documentation", which is the question the
de-AI mode actually has to answer, because that is the register agents write in.

## Current state

Built by `fetch_core.py` and `fetch_docs.py`, fitted by
`../corpora/refit.sh registers`:

| register | documents | source |
|---|---|---|
| news, opinion-blog, personal-blog, reviews, forum, sports-report | 1,200 each | CORE |
| info-blog | 1,194 | CORE |
| how-to | 1,007 | CORE |
| advice | 794 | CORE |
| marketing | 693 | CORE (`IP` + `DS`) |
| academic | 537 | CORE |
| forum-qa | 506 | CORE |
| encyclopedic | 450 | CORE |
| legal-terms | 145 | CORE |
| recipe | 141 | CORE |
| **software-docs** | see `build/docs-qc.json` | 48 projects at their last pre-2022 commit |

`../battery/matrix.py` still declares `tech-blog` and `agent-prose`, and
neither exists. `tech-blog` cannot be built from CORE — see the paragraph
problem below — and needs a source with its markdown intact.

### The marketing claim, now that it can be tested

It was unfalsifiable before, because *buy now* and *run this command first* are
both imperatives and there was no instructional register in the battery to tell
them apart. With `how-to`, `recipe`, `advice` and `software-docs` on disk,
median rates over 20 documents each:

| register | `dev:imperative_rate` |
|---|---|
| recipe | **6.73** |
| advice | 4.31 |
| software-docs | 3.00 |
| how-to | 2.25 |
| **marketing** | **0.54** |
| news / encyclopedic / reviews | 0.00 |

**Marketing is below four instructional registers and 12× below recipe.** The
other two dimensions in the claim barely fire at all: `pressure` is non-zero in
1 of 20 marketing documents and *higher* on news, `directive` in 5 of 20.

The claim as written is not supported. `registers/imperatives.py` reruns this,
and `../experiments/FINDINGS.md` #21 records the alternative explanation worth
keeping in mind — this marketing corpus is 2012–2014 web persuasion, and the
pack may simply not generalise backwards from modern landing copy.

## What is missing, in priority order

Priority is set by what an agent produces, since that is what the critic is
pointed at.

**Tier 1 — agents write these constantly and we have no corpus for any of them**

| register | why |
|---|---|
| **software-docs** | README, API reference, guides, changelogs. The single most common thing an agent writes, and the one with no substitute in any existing corpus. |
| **encyclopedic** | Neutral-point-of-view reference prose. This is what LLM output *defaults to*, so it is the baseline the de-AI mode has to measure against. |
| **forum-technical** | Explaining something to a peer. The `Register::ForumComment` variant and the `handprint hn` extractor already exist; the corpus does not. |

**Tier 2 — contrast anchors, without which the tier-1 numbers mean nothing**

| register | why |
|---|---|
| **news** | The unmarked professional baseline. Without it there is no "plain" pole and every register looks marked. |
| **academic** | Where the published LLM-signature work lives (nominalization, participial clauses). The `agent-prose` claims are borrowed from that literature and have never been checked against the register they came from. |
| **workplace-email** | The register of "write the PR description". Semi-formal, first-person, addressed to a known reader. |
| **how-to** | The imperative control described above. Highest value per unit of effort in this table. |

**Tier 3 — poles and outliers, cheap to add once the pipeline exists**

`legal-terms` (formality ceiling), `reviews` (the involved, evaluative pole and
marketing's honest cousin), `personal-blog` (first-person informal human
anchor), `plain-government` (deliberately plain professional prose — the
anti-marketing control), `recipe` (extreme imperative plus ellipsis; a useful
outlier for almost no work).

## Sources

Grouped by what the licence lets us do, because that is the constraint that
actually bites — see `../licenses/check.py`.

### CORE — the Corpus of Online Registers of English

The largest single win. **47 human-labelled sub-registers**, ~34,000 documents,
compiled by Biber, Egbert and Davies from a web sample that predates their 2015
publication — so the texts are **pre-2016 at the latest**, and LLM
contamination is not merely unlikely, it is chronologically impossible.

- <https://github.com/TurkuNLP/CORE-corpus> — `train/dev/test.tsv.gz`, three
  columns: labels, document id, text.
- Annotations **CC BY-SA 4.0**. The *texts* are scraped web pages under their
  authors' copyright, so the corpus is **local-only**, exactly like the
  in-copyright authors. Fitted profiles ship; text does not.

Measured from the dev split (multiply by ~7 for the full corpus):

| label | register | ~full corpus | median words |
|---|---|---|---|
| NE | news report | 8,239 | 578 |
| OB | opinion blog | 3,640 | 790 |
| PB | personal blog | 2,296 | 633 |
| DF | discussion forum | 1,498 | 609 |
| IP | informational persuasion | 1,449 | 596 |
| RV | review | 1,435 | 757 |
| HI | how-to instructional | 1,421 | 654 |
| IB | information blog | 1,365 | 745 |
| HT | how-to | 1,043 | 691 |
| DS | description with intent to sell | 1,022 | 519 |
| QA | question/answer forum | 861 | 390 |
| AV | advice | 805 | 680 |
| RA | research article | 658 | 586 |
| EN | encyclopedia article | 399 | 2,442 |
| LT | legal terms and conditions | 140 | 999 |
| RE | recipe | 133 | 492 |
| TS / TR / AD | technical support / report / advertisement | 42 / 35 / 35 | too few |

**The length caveat, measured rather than assumed.** Median document is ~600
words; only about 30% reach 1,000 words and 10% reach 2,000. Against
handprint's own tiers that puts `mfw` and `char_ngram` below *supported* for
most documents. Consequences:

* Fitting a register reference over several hundred documents is fine — the
  aggregate has plenty of tokens.
* **Per-document ranking is not fine**, so suite (a) of the battery will be weak
  on CORE and should say so rather than report a number.
* If per-document power is wanted, concatenate by source site into ~2,500-word
  documents, the same treatment Discworld novels get.

CORE does **not** cover software documentation. `TS` and `TR` have 42 and 35
documents, and `HI`/`HT` are consumer how-tos, not API references.

### Software documentation — build it, no corpus exists

Pick 40–60 projects whose docs carry a permissive licence, check out a git tag
dated **on or before 2021-12**, and extract prose from `.md`/`.rst`. Diversity
comes free: fifty doc teams are fifty house styles.

Clean starting set: **Rust Book** (MIT/Apache-2.0), **Kubernetes docs**
(CC BY 4.0), **PostgreSQL** (PostgreSQL licence), **SQLite** (public domain),
**Python** (PSF), **Django** (BSD).

Avoid **MDN** (CC BY-SA) and **GNU manuals** (GFDL) unless they are kept in
their own isolated pack — share-alike leaking into a sibling pack is failure
class 3 in `../licenses/check.py`.

### Legal — the only genuinely shippable option here

**Caselaw Access Project / CourtListener.** US court opinions are **public
domain**: no local-only caveat, no attribution obligation, text-bearing
artifacts may ship. CAP covers official reports through 2020, so the pre-2022
rule is satisfied by the source itself. Thousands of distinct judges is real
diversity, not a house style.

- <https://case.law/> · <https://www.courtlistener.com/help/api/bulk-data/>

### Academic

**PMC Open Access Subset**, the *commercial-use* group (CC0, CC BY, CC BY-SA,
CC BY-ND), filtered to publication dates ≤ 2021. Bulk FTP, plain text
available. Keep the CC BY-SA portion out of any pack that also carries
something else.

- <https://pmc.ncbi.nlm.nih.gov/tools/ftp/>

### News

**CC-NEWS** (Common Crawl, 2016→). WARC filenames carry the date, so the
pre-2022 filter is a filename glob. Thousands of outlets. Publisher copyright →
local-only.

- <https://data.commoncrawl.org/crawl-data/CC-NEWS/index.html>

### Workplace email

**Enron** — ~500,000 messages from ~150 employees, 1999–2002, released by FERC
and the standard corpus for this register. Non-commercial research use,
local-only. Eighteen years pre-LLM.

**Flag it honestly:** 150 authors inside *one company, one industry, one
era*. It is Enron's house voice as much as it is "business email", and a
register reference fitted on it alone will learn energy-trading English. It is
the best available and it is not representative.

- <https://www.cs.cmu.edu/~enron/>

### Forum and Q&A

**Stack Exchange dumps**, CC BY-SA 4.0. Snapshots up to 2021 are on the
Internet Archive; from July 2024 new dumps moved behind a login whose terms
forbid LLM *training* — worth reading before relying on the newer ones, though
fitting style statistics is not training. **Hacker News** is already reachable
through `handprint hn`.

- <https://archive.org/details/stackexchange>

### Personal blogs

**Blog Authorship Corpus** — 681,288 posts from **19,320 bloggers**, collected
from blogger.com in August 2004. Unbeatable for author diversity and 18 years
pre-LLM. Non-commercial research use.

Do **not** use it for `tech-blog`: it is diary prose, not technical writing.

### Plain government English

**GOV.UK** content under the **Open Government Licence** (permissive,
commercial use allowed) and **plainlanguage.gov** (US federal work). Small, but
a genuinely distinct register — professional prose deliberately written plain —
and the natural control against marketing.

### Marketing — the hard one

No clean open dataset of landing-page copy exists. Best available, in order:

1. **CORE `DS` + `IP` + `AD`** ≈ 2,500 documents of human-labelled selling and
   persuasive web text, 2012–2014.
2. **SEC EDGAR** 8-K press-release exhibits — corporate promotional register,
   thousands of companies, trivially date-filtered, public filings.
3. **Wayback Machine** captures of landing pages with a pre-2022 timestamp.
   Local-only, but you control the sampling frame: 300 sites across 10
   industries beats whatever a scrape happens to catch.
4. **Webis-Clickbait-17** for headlines specifically (already noted in
   `../corpora/README.md`).

Recommended: 1 + 2. Web marketing prose and the corporate-promotional variant
are different enough to be worth separating, and both are diverse across
hundreds of authors.

## Making "pre-2022" checkable

A date on a dataset is not enough, because several of these are *rolling*. The
cutoff has to be enforced per item and recorded, the same way `pd_basis` is:

| source | how the cutoff is enforced |
|---|---|
| CC-NEWS | WARC filename date ≤ 2021-12 |
| PMC OA | publication date field |
| Stack Exchange | dump snapshot ≤ 2021, or filter `CreationDate` |
| software docs | `git checkout` a tag dated ≤ 2021-12 — **not `main`** |
| CORE, Enron, Blog Authorship, CAP | fixed and safely old |

Record the cutoff and the item count per source in the corpus manifest.
Otherwise "pre-2022" is a claim nobody can check a year from now, including us.

## Making "diverse" checkable

Not copying a single voice needs a cap, not a hope. Per register:

* **≥100 distinct sources** (domains, authors, projects, courts).
* **No source above ~2% of the register's documents.**
* Source count and the top-source share recorded in provenance.

Enron fails the first test as a standalone business-email corpus and should be
labelled as such rather than quietly counted as 150 authors.
