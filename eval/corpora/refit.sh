#!/usr/bin/env bash
# Fit and calibrate one reference per author, in parallel.
#
# The feature set is the matrix's: no `contrast` family, because it reports
# topic rather than style (eval/experiments/adams-pratchett/RESULTS.md), and
# every reference is calibrated against the *same* background so cells are
# comparable to each other.
set -uo pipefail
cd "$(dirname "$0")/.."   # eval/

HANDPRINT=${HANDPRINT:-../target/release/handprint}
CORPORA=${CORPORA:-corpora}
BUILD=${BUILD:-build}
JOBS=${JOBS:-6}
MIN_DOCS=${MIN_DOCS:-15}
BG_DOCS=${BG_DOCS:-40}

# Authors and registers are fitted separately, against separate backgrounds.
# A background is what "unusual" is measured against, so an author must be
# unusual relative to other authors and a register relative to other
# registers. Pooling them makes every author's calibration depend on how many
# web registers happen to be on disk, which is not a property of the author.
KIND=${1:-authors}
REGISTER_DIRS="news opinion-blog personal-blog info-blog forum forum-qa \
marketing how-to reviews academic encyclopedic legal-terms advice recipe \
sports-report tech-blog agent-prose software-docs"

case "$KIND" in
    authors)   OUT=${OUT:-$BUILD/refs/v3};      BG=${BG:-$BUILD/background-v3} ;;
    registers) OUT=${OUT:-$BUILD/refs/register}; BG=${BG:-$BUILD/background-register} ;;
    *) echo "usage: refit.sh [authors|registers]"; exit 2 ;;
esac

# Authors and registers want different packs. The register set adds three
# the author set has no use for, and one of them is the reason a blind judge
# beat the critic on the de-AI test: `doc-style` carries the weasel words,
# wordy phrases and clichés pack, which is the closest thing in the crate to
# the tells the judge actually named. `hyland` supplies the hedge and booster
# categories the register matrix's claims are written in terms of, and
# `marketing` the pressure and directive categories. Omitting them made three
# of the battery's own claims untestable — see FINDINGS #21 and #28.
BASE_FEATURES=punct,sentence-rhythm,punchline,biber,readability,frames,formality,devices,syntax,mfw,ngrams
case "$KIND" in
    registers) FEATURES=$BASE_FEATURES,doc-style,hyland,marketing ;;
    *)         FEATURES=$BASE_FEATURES ;;
esac
export KIND REGISTER_DIRS

mkdir -p "$OUT" "$BUILD/refit-logs"

# ------------------------------------------------------------------ background
# An author's "unusual" dimensions are only unusual relative to a population.
# Sampling a fixed number of documents per author keeps Pratchett's 1,475 from
# outvoting Machiavelli's 18 in what counts as normal.
echo "== $KIND background: $BG_DOCS documents each =="
rm -rf "$BG"
python3 - "$CORPORA" "$BG" "$BG_DOCS" "$MIN_DOCS" "$KIND" "$REGISTER_DIRS" <<'PY'
import random, shutil, sys
from pathlib import Path
corpora, bg, per, floor = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
kind, registers = sys.argv[5], set(sys.argv[6].split())
for entry in sorted(p for p in corpora.iterdir() if p.is_dir() and p.name != "__pycache__"):
    if (kind == "registers") != (entry.name in registers):
        continue
    docs = sorted(entry.glob("*.md"))
    if len(docs) < floor:
        continue
    # Seeded by name so the background is the same on every re-run: a
    # calibration that moves when nothing changed is not a calibration.
    random.Random(f"background:{entry.name}").shuffle(docs)
    target = bg / entry.name
    target.mkdir(parents=True, exist_ok=True)
    for doc in docs[:per]:
        shutil.copy2(doc, target / doc.name)
print(f"{len(list(bg.iterdir()))} background entries for {kind}")
PY

# ------------------------------------------------------------------ references
fit_one() {
    author=$1; corpora=$2; out=$3; bg=$4; features=$5; logs=$6
    src=$corpora/$author
    log=$logs/$author.log
    # `--metric burrows` explicitly. Burrows's Delta is the right choice for
    # one-class membership, which is the question a reference answers, and it
    # is what the calibration below is fitted for. Leaving the default in
    # place produces a reference whose stated metric and whose calibration
    # disagree, which nothing warns about.
    if ! $HANDPRINT fit "$src" -o "$out/$author.json" --no-config \
            --name "$author" --version "v3-$KIND" --metric burrows \
            --features "$features" > "$log" 2>&1; then
        echo "FIT FAILED $author"; return 1
    fi
    if ! $HANDPRINT calibrate "$out/$author.json" --background "$bg" \
            --metric burrows >> "$log" 2>&1; then
        echo "CALIBRATE FAILED $author"; return 1
    fi
    echo "ok $author"
}
export -f fit_one
export HANDPRINT KIND

authors=$(python3 - "$CORPORA" "$MIN_DOCS" "$KIND" "$REGISTER_DIRS" <<'PY'
import sys
from pathlib import Path
corpora, floor = Path(sys.argv[1]), int(sys.argv[2])
kind, registers = sys.argv[3], set(sys.argv[4].split())
for a in sorted(p for p in corpora.iterdir() if p.is_dir() and p.name != "__pycache__"):
    if (kind == "registers") != (a.name in registers):
        continue
    if len(list(a.glob("*.md"))) >= floor:
        print(a.name)
PY
)
count=$(echo "$authors" | wc -l)
echo "== fitting $count references, $JOBS at a time =="
echo "$authors" | xargs -P "$JOBS" -I{} bash -c \
    'fit_one "$@"' _ {} "$CORPORA" "$OUT" "$BG" "$FEATURES" "$BUILD/refit-logs"

echo
echo "== calibrated references =="
for f in "$OUT"/*.json; do
    printf '%-34s %s\n' "$(basename "$f" .json)" \
        "$($HANDPRINT pack show "$f" 2>/dev/null | grep -c 'calibrated for')"
done
