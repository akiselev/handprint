#!/usr/bin/env python3
"""Extract prose from EPUBs into markdown, with a quality report.

The stdlib-only stand-in for `pandoc -t gfm` from `README.md`, for machines
without pandoc. Output is the same shape — one markdown file per book, chapter
headings as `#` — so `chapterize.py` consumes it unchanged.

Three things here are not incidental, because each one silently corrupts a
reference rather than failing:

**Inline elements must not break words.** Publisher EPUBs open chapters with
`<span class="dropcaps">T</span>he house stood`. A converter that treats every
tag as a block boundary yields `T he house`, and every chapter opening in the
corpus donates a spurious one-letter word to the frequency table.

**Front and back matter is the publisher's voice.** Copyright pages, "Books by
this author", and the About the Author blurb are written by someone who is not
the author, and they recur in every book — so they are exactly the text a
reference over-fits to. This is the EPUB-side version of the unstripped
Gutenberg licence that `handprint gutenberg` warns about.

**Some EPUBs are bad scans.** Text reflowed from a hard-wrapped source without
restoring the spaces produces `the blood that was gobbing out of ithad slowed`
— a real line from one of these files. Glued words wreck token counts and the
frequent-word family. They are undetectable by eye at corpus scale, so the
converter counts them and `--max-glue-rate` refuses the book.

    python3 epub2md.py 'books/*.epub' --out /tmp/md --report /tmp/qc.json
"""

from __future__ import annotations

import argparse
import html
import json
import posixpath
import re
import sys
import unicodedata
import zipfile
from html.parser import HTMLParser
from pathlib import Path
from xml.etree import ElementTree

# Elements that end the run of text. Everything not listed is inline and must
# join without a space — see the dropcaps note above.
BLOCK = {
    "p", "div", "h1", "h2", "h3", "h4", "h5", "h6", "li", "ul", "ol", "dl",
    "dd", "dt", "blockquote", "pre", "table", "tr", "td", "th", "section",
    "article", "header", "footer", "aside", "figure", "figcaption", "hr",
    "body", "nav", "main",
}
HEADINGS = {"h1": 1, "h2": 2, "h3": 3, "h4": 3, "h5": 3, "h6": 3}
SKIP = {"head", "style", "script", "title", "svg"}

# Matched against the spine item's file name. `index` is deliberately absent:
# two of these books use `index_split_000.html` for the novel itself.
FRONT_BACK = re.compile(
    r"cover|titlepage|title_?page|^title|toc|contents|copyright|dedication"
    r"|also_?by|^also$|about_?author|about_?the_?author|about_?book|^author$"
    r"|praise|acknowledg"
    r"|publisher|colophon|imprint|frontmatter|backmatter|teaser|advert"
    r"|introducingdiscworld|newsletter|ad_?card|ebookreg|signup|mailinglist",
    re.IGNORECASE,
)

# Phrases that belong to a publisher rather than to a novelist. Two hits in a
# short document is the drop rule; the report lists which fired.
BOILERPLATE = (
    "all rights reserved",
    "a cip catalogue record",
    "cataloguing-in-publication",
    "isbn",
    "first published in great britain",
    "published by",
    "random house",
    "transworld publishers",
    "harpercollins",
    "no part of this publication may be",
    "this ebook is copyright material",
    "www.",
    "printed and bound",
    "the moral right of the author",
    "is a work of fiction",
    "penguin books",
    "victor gollancz",
    "doubleday",
    "table of contents",
    "by the same author",
    "for more information",
)

# Back matter that a publisher appends to the *end of a real chapter file*
# instead of giving it a spine item of its own. `classify` cannot see it: the
# document is 2,700 words of Thor storming into Valhalla followed by 150 words
# of mailing-list pitch, so it is neither short enough nor boilerplate-dense
# enough to drop, and dropping it whole would cost the chapter.
#
# Two kinds of marker, because one rule for both cuts novels in half.
#
# A *heading* must be the whole of a short line. The looser "line contains
# `books by ...`" version deleted 765 words of Guards! Guards! from
# "he explored the spines of the books by his side", and 4,513 words of
# Monstrous Regiment. Every back-matter heading in this collection is its own
# line of five words or fewer; every false positive was a clause inside a
# sentence. `fullmatch` on a short line separates them exactly.
TAIL_HEADINGS = (
    r"also by .{0,40}",
    r"books by .{0,40}",
    r"by the same author",
    r"the discworld\W* series",
    r"have you read them all\??",
    r"about the author",
    r"other books by .{0,40}",
)
TAIL_HEADING_RE = re.compile(
    "|".join(f"(?:{p})" for p in TAIL_HEADINGS), re.IGNORECASE
)
MAX_HEADING_WORDS = 8

