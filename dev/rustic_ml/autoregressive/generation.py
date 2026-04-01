"""
Random GraphSpec generation for the autoregressive model.

Uses GraphSpec.random() from rustic_py to generate full hierarchical specs
(multi-source, filters, DAG connections) compatible with the new token vocabulary.
"""
from __future__ import annotations

import numpy as np

from rustic_py import GraphSpec
from .vocab import Vocabulary


def random_ar_spec(
    vocab: Vocabulary,
    complexity: float | None = None,
    max_filters: int | None = None,
    waveform: str | None = None,
) -> dict:
    """Generate a random GraphSpec dict compatible with the new token vocabulary.

    Args:
        vocab:       Vocabulary instance (unused but kept for API compatibility).
        complexity:  Graph complexity in [0.0, 1.0]. If None, samples uniformly.
        max_filters: Ignored — filter count is controlled by complexity.
                     Kept for backwards compatibility with ARDataset callers.
        waveform:    Ignored — waveform is chosen randomly by GraphSpec.random().
                     Kept for backwards compatibility with ARDataset callers.

    Returns:
        A dict from GraphSpec.to_spec(), compatible with spec_to_sequence().
    """
    if complexity is None:
        complexity = float(np.random.uniform(0.0, 0.5))
    return GraphSpec.random(complexity=complexity).to_spec()
