# Preparing a Project Gutenberg corpus

`handprint gutenberg` turns a directory of raw Gutenberg text files into
per-chapter documents you can fit a reference on. It does three things, and the
third one is the reason the other two exist.

## What you need

The CLI, and a directory of `.txt` files sorted into one folder per author:

raw/mark-twain/pg76.txt, raw/jane-austen/pg1342.txt, and so on. The folder name
is the author attribution — nothing reads the file header — so a text filed in
the wrong folder produces a reference that has quietly learned the wrong
person.

## The three stages

**Boilerplate.** Every Gutenberg file is wrapped in a licence. Left in, it is
the most consistent prose in the corpus (it is identical across every book) and
a fitted reference will happily report Project Gutenberg's legal department as
the author's voice.

**Editorial matter.** Transcriber's notes, publication slugs ("First published
in 1880"), plate indexes. These were written by whoever prepared the etext,
sometimes a century after the author died. The Pudd'nhead Wilson file carries
1,505 words of modern historical essay filed under Mark Twain.

**Chapterisation.** One document per chapter, not per book. A corpus of whole
books has no within-author variance to estimate from, and every band you fit on
it will be a point rather than an interval.

## Running it

The `--jsonl` output is optional. It is also the only machine-readable record
of what went in, so skip it and the next question about provenance has no
answer.

## Reading the output

Two things get printed and both matter more than the success count.

Files with no recognised markers are listed by name. That file still has its
licence attached. It has not failed — it has silently succeeded at the wrong
thing, which is worse, and the list is the only place it shows up.

Removed editorial blocks are listed with their rule, word count and first
fourteen words. Skim them. Each one is a judgement about who wrote something,
and a couple of them will be wrong: an early version of the plate-index rule
deleted Watson's list of Sherlock Holmes's limits, on the grounds that it was a
numbered run of short items, which it is.

## Two things it will not do for you

The chapteriser only recognises headings it knows (CHAPTER, BOOK, roman
numerals). Books without them come out as one 100,000-word document. Run those
back through `chapterize.py --target-words 2500 --min-words 0`, or document
length ends up confounded with author.

It also will not notice that you fed it the same book twice under two names.
Nothing here deduplicates across authors.

## Before you fit anything

Count the documents and the words, and compare them against the last run. A
rebuild that quietly produced 600,000 fewer words than yesterday looks exactly
like a rebuild that worked — that is how a whole subdirectory of books went
missing here, and nothing in any log mentioned it.
