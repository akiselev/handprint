"""Non-circular evaluation for handprint.

Nothing in this package may import or reimplement handprint's own features.
Every judge here is an external model or an external detector; that is the
entire point of the package.
"""

from .loop import CritiqueLoop, LoopResult, Rewriter
from .report import AwayTowards, away_towards, tpr_at_fpr

__all__ = [
    "AwayTowards",
    "CritiqueLoop",
    "LoopResult",
    "Rewriter",
    "away_towards",
    "tpr_at_fpr",
]
