"""
Vocabulary built dynamically from rustic_py metadata.

Two-head architecture:
  - Continuous head: bounded float values, shape (cont_width,) per VALS position.
    ADSR duration fields use log-scale normalisation.
  - Categorical head: integer class values, shape (cat_width,) per VALS position.
    Used for MIDI notes and small-integer connection/harmonic indices.

Token layout order (fixed prefix, then auto-generated):
  Special:     <SOS>  <EOS>  <PAD>  <VALS>
  Note:        NOTE
  MultiSource: <SOMS> <EOMS>  MM:sum  MM:avg  MM:max
  Source:      <SOSD> <EOSD>
               WF:<type> ... (auto from available_sources)
               FR:identity  FR:harmonic  FR:offset  FR:ratio  FR:constant  FR:semitones
  Envelope:    <SOED> <EOED>  ATK  DCY  SUS  REL
  Filter:      <SOFD> <EOFD>
               FT:<type> ... (auto from available_filters)
  Connection:  CN:source_sink  CN:source_filter  CN:filter_filter  CN:filter_sink
"""
from __future__ import annotations

import math
from dataclasses import dataclass, field

import numpy as np

# ── ADSR normalisation constants ──────────────────────────────────────────────
_ADSR_DUR_MIN: float = 0.001
_ADSR_DUR_MAX: float = 4.0
_ADSR_AMP_MIN: float = 0.0
_ADSR_AMP_MAX: float = 1.0

# ── Note / timing constants ────────────────────────────────────────────────────
_NOTE_CLASSES: int = 128
_TIMING_MIN: float = 0.0
_TIMING_MAX: float = 20.0   # seconds (generous upper bound)

# ── Frequency relation value ranges ───────────────────────────────────────────
_HARMONIC_CLASSES: int = 32   # integer harmonic multipliers 1–32
_FR_OFFSET_MIN: float = -2000.0
_FR_OFFSET_MAX: float =  2000.0
_FR_RATIO_MIN:  float =  0.1
_FR_RATIO_MAX:  float =  10.0
_FR_CONST_MIN:  float =  20.0
_FR_CONST_MAX:  float =  8000.0
_FR_SEMI_MIN:   float = -24.0
_FR_SEMI_MAX:   float =  24.0

# ── Connection index size ──────────────────────────────────────────────────────
_CN_IDX_CLASSES: int = 32   # supports up to 32 sources or filters per graph


def _extract_param_info(
    params_raw: list[dict],
    *,
    wrapped: bool,
) -> list[tuple[str, float, float]]:
    """Return [(field_name, min, max), …] from metadata param lists.

    Args:
        params_raw: list from available_sources() ``parameters`` key (wrapped=False)
                    or from available_filters() ``inputs`` list (wrapped=True).
        wrapped: if True each entry is ``{'parameter': {kind: data} | None}``
                 (filter style); if False each entry is ``{kind: data}`` directly.
    """
    result: list[tuple[str, float, float]] = []
    for entry in params_raw:
        if wrapped:
            p = entry.get("parameter")
            if p is None:
                continue
        else:
            p = entry
        (kind, data), = p.items()
        name: str = data["field_name"]
        if kind in ("Range", "Float"):
            lo, hi = float(data.get("min", 0)), float(data.get("max", 1))
        elif kind == "Int":
            lo = float(data.get("min", 0))
            hi_raw = data.get("max", 127)
            hi = float(hi_raw) if hi_raw is not None else 127.0
        elif kind == "Toggle":
            lo, hi = 0.0, 1.0
        else:
            lo, hi = 0.0, 1.0
        result.append((name, lo, hi))
    return result


