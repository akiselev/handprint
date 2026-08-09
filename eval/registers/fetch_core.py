#!/usr/bin/env python3
"""Turn the Corpus of Online Registers of English into register corpora.

CORE is 47 human-labelled web registers from a sample predating its 2015
publication, which makes LLM contamination chronologically impossible rather
than merely unlikely. It is the only source in this project that gives a dozen
registers from one download, and it arrives with three defects that a naive
loader passes straight into the references.

**Every document is one paragraph.** One document in 4,845 contains a newline.
The corpus is an HTML-to-plaintext export, and a plaintext export is exactly
what `../corpora/README.md` warns destroys the apparatus signal. Consequences,
recorded here because they are invisible afterwards:

* `dev:device_para_share` and `frame:figure_para_share` measure nothing — the
  whole document is one paragraph, so the share is 1.0 or 0.0.
* `md:link_rate` and `md:footnote_rate` are structurally zero. The `tech-blog`
  claim in `../battery/matrix.py` rests on those two dimensions and **cannot be
  tested on CORE at all**. That register needs a source with its markup intact.

Nothing here can fix that. It is reported per register in the QC file so a
reader knows which dimensions to disbelieve.

**Space before punctuation.** Stripping tags leaves the space that separated
them: `Nothing Lasts Forever , the sequel`. Median 1.07 per 1k words, 20% of
documents above 5, one at 103. The `punct` family is an entire feature family
and this is a typographic artifact of the scrape, not a habit of any writer, so
it is repaired — deterministically, and only where a repair is unambiguous.

**Hybrid documents.** 617 of 4,845 carry three or four labels, meaning the
annotators saw more than one register in one page. Those are dropped. An author
whose corpus spans two registers is a bad target (`../experiments/FINDINGS.md`
#5); a *register* assembled from documents that are two registers is the same
mistake with the same consequence — bands wide enough to admit anything.

    python3 fetch_core.py --out ../corpora --cache /tmp/core
"""

from __future__ import annotations

import argparse
import collections
import gzip
import json
import re
import shutil
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent

BASE = "https://raw.githubusercontent.com/TurkuNLP/CORE-corpus/master"
SPLITS = ("train.tsv.gz", "dev.tsv.gz", "test.tsv.gz")

#: CORE sub-register -> the register corpus it feeds. Several sub-registers map
#: to one corpus where the distinction is finer than anything measured here:
#: `HI` and `HT` are both "instructions for doing a thing", and `IP` and `DS`
#: are both "prose written to sell you something".
#:
#: Registers absent from this map are absent on purpose. `AD` (advertisement),
#: `TS` (technical support) and `TR` (technical report) have 35–42 documents in
#: the whole corpus, which is fewer than Hesse — see FINDINGS #11 for what a
#: thin reference does to every ranking it appears in.
REGISTERS: dict[str, str] = {
    "NE": "news",
    "OB": "opinion-blog",
    "PB": "personal-blog",
    "IB": "info-blog",
    "TB": "personal-blog",
    "DF": "forum",
    "QA": "forum-qa",
    "IP": "marketing",
    "DS": "marketing",
    "HI": "how-to",
    "HT": "how-to",
    "RV": "reviews",
    "RA": "academic",
    "EN": "encyclopedic",
    "LT": "legal-terms",
    "AV": "advice",
    "RE": "recipe",
    "SR": "sports-report",
}

#: Site chrome that survived the scrape. Counted, not stripped: "Click here" is
#: navigation on a news page and an instruction in a how-to, and a rule that
#: cannot tell them apart should not be deleting either. Documents above
#: `--max-chrome` hits are dropped whole.
CHROME = re.compile(
    r"\b(Search in:|Skip to (?:main )?content|Sign in|Log in|Subscribe now|"
    r"Advertisement|Cookie Policy|Privacy Policy|Terms of Use|All rights reserved|"
    r"Follow us on|Share this (?:article|page)|Leave a (?:Reply|Comment)|"
    r"Posted in|Filed under|Tagged with|Related Posts|You may also like)\b",
    re.IGNORECASE,
)

# Repairs. Each is unambiguous: no English orthography puts a space before a
# comma, and none doubles a space inside a sentence.
SPACE_BEFORE_PUNCT = re.compile(r"[ \t]+([,.;:!?%])")
SPACE_INSIDE_QUOTE = re.compile(r"([“‘(\[])[ \t]+")
SPACE_BEFORE_CLOSE = re.compile(r"[ \t]+([”’)\]])")
SPACED_APOSTROPHE = re.compile(r"(\w)[ \t]+([’'])[ \t]*(s|t|re|ve|ll|d|m)\b")
MULTISPACE = re.compile(r"[ \t]{2,}")


