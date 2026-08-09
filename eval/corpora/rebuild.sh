#!/usr/bin/env bash
# Rebuild every corpus from raw/ and the local EPUBs, in parallel.
#
# Idempotent by construction: each author directory is removed before it is
# written, so a re-run cannot leave documents from a previous rule set lying
# next to documents from this one. That failure is invisible afterwards — the
# stale files look exactly like corpus.
set -uo pipefail
cd "$(dirname "$0")/.."   # eval/

HANDPRINT=${HANDPRINT:-../target/release/handprint}
CORPORA=${CORPORA:-corpora}
BUILD=${BUILD:-build}
JOBS=${JOBS:-8}
LOG=$BUILD/rebuild
mkdir -p "$LOG"

echo "== clearing previous corpora =="
find "$CORPORA" -mindepth 1 -maxdepth 1 -type d ! -name '__pycache__' -exec rm -rf {} +

# ---------------------------------------------------------------- Gutenberg
# One pass over raw/, which is already laid out as raw/{author}/*.txt. The
# crate strips licence boilerplate, transcriber's notes and the older sign-off
# line, then chapterizes; it reports every editorial block it removed.
echo "== gutenberg =="
$HANDPRINT gutenberg --in raw --out "$CORPORA" --jsonl "$BUILD/gutenberg.jsonl" \
    > "$LOG/gutenberg.log" 2>&1
gutenberg_status=$?
grep -E "^(removed|warning)" -A 40 "$LOG/gutenberg.log" | head -40

# Books whose chapter markers the crate did not recognise come out as one
# 100k-word document. Re-cut every Gutenberg author to a uniform size so
# document length is not confounded with author, and let the drop-below and
# dedupe rules run over the result.
echo "== re-cutting gutenberg authors to 2500w =="
resize_one() {
    author=$1
    dir="$2/$author"
    tmp=$(mktemp -d)
    mv "$dir"/*.md "$tmp"/ 2>/dev/null || return 0
    python3 corpora/chapterize.py "$tmp"/*.md --author "$author" --out "$2" \
        --target-words 2500 --min-words 0 > "$3/resize-$author.log" 2>&1
    rm -rf "$tmp"
}
export -f resize_one
find "$CORPORA" -mindepth 1 -maxdepth 1 -type d ! -name '__pycache__' \
    -printf '%f\n' \
    | xargs -P "$JOBS" -I{} bash -c 'resize_one "$@"' _ {} "$CORPORA" "$LOG"

# ---------------------------------------------------------------- EPUB
# In-copyright, local-only, and never committed. Each author is one job.
echo "== epubs =="
epub_one() {
    author=$1
    md=$(mktemp -d)
    # Recursive: the Pratchett collection keeps the Tiffany Aching books in a
    # `ya/` subdirectory, and a non-recursive glob silently loses seven books —
    # a fifth of the corpus, and the fifth that contains the duplicated
    # preview chapters the dedupe rule exists for.
    mapfile -t epubs < <(find ../corpus/"$author" -name '*.epub' | sort)
    python3 corpora/epub2md.py "${epubs[@]}" --out "$md" \
        --exclude corpora/exclude.json --report "$3/qc-$author.json" \
        > "$3/epub-$author.log" 2>&1
    shopt -s nullglob
    files=("$md"/*.md)
    if [ ${#files[@]} -gt 0 ]; then
        python3 corpora/chapterize.py "${files[@]}" --author "$author" \
            --out "$2" --target-words 2500 >> "$3/epub-$author.log" 2>&1
    fi
    rm -rf "$md"
}
export -f epub_one
ls ../corpus | xargs -P "$JOBS" -I{} bash -c 'epub_one "$@"' _ {} "$CORPORA" "$LOG"

echo
echo "== result =="
for d in "$CORPORA"/*/; do
    [ "$(basename "$d")" = "__pycache__" ] && continue
    printf '%-24s %4s docs %10s words\n' "$(basename "$d")" \
        "$(ls "$d" | wc -l)" "$(cat "$d"/*.md 2>/dev/null | wc -w)"
done
exit $gutenberg_status
