#!/usr/bin/env python3
"""Build the software-documentation register from permissively licensed projects.

The register an agent writes in more than any other, and the one no existing
corpus covers: CORE has 42 documents of technical support and 35 of technical
report in its entire 34,000, and its `HI`/`HT` classes are consumer how-tos.

Three things make this corpus different from CORE, and all three are the point:

* **The markup survives.** `md:link_rate` and `md:footnote_rate` are
  structurally zero on CORE because it is an HTML-to-plaintext export. Here the
  markdown is the source, so the apparatus dimensions are measurable — and the
  `tech-blog` claim in `../battery/matrix.py` rests on exactly those two.
* **Diversity is free.** Fifty projects are fifty documentation teams with
  fifty house styles. No sampling frame to defend.
* **The date is exact.** Every project is checked out at its last commit before
  `--before`, so "pre-2022" is a commit hash and a timestamp rather than a
  claim about a dataset.

Code blocks are stripped. A documentation page is half code, and a reference
fitted over unstripped pages measures Python, not the person explaining it.
Inline code and links stay: those are prose apparatus, and deleting them would
remove the signal this corpus exists to supply.

    python3 fetch_docs.py --out ../corpora --cache /tmp/docs-cache
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent

#: (name, repo, licence, doc paths). Licence is the *documentation* licence,
#: which is not always the project's code licence. Share-alike sources are
#: deliberately absent — MDN (CC BY-SA), the Redis docs (CC BY-SA), the
#: Express site (CC BY-SA) and the GNU manuals (GFDL) would each drag a
#: copyleft obligation into a pack that carries other things, which is failure
#: class 3 in `../licenses/check.py`. GPL-documented projects (Ansible, git)
#: are out for the same reason.
PROJECTS: list[tuple[str, str, str, list[str]]] = [
    ("rust-book",      "https://github.com/rust-lang/book",             "MIT OR Apache-2.0", ["src"]),
    ("rust-by-example","https://github.com/rust-lang/rust-by-example",  "MIT OR Apache-2.0", ["src"]),
    ("cargo",          "https://github.com/rust-lang/cargo",            "MIT OR Apache-2.0", ["src/doc/src"]),
    ("rust-reference", "https://github.com/rust-lang/reference",        "MIT OR Apache-2.0", ["src"]),
    ("node",           "https://github.com/nodejs/node",               "MIT",               ["doc/api"]),
    ("rails-guides",   "https://github.com/rails/rails",               "MIT",               ["guides/source"]),
    ("jekyll",         "https://github.com/jekyll/jekyll",             "MIT",               ["docs/_docs"]),
    ("eslint",         "https://github.com/eslint/eslint",             "MIT",               ["docs/user-guide", "docs/developer-guide"]),
    ("prettier",       "https://github.com/prettier/prettier",         "MIT",               ["docs"]),
    ("jest",           "https://github.com/jestjs/jest",               "MIT",               ["docs"]),
    ("babel",          "https://github.com/babel/website",             "MIT",               ["docs"]),
    ("vue-docs",       "https://github.com/vuejs/docs",                "MIT",               ["src/guide"]),
    ("svelte",         "https://github.com/sveltejs/svelte",           "MIT",               ["site/content/docs", "documentation"]),
    ("nextjs",         "https://github.com/vercel/next.js",            "MIT",               ["docs"]),
    ("gatsby",         "https://github.com/gatsbyjs/gatsby",           "MIT",               ["docs/docs"]),
    ("storybook",      "https://github.com/storybookjs/storybook",     "MIT",               ["docs"]),
    ("tailwind",       "https://github.com/tailwindlabs/tailwindcss.com", "MIT",            ["src/pages/docs"]),
    ("fastapi",        "https://github.com/fastapi/fastapi",           "MIT",               ["docs/en/docs"]),
    ("sqlalchemy",     "https://github.com/sqlalchemy/sqlalchemy",     "MIT",               ["doc/build"]),
    ("flask",          "https://github.com/pallets/flask",             "BSD-3-Clause",      ["docs"]),
    ("requests",       "https://github.com/psf/requests",              "Apache-2.0",        ["docs"]),
    ("django",         "https://github.com/django/django",             "BSD-3-Clause",      ["docs"]),
    ("drf",            "https://github.com/encode/django-rest-framework", "BSD-3-Clause",   ["docs"]),
    ("celery",         "https://github.com/celery/celery",             "BSD-3-Clause",      ["docs"]),
    ("scrapy",         "https://github.com/scrapy/scrapy",             "BSD-3-Clause",      ["docs"]),
    ("numpy",          "https://github.com/numpy/numpy",               "BSD-3-Clause",      ["doc/source/user", "doc/source/dev"]),
    ("pandas",         "https://github.com/pandas-dev/pandas",         "BSD-3-Clause",      ["doc/source/user_guide", "doc/source/development"]),
    ("scikit-learn",   "https://github.com/scikit-learn/scikit-learn", "BSD-3-Clause",      ["doc/modules", "doc/developers"]),
    ("scipy",          "https://github.com/scipy/scipy",               "BSD-3-Clause",      ["doc/source/tutorial", "doc/source/dev"]),
    ("matplotlib",     "https://github.com/matplotlib/matplotlib",     "BSD-compatible",    ["doc/users", "doc/devel"]),
    ("jupyter-nb",     "https://github.com/jupyter/notebook",          "BSD-3-Clause",      ["docs/source"]),
    ("cpython",        "https://github.com/python/cpython",            "PSF-2.0",           ["Doc/tutorial", "Doc/howto", "Doc/faq"]),
    ("pytorch",        "https://github.com/pytorch/pytorch",           "BSD-3-Clause",      ["docs/source/notes"]),
    ("tensorflow-docs","https://github.com/tensorflow/docs",           "Apache-2.0",        ["site/en/guide"]),
    ("kubernetes",     "https://github.com/kubernetes/website",        "CC-BY-4.0",         ["content/en/docs/concepts", "content/en/docs/tasks"]),
    ("helm",           "https://github.com/helm/helm-www",             "Apache-2.0",        ["content/en/docs"]),
    ("istio",          "https://github.com/istio/istio.io",            "Apache-2.0",        ["content/en/docs/concepts", "content/en/docs/ops"]),
    ("envoy",          "https://github.com/envoyproxy/envoy",          "Apache-2.0",        ["docs/root/intro", "docs/root/configuration"]),
    ("prometheus",     "https://github.com/prometheus/docs",           "Apache-2.0",        ["content/docs"]),
    ("docker",         "https://github.com/docker/docs",               "Apache-2.0",        ["engine", "get-started"]),
    ("airflow",        "https://github.com/apache/airflow",            "Apache-2.0",        ["docs/apache-airflow"]),
    ("spark",          "https://github.com/apache/spark",              "Apache-2.0",        ["docs"]),
    ("hugo-docs",      "https://github.com/gohugoio/hugoDocs",         "Apache-2.0",        ["content/en/documentation", "content/en/getting-started"]),
    ("typescript",     "https://github.com/microsoft/TypeScript-Website", "Apache-2.0",     ["packages/documentation/copy/en/handbook-v2"]),
    ("llvm",           "https://github.com/llvm/llvm-project",         "Apache-2.0 WITH LLVM-exception", ["llvm/docs"]),
    ("openssl",        "https://github.com/openssl/openssl",           "Apache-2.0",        ["doc/man7"]),
    ("grpc",           "https://github.com/grpc/grpc.io",              "Apache-2.0",        ["content/en/docs"]),
    ("terraform",      "https://github.com/hashicorp/terraform",       "MPL-2.0",           ["website/docs"]),
]

DOC_SUFFIXES = {".md", ".rst", ".markdown", ".mdx", ".txt"}

# YAML / TOML front matter, and Jekyll-style include tags.
FRONT_MATTER = re.compile(r"\A---\n.*?\n---\n", re.S)
TOML_MATTER = re.compile(r"\A\+\+\+\n.*?\n\+\+\+\n", re.S)
LIQUID = re.compile(r"\{%.*?%\}|\{\{.*?\}\}", re.S)
FENCED = re.compile(r"^([ \t]*)(```|~~~).*?^\1\2[ \t]*$", re.S | re.M)
HTML_BLOCK = re.compile(r"<(script|style|table|pre|code)\b.*?</\1>", re.S | re.I)
RST_DIRECTIVE = re.compile(r"^\.\. [a-z-]+::.*?(?=\n\S|\Z)", re.S | re.M)
RST_LITERAL = re.compile(r"::\n\n((?:[ \t]+\S.*\n|\n)+)", re.M)
INDENTED_CODE = re.compile(r"^(?:(?: {4}|\t)\S.*\n|\n)+", re.M)
BADGE = re.compile(r"^\s*\[!\[.*$", re.M)
SETEXT = re.compile(r"^[=~^\-*+#]{4,}\s*$", re.M)


def strip_code(text: str, suffix: str) -> str:
    """Remove code, keep prose apparatus.

    Fenced blocks, HTML blocks, front matter and template tags go. Inline code
    spans and links stay: `md:link_rate` is the reason this corpus exists and
    stripping links to be tidy would delete the measurement.
    """
    text = FRONT_MATTER.sub("", text)
    text = TOML_MATTER.sub("", text)
    text = FENCED.sub("\n", text)
    text = HTML_BLOCK.sub("\n", text)
    text = LIQUID.sub("", text)
    text = BADGE.sub("", text)
    if suffix == ".rst":
        text = RST_DIRECTIVE.sub("\n", text)
        text = RST_LITERAL.sub("\n", text)
        text = SETEXT.sub("", text)
    # Indented code last, and only outside rst, where indentation is structural.
    if suffix != ".rst":
        text = INDENTED_CODE.sub("\n", text)
    return re.sub(r"\n{3,}", "\n\n", text).strip()


def prose_ratio(text: str) -> float:
    """Share of lines that look like sentences rather than markup or lists."""
    lines = [l for l in text.splitlines() if l.strip()]
    if not lines:
        return 0.0
    sentence_like = sum(
        1 for l in lines
        if len(l.split()) >= 8 and not l.lstrip().startswith(("#", "|", "-", "*", ">", ".."))
    )
    return sentence_like / len(lines)


def run(args: list[str], cwd: Path | None = None, timeout: int = 900) -> tuple[int, str]:
    out = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    return out.returncode, (out.stdout + out.stderr)


def checkout(name: str, repo: str, cache: Path, before: str,
             paths: list[str], timeout: int) -> tuple[Path, str, str] | None:
    """Clone blob-less, find the last commit before the cutoff, fetch only docs."""
    work = cache / name
    if not (work / ".git").exists():
        work.parent.mkdir(parents=True, exist_ok=True)
        code, log = run(["git", "clone", "--filter=blob:none", "--no-checkout",
                         "--single-branch", repo, str(work)], timeout=timeout)
        if code != 0:
            print(f"  {name}: clone failed — {log.strip().splitlines()[-1][:100]}")
            shutil.rmtree(work, ignore_errors=True)
            return None
    # `git rev-list --before` is not a UTC comparison. It resolves a bare date
    # in the *local* timezone and compares against the committer date, and the
    # result is that 17 of 44 projects here came back with commits several
    # hours into 2022 while the run reported itself as pre-2022. The cutoff is
    # the whole point of this corpus, so it is verified in epoch seconds, where
    # there is no timezone left to be wrong about.
    cutoff = int(datetime.strptime(before, "%Y-%m-%d")
                 .replace(tzinfo=timezone.utc).timestamp())
    code, out = run(["git", "rev-list", "-1", f"--before={before}", "HEAD"], cwd=work)
    sha = out.strip().splitlines()[-1] if out.strip() else ""
    if code != 0 or not sha:
        print(f"  {name}: no commit before {before}")
        return None
    # Walk back until the committer timestamp is genuinely below the cutoff.
    # A handful of steps at most; the bound stops a pathological history from
    # turning this into a full traversal.
    for _ in range(200):
        code, stamp = run(["git", "show", "-s", "--format=%ct", sha], cwd=work)
        try:
            committed_at = int(stamp.strip().splitlines()[-1])
        except (ValueError, IndexError):
            break
        if committed_at < cutoff:
            break
        code, out = run(["git", "rev-list", "-1", f"{sha}^"], cwd=work)
        parent = out.strip().splitlines()[-1] if out.strip() else ""
        if not parent or parent == sha:
            break
        sha = parent
    else:
        print(f"  {name}: could not reach a commit before {before}")
        return None
    code, when = run(["git", "show", "-s", "--format=%cI", sha], cwd=work)
    # Checking out only the doc paths is what keeps a blob-less clone cheap:
    # nothing else in the repository is ever downloaded.
    code, log = run(["git", "checkout", sha, "--", *paths], cwd=work, timeout=timeout)
    if code != 0:
        code, log = run(["git", "checkout", sha], cwd=work, timeout=timeout)
        if code != 0:
            print(f"  {name}: checkout failed — {log.strip().splitlines()[-1][:100]}")
            return None
    return work, sha, when.strip().splitlines()[-1] if when.strip() else "?"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=HERE.parent / "corpora")
    parser.add_argument("--cache", type=Path, default=Path("/tmp/docs-cache"))
    parser.add_argument("--report", type=Path, default=HERE.parent / "build/docs-qc.json")
    parser.add_argument("--before", default="2022-01-01",
                        help="checkout the last commit strictly before this date")
    parser.add_argument("--min-words", type=int, default=300)
    parser.add_argument("--max-words", type=int, default=6000)
    parser.add_argument("--min-prose", type=float, default=0.35,
                        help="minimum share of sentence-like lines; below this "
                             "the page is a table, a changelog or an API stub")
    parser.add_argument("--per-project", type=int, default=30,
                        help="cap per project so no one doc team dominates")
    parser.add_argument("--only", nargs="*", help="restrict to these project names")
    parser.add_argument("--timeout", type=int, default=900)
    args = parser.parse_args()

    projects = [p for p in PROJECTS if not args.only or p[0] in args.only]
    target = args.out / "software-docs"
    target.mkdir(parents=True, exist_ok=True)
    for stale in target.glob("*.md"):
        stale.unlink()

    manifest = []
    total_docs = total_words = 0
    for name, repo, licence, paths in projects:
        print(f"{name} …")
        got = checkout(name, repo, args.cache, args.before, paths, args.timeout)
        if got is None:
            manifest.append({"project": name, "status": "failed"})
            continue
        work, sha, when = got

        candidates = []
        for relative in paths:
            root = work / relative
            if not root.exists():
                continue
            for path in sorted(root.rglob("*")):
                if path.is_file() and path.suffix.lower() in DOC_SUFFIXES:
                    candidates.append(path)
        kept = []
        for path in candidates:
            try:
                raw = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            body = strip_code(raw, path.suffix.lower())
            words = len(body.split())
            if words < args.min_words or words > args.max_words:
                continue
            if prose_ratio(body) < args.min_prose:
                continue
            kept.append((path.relative_to(work), body, words))
        # Deterministic: sorted, then capped. A shuffle would make two runs of
        # the same commit produce different corpora.
        kept.sort(key=lambda k: str(k[0]))
        kept = kept[: args.per_project]
        for relative, body, words in kept:
            slug = re.sub(r"[^a-z0-9]+", "-", str(relative).lower()).strip("-")
            (target / f"docs-{name}-{slug}.md").write_text(body + "\n", encoding="utf-8")
            total_words += words
        total_docs += len(kept)
        code, epoch = run(["git", "show", "-s", "--format=%ct", sha], cwd=work)
        manifest.append({
            "project": name, "repo": repo, "licence": licence,
            "commit": sha, "committed": when,
            "committed_epoch": int(epoch.strip().splitlines()[-1]) if epoch.strip() else None,
            "candidates": len(candidates), "kept": len(kept),
            "words": sum(w for _, _, w in kept),
            "status": "ok" if kept else "no-usable-pages",
        })
        print(f"  {sha[:9]} {when[:10]}  {len(kept):3}/{len(candidates)} pages, "
              f"{sum(w for _, _, w in kept):,} words  [{licence}]")

    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps({
        "cutoff": args.before, "projects": manifest,
        "documents": total_docs, "words": total_words,
    }, indent=1), encoding="utf-8")

    ok = [m for m in manifest if m.get("status") == "ok"]
    print(f"\n{total_docs} documents, {total_words:,} words from {len(ok)} projects")
    if ok:
        biggest = max(ok, key=lambda m: m["kept"])
        print(f"largest single project: {biggest['project']} "
              f"({100 * biggest['kept'] / total_docs:.1f}% of documents)")
    failed = [m["project"] for m in manifest if m.get("status") != "ok"]
    if failed:
        print(f"no usable pages from: {', '.join(failed)}")
    print(f"wrote {args.report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
