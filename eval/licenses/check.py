#!/usr/bin/env python3
"""License hygiene: assert that loader-only and local-only resources never ship.

Runs in CI and needs no corpora. Three classes of failure, all of which look
identical from inside the repository and completely different from outside it:

1. A **loader-only** table (NRC EmoLex, Warriner VAD, Brysbaert concreteness,
   Pavlick-Tetreault formality) compiled into a shipped pack. handprint ships
   the feature and the loader; the user supplies those tables.
2. A **local-only** corpus (Folger Shakespeare TEI, or any in-copyright author)
   committed under `eval/corpora/`.
3. A **share-alike** derivation whose obligation has leaked into a sibling
   pack. CC-BY-SA attaches to the Wikipedia signs-of-AI list and to nothing
   else in the binary.

The check is a grep, deliberately. A subtler test would be easier to satisfy
accidentally, and the thing being defended is a legal claim rather than a
behaviour.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# Markers that must never appear in a *shipped* pack: a bundled table drawn
# from one of these resources. The names are matched case-insensitively inside
# the pack data, not in prose — a doc comment that says "NOT bundled" is the
# point, so only pack contents are searched.
LOADER_ONLY = {
    "nrc": r"\bnrc[- ]?(emolex|vad)\b",
    "warriner": r"\bwarriner\b",
    "brysbaert": r"\bbrysbaert\b",
    "pavlick": r"\bpavlick\b",
    "folger": r"\bfolger\b",
    "sensorimotor": r"\blancaster sensorimotor\b",
}

# Corpora that may exist on a machine and must never be in git.
LOCAL_ONLY_DIRS = [
    "douglas-adams",
    "terry-pratchett",
    "charles-bukowski",
    "hunter-s-thompson",
    "folger",
    "shakespeare-folger",
]

# Where bundled pack *data* lives. Prose elsewhere may name these resources
# freely — saying "we do not ship Brysbaert" is the documentation working.
PACK_DIR = REPO / "crates" / "handprint-core" / "src" / "feature" / "packs"


TABLE_START = re.compile(r"^\s*(?:pub\s+)?const\s+[A-Z_0-9]+\s*:\s*&\[[^\]]*\]\s*=\s*&\[\s*$")


def pack_data_lines(path: Path) -> list[tuple[int, str]]:
    """The rows of a pack's data tables.

    Only the tables. Provenance and documentation *must* name these resources —
    "Brysbaert, Warriner & Kuperman (NOT bundled)" in a `sources` entry is the
    licensing discipline working, and a checker that flagged it would push
    people toward deleting the attribution rather than the data. What must
    never appear is a resource's *contents*, which live in the `const` tables.
    """
    out: list[tuple[int, str]] = []
    inside = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        stripped = line.strip()
        if TABLE_START.match(line):
            inside = True
            continue
        if inside and stripped == "];":
            inside = False
            continue
        if not inside or stripped.startswith("//"):
            continue
        out.append((number, line))
    return out


def check_packs() -> list[str]:
    failures: list[str] = []
    for path in sorted(PACK_DIR.rglob("*.rs")):
        for number, line in pack_data_lines(path):
            for name, pattern in LOADER_ONLY.items():
                if re.search(pattern, line, re.IGNORECASE):
                    failures.append(
                        f"{path.relative_to(REPO)}:{number}: bundled pack data mentions "
                        f"the loader-only resource {name!r}: {line.strip()[:80]}"
                    )
    return failures


def check_tracked_corpora() -> list[str]:
    """No corpus text may be tracked by git, whatever its licence."""
    try:
        tracked = subprocess.run(
            ["git", "ls-files"],
            cwd=REPO,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.splitlines()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ["could not run `git ls-files`; corpus check skipped"]

    # The scaffolding that tells people how to build a corpus is not a corpus.
    # Listed explicitly rather than matched by extension, so that adding a
    # `.md` under `eval/corpora/` is a decision someone makes here.
    ALLOWED = {
        "eval/corpora/README.md",
        "eval/corpora/chapterize.py",
        "eval/corpora/epub2md.py",
        "eval/corpora/fetch_gutenberg.py",
        # Names the *sections* of in-copyright books that a different person
        # wrote, so they can be dropped. It carries no corpus text.
        "eval/corpora/exclude.json",
    }

    failures = []
    for path in tracked:
        if path in ALLOWED:
            continue
        lowered = path.lower()
        if lowered.startswith("eval/corpora/") or lowered.startswith("corpus/"):
            failures.append(f"{path}: corpus content must not be tracked")
        for name in LOCAL_ONLY_DIRS:
            if f"/{name}/" in f"/{lowered}":
                failures.append(f"{path}: local-only corpus {name!r} is tracked")
    return failures


def check_share_alike() -> list[str]:
    """CC-BY-SA must attach to exactly one pack file."""
    failures = []
    sa_files = []
    for path in sorted(PACK_DIR.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        if 'license: "CC-BY-SA' in text or "CC-BY-SA-4.0" in text:
            sa_files.append(path.name)
    expected = {"misc.rs"}
    unexpected = set(sa_files) - expected
    if unexpected:
        failures.append(
            "share-alike licence declared outside the isolated derivation: "
            + ", ".join(sorted(unexpected))
        )
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verbose", action="store_true", help="list what was checked"
    )
    args = parser.parse_args()

    failures = check_packs() + check_tracked_corpora() + check_share_alike()

    if args.verbose:
        print(f"checked {len(list(PACK_DIR.rglob('*.rs')))} pack file(s)")
        print(f"loader-only resources: {', '.join(sorted(LOADER_ONLY))}")

    if failures:
        print("license hygiene FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print("license hygiene OK: no loader-only table bundled, no corpus tracked, "
          "share-alike isolated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
