"""External AI-text detectors."""

from __future__ import annotations

from typing import Protocol

import numpy as np


class Detector(Protocol):
    """Scores text for machine-generation. Higher means more machine-like."""

    def score(self, texts: list[str]) -> np.ndarray: ...


def binoculars() -> Detector:
    """Binoculars (zero-shot, two-model perplexity ratio).

    Chosen because it is zero-shot and reproducible. Supervised detectors score
    better but are opaque and cannot be re-run independently, which defeats the
    purpose of an external judge.

    Not implemented: see eval/README.md.
    """
    raise NotImplementedError("Binoculars wrapper is not implemented yet")
