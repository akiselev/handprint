"""Scoring, in spaces handprint knows nothing about."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np


@dataclass
class AwayTowards:
    """How far a rewrite moved in an external embedding space.

    `away` is the cosine distance gained from the source style; `towards` is
    the cosine distance lost to the target style. Reporting both matters:
    a rewrite can move away from the source without moving toward anything,
    which reads as success on a one-sided metric and is not.
    """

    away: float
    towards: float

    @property
    def net(self) -> float:
        return self.towards - self.away


def cosine(a: np.ndarray, b: np.ndarray) -> float:
    denom = float(np.linalg.norm(a) * np.linalg.norm(b))
    return 0.0 if denom == 0.0 else float(np.dot(a, b) / denom)


def away_towards(
    original: np.ndarray,
    rewritten: np.ndarray,
    source_style: np.ndarray,
    target_style: np.ndarray,
) -> AwayTowards:
    """Away/towards scores in the style of STYLL."""
    return AwayTowards(
        away=cosine(original, source_style) - cosine(rewritten, source_style),
        towards=cosine(rewritten, target_style) - cosine(original, target_style),
    )


def tpr_at_fpr(scores_positive: np.ndarray, scores_negative: np.ndarray, fpr: float) -> float:
    """True-positive rate at a fixed false-positive rate.

    Accuracy alone is not reportable for a detector: it hides the operating
    point, and the operating point is the entire question. A detector that is
    95% accurate at a 30% false-positive rate is useless for accusing anyone.
    """
    if len(scores_negative) == 0 or len(scores_positive) == 0:
        return float("nan")
    threshold = float(np.quantile(scores_negative, 1.0 - fpr))
    return float(np.mean(scores_positive >= threshold))
