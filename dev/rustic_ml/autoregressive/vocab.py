"""
Vocabulary built dynamically from rustic_py metadata.

Reads available_filters() and available_sources() at import time to construct:
  - A token table (special tokens + NOTE + one per source type + one per filter type)
  - Per-token parameter layouts (ordered field names)
  - Per-field normalisation ranges ([min, max] → [0, 1])
  - The fixed VALUES vector width (max param count over all node types)
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

import numpy as np

# ── NOTE token ranges (shared with rustic_ml.data.encoding constants) ─────────
_NOTE_MIN: float = 36.0
_NOTE_MAX: float = 84.0
_TIMING_MIN: float = 0.0
_TIMING_MAX: float = 1.0   # duration is always 1.0 s


def _extract_param_info(
    params_raw: list[dict],
    *,
    wrapped: bool,
) -> list[tuple[str, float, float]]:
    """Return [(field_name, min, max), ...] from metadata param lists.

    Args:
        params_raw: raw list from available_sources() ``parameters`` key (wrapped=False)
                    or from available_filters() ``inputs`` list (wrapped=True).
        wrapped: if True each entry is ``{'parameter': {kind: data} | None}``
                 (filter style); if False each entry is ``{kind: data}`` directly
                 (source style).
    """
    result: list[tuple[str, float, float]] = []
    for entry in params_raw:
        if wrapped:
            p = entry.get("parameter")
            if p is None:
                continue  # audio signal input, not a parameter
        else:
            p = entry
        (kind, data), = p.items()
        name: str = data["field_name"]
        if kind in ("Range", "Float"):
            lo, hi = float(data.get("min", 0)), float(data.get("max", 0))
        elif kind == "Int":
            print(data)
            lo = float(data.get("min", 0))
            hi = data.get("max", 127)
            if hi is not None:
                hi = float(hi)
            else:
                hi = 0.0
        elif kind == "Toggle":
            lo, hi = 0.0, 1.0
        else:
            lo, hi = 0.0, 1.0  # fallback for unknown kinds
        result.append((name, lo, hi))
    return result


@dataclass
class Vocabulary:
    """Token table + parameter metadata for the autoregressive graph model.

    Attributes:
        tokens:       mapping token-name → integer id
        id_to_token:  inverse mapping
        param_layout: token-id → ordered list of field names (only for structural tokens)
        param_ranges: field-name → (min, max) for [0,1] normalisation
        values_width: fixed width of the VALUES vector (max params over all nodes)

    Special token IDs are exposed as class constants:
        SOS, EOS, PAD, VALUES_TOK, NOTE_TOK
    """
    tokens: dict[str, int] = field(default_factory=dict)
    id_to_token: dict[int, str] = field(default_factory=dict)
    param_layout: dict[int, list[str]] = field(default_factory=dict)
    param_ranges: dict[str, tuple[float, float]] = field(default_factory=dict)
    values_width: int = 0

    # ── special token names ────────────────────────────────────────────────
    SOS_NAME = "<SOS>"
    EOS_NAME = "<EOS>"
    PAD_NAME = "<PAD>"
    VALUES_NAME = "<VALUES>"
    NOTE_NAME = "NOTE"

    # ── convenience ID properties ──────────────────────────────────────────
    @property
    def sos(self) -> int:
        return self.tokens[self.SOS_NAME]

    @property
    def eos(self) -> int:
        return self.tokens[self.EOS_NAME]

    @property
    def pad(self) -> int:
        return self.tokens[self.PAD_NAME]

    @property
    def values_tok(self) -> int:
        return self.tokens[self.VALUES_NAME]

    @property
    def note_tok(self) -> int:
        return self.tokens[self.NOTE_NAME]

    # ── size ───────────────────────────────────────────────────────────────
    def __len__(self) -> int:
        return len(self.tokens)

    # ── normalisation helpers ──────────────────────────────────────────────
    def normalize(self, token_id: int, raw: np.ndarray) -> np.ndarray:
        """Map raw parameter values to [0, 1] for the given structural token.

        Only the positions corresponding to the token's param_layout are
        normalised; remaining positions (padding) are left as-is.
        """
        out = np.zeros(self.values_width, dtype=np.float32)
        layout = self.param_layout.get(token_id, [])
        for i, name in enumerate(layout):
            lo, hi = self.param_ranges[name]
            span = hi - lo
            out[i] = float(raw[i] - lo) / span if span > 0 else 0.0
        return out

    def denormalize(self, token_id: int, norm: np.ndarray) -> np.ndarray:
        """Map [0, 1] values back to raw parameter values."""
        out = np.zeros(self.values_width, dtype=np.float32)
        layout = self.param_layout.get(token_id, [])
        for i, name in enumerate(layout):
            lo, hi = self.param_ranges[name]
            out[i] = float(norm[i]) * (hi - lo) + lo
        return out

    # ── source/filter token predicates ────────────────────────────────────
    def is_source_token(self, token_id: int) -> bool:
        name = self.id_to_token.get(token_id, "")
        return name.startswith("source:")

    def is_filter_token(self, token_id: int) -> bool:
        name = self.id_to_token.get(token_id, "")
        return name.startswith("filter:")

    def source_type_id(self, token_id: int) -> str:
        """Return the rustic_py type_id string for a source token."""
        return self.id_to_token[token_id][len("source:"):]

    def filter_type_id(self, token_id: int) -> str:
        """Return the rustic_py type_id string for a filter token."""
        return self.id_to_token[token_id][len("filter:"):]

    # ── factory ───────────────────────────────────────────────────────────
    @classmethod
    def from_rustic(cls) -> "Vocabulary":
        """Build a Vocabulary from live rustic_py metadata.

        Requires rustic_py to be importable (i.e. the Rust extension must be
        built and installed in the active environment).
        """
        from rustic_py.rustic_py import available_filters, available_sources  # type: ignore[import]

        vocab = cls()
        idx = 0

        # ── special tokens ─────────────────────────────────────────────
        for name in (cls.SOS_NAME, cls.EOS_NAME, cls.PAD_NAME, cls.VALUES_NAME):
            vocab.tokens[name] = idx
            vocab.id_to_token[idx] = name
            idx += 1

        # ── NOTE pseudo-token ──────────────────────────────────────────
        note_id = idx
        vocab.tokens[cls.NOTE_NAME] = note_id
        vocab.id_to_token[note_id] = cls.NOTE_NAME
        idx += 1

        note_fields = ["note", "note_on", "note_off"]
        vocab.param_layout[note_id] = note_fields
        vocab.param_ranges["note"] = (_NOTE_MIN, _NOTE_MAX)
        vocab.param_ranges["note_on"] = (_TIMING_MIN, _TIMING_MAX)
        vocab.param_ranges["note_off"] = (_TIMING_MIN, _TIMING_MAX)

        # ── source tokens ──────────────────────────────────────────────
        for src_meta in available_sources():
            type_id: str = src_meta["type_id"]
            tok_name = f"source:{type_id}"
            tok_id = idx
            vocab.tokens[tok_name] = tok_id
            vocab.id_to_token[tok_id] = tok_name
            idx += 1

            params = _extract_param_info(src_meta["parameters"], wrapped=False)
            layout = []
            for fname, lo, hi in params:
                layout.append(fname)
                # param_ranges: last write wins (same field may appear in
                # multiple source types with identical ranges, but we keep the
                # broadest range seen so normalisation is consistent)
                if fname not in vocab.param_ranges:
                    vocab.param_ranges[fname] = (lo, hi)
                else:
                    existing_lo, existing_hi = vocab.param_ranges[fname]
                    vocab.param_ranges[fname] = (
                        min(existing_lo, lo),
                        max(existing_hi, hi),
                    )
            vocab.param_layout[tok_id] = layout

        # ── filter tokens ──────────────────────────────────────────────
        for flt_meta in available_filters():
            type_id = flt_meta["type_id"]
            tok_name = f"filter:{type_id}"
            tok_id = idx
            vocab.tokens[tok_name] = tok_id
            vocab.id_to_token[tok_id] = tok_name
            idx += 1

            params = _extract_param_info(flt_meta["inputs"], wrapped=True)
            layout = []
            for fname, lo, hi in params:
                layout.append(fname)
                if fname not in vocab.param_ranges:
                    vocab.param_ranges[fname] = (lo, hi)
                else:
                    existing_lo, existing_hi = vocab.param_ranges[fname]
                    vocab.param_ranges[fname] = (
                        min(existing_lo, lo),
                        max(existing_hi, hi),
                    )
            vocab.param_layout[tok_id] = layout

        # ── values_width ───────────────────────────────────────────────
        vocab.values_width = max(
            len(layout) for layout in vocab.param_layout.values()
        ) if vocab.param_layout else 1

        return vocab

    # ── pretty-print ──────────────────────────────────────────────────────
    def summary(self) -> str:
        lines = [
            f"Vocabulary  ({len(self)} tokens, values_width={self.values_width})",
            f"{'ID':>4}  {'Token':<32}  {'# params':>8}  Fields",
            "-" * 80,
        ]
        for name, tid in sorted(self.tokens.items(), key=lambda kv: kv[1]):
            layout = self.param_layout.get(tid, [])
            fields_preview = ", ".join(layout[:4])
            if len(layout) > 4:
                fields_preview += f", … (+{len(layout) - 4})"
            lines.append(f"{tid:>4}  {name:<32}  {len(layout):>8}  {fields_preview}")
        return "\n".join(lines)
