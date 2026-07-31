#!/usr/bin/env bash
# Regenerate the capstone from Gutenberg IDs and seeds.
#
# Everything before the transfers themselves is deterministic and needs no
# model: fetch, strip, chapterize, fit, calibrate. The transfers need an agent
# and are driven by `transfer.py`, which takes the model as an argument.
#
# A number produced by a run whose manifest is missing is not reproducible and
# should not be quoted. The manifest is written first, on purpose.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD="${BUILD:-$ROOT/build/voice-transfer}"
RAW="$BUILD/raw"
CORPORA="$BUILD/corpora"
HANDPRINT="${HANDPRINT:-cargo run --quiet --manifest-path $ROOT/../Cargo.toml --release --bin handprint --}"
SEED="${SEED:-20260801}"

mkdir -p "$RAW" "$CORPORA" "$BUILD/refs" "$BUILD/artifacts"

# Gutenberg ids. Kept here rather than in a config file so that the corpus a
# run used is visible in the same diff as the code that used it.
declare -A BOOKS=(
  [wodehouse]="8164 7471 2233 6753"          # My Man Jeeves, Right Ho Jeeves, ...
  [twain]="3178 74 76 245 119"               # Jumping Frog, Tom Sawyer, Huck Finn, ...
  [poe]="2147 2148 2149 2150 25525"          # Works, vols I-IV
  [austen]="1342 161 141 158 121"            # Pride and Prejudice, Sense and Sensibility, ...
)
# Hemingway's US-PD works are not all on Gutenberg; place them in
# raw/hemingway-early/ by hand. The script does not pretend otherwise.

fetch() {
  local author="$1" id="$2"
  local out="$RAW/$author/pg$id.txt"
  [ -f "$out" ] && return 0
  mkdir -p "$RAW/$author"
  echo "fetching $author/$id"
  curl -fsSL "https://www.gutenberg.org/cache/epub/$id/pg$id.txt" -o "$out" \
    || curl -fsSL "https://www.gutenberg.org/files/$id/$id-0.txt" -o "$out"
}

echo "== fetching"
for author in "${!BOOKS[@]}"; do
  for id in ${BOOKS[$author]}; do fetch "$author" "$id"; done
done
if [ ! -d "$RAW/hemingway-early" ]; then
  echo "note: raw/hemingway-early is absent. Its US-PD works are not all on"
  echo "      Gutenberg; place them there by hand. T1 and T2 need it."
fi

echo "== stripping and chapterizing"
$HANDPRINT gutenberg --in "$RAW" --out "$CORPORA" --jsonl "$BUILD/gutenberg.jsonl"

echo "== fitting and calibrating"
FEATURES="punct,sentence-rhythm,punchline,biber,readability,frames,formality,devices,syntax,ngrams"
for author_dir in "$CORPORA"/*/; do
  author="$(basename "$author_dir")"
  ref="$BUILD/refs/$author.json"
  # Signature lexis is derived, never hand-listed: contrast this author
  # against every other, and feed the result back in as a feature.
  vocab="$BUILD/refs/$author.vocab.json"
  $HANDPRINT contrast "$author_dir" "$CORPORA" --min-count 5 --z-threshold 2.0 \
    --out "$vocab" >/dev/null
  $HANDPRINT fit "$author_dir" -o "$ref" --no-config \
    --name "$author" --version "capstone" --features "$FEATURES" --vocab "$vocab"
  $HANDPRINT calibrate "$ref" --background "$CORPORA" --metric burrows
  echo "  $author -> $ref"
done

echo "== manifest"
# Written before the transfers run, not after: a number whose manifest is
# missing is not reproducible and should not be quoted.
HANDPRINT_VERSION="$($HANDPRINT --version 2>/dev/null | tail -1)"
python3 "$ROOT/experiments/voice-transfer/manifest.py" \
  --out "$BUILD/manifest.json" \
  --seed "$SEED" \
  --features "$FEATURES" \
  --handprint "$HANDPRINT_VERSION"

echo
echo "References are ready. The transfers need an agent:"
echo "  python3 $ROOT/experiments/voice-transfer/transfer.py \\"
echo "      --build $BUILD --transfer T1 --arm c"
echo
echo "Reminder: Wodehouse and Hemingway artifacts that carry text stay local."
