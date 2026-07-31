"""The headline experiment: does the loop actually work?

Run a manual pilot before trusting anything downstream of it. The experiment is
designed to be *falsifiable* - see eval/README.md for what a failure looks like.
"""

from __future__ import annotations

import argparse
from pathlib import Path

from harness import CritiqueLoop


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--drafts", type=Path, required=True)
    parser.add_argument("--workdir", type=Path, default=Path("/tmp/handprint-eval"))
    parser.add_argument("--max-iterations", type=int, default=10)
    args = parser.parse_args()

    loop = CritiqueLoop(reference=args.reference, max_iterations=args.max_iterations)
    print(f"loop configured against {loop.reference}")
    print(
        "not runnable yet: plug in a Rewriter and the embedding/detector "
        "wrappers from harness.embeddings and harness.detectors"
    )
    _ = args.drafts, args.workdir
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
