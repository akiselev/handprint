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
for book in ~/books/adams/*.epub; do
  pandoc "$book" -t gfm -o "/tmp/$(basename "$book" .epub).md"
done
python3 corpora/chapterize.py /tmp/*.md --author douglas-adams --out corpora
```

`pandoc -t gfm` keeps chapter headings as `#` sections, which makes the
splitter trivial. `chapterize.py` is the markdown-side counterpart of the
crate's `handprint gutenberg`, which handles plain-text Gutenberg files.

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
mkdir -p raw/twain && curl -o raw/twain/pg3178.txt https://www.gutenberg.org/files/3178/3178-0.txt
make gutenberg
```

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
