"""
Spec ↔ token-sequence serialisation.

A GraphSpec dict is encoded as a flat integer token sequence paired with a
values matrix of shape (seq_len, values_width).  Only positions that correspond
to a <VALUES> token carry non-zero values; all other rows are zero.

Sequence structure (linear chain, left-to-right):
    SOS
    NOTE  →  <VALUES>  [note, note_on, note_off, 0…]
    <waveform_type>  →  <VALUES>  [param_0, param_1, …, 0…]
    [<filter_type>  →  <VALUES>  [param_0, …, 0…]] × n_filters
    EOS
"""
from __future__ import annotations

from typing import Any

import numpy as np

from .vocab import Vocabulary


def spec_to_sequence(
    spec: dict[str, Any],
    vocab: Vocabulary,
) -> tuple[list[int], np.ndarray]:
    """Encode a GraphSpec dict into a token sequence + values matrix.

    Args:
        spec:  Plain dict compatible with ``rustic_py.render()``.
        vocab: A :class:`Vocabulary` built via ``Vocabulary.from_rustic()``.

    Returns:
        token_ids: integer list, length = sequence length
        values:    float32 array of shape (seq_len, values_width); only rows
                   at <VALUES> positions are non-zero.
    """
    W = vocab.values_width
    token_ids: list[int] = []
    rows: list[np.ndarray] = []

    def _append(tok_id: int, row: np.ndarray | None = None) -> None:
        token_ids.append(tok_id)
        rows.append(row if row is not None else np.zeros(W, dtype=np.float32))

    # SOS
    _append(vocab.sos)

    # NOTE + VALUES(note, note_on, note_off)
    _append(vocab.note_tok)
    raw_note = np.array([spec["note"], spec["note_on"], spec["note_off"]], dtype=np.float32)
    _append(vocab.values_tok, vocab.normalize(vocab.note_tok, raw_note))

    # SOURCE type + VALUES(source params)
    waveform = spec["source"]["waveform"]
    src_tok_name = f"source:{waveform}"
    if src_tok_name not in vocab.tokens:
        raise ValueError(f"Unknown waveform {waveform!r}; not in vocabulary")
    src_tok_id = vocab.tokens[src_tok_name]
    _append(src_tok_id)

    src_layout = vocab.param_layout[src_tok_id]
    src_raw = np.zeros(len(src_layout), dtype=np.float32)
    src_dict = spec["source"]
    for i, fname in enumerate(src_layout):
        src_raw[i] = float(src_dict.get(fname, 0.0))
    _append(vocab.values_tok, vocab.normalize(src_tok_id, src_raw))

    # FILTER(s): each filter → VALUES(filter params)
    for flt in spec.get("filters", []):
        flt_type = flt["type"]
        flt_tok_name = f"filter:{flt_type}"
        if flt_tok_name not in vocab.tokens:
            raise ValueError(f"Unknown filter type {flt_type!r}; not in vocabulary")
        flt_tok_id = vocab.tokens[flt_tok_name]
        _append(flt_tok_id)

        flt_layout = vocab.param_layout[flt_tok_id]
        flt_raw = np.zeros(len(flt_layout), dtype=np.float32)
        flt_params = flt.get("params", {})
        for i, fname in enumerate(flt_layout):
            flt_raw[i] = float(flt_params.get(fname, 0.0))
        _append(vocab.values_tok, vocab.normalize(flt_tok_id, flt_raw))

    # EOS
    _append(vocab.eos)

    values = np.stack(rows, axis=0)  # (seq_len, W)
    return token_ids, values


def sequence_to_spec(
    token_ids: list[int] | np.ndarray,
    values: np.ndarray,
    vocab: Vocabulary,
    *,
    sample_rate: float = 44100.0,
    block_size: int = 512,
    duration: float = 1.0,
    frequency_relation: str = "identity",
) -> dict[str, Any]:
    """Decode a token sequence + values matrix back into a GraphSpec dict.

    Walks the token sequence; when a structural token (NOTE / source / filter)
    is encountered, the *next* <VALUES> row is used to reconstruct parameters.

    Args:
        token_ids: integer sequence (may contain EOS; parsing stops there).
        values:    float32 array, shape (seq_len, values_width).
        vocab:     Vocabulary instance.
        sample_rate, block_size, duration, frequency_relation:
                   fixed scalar fields for the spec dict.

    Returns:
        A dict compatible with ``rustic_py.render()``.

    Raises:
        ValueError: if the sequence is malformed (e.g. missing VALUES token
                    after a structural token, or missing source token).
    """
    ids = list(token_ids)

    # strip leading SOS / trailing EOS+PAD
    while ids and ids[0] in (vocab.sos, vocab.pad):
        ids.pop(0)
        values = values[1:]
    while ids and ids[-1] in (vocab.eos, vocab.pad):
        ids.pop()
        values = values[:-1]

    spec: dict[str, Any] = {
        "note": 60,
        "note_on": 0.0,
        "note_off": 0.5,
        "duration": duration,
        "sample_rate": sample_rate,
        "block_size": block_size,
        "source": {
            "waveform": "sine",
            "frequency_relation": frequency_relation,
            "attack": 0.01,
            "decay": 0.1,
            "sustain": 0.8,
            "release": 0.2,
        },
        "filters": [],
    }

    i = 0
    last_structural: int | None = None

    while i < len(ids):
        tok = ids[i]
        tok_name = vocab.id_to_token.get(tok, "")

        if tok == vocab.values_tok:
            if last_structural is None:
                i += 1
                continue  # orphan VALUES token; skip

            raw = vocab.denormalize(last_structural, values[i])
            layout = vocab.param_layout.get(last_structural, [])

            if last_structural == vocab.note_tok:
                # layout = ['note', 'note_on', 'note_off']
                param_dict = {fname: float(raw[j]) for j, fname in enumerate(layout)}
                spec["note"] = int(round(param_dict.get("note", 60)))
                spec["note_on"] = float(param_dict.get("note_on", 0.0))
                spec["note_off"] = float(param_dict.get("note_off", 0.5))

            elif vocab.is_source_token(last_structural):
                param_dict = {fname: float(raw[j]) for j, fname in enumerate(layout)}
                # Update source dict; waveform was already set when we saw the source token
                for fname, val in param_dict.items():
                    spec["source"][fname] = val

            elif vocab.is_filter_token(last_structural):
                param_dict = {fname: float(raw[j]) for j, fname in enumerate(layout)}
                spec["filters"].append({
                    "type": vocab.filter_type_id(last_structural),
                    "params": param_dict,
                })

            last_structural = None

        elif tok == vocab.note_tok:
            last_structural = tok

        elif vocab.is_source_token(tok):
            spec["source"]["waveform"] = vocab.source_type_id(tok)
            last_structural = tok

        elif vocab.is_filter_token(tok):
            last_structural = tok

        i += 1

    return spec