# A *phrase* may appear anywhere, because no novelist writes these.
TAIL_PHRASES = (
    "we hope you enjoyed reading",
    "thank you for purchasing this",
    "join our mailing list",
    "click here to sign up",
    "sign up for our newsletter",
)


def is_tail_marker(line: str) -> bool:
    """Whether this line begins publisher back matter."""
    stripped = line.strip().lstrip("#").strip().rstrip(".:")
    if not stripped:
        return False
    lowered = stripped.lower()
    if any(phrase in lowered for phrase in TAIL_PHRASES):
        return True
    return (
        len(stripped.split()) <= MAX_HEADING_WORDS
        and TAIL_HEADING_RE.fullmatch(stripped) is not None
    )

WORD = re.compile(r"[A-Za-z][a-z]+")

# A closed set of function words. `ithad` and `himthat` are both halves of this
# list; a real English word rarely is. Used only to *count* the defect — the
# repair is to drop the book, not to guess where the space went.
FUNCTION_WORDS = frozenset("""
about after again against all almost also although always among and another any
are around because been before being both but came can come could did does
down each even ever every for from get got had has have her here him his how
into its just like made make many may might more most much must never new not
now off one only other our out over own said same she should since some still
such take than that the their them then there these they this those though
three through time too two under until upon very was way well went were what
when where which while who why will with without would you your
""".split())

# Closed compounds whose halves are both function words. Without these the
# detector reports every `into` and `another` as a defect.
COMPOUNDS = frozenset("""
into another however whatever whenever wherever whoever somewhat cannot himself
herself myself yourself itself themselves ourselves therefore thereby wherein
whereby somehow someone something sometime sometimes somebody anyone anything
anybody anyway anywhere everyone everything everybody everywhere everyday
nothing nobody nowhere within without upon onto also although always altogether
already alright become became becoming before beforehand behind beside besides
between beyond throughout thereafter whereas hereafter moreover nevertheless
notwithstanding outside inside overall underneath forever
somewhere someplace anymore whereupon whereof whereat wherefore whatsoever
forget forgets forgetting forgot forgotten forgive forgave forgiven
overcome overcame overtime overall overtake overtook overhead overnight
underwent undergo undergone understand understood undertake undertook
whichever whatnot however therein thereof thereon thereto herein hereby
income outcome outside downstairs upstairs alone amount
""".split())


