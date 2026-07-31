"""External embedding spaces. None of these is a handprint feature."""

from __future__ import annotations

from typing import Protocol

import numpy as np


class Embedder(Protocol):
    """Encodes text into a fixed-width vector."""

    def encode(self, texts: list[str]) -> np.ndarray: ...


LUAR = "rrivera1849/LUAR-MUD"
"""Authorship representation. Answers 'who wrote this'."""

STYLE_EMBEDDING = "AnnaWegmann/Style-Embedding"
"""Style-only representation, trained to be content-invariant."""

STYLE_DISTANCE = "StyleDistance/styledistance"
"""Style-only representation with explicit stylistic-feature supervision."""

SBERT = "sentence-transformers/all-mpnet-base-v2"
"""Meaning. Used as a *gate*: a rewrite that drops below the similarity floor
is rejected regardless of how well it scores on style."""


def load(name: str) -> Embedder:
    """Load an embedder by model name.

    Not implemented: the harness is scaffolding, and no number in the
    documentation comes from this directory.
    """
    raise NotImplementedError(
        f"model wrappers are not implemented yet (requested {name!r}); "
        "see eval/README.md"
    )
