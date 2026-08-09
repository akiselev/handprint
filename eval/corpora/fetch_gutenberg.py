#!/usr/bin/env python3
"""Fetch Project Gutenberg texts by ID and file them under the author they name.

The author comes out of the file's own header rather than out of a table in
this script. A hand-maintained id-to-author map is a second place to be wrong,
and being wrong there is invisible: the text lands in `raw/twain/` and the
reference learns Dickens. `handprint gutenberg` downstream strips the licence
boilerplate and chapterizes.

    python3 fetch_gutenberg.py --out raw 74 76 1342 1661
"""

from __future__ import annotations

import argparse
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

# Two layouts, because Gutenberg has served both for years and which one an ID
# answers on is not predictable from the ID.
MIRRORS = (
    "https://www.gutenberg.org/cache/epub/{id}/pg{id}.txt",
    "https://www.gutenberg.org/files/{id}/{id}-0.txt",
    "https://www.gutenberg.org/files/{id}/{id}.txt",
)

TITLE = re.compile(r"^\s*Title:\s*(.+?)\s*$", re.MULTILINE)
AUTHOR = re.compile(r"^\s*Author:\s*(.+?)\s*$", re.MULTILINE)

# "Mark Twain (Samuel Clemens)" -> mark-twain. The parenthetical is a real name
# or a pseudonym depending on the book, and either way it splits one author
# into two directories, which silently halves the corpus.
PAREN = re.compile(r"\s*\(.*?\)\s*")


def slug(name: str) -> str:
    name = PAREN.sub(" ", name)
    out, dash = [], False
    for character in name.lower():
        if character.isalnum():
            out.append(character)
            dash = False
        elif not dash and out:
            out.append("-")
            dash = True
    return "".join(out).strip("-")


def surname_first(author: str) -> str:
    """`Doyle, Arthur Conan` and `Arthur Conan Doyle` are one author."""
    if "," in author:
        last, _, first = author.partition(",")
        return f"{first.strip()} {last.strip()}".strip()
    return author.strip()


def fetch(book_id: int) -> str | None:
    for template in MIRRORS:
        url = template.format(id=book_id)
        try:
            with urllib.request.urlopen(url, timeout=60) as response:
                return response.read().decode("utf-8-sig", errors="replace")
        except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError):
            continue
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ids", nargs="+", type=int, help="Gutenberg book IDs")
    parser.add_argument("--out", type=Path, default=Path("raw"))
    args = parser.parse_args()

    unresolved: list[int] = []
    written = 0
    for book_id in args.ids:
        text = fetch(book_id)
        if text is None:
            print(f"  {book_id}: FETCH FAILED", flush=True)
            unresolved.append(book_id)
            continue

        author_match = AUTHOR.search(text[:4000])
        title_match = TITLE.search(text[:4000])
        if not author_match:
            # No header means no reliable author, and guessing would put text
            # into a reference that did not write it.
            print(f"  {book_id}: no Author: header, skipped", flush=True)
            unresolved.append(book_id)
            continue

        author = slug(surname_first(author_match.group(1)))
        title = title_match.group(1) if title_match else "?"
        target = args.out / author
        target.mkdir(parents=True, exist_ok=True)
        (target / f"pg{book_id}.txt").write_text(text, encoding="utf-8")
        written += 1
        print(f"  {book_id}: {author:24s} {title[:52]}", flush=True)

    print(f"\nwrote {written} text(s) under {args.out}")
    if unresolved:
        print(f"unresolved ids: {' '.join(str(i) for i in unresolved)}", file=sys.stderr)
    return 1 if unresolved and not written else 0


if __name__ == "__main__":
    raise SystemExit(main())
