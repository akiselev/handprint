# `corpora/` — how to assemble the battery corpora locally

**Nothing in this directory is committed.** `.gitignore` excludes it, the
license-hygiene check fails the build if anything under it is tracked, and the
reason is not caution: four of the five battery authors are in copyright, and
the Folger Shakespeare TEI is non-commercial. What ships from the battery is
metrics and fitted profiles, never text.

The layout every script expects:

```
corpora/
  douglas-adams/       hhgg-01-ch01.md, hhgg-01-ch02.md, ...
  terry-pratchett/
  charles-bukowski/
  hunter-s-thompson/
  shakespeare/
  tech-blog/
  marketing/
  agent-prose/
```

One file per chapter, poem or post. **Per-chapter, not per-book** — that is
what makes within-author variance measurable, and within-author variance is
what a two-sided band and a same-author calibration are made of. A corpus of
whole books gives the calibration nothing to estimate from.

## Fiction and journalism (epub → markdown)

```sh
python3 corpora/epub2md.py ~/books/adams/*.epub \
    --out /tmp/md --exclude corpora/exclude.json --report /tmp/qc.json
python3 corpora/chapterize.py /tmp/md/*.md \
    --author douglas-adams --out corpora --target-words 2500
```

`epub2md.py` is stdlib-only, so it needs no pandoc. `pandoc -t gfm` works too
and produces the same shape; what it does not produce is the QC report, and
three of the failures below are invisible without one. `chapterize.py` is the
markdown-side counterpart of the crate's `handprint gutenberg`, which handles
plain-text Gutenberg files.

Read `/tmp/qc.json` before fitting anything. Four things go wrong here, and
none of them announce themselves:

**Bad scans.** One Adams EPUB in this collection reflows a hard-wrapped source
without restoring the spaces: `the blood that was gobbing out of ithad slowed`.
`epub2md.py` counts words that look like two function words fused and rejects
the book above `--max-glue-rate`. Clean books in this collection score 0.00–0.05
per 1k; the bad one scores 5.6, and that number is a *floor*, since the detector
only catches fusions where both halves are function words. There is no repair —
drop the book.

**Publisher voice.** Copyright pages, "Books by this author" and the About the
Author blurb recur in every book by the same author, so they are exactly what a
reference over-fits. Dropped by file name, by boilerplate phrases, and by the
alt text of the heading images Pratchett's EPUBs use for section titles.

**Other people's prose.** *The Salmon of Doubt* carries an introduction by
Stephen Fry and an epilogue by Richard Dawkins; *A Blink of the Screen* opens
with a foreword by A. S. Byatt; the omnibus *Hitchhiker's* opens with one by
Neil Gaiman. Nothing in the markup distinguishes those from the author's own
front matter — only knowing who wrote them does. They are listed in
`exclude.json`, which reports any entry that stops matching.

Note what is *not* excluded: Pratchett's footnotes, author's notes and Feegle
glossaries, and Adams's "A GUIDE TO THE GUIDE". Those are the author writing,
and several are the most characteristic prose in the book.

**Books without chapters.** Discworld novels have none. Their EPUBs are one
90,000-word document or a few `Chapter_1a`..`Chapter_1h` production splits, so
a heading walk yields three documents for a whole novel — and three documents
per author give a calibration nothing to estimate from. `--target-words 2500`
cuts at paragraph boundaries instead. Equal-length documents are arguably
better than real chapters anyway: chapter length varies enormously and several
dimensions are length-sensitive, so a fixed size removes a confound.

Bukowski's poems go in one file per poem: the verse family's line-geometry
dimensions are about the shape of a poem, and concatenating twenty of them
measures the concatenation.

## Shakespeare

Folger's TEI XML has the best structure — verse and prose are marked
explicitly, which is what `VersePack` prefers over its own heuristic — and is
**non-commercial licensed**, so it is a local battery input and never a shipped
pack. MIT/Moby is the public-domain fallback with weaker structure.

## Public-domain capstone authors

These have a supported path, because they are the ones whose artifacts ship:

```sh
python3 corpora/fetch_gutenberg.py --out raw 74 76 1342 1661 8164
make gutenberg
```

`fetch_gutenberg.py` files each text under the author named in its own header
rather than under an author this script was told to expect. A hand-maintained
id-to-author map is a second place to be wrong and the only invisible one: the
text lands in `raw/twain/`, the fit succeeds, and the reference has learned
Dickens.

The Gutenberg chapterizer finds no markers in some books and emits one
100,000-word document for the whole text. Run those back through
`chapterize.py --target-words 2500 --min-words 0` so the background population
is the same document size as everything else; otherwise document length is
confounded with author.

`handprint gutenberg` strips the licence boilerplate and chapterizes. It warns
loudly about any file whose markers it did not recognize — an unstripped file
teaches the reference Project Gutenberg's legal department and reports it as
the author's voice.

## Registers

* **tech-blog** — posts saved as markdown. Keep the original formatting: the
  apparatus dimensions (`md:link_rate`, `md:footnote_rate`) are the point, and
  a plaintext export destroys exactly the signal being measured.
* **marketing** — landing pages and product copy, one file each. Webis-Clickbait-17
  is a free graded corpus for the headline dimension.
* **agent-prose** — `handprint extract claude-code --out agent.jsonl`, then
  split by model. **Private data**: any vocabulary derived from it goes through
  `handprint contrast --private` and a manual review before it leaves the
  machine, and the raw transcripts never do.

## What the battery does with them

`battery/run.py` holds out three documents per author before fitting, so suite
(a) ranks documents the reference has never seen. Ranking a document that was
in the fitting corpus proves nothing.