def repair(text: str) -> tuple[str, int]:
    """Undo the typography the tag-stripper introduced.

    Returns the text and the number of *substitutions*. Counting changed
    character positions instead — the obvious implementation — reports 33
    million repairs on the news register, because deleting one space shifts
    every character after it and they all compare unequal. A repair count that
    large is not a measurement, it is a diff artifact, and it hides the thing
    the count exists to show: whether a rule fired more than the corpus can
    justify.
    """
    substitutions = 0
    for pattern, replacement in (
        (SPACED_APOSTROPHE, r"\1\2\3"),
        (SPACE_BEFORE_PUNCT, r"\1"),
        (SPACE_INSIDE_QUOTE, r"\1"),
        (SPACE_BEFORE_CLOSE, r"\1"),
        (MULTISPACE, " "),
    ):
        text, count = pattern.subn(replacement, text)
        substitutions += count
    return text.strip(), substitutions


def download(cache: Path) -> list[Path]:
    cache.mkdir(parents=True, exist_ok=True)
    paths = []
    for name in SPLITS:
        target = cache / name
        if not target.exists():
            print(f"fetching {name} …")
            urllib.request.urlretrieve(f"{BASE}/{name}", target)
        paths.append(target)
    return paths


def read(paths: list[Path]):
    for path in paths:
        with gzip.open(path, "rt", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                fields = line.rstrip("\n").split("\t")
                if len(fields) < 3:
                    continue
                yield fields[0].split(), fields[1], "\t".join(fields[2:])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=HERE.parent / "corpora")
    parser.add_argument("--cache", type=Path, default=Path("/tmp/core"))
    parser.add_argument("--report", type=Path, default=HERE.parent / "build/core-qc.json")
    parser.add_argument("--min-words", type=int, default=300,
                        help="below this even the `usable` tier fails for "
                             "char_ngram and device")
    parser.add_argument("--max-chrome", type=int, default=2,
                        help="drop documents with more than this many site-chrome hits")
    parser.add_argument("--max-per-register", type=int, default=1200)
    parser.add_argument("--prefix", default="core-")
    args = parser.parse_args()

    paths = download(args.cache)

    kept: dict[str, list[tuple[str, str]]] = collections.defaultdict(list)
    dropped = collections.Counter()
    repairs = collections.Counter()
    seen_labels = collections.Counter()

    for labels, doc_id, text in read(paths):
        seen_labels[len(labels)] += 1
        # Exactly one top-level class and one sub-register. See the module docs:
        # three or four labels means the annotators saw more than one register
        # on the page, and a register corpus is the wrong home for those.
        if len(labels) != 2:
            dropped["hybrid-or-toplevel-only"] += 1
            continue
        sub = labels[1]
        register = REGISTERS.get(sub)
        if register is None:
            dropped[f"register-not-wanted:{sub}"] += 1
            continue
        chrome = len(CHROME.findall(text))
        if chrome > args.max_chrome:
            dropped[f"chrome:{register}"] += 1
            continue
        body, changed = repair(text)
        if len(body.split()) < args.min_words:
            dropped[f"short:{register}"] += 1
            continue
        repairs[register] += changed
        kept[register].append((doc_id, body))

    args.out.mkdir(parents=True, exist_ok=True)
    summary = {}
    for register, docs in sorted(kept.items()):
        # Deterministic order, then a cap. Sorting by id rather than shuffling
        # keeps a re-run byte-identical, which is what makes a refit comparable
        # to the one before it.
        docs.sort()
        capped = docs[: args.max_per_register]
        directory = args.out / register
        if directory.exists():
            shutil.rmtree(directory)
        directory.mkdir(parents=True)
        words = 0
        for doc_id, body in capped:
            (directory / f"{args.prefix}{doc_id}.md").write_text(
                body + "\n", encoding="utf-8")
            words += len(body.split())
        summary[register] = {
            "documents": len(capped),
            "available": len(docs),
            "words": words,
            "median_words": sorted(len(b.split()) for _, b in capped)[len(capped) // 2],
            "typography_repairs": repairs[register],
            # Every paragraph-shaped and markup-shaped dimension is dead on
            # this source. Recorded per register so it travels with the data.
            "unusable_dimensions": [
                "dev:device_para_share", "frame:figure_para_share",
                "md:link_rate", "md:footnote_rate",
            ],
            "source_diversity": "unknown — CORE ships no URL, so the "
                                "per-domain cap cannot be enforced or verified",
        }
        print(f"{register:16} {len(capped):5} docs  {words:9,} words  "
              f"(of {len(docs)} available, {repairs[register]} repairs)")

    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps({
        "summary": summary,
        "dropped": dict(dropped.most_common()),
        "labels_per_document": dict(sorted(seen_labels.items())),
    }, indent=1), encoding="utf-8")
    print(f"\nwrote {args.report}")
    total = sum(v["documents"] for v in summary.values())
    print(f"{total} documents across {len(summary)} registers")
    thin = [r for r, v in summary.items() if v["documents"] < 100]
    if thin:
        print(f"thin (<100 docs), treat their references with FINDINGS #11 in "
              f"mind: {', '.join(thin)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
