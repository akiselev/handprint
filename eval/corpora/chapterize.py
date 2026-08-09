#!/usr/bin/env python3
"""Split pandoc-produced markdown into per-chapter documents.

The markdown-side counterpart of the crate's `handprint gutenberg`, which
handles plain-text Gutenberg files. Here the input came through
`pandoc -t gfm`, so chapters are already `#` headings and the split is a
heading walk rather than a heuristic.

Per-chapter is the whole point. A corpus of whole books gives a same-author
calibration nothing to estimate from, and a two-sided band needs within-author
variance to have a width.

    python3 chapterize.py /tmp/*.md --author douglas-adams --out corpora
"""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

HEADING = re.compile(r"^(#{1,3})\s+(.*)$")


def slug(name: str) -> str:
    out = []
    dash = False
    for character in name.lower():
        if character.isalnum():
            out.append(character)
            dash = False
        elif not dash and out:
            out.append("-")
            dash = True
    return "".join(out).strip("-") or "untitled"


def split(text: str, min_words: int) -> list[tuple[str | None, str]]:
    """Split on headings, merging anything below the word floor forward."""
    chapters: list[tuple[str | None, list[str]]] = []
    heading: str | None = None
    buffer: list[str] = []
    seen = False

    for line in text.splitlines():
        match = HEADING.match(line)
        if match:
            if seen and any(l.strip() for l in buffer):
                chapters.append((heading, buffer))
            heading = match.group(2).strip()
            buffer = []
            seen = True
            continue
        buffer.append(line)
    if seen and any(l.strip() for l in buffer):
        chapters.append((heading, buffer))
    if not chapters and text.strip():
        chapters = [(None, text.splitlines())]

    # Merge short sections into their predecessor: a dedication or a
    # one-paragraph epigraph is not a document, and a corpus of hundred-word
    # documents estimates a calibration from noise.
    merged: list[tuple[str | None, list[str]]] = []
    for heading, body in chapters:
        words = sum(len(l.split()) for l in body)
        if words < min_words and merged:
            merged[-1][1].extend(["", *body])
            continue
        merged.append((heading, list(body)))

    # A leading runt has no predecessor to merge back into, so the rule above
    # silently keeps it. That is how a table of contents, a colophon or a
    # one-word title page becomes document 001 of an author's corpus — and a
    # 1-word document is a per-document vector computed from nothing, which
    # widens every band it touches. Merge it forward instead.
    while len(merged) > 1 and sum(len(l.split()) for l in merged[0][1]) < min_words:
        head, body = merged.pop(0)
        merged[0] = (merged[0][0], [*body, "", *merged[0][1]])

    return [(h, "\n".join(b).strip()) for h, b in merged]


def resize(body: str, target: int) -> list[str]:
    """Cut an over-long section into ~`target`-word pieces at paragraph breaks.

    Discworld novels have no chapters. Their EPUBs are one 90,000-word document,
    or a handful of arbitrary `Chapter_1a`..`Chapter_1h` splits that are
    production artifacts rather than structure — so a heading walk yields three
    documents for a whole novel, and a three-document author has no measurable
    within-author variance for a calibration to estimate from.

    Equal-length pieces are arguably better than real chapters anyway: chapter
    length varies enormously, and several dimensions are length-sensitive, so
    documents of one size take a confound out of the comparison. The cut is
    always at a paragraph boundary, never mid-sentence.
    """
    paragraphs = [p for p in re.split(r"\n\s*\n", body) if p.strip()]
    pieces: list[str] = []
    current: list[str] = []
    count = 0
    for paragraph in paragraphs:
        current.append(paragraph)
        count += len(paragraph.split())
        if count >= target:
            pieces.append("\n\n".join(current))
            current, count = [], 0
    if current:
        # A short tail is appended to the last piece rather than kept as a runt
        # document, which would be measured as an unusually short one.
        if pieces and count < target // 2:
            pieces[-1] += "\n\n" + "\n\n".join(current)
        else:
            pieces.append("\n\n".join(current))
    return pieces or [body]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="+", type=Path)
    parser.add_argument("--author", required=True)
    parser.add_argument("--out", type=Path, default=Path("corpora"))
    parser.add_argument(
        "--min-words",
        type=int,
        default=1500,
        help="sections below this merge into the previous one",
    )
    parser.add_argument(
        "--target-words",
        type=int,
        default=0,
        help="also cut sections longer than this into ~this many words, at "
        "paragraph boundaries. Required for books without chapters.",
    )
    parser.add_argument(
        "--drop-below",
        type=int,
        default=400,
        help="discard documents shorter than this. Below a few hundred words "
        "a per-document vector is mostly imputation, and it still counts as "
        "one draw when the band is fitted.",
    )
    args = parser.parse_args()

    target = args.out / args.author
    target.mkdir(parents=True, exist_ok=True)
    written = 0
    runts: list[tuple[str, int]] = []
    # Content hash -> the document that claimed it first. Several of these
    # EPUBs append the opening chapters of the *next* book as a preview, so
    # the same three chapters arrive twice under two book slugs and get
    # double weight in every band the author has.
    seen: dict[str, str] = {}
    duplicates: list[tuple[str, str]] = []
    for path in args.inputs:
        book = slug(path.stem)
        chapters = split(path.read_text(encoding="utf-8"), args.min_words)
        index = 0
        for heading, body in chapters:
            # Dedupe *here*, on whole sections, not on the pieces written
            # below. Several of these EPUBs append the opening chapters of the
            # next book as a preview, so the same chapter arrives under two
            # book slugs — but the resize below cuts cumulatively from the top
            # of each document, so the same prose comes out at different
            # offsets and no two pieces are ever byte-identical. Comparing
            # pieces finds nothing; comparing sections finds all of it.
            fingerprint = hashlib.sha1(
                " ".join(body.split()).encode("utf-8")
            ).hexdigest()
            if fingerprint in seen:
                duplicates.append((f"{book}-ch{index + 1:03d}", seen[fingerprint]))
                continue
            seen[fingerprint] = f"{book}-ch{index + 1:03d}"
            if args.target_words and len(body.split()) > args.target_words * 1.5:
                pieces = resize(body, args.target_words)
            else:
                pieces = [body]
            for offset, piece in enumerate(pieces):
                index += 1
                name = f"{book}-ch{index:03d}.md"
                # The heading rides on the first piece only: repeating it would
                # put the same chapter title into twenty documents, and the
                # frequent-word family would learn it.
                content = (
                    f"# {heading}\n\n{piece}\n"
                    if heading and offset == 0
                    else f"{piece}\n"
                )
                words = len(content.split())
                if words < args.drop_below:
                    runts.append((name, words))
                    continue
                (target / name).write_text(content, encoding="utf-8")
                written += 1
    print(f"wrote {written} chapter document(s) to {target}")
    if runts:
        print(f"dropped {len(runts)} document(s) below {args.drop_below} words: "
              + ", ".join(f"{n} ({w}w)" for n, w in runts[:8]))
    if duplicates:
        print(f"dropped {len(duplicates)} duplicate document(s): "
              + ", ".join(f"{n} == {first}" for n, first in duplicates[:6]))
    if written < 20:
        print(
            "warning: fewer than 20 documents. Calibration samples same-author "
            "pairs, and there are only so many pairs to draw from."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
