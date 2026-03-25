"""
Random GraphSpec generation with variable filter chains.

Extends the existing ``rustic_ml.data.generation.random_spec()`` to include a
randomly sampled filter chain.  Each filter's parameters are sampled uniformly
within the ranges defined by the Vocabulary (which in turn comes from
``available_filters()`` metadata).
"""
from __future__ import annotations

import numpy as np

from rustic_ml.data.generation import (
    random_spec as _random_source_spec,
    ADSR_MIN,
    ADSR_MAX,
)
from .vocab import Vocabulary


def random_ar_spec(
    vocab: Vocabulary,
    max_filters: int = 3,
    waveform: str | None = None,
) -> dict:
    """Generate a random GraphSpec dict with a variable-length filter chain.

    Starts from the existing ``random_spec()`` (source + timing) and appends
    0 to ``max_filters`` randomly chosen filters, each with parameters sampled
    within their metadata-defined ranges.

    Args:
        vocab:       Vocabulary instance (used for filter token list and ranges).
        max_filters: Maximum number of filters to append (inclusive).
        waveform:    Fix the waveform type, or None for random.

    Returns:
        A dict compatible with ``rustic_py.render()``.
    """
    spec = _random_source_spec(waveform=waveform)

    filter_token_ids = [
        tid for tid, name in vocab.id_to_token.items()
        if name.startswith("filter:")
    ]

    n_filters = int(np.random.randint(0, max_filters + 1))
    filters = []

    for _ in range(n_filters):
        flt_tok_id = filter_token_ids[int(np.random.randint(0, len(filter_token_ids)))]
        layout = vocab.param_layout.get(flt_tok_id, [])
        params: dict[str, float] = {}

        for fname in layout:
            lo, hi = vocab.param_ranges.get(fname, (0.0, 1.0))
            params[fname] = float(np.random.uniform(lo, hi))

        filters.append({
            "type": vocab.filter_type_id(flt_tok_id),
            "params": params,
        })

    spec["filters"] = filters
    return spec
