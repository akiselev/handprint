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
    return [(h, "\n".join(b).strip()) for h, b in merged]


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
    args = parser.parse_args()

    target = args.out / args.author
    target.mkdir(parents=True, exist_ok=True)
    written = 0
    for path in args.inputs:
        book = slug(path.stem)
        chapters = split(path.read_text(encoding="utf-8"), args.min_words)
        for index, (heading, body) in enumerate(chapters, 1):
            name = f"{book}-ch{index:02d}.md"
            content = f"# {heading}\n\n{body}\n" if heading else f"{body}\n"
            (target / name).write_text(content, encoding="utf-8")
            written += 1
    print(f"wrote {written} chapter document(s) to {target}")
    if written < 20:
        print(
            "warning: fewer than 20 documents. Calibration samples same-author "
            "pairs, and there are only so many pairs to draw from."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
