"""Drive `handprint critique --json` against a rewriter."""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Protocol


class Rewriter(Protocol):
    """Anything that can rewrite a draft given a critique report.

    The harness never assumes this is a model. A template rewriter is a valid
    implementation and is what the integration tests use.
    """

    def rewrite(self, draft: str, report: dict) -> str: ...


@dataclass
class LoopResult:
    """One run of the loop."""

    original: str
    final: str
    reports: list[dict] = field(default_factory=list)

    @property
    def iterations(self) -> int:
        return len(self.reports)

    @property
    def passed(self) -> bool:
        return bool(self.reports) and self.reports[-1]["verdict"]["pass"] is True

    @property
    def canary_ok(self) -> bool:
        """False if the canary tripped at any point.

        A run that passes with a tripped canary is a *failure* of the
        experiment, not a success: it means the rewriter optimized the reported
        features rather than the style.
        """
        return all(r["guards"]["canary_ok"] for r in self.reports)

    @property
    def drift_ok(self) -> bool:
        return all(r["guards"]["drift_ok"] for r in self.reports)


@dataclass
class CritiqueLoop:
    """Runs the handprint binary in a loop."""

    reference: Path
    binary: str = "handprint"
    max_iterations: int = 10
    extra_args: list[str] = field(default_factory=list)

    def critique(self, draft: str, workdir: Path) -> dict:
        path = workdir / "draft.txt"
        path.write_text(draft, encoding="utf-8")
        proc = subprocess.run(
            [self.binary, "critique", "-r", str(self.reference), str(path), *self.extra_args],
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode == 2:
            raise RuntimeError(f"handprint critique errored: {proc.stderr.strip()}")
        return json.loads(proc.stdout)

    def run(self, draft: str, rewriter: Rewriter, workdir: Path) -> LoopResult:
        workdir.mkdir(parents=True, exist_ok=True)
        result = LoopResult(original=draft, final=draft)
        for _ in range(self.max_iterations):
            report = self.critique(result.final, workdir)
            result.reports.append(report)
            if report["verdict"]["pass"] is True:
                break
            rewritten = rewriter.rewrite(result.final, report)
            if rewritten == result.final:
                break
            result.final = rewritten
        return result