@dataclass
class Vocabulary:
    """Token table + two-head parameter metadata for the autoregressive graph model.

    Attributes:
        tokens:       token-name → integer id
        id_to_token:  integer id → token-name
        cont_layout:  token-id → ordered list of continuous field names
        cat_layout:   token-id → ordered list of categorical field names
        cont_ranges:  field-name → (lo, hi) for [0,1] normalisation
        log_fields:   set of field names that use log-scale normalisation
        cat_n_classes: field-name → number of classes
        cont_width:   max number of continuous values any VALS position carries
        cat_width:    max number of categorical values any VALS position carries
    """
    tokens:       dict[str, int]           = field(default_factory=dict)
    id_to_token:  dict[int, str]           = field(default_factory=dict)
    cont_layout:  dict[int, list[str]]     = field(default_factory=dict)
    cat_layout:   dict[int, list[str]]     = field(default_factory=dict)
    cont_ranges:  dict[str, tuple[float, float]] = field(default_factory=dict)
    log_fields:   set[str]                 = field(default_factory=set)
    cat_n_classes: dict[str, int]          = field(default_factory=dict)
    cont_width:   int = 0
    cat_width:    int = 0

    # ── token name constants ──────────────────────────────────────────────────
    SOS_NAME  = "<SOS>"
    EOS_NAME  = "<EOS>"
    PAD_NAME  = "<PAD>"
    VALS_NAME = "<VALS>"
    NOTE_NAME = "NOTE"

    SOMS_NAME   = "<SOMS>"
    EOMS_NAME   = "<EOMS>"
    MM_SUM_NAME = "MM:sum"
    MM_AVG_NAME = "MM:avg"
    MM_MAX_NAME = "MM:max"

    SOSD_NAME = "<SOSD>"
    EOSD_NAME = "<EOSD>"

    FR_IDENTITY_NAME  = "FR:identity"
    FR_HARMONIC_NAME  = "FR:harmonic"
    FR_OFFSET_NAME    = "FR:offset"
    FR_RATIO_NAME     = "FR:ratio"
    FR_CONSTANT_NAME  = "FR:constant"
    FR_SEMITONES_NAME = "FR:semitones"

    SOED_NAME = "<SOED>"
    EOED_NAME = "<EOED>"
    ATK_NAME  = "ATK"
    DCY_NAME  = "DCY"
    SUS_NAME  = "SUS"
    REL_NAME  = "REL"

    SOFD_NAME = "<SOFD>"
    EOFD_NAME = "<EOFD>"

    CN_SOURCE_SINK_NAME   = "CN:source_sink"
    CN_SOURCE_FILTER_NAME = "CN:source_filter"
    CN_FILTER_FILTER_NAME = "CN:filter_filter"
    CN_FILTER_SINK_NAME   = "CN:filter_sink"

    # ── convenience ID properties ─────────────────────────────────────────────
    @property
    def sos(self) -> int:       return self.tokens[self.SOS_NAME]
    @property
    def eos(self) -> int:       return self.tokens[self.EOS_NAME]
    @property
    def pad(self) -> int:       return self.tokens[self.PAD_NAME]
    @property
    def vals_tok(self) -> int:  return self.tokens[self.VALS_NAME]
    @property
    def note_tok(self) -> int:  return self.tokens[self.NOTE_NAME]
    @property
    def atk_tok(self) -> int:   return self.tokens[self.ATK_NAME]
    @property
    def dcy_tok(self) -> int:   return self.tokens[self.DCY_NAME]
    @property
    def sus_tok(self) -> int:   return self.tokens[self.SUS_NAME]
    @property
    def rel_tok(self) -> int:   return self.tokens[self.REL_NAME]

    def __len__(self) -> int:
        return len(self.tokens)

    # ── token type predicates ─────────────────────────────────────────────────
    def is_wf_token(self, tid: int) -> bool:
        return self.id_to_token.get(tid, "").startswith("WF:")

    def is_ft_token(self, tid: int) -> bool:
        return self.id_to_token.get(tid, "").startswith("FT:")

    def is_fr_token(self, tid: int) -> bool:
        return self.id_to_token.get(tid, "").startswith("FR:")

    def is_cn_token(self, tid: int) -> bool:
        return self.id_to_token.get(tid, "").startswith("CN:")

    def is_mm_token(self, tid: int) -> bool:
        return self.id_to_token.get(tid, "").startswith("MM:")

    def is_adsr_segment_token(self, tid: int) -> bool:
        return self.id_to_token.get(tid, "") in (
            self.ATK_NAME, self.DCY_NAME, self.SUS_NAME, self.REL_NAME
        )

    # ── token type decoders ───────────────────────────────────────────────────
    def wf_type_id(self, tid: int) -> str:
        """Waveform type string from a WF:* token."""
        return self.id_to_token[tid][3:]

    def ft_type_id(self, tid: int) -> str:
        """Filter type string from a FT:* token."""
        return self.id_to_token[tid][3:]

    def fr_relation(self, tid: int) -> str:
        """Frequency relation string from a FR:* token."""
        return self.id_to_token[tid][3:]

    def mm_mode(self, tid: int) -> str:
        """Mix mode string (as expected by MultiSourceSpec) from a MM:* token."""
        mm = self.id_to_token[tid][3:]
        return {"sum": "Sum", "avg": "Average", "max": "Max"}.get(mm, mm.capitalize())

    def has_vals(self, tid: int) -> bool:
        """True if this token is followed by a <VALS> in the sequence."""
        return bool(self.cont_layout.get(tid) or self.cat_layout.get(tid))

    # ── normalisation ─────────────────────────────────────────────────────────
    def normalize_cont(self, token_id: int, raw: np.ndarray) -> np.ndarray:
        """Normalise continuous values to [0, 1] for the given structural token.

        Fields in ``log_fields`` use log-scale normalisation.
        Output is always of length ``cont_width`` (zero-padded beyond active fields).
        """
        out = np.zeros(self.cont_width, dtype=np.float32)
        layout = self.cont_layout.get(token_id, [])
        raw_arr = np.asarray(raw, dtype=np.float32)
        for i, name in enumerate(layout):
            if i >= len(raw_arr):
                break
            lo, hi = self.cont_ranges[name]
            v = float(raw_arr[i])
            if name in self.log_fields:
                lo_l = math.log(max(lo, 1e-12))
                hi_l = math.log(max(hi, 1e-12))
                span = hi_l - lo_l
                out[i] = (math.log(max(v, 1e-12)) - lo_l) / span if span > 0 else 0.0
            else:
                span = hi - lo
                out[i] = float(v - lo) / span if span > 0 else 0.0
        return out

    def denormalize_cont(self, token_id: int, norm: np.ndarray) -> np.ndarray:
        """Denormalise continuous values from [0, 1] back to raw values."""
        out = np.zeros(self.cont_width, dtype=np.float32)
        layout = self.cont_layout.get(token_id, [])
        for i, name in enumerate(layout):
            if i >= len(norm):
                break
            lo, hi = self.cont_ranges[name]
            v = float(norm[i])
            if name in self.log_fields:
                lo_l = math.log(max(lo, 1e-12))
                hi_l = math.log(max(hi, 1e-12))
                out[i] = math.exp(v * (hi_l - lo_l) + lo_l)
            else:
                out[i] = float(np.clip(v * (hi - lo) + lo, lo, hi))
        return out

    # ── factory ───────────────────────────────────────────────────────────────
    @classmethod
    def from_rustic(cls) -> "Vocabulary":
        """Build a Vocabulary from live rustic_py metadata."""
        from rustic_py.rustic_py import available_filters, available_sources  # type: ignore[import]

        v = cls()
        idx = 0

        def _add(name: str) -> int:
            nonlocal idx
            v.tokens[name] = idx
            v.id_to_token[idx] = name
            r = idx
            idx += 1
            return r

        def _set_cont(tid: int, fields: list[tuple[str, float, float]],
                      log: list[str] | None = None) -> None:
            layout = []
            for name, lo, hi in fields:
                layout.append(name)
                if name not in v.cont_ranges:
                    v.cont_ranges[name] = (lo, hi)
                else:
                    el, eh = v.cont_ranges[name]
                    v.cont_ranges[name] = (min(el, lo), max(eh, hi))
                if log and name in log:
                    v.log_fields.add(name)
            v.cont_layout[tid] = layout

        def _set_cat(tid: int, fields: list[tuple[str, int]]) -> None:
            layout = []
            for name, n_classes in fields:
                layout.append(name)
                v.cat_n_classes[name] = n_classes
            v.cat_layout[tid] = layout

        def _adsr_fields(prefix: str) -> list[tuple[str, float, float]]:
            return [
                (f"{prefix}_dur",  _ADSR_DUR_MIN, _ADSR_DUR_MAX),
                (f"{prefix}_peak", _ADSR_AMP_MIN, _ADSR_AMP_MAX),
                (f"{prefix}_ct",   _ADSR_DUR_MIN, _ADSR_DUR_MAX),
                (f"{prefix}_cp",   _ADSR_AMP_MIN, _ADSR_AMP_MAX),
            ]

        # ── special tokens (no values) ────────────────────────────────────
        for name in (cls.SOS_NAME, cls.EOS_NAME, cls.PAD_NAME, cls.VALS_NAME):
            _add(name)

        # ── NOTE: cat(note 0-127) + cont(note_on, note_off) ───────────────
        note_id = _add(cls.NOTE_NAME)
        _set_cat(note_id, [("note", _NOTE_CLASSES)])
        _set_cont(note_id, [
            ("note_on",  _TIMING_MIN, _TIMING_MAX),
            ("note_off", _TIMING_MIN, _TIMING_MAX),
        ])

        # ── Multi-source structure ─────────────────────────────────────────
        _add(cls.SOMS_NAME)
        _add(cls.EOMS_NAME)

        _add(cls.MM_SUM_NAME)
        _add(cls.MM_AVG_NAME)
        _add(cls.MM_MAX_NAME)

        # ── Source definition structure ────────────────────────────────────
        _add(cls.SOSD_NAME)
        _add(cls.EOSD_NAME)

        # WF:* tokens — auto-generated, no values (waveform params not in SourceSpec)
        for src_meta in available_sources():
            _add(f"WF:{src_meta['type_id']}")

        # FR:* tokens
        _add(cls.FR_IDENTITY_NAME)   # no values

        fr_harmonic_id = _add(cls.FR_HARMONIC_NAME)
        _set_cat(fr_harmonic_id, [("harmonic_n", _HARMONIC_CLASSES)])

        fr_offset_id = _add(cls.FR_OFFSET_NAME)
        _set_cont(fr_offset_id, [("fr_offset_hz", _FR_OFFSET_MIN, _FR_OFFSET_MAX)])

        fr_ratio_id = _add(cls.FR_RATIO_NAME)
        _set_cont(fr_ratio_id, [("fr_ratio", _FR_RATIO_MIN, _FR_RATIO_MAX)],
                  log=["fr_ratio"])

        fr_const_id = _add(cls.FR_CONSTANT_NAME)
        _set_cont(fr_const_id, [("fr_const_hz", _FR_CONST_MIN, _FR_CONST_MAX)],
                  log=["fr_const_hz"])

        fr_semi_id = _add(cls.FR_SEMITONES_NAME)
        _set_cont(fr_semi_id, [("fr_semitones", _FR_SEMI_MIN, _FR_SEMI_MAX)])

        # ── Envelope structure ────────────────────────────────────────────
        _add(cls.SOED_NAME)
        _add(cls.EOED_NAME)

        atk_id = _add(cls.ATK_NAME)
        _set_cont(atk_id, _adsr_fields("atk"), log=["atk_dur"])

        dcy_id = _add(cls.DCY_NAME)
        _set_cont(dcy_id, _adsr_fields("dcy"), log=["dcy_dur"])

        sus_id = _add(cls.SUS_NAME)
        _set_cont(sus_id, [("sus_level", _ADSR_AMP_MIN, _ADSR_AMP_MAX)])

        rel_id = _add(cls.REL_NAME)
        _set_cont(rel_id, _adsr_fields("rel"), log=["rel_dur"])

        # ── Filter structure ──────────────────────────────────────────────
        _add(cls.SOFD_NAME)
        _add(cls.EOFD_NAME)

        # FT:* tokens — auto-generated from available_filters()
        for flt_meta in available_filters():
            type_id = flt_meta["type_id"]
            ft_id = _add(f"FT:{type_id}")
            params = _extract_param_info(flt_meta["inputs"], wrapped=True)
            if params:
                _set_cont(ft_id, params)

        # ── Connection tokens ─────────────────────────────────────────────
        cn_ss_id = _add(cls.CN_SOURCE_SINK_NAME)
        _set_cat(cn_ss_id, [("cn_src", _CN_IDX_CLASSES)])

        cn_sf_id = _add(cls.CN_SOURCE_FILTER_NAME)
        _set_cat(cn_sf_id, [("cn_src", _CN_IDX_CLASSES), ("cn_flt", _CN_IDX_CLASSES)])

        cn_ff_id = _add(cls.CN_FILTER_FILTER_NAME)
        _set_cat(cn_ff_id, [("cn_flt_out", _CN_IDX_CLASSES), ("cn_flt_in", _CN_IDX_CLASSES)])

        cn_fs_id = _add(cls.CN_FILTER_SINK_NAME)
        _set_cat(cn_fs_id, [("cn_flt", _CN_IDX_CLASSES)])

        # ── Compute head widths ───────────────────────────────────────────
        all_cont = [len(layout) for layout in v.cont_layout.values()]
        all_cat  = [len(layout) for layout in v.cat_layout.values()]
        v.cont_width = max(all_cont) if all_cont else 1
        v.cat_width  = max(all_cat)  if all_cat  else 1

        return v

    # ── pretty-print ──────────────────────────────────────────────────────────
    def summary(self) -> str:
        lines = [
            f"Vocabulary  ({len(self)} tokens, cont_width={self.cont_width}, cat_width={self.cat_width})",
            f"{'ID':>4}  {'Token':<32}  {'cont':>5}  {'cat':>5}  Fields",
            "-" * 72,
        ]
        for name, tid in sorted(self.tokens.items(), key=lambda kv: kv[1]):
            nc = len(self.cont_layout.get(tid, []))
            nk = len(self.cat_layout.get(tid, []))
            cont_fields = ", ".join(self.cont_layout.get(tid, []))
            cat_fields  = ", ".join(self.cat_layout.get(tid, []))
            fields_str  = " | ".join(filter(None, [cont_fields, cat_fields]))
            lines.append(f"{tid:>4}  {name:<32}  {nc:>5}  {nk:>5}  {fields_str}")
        return "\n".join(lines)
