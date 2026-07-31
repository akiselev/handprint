"""Thin subprocess wrapper around the `handprint` binary.

The battery drives the shipped CLI rather than linking the library, for the
same reason the eval harness uses external judges: a battery that called into
the crate would be testing the crate's own view of itself, and the thing being
checked is what a user gets.
"""

from __future__ import annotations

import json
import shlex
import subprocess
from dataclasses import dataclass
from pathlib import Path


class HandprintError(RuntimeError):
    """A `handprint` invocation failed in a way the battery cannot continue past."""


@dataclass
class Handprint:
    """A configured `handprint` command line."""

    command: str = "handprint"
    cwd: Path | None = None

    def run(self, *args: str, allow: tuple[int, ...] = (0,)) -> str:
        argv = shlex.split(self.command) + list(args)
        result = subprocess.run(
            argv,
            cwd=self.cwd,
            capture_output=True,
            text=True,
        )
        if result.returncode not in allow:
            raise HandprintError(
                f"{' '.join(argv)}\nexit {result.returncode}\n{result.stderr}"
            )
        return result.stdout

    # -- fitting ----------------------------------------------------------

    def fit(
        self,
        corpus: Path,
        out: Path,
        *,
        name: str,
        features: list[str],
        version: str = "battery",
        extra: list[str] | None = None,
    ) -> Path:
        args = [
            "fit",
            str(corpus),
            "-o",
            str(out),
            "--no-config",
            "--name",
            name,
            "--version",
            version,
            "--features",
            ",".join(features),
        ]
        self.run(*args, *(extra or []))
        return out

    def calibrate(
        self, reference: Path, background: Path, *, metric: str = "burrows"
    ) -> Path:
        self.run(
            "calibrate",
            str(reference),
            "--background",
            str(background),
            "--metric",
            metric,
        )
        return reference

    def contrast_vocab(
        self,
        a: Path,
        b: Path,
        out: Path,
        *,
        z_threshold: float = 2.0,
        min_count: int = 5,
    ) -> Path:
        """Derive a signature vocabulary: `contrast a b --out`.

        This is the mechanism the plan insists on — an author's signature lexis
        is *derived*, and the hand-curated list is a golden test rather than the
        input.
        """
        self.run(
            "contrast",
            str(a),
            str(b),
            "--min-count",
            str(min_count),
            "--z-threshold",
            str(z_threshold),
            "--out",
            str(out),
        )
        return out

    # -- measuring --------------------------------------------------------

    def critique(self, reference: Path, draft: Path, *, max_findings: int = 200) -> dict:
        # Exit 1 means "findings", which is the normal case here, and exit 2
        # means "gate unevaluated", which the caller inspects rather than
        # crashing on.
        out = self.run(
            "critique",
            "-r",
            str(reference),
            str(draft),
            "--max-findings",
            str(max_findings),
            allow=(0, 1, 2),
        )
        return json.loads(out)

    def profile(self, reference: Path, doc: Path) -> dict:
        return json.loads(self.run("profile", "-r", str(reference), str(doc), "--json"))

    def compare(self, reference: Path, a: Path, b: Path, *, metric: str = "burrows") -> dict:
        return json.loads(
            self.run(
                "compare",
                "-r",
                str(reference),
                str(a),
                str(b),
                "--metric",
                metric,
                "--json",
            )
        )

    def rank(self, reference: Path, query: Path, candidates: Path, *, metric: str = "burrows") -> list[dict]:
        return json.loads(
            self.run(
                "rank",
                "-r",
                str(reference),
                str(query),
                str(candidates),
                "--metric",
                metric,
                "--json",
            )
        )

    def verify(
        self,
        reference: Path,
        query: Path,
        target: Path,
        impostors: Path,
        *,
        iterations: int = 100,
    ) -> dict:
        return json.loads(
            self.run(
                "verify",
                "-r",
                str(reference),
                str(query),
                "--target",
                str(target),
                "--impostors",
                str(impostors),
                "--iterations",
                str(iterations),
                "--json",
            )
        )


def parse_profile_table(text: str) -> list[tuple[str, float]]:
    """Read `handprint profile`'s "most unusual dimensions" table.

    The table is what a person looks at, and the interpretability suite asserts
    on the same thing rather than on a private re-derivation of z-scores. Rows
    are `dimension observed corpus z`.
    """
    rows: list[tuple[str, float]] = []
    started = False
    for line in text.splitlines():
        if line.startswith("most unusual dimensions"):
            started = True
            continue
        if not started:
            continue
        parts = line.split()
        if len(parts) != 4 or parts[0] == "dimension":
            continue
        try:
            rows.append((parts[0], float(parts[3])))
        except ValueError:
            continue
    return rows