class Extract(HTMLParser):
    """Collect an XHTML body as a list of (heading_level, text) blocks."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.blocks: list[tuple[int, str]] = []
        self.alts: list[str] = []
        self._buffer: list[str] = []
        self._heading = 0
        self._skip = 0

    def _flush(self) -> None:
        text = "".join(self._buffer)
        self._buffer = []
        text = text.replace("\xa0", " ")
        # Collapse horizontal runs but keep the line breaks <br> introduced.
        text = re.sub(r"[^\S\n]+", " ", text)
        text = "\n".join(line.strip() for line in text.split("\n"))
        text = re.sub(r"\n{2,}", "\n", text).strip()
        if text:
            self.blocks.append((self._heading, text))
        self._heading = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag in SKIP:
            self._skip += 1
            return
        if self._skip:
            return
        if tag == "img":
            # Pratchett's EPUBs set section titles as images; the alt text is
            # the only place "About the Author" appears in the document.
            alt = dict(attrs).get("alt")
            if alt:
                self.alts.append(alt)
            return
        if tag == "br":
            self._buffer.append("\n")
            return
        if tag in BLOCK:
            level = HEADINGS.get(tag, 0)
            self._flush()
            self._heading = level

    def handle_endtag(self, tag: str) -> None:
        if tag in SKIP:
            self._skip = max(0, self._skip - 1)
            return
        if self._skip:
            return
        if tag in BLOCK:
            self._flush()

    def handle_data(self, data: str) -> None:
        if not self._skip:
            self._buffer.append(data)

    def close(self) -> None:  # noqa: D102
        super().close()
        self._flush()


def spine_documents(archive: zipfile.ZipFile) -> list[str]:
    """Reading order, from container.xml through the OPF spine.

    Falling back to `namelist()` order would be worse than failing: the archive
    order of one of these books puts chapter 40 first, and a corpus whose
    documents are shuffled inside a book still *looks* fine.
    """
    container = archive.read("META-INF/container.xml").decode("utf-8", "replace")
    root = ElementTree.fromstring(container)
    rootfile = root.find(".//{urn:oasis:names:tc:opendocument:xmlns:container}rootfile")
    if rootfile is None or not rootfile.get("full-path"):
        raise ValueError("container.xml names no rootfile")
    opf_path = rootfile.get("full-path")

    opf = ElementTree.fromstring(archive.read(opf_path).decode("utf-8", "replace"))
    ns = {"opf": "http://www.idpf.org/2007/opf"}
    manifest = {}
    for item in opf.findall(".//opf:manifest/opf:item", ns):
        if item.get("id") and item.get("href"):
            manifest[item.get("id")] = item.get("href")

    base = posixpath.dirname(opf_path)
    names = set(archive.namelist())
    documents = []
    for ref in opf.findall(".//opf:spine/opf:itemref", ns):
        idref = ref.get("idref")
        href = manifest.get(idref, idref)  # some EPUBs use the file name as id
        if not href:
            continue
        href = html.unescape(href.split("#")[0])
        path = posixpath.normpath(posixpath.join(base, href)) if base else href
        if path in names:
            documents.append(path)
    return documents


def load_excludes(path: Path | None) -> dict[str, list[str]]:
    """Per-book sections that are not the author's, listed by hand.

    The heuristics below cannot reach these. `The Salmon of Doubt` carries an
    introduction by Stephen Fry and an epilogue by Richard Dawkins; `A Blink of
    the Screen` opens with a foreword by A. S. Byatt. Nothing in the file names
    or the markup distinguishes those from the author's own front matter — only
    knowing who wrote them does, so they are listed rather than guessed at.

    Leaving them in is the failure this whole file is about: a reference fitted
    on Adams that has quietly learned Dawkins, in the sections most likely to
    be stylistically extreme.
    """
    if path is None:
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def classify(name: str, text: str, alts: list[str], words: int) -> str | None:
    """Return a drop reason, or None to keep."""
    stem = Path(name).stem
    if FRONT_BACK.search(stem):
        return f"filename:{stem}"
    for alt in alts:
        if FRONT_BACK.search(alt.replace(" ", "")):
            return f"image-title:{alt[:40]}"
    lowered = text.lower()
    hits = [phrase for phrase in BOILERPLATE if phrase in lowered]
    if len(hits) >= 3 or (len(hits) >= 2 and words < 2000):
        return "boilerplate:" + ",".join(hits[:3])
    # A whole document that is nothing but back matter. The file-name rules
    # above depend on knowing that Simon & Schuster calls its newsletter page
    # `ebookregback.html`, which is exactly the kind of knowledge that does not
    # transfer to the next publisher. This rule needs only the words on the
    # page, and is length-gated so it cannot fire on a chapter.
    if words < 500:
        for line in text.splitlines():
            if is_tail_marker(line):
                return f"back-matter:{line.strip()[:40]}"
    if words < 40:
        return f"too-short:{words}w"
    return None


def cut_tail(body: str) -> tuple[str, int, str]:
    """Drop appended back matter, returning the body, words cut, and the match.

    Only the *last* third of the document is searched. A Discworld novel can
    open on a title list, and the two cases want opposite treatment: at the
    front the list is the document and `classify` drops it; at the back it is a
    tail on a real chapter. Searching the whole document would let a false
    match near the top delete the chapter.
    """
    lines = body.splitlines()
    floor = (len(lines) * 2) // 3
    for i in range(floor, len(lines)):
        if is_tail_marker(lines[i]):
            stripped = lines[i].strip()
            removed = len("\n".join(lines[i:]).split())
            # A "cut" that takes most of the document means the match was the
            # document, not a tail on one. Let `classify` handle that.
            if removed > len(body.split()) * 0.6:
                return body, 0, ""
            return "\n".join(lines[:i]).rstrip(), removed, stripped[:60]
    return body, 0, ""


def glue_rate(text: str) -> tuple[float, list[str]]:
    """Per-1k-token rate of words that look like two function words fused."""
    tokens = WORD.findall(text)
    if not tokens:
        return 0.0, []
    suspects = []
    for token in tokens:
        lowered = token.lower()
        if len(lowered) < 5 or lowered in FUNCTION_WORDS or lowered in COMPOUNDS:
            continue
        for cut in range(2, len(lowered) - 1):
            if (
                lowered[:cut] in FUNCTION_WORDS
                and lowered[cut:] in FUNCTION_WORDS
            ):
                suspects.append(token)
                break
    return 1000.0 * len(suspects) / len(tokens), suspects


def normalize(text: str) -> str:
    """Compatibility-fold the typographic characters that are not style.

    Curly quotes and dashes stay: they are punctuation features and the whole
    `punct` family is about them. Ligatures and odd spaces go, because they are
    typesetting artifacts that would read as vocabulary.
    """
    text = text.replace("ﬁ", "fi").replace("ﬂ", "fl")
    text = text.replace("​", "").replace("﻿", "")
    text = "".join(
        " " if unicodedata.category(c) == "Zs" and c != " " else c for c in text
    )
    return text


def convert(path: Path, keep_emphasis: bool, excludes: list[str]) -> dict:
    archive = zipfile.ZipFile(path)
    kept: list[str] = []
    dropped: list[dict] = []
    trimmed: list[dict] = []
    unused = set(excludes)
    for name in spine_documents(archive):
        try:
            raw = archive.read(name).decode("utf-8", "replace")
        except KeyError:
            continue
        parser = Extract()
        try:
            parser.feed(raw)
            parser.close()
        except Exception as error:  # a malformed document is not fatal
            dropped.append({"doc": name, "reason": f"parse-error:{error}"})
            continue

        body = "\n\n".join(
            ("#" * level + " " + text) if level else text
            for level, text in parser.blocks
        )
        body = normalize(html.unescape(body))
        words = len(body.split())

        # A manifest entry matches the spine path or the section's opening
        # words, because some of these EPUBs name every file `index_split_027`.
        opening = " ".join(body.split())[:100]
        hit = next(
            (e for e in excludes if e in name or e.lower() in opening.lower()), None
        )
        if hit:
            unused.discard(hit)
            dropped.append({"doc": name, "reason": f"excluded:{hit}", "words": words})
            continue

        reason = classify(name, body, parser.alts, words)
        if reason:
            dropped.append({"doc": name, "reason": reason, "words": words})
            continue
        body, tail_words, matched = cut_tail(body)
        if tail_words:
            trimmed.append({"doc": name, "words": tail_words, "matched": matched})
        kept.append(body)

    text = "\n\n".join(kept).strip() + "\n"
    if not keep_emphasis:
        pass  # emphasis was never emitted; the flag documents the default
    rate, suspects = glue_rate(text)
    return {
        "book": path.stem,
        "words": len(text.split()),
        "kept_docs": len(kept),
        "dropped": dropped,
        # Back matter cut off the end of a document that was otherwise kept.
        "trimmed": trimmed,
        # A manifest entry that matches nothing is a silent no-op, and the
        # section it was meant to remove is in the corpus.
        "stale_excludes": sorted(unused),
        "glue_per_1k": round(rate, 3),
        "glue_examples": sorted(set(suspects))[:12],
        "text": text,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="+", type=Path, help=".epub files")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--report", type=Path, help="write the QC report as JSON")
    parser.add_argument(
        "--min-words",
        type=int,
        default=8000,
        help="reject a book yielding less than this; catches broken archives",
    )
    parser.add_argument(
        "--max-glue-rate",
        type=float,
        default=1.0,
        help="reject a book above this many fused words per 1k tokens",
    )
    parser.add_argument(
        "--keep-emphasis",
        action="store_true",
        help="(reserved) emit <em> as markdown; off, since a corpus of "
        "asterisks would be measured by the punctuation family",
    )
    parser.add_argument(
        "--exclude",
        type=Path,
        help="JSON manifest of per-book sections written by someone other "
        "than the author (see exclude.json)",
    )
    args = parser.parse_args()

    excludes = load_excludes(args.exclude)
    args.out.mkdir(parents=True, exist_ok=True)
    report, accepted, rejected = [], 0, 0
    for path in sorted(args.inputs):
        try:
            result = convert(path, args.keep_emphasis, excludes.get(path.stem, []))
        except Exception as error:
            print(f"REJECT {path.name}: {error}", file=sys.stderr)
            report.append({"book": path.stem, "rejected": f"error:{error}"})
            rejected += 1
            continue

        text = result.pop("text")
        problems = []
        if result["words"] < args.min_words:
            problems.append(f"only {result['words']} words")
        if result["glue_per_1k"] > args.max_glue_rate:
            problems.append(
                f"glued-word rate {result['glue_per_1k']}/1k "
                f"(e.g. {', '.join(result['glue_examples'][:4])})"
            )
        if problems:
            result["rejected"] = "; ".join(problems)
            print(f"REJECT {path.name}: {result['rejected']}", file=sys.stderr)
            rejected += 1
        else:
            (args.out / f"{path.stem}.md").write_text(text, encoding="utf-8")
            accepted += 1
            print(
                f"  {path.stem[:44]:46s} {result['words']:7d}w  "
                f"{result['kept_docs']:3d} docs  glue {result['glue_per_1k']:.2f}/1k"
            )
        if result.get("stale_excludes"):
            # A manifest entry that matches nothing removed nothing, and the
            # section it names is still in the corpus.
            print(
                f"    STALE exclude(s) for {path.stem}: "
                f"{', '.join(result['stale_excludes'])}",
                file=sys.stderr,
            )
        report.append(result)

    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"\n{accepted} book(s) written to {args.out}, {rejected} rejected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
