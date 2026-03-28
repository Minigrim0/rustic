"""
Spec ↔ token-sequence serialisation for the hierarchical graph vocabulary.

A GraphSpec dict is encoded as a flat integer token sequence paired with two
values matrices:
  - cont_values: shape (seq_len, cont_width) — normalised continuous params
  - cat_values:  shape (seq_len, cat_width)  — integer categorical params

Only positions corresponding to a <VALS> token carry non-zero values; all
other rows are zero.

Sequence structure (hierarchical):
    SOS
    NOTE  <VALS(cat:note; cont:note_on, note_off)>
    <SOMS>
      MM:avg | MM:sum | MM:max
      <SOSD>
        WF:<type>
        FR:<relation>  [<VALS(...)>]        # only if relation ≠ identity
        <SOED>
          ATK <VALS(cont:dur,peak,ct,cp)>
          DCY <VALS(cont:dur,peak,ct,cp)>
          SUS <VALS(cont:level)>
          REL <VALS(cont:dur,peak,ct,cp)>
        <EOED>
      <EOSD>
      ...                                   # more <SOSD>...<EOSD> blocks
      <SOED>...<EOED>                       # glob_ampl envelope
    <EOMS>
    ...                                     # more <SOMS>...<EOMS> blocks
    <SOFD>
      FT:<type>  <VALS(cont:params...)>
    <EOFD>
    ...                                     # more filter blocks
    CN:source_sink   <VALS(cat:src_idx)>
    CN:source_filter <VALS(cat:src_idx, flt_idx)>
    CN:filter_filter <VALS(cat:flt_out, flt_in)>
    CN:filter_sink   <VALS(cat:flt_idx)>
    ...
    EOS
"""
from __future__ import annotations

from typing import Any

import numpy as np

from .vocab import Vocabulary

# Default ADSR for round-trip defaults
_DEFAULT_ADSR: dict[str, Any] = {
    "attack":  [0.01, 1.0, 0.01, 0.0],
    "decay":   [0.1,  0.8, 0.1,  1.0],
    "sustain": 0.8,
    "release": [0.2,  0.0, 0.0,  0.0],
}


def spec_to_sequence(
    spec: dict[str, Any],
    vocab: Vocabulary,
) -> tuple[list[int], np.ndarray, np.ndarray]:
    """Encode a GraphSpec dict into a token sequence + two values matrices.

    Args:
        spec:  Plain dict from ``GraphSpec.to_spec()``.
        vocab: A :class:`Vocabulary` built via ``Vocabulary.from_rustic()``.

    Returns:
        token_ids:    integer list, length = sequence length
        cont_values:  float32 array shape (seq_len, cont_width)
        cat_values:   int64 array   shape (seq_len, cat_width)
    """
    CW = vocab.cont_width
    KW = vocab.cat_width
    token_ids: list[int] = []
    cont_rows: list[np.ndarray] = []
    cat_rows:  list[np.ndarray] = []

    def _append(tok_id: int,
                cont: np.ndarray | None = None,
                cat:  np.ndarray | None = None) -> None:
        token_ids.append(tok_id)
        cont_rows.append(cont if cont is not None else np.zeros(CW, np.float32))
        cat_rows.append(cat   if cat  is not None else np.zeros(KW, np.int64))

    def _vals(context_tok: int,
              raw_cont: list[float] | None = None,
              raw_cat:  list[int]   | None = None) -> None:
        """Append a <VALS> token with values normalised for context_tok."""
        cont = vocab.normalize_cont(
            context_tok,
            np.array(raw_cont or [], dtype=np.float32),
        )
        cat = np.zeros(KW, dtype=np.int64)
        for j, v in enumerate(raw_cat or []):
            cat[j] = int(v)
        _append(vocab.vals_tok, cont, cat)

    def _adsr(adsr: dict[str, Any]) -> None:
        """Encode one ADSR envelope block."""
        _append(vocab.tokens[vocab.SOED_NAME])

        atk = adsr.get("attack", _DEFAULT_ADSR["attack"])
        _append(vocab.atk_tok)
        _vals(vocab.atk_tok, [atk[0], atk[1], atk[2], atk[3]])

        dcy = adsr.get("decay", _DEFAULT_ADSR["decay"])
        _append(vocab.dcy_tok)
        _vals(vocab.dcy_tok, [dcy[0], dcy[1], dcy[2], dcy[3]])

        _append(vocab.sus_tok)
        _vals(vocab.sus_tok, [adsr.get("sustain", 0.8)])

        rel = adsr.get("release", _DEFAULT_ADSR["release"])
        _append(vocab.rel_tok)
        _vals(vocab.rel_tok, [rel[0], rel[1], rel[2], rel[3]])

        _append(vocab.tokens[vocab.EOED_NAME])

    # ── SOS ───────────────────────────────────────────────────────────────────
    _append(vocab.sos)

    # ── NOTE ──────────────────────────────────────────────────────────────────
    _append(vocab.note_tok)
    _vals(vocab.note_tok,
          raw_cont=[spec["note_on"], spec["note_off"]],
          raw_cat=[spec["note"]])

    # ── MultiSource blocks ────────────────────────────────────────────────────
    for ms in spec.get("sources", []):
        _append(vocab.tokens[vocab.SOMS_NAME])

        # Mix mode
        mm_map = {
            "Sum":     vocab.MM_SUM_NAME,
            "Average": vocab.MM_AVG_NAME,
            "Max":     vocab.MM_MAX_NAME,
        }
        mm_name = mm_map.get(ms["mix_mode"], vocab.MM_AVG_NAME)
        _append(vocab.tokens[mm_name])

        # Individual sources
        for src in ms.get("sources", []):
            _append(vocab.tokens[vocab.SOSD_NAME])

            # Waveform
            wf_name = f"WF:{src['waveform']}"
            if wf_name not in vocab.tokens:
                raise ValueError(f"Unknown waveform {src['waveform']!r}")
            _append(vocab.tokens[wf_name])

            # Frequency relation
            fr = src.get("frequency_relation", "identity")
            fr_name = f"FR:{fr}"
            if fr_name not in vocab.tokens:
                raise ValueError(f"Unknown frequency relation {fr!r}")
            fr_tok = vocab.tokens[fr_name]
            _append(fr_tok)
            if vocab.has_vals(fr_tok):
                fr_val = src.get("frequency_relation_value", 0.0)
                if vocab.cat_layout.get(fr_tok):     # harmonic → categorical
                    _vals(fr_tok, raw_cat=[int(fr_val)])
                else:                                 # offset / ratio / constant / semitones → continuous
                    _vals(fr_tok, raw_cont=[float(fr_val)])

            # Source envelope
            _adsr(src["envelope"])

            _append(vocab.tokens[vocab.EOSD_NAME])

        # Global amplitude envelope
        _adsr(ms["glob_ampl"])

        _append(vocab.tokens[vocab.EOMS_NAME])

    # ── Filter blocks ─────────────────────────────────────────────────────────
    for flt in spec.get("filters", []):
        _append(vocab.tokens[vocab.SOFD_NAME])

        ft_name = f"FT:{flt['type']}"
        if ft_name not in vocab.tokens:
            raise ValueError(f"Unknown filter type {flt['type']!r}")
        ft_tok = vocab.tokens[ft_name]
        _append(ft_tok)

        if vocab.cont_layout.get(ft_tok):
            layout   = vocab.cont_layout[ft_tok]
            flt_params = flt.get("params", {})
            raw_cont = [float(flt_params.get(name, 0.0)) for name in layout]
            _vals(ft_tok, raw_cont=raw_cont)

        _append(vocab.tokens[vocab.EOFD_NAME])

    # ── Connection tokens ─────────────────────────────────────────────────────
    for conn in spec.get("connections", []):
        if "SourceSink" in conn:
            cn_tok = vocab.tokens[vocab.CN_SOURCE_SINK_NAME]
            _append(cn_tok)
            _vals(cn_tok, raw_cat=[conn["SourceSink"]["source"]])

        elif "SourceFilter" in conn:
            cn_tok = vocab.tokens[vocab.CN_SOURCE_FILTER_NAME]
            _append(cn_tok)
            _vals(cn_tok, raw_cat=[
                conn["SourceFilter"]["source"],
                conn["SourceFilter"]["filter"],
            ])

        elif "FilterFilter" in conn:
            cn_tok = vocab.tokens[vocab.CN_FILTER_FILTER_NAME]
            _append(cn_tok)
            _vals(cn_tok, raw_cat=[
                conn["FilterFilter"]["filter_out"],
                conn["FilterFilter"]["filter_in"],
            ])

        elif "FilterSink" in conn:
            cn_tok = vocab.tokens[vocab.CN_FILTER_SINK_NAME]
            _append(cn_tok)
            _vals(cn_tok, raw_cat=[conn["FilterSink"]["filter"]])

    # ── EOS ───────────────────────────────────────────────────────────────────
    _append(vocab.eos)

    return (
        token_ids,
        np.stack(cont_rows, axis=0),
        np.stack(cat_rows,  axis=0),
    )


def sequence_to_spec(
    token_ids:   list[int] | np.ndarray,
    cont_values: np.ndarray,
    cat_values:  np.ndarray,
    vocab:       Vocabulary,
    *,
    sample_rate: float = 44100.0,
    block_size:  int   = 512,
    duration:    float = 1.0,
) -> dict[str, Any]:
    """Decode a token sequence + values matrices back into a GraphSpec dict.

    Args:
        token_ids:    integer sequence (parsing stops at EOS).
        cont_values:  float32 array shape (seq_len, cont_width).
        cat_values:   int64 array   shape (seq_len, cat_width).
        vocab:        Vocabulary instance.
        sample_rate, block_size, duration: scalar fields for the output spec.

    Returns:
        A dict compatible with ``rustic_py.render()``.
    """
    ids = list(token_ids)

    # Find the active range (strip SOS/PAD prefix and EOS/PAD suffix)
    start = 0
    while start < len(ids) and ids[start] in (vocab.sos, vocab.pad):
        start += 1
    end = len(ids)
    while end > start and ids[end - 1] in (vocab.eos, vocab.pad):
        end -= 1

    spec: dict[str, Any] = {
        "note": 60,
        "note_on": 0.0,
        "note_off": 0.5,
        "duration": duration,
        "sample_rate": sample_rate,
        "block_size": block_size,
        "sources": [],
        "filters": [],
        "connections": [],
    }

    # ── parsing state ─────────────────────────────────────────────────────────
    last_structural: int | None = None

    current_ms:      dict | None = None   # MultiSourceSpec being built
    current_src:     dict | None = None   # SourceSpec being built
    current_filter:  dict | None = None   # filter being built
    current_adsr:    dict | None = None   # ADSR dict being built

    soms_tok   = vocab.tokens[vocab.SOMS_NAME]
    eoms_tok   = vocab.tokens[vocab.EOMS_NAME]
    sosd_tok   = vocab.tokens[vocab.SOSD_NAME]
    eosd_tok   = vocab.tokens[vocab.EOSD_NAME]
    soed_tok   = vocab.tokens[vocab.SOED_NAME]
    eoed_tok   = vocab.tokens[vocab.EOED_NAME]
    sofd_tok   = vocab.tokens[vocab.SOFD_NAME]
    eofd_tok   = vocab.tokens[vocab.EOFD_NAME]
    fr_id_tok  = vocab.tokens[vocab.FR_IDENTITY_NAME]

    def _read_cont(i: int, tok_id: int) -> dict[str, float]:
        raw    = vocab.denormalize_cont(tok_id, cont_values[i])
        layout = vocab.cont_layout.get(tok_id, [])
        return {name: float(raw[j]) for j, name in enumerate(layout)}

    def _read_cat(i: int, tok_id: int) -> dict[str, int]:
        layout = vocab.cat_layout.get(tok_id, [])
        return {name: int(cat_values[i][j]) for j, name in enumerate(layout)}

    for i in range(start, end):
        tok = ids[i]

        # ── <VALS>: decode values for the preceding structural token ──────────
        if tok == vocab.vals_tok:
            if last_structural is None:
                continue

            ls = last_structural

            if ls == vocab.note_tok:
                cont = _read_cont(i, ls)
                cat  = _read_cat(i, ls)
                spec["note"]     = cat.get("note", 60)
                spec["note_on"]  = cont.get("note_on",  0.0)
                spec["note_off"] = cont.get("note_off", 0.5)

            elif ls in (vocab.atk_tok, vocab.dcy_tok, vocab.sus_tok, vocab.rel_tok):
                if current_adsr is not None:
                    cont = _read_cont(i, ls)
                    if ls == vocab.atk_tok:
                        current_adsr["attack"] = [
                            cont.get("atk_dur", 0.01), cont.get("atk_peak", 1.0),
                            cont.get("atk_ct",  0.01), cont.get("atk_cp",  0.0),
                        ]
                    elif ls == vocab.dcy_tok:
                        current_adsr["decay"] = [
                            cont.get("dcy_dur", 0.1),  cont.get("dcy_peak", 0.8),
                            cont.get("dcy_ct",  0.1),  cont.get("dcy_cp",  1.0),
                        ]
                    elif ls == vocab.sus_tok:
                        current_adsr["sustain"] = cont.get("sus_level", 0.8)
                    elif ls == vocab.rel_tok:
                        current_adsr["release"] = [
                            cont.get("rel_dur", 0.2),  cont.get("rel_peak", 0.0),
                            cont.get("rel_ct",  0.0),  cont.get("rel_cp",  0.0),
                        ]

            elif vocab.is_fr_token(ls) and ls != fr_id_tok:
                if current_src is not None:
                    if vocab.cat_layout.get(ls):
                        cat = _read_cat(i, ls)
                        current_src["frequency_relation_value"] = cat.get("harmonic_n", 1)
                    elif vocab.cont_layout.get(ls):
                        cont = _read_cont(i, ls)
                        layout = vocab.cont_layout[ls]
                        if layout:
                            current_src["frequency_relation_value"] = cont[layout[0]]

            elif vocab.is_ft_token(ls) and current_filter is not None:
                cont   = _read_cont(i, ls)
                layout = vocab.cont_layout.get(ls, [])
                current_filter["params"] = {name: cont[name] for name in layout}

            elif vocab.is_cn_token(ls):
                cat = _read_cat(i, ls)
                tok_name = vocab.id_to_token[ls]
                if tok_name == vocab.CN_SOURCE_SINK_NAME:
                    spec["connections"].append(
                        {"SourceSink": {"source": cat.get("cn_src", 0), "sink": 0}}
                    )
                elif tok_name == vocab.CN_SOURCE_FILTER_NAME:
                    spec["connections"].append(
                        {"SourceFilter": {"source": cat.get("cn_src", 0),
                                          "filter": cat.get("cn_flt", 0)}}
                    )
                elif tok_name == vocab.CN_FILTER_FILTER_NAME:
                    spec["connections"].append(
                        {"FilterFilter": {"filter_out": cat.get("cn_flt_out", 0),
                                          "filter_in":  cat.get("cn_flt_in", 0)}}
                    )
                elif tok_name == vocab.CN_FILTER_SINK_NAME:
                    spec["connections"].append(
                        {"FilterSink": {"filter": cat.get("cn_flt", 0), "sink": 0}}
                    )

            last_structural = None
            continue

        # ── structural tokens ─────────────────────────────────────────────────

        if tok == vocab.note_tok:
            last_structural = tok

        elif tok == soms_tok:
            current_ms = {
                "sources": [],
                "base_frequency": 440.0,
                "mix_mode": "Average",
                "glob_ampl": dict(_DEFAULT_ADSR),
            }

        elif tok == eoms_tok:
            if current_ms is not None:
                spec["sources"].append(current_ms)
            current_ms = None

        elif vocab.is_mm_token(tok) and current_ms is not None:
            current_ms["mix_mode"] = vocab.mm_mode(tok)

        elif tok == sosd_tok:
            current_src = {
                "waveform": "sine",
                "frequency_relation": "identity",
                "envelope": dict(_DEFAULT_ADSR),
            }

        elif tok == eosd_tok:
            if current_src is not None and current_ms is not None:
                current_ms["sources"].append(current_src)
            current_src = None

        elif vocab.is_wf_token(tok):
            if current_src is not None:
                current_src["waveform"] = vocab.wf_type_id(tok)

        elif vocab.is_fr_token(tok):
            if current_src is not None:
                current_src["frequency_relation"] = vocab.fr_relation(tok)
            last_structural = tok  # may be followed by VALS

        elif tok == soed_tok:
            current_adsr = dict(_DEFAULT_ADSR)

        elif tok == eoed_tok:
            if current_adsr is not None:
                # Attach ADSR to the innermost open context
                if current_src is not None:
                    current_src["envelope"] = current_adsr
                elif current_ms is not None:
                    current_ms["glob_ampl"] = current_adsr
            current_adsr = None

        elif vocab.is_adsr_segment_token(tok):
            last_structural = tok

        elif tok == sofd_tok:
            current_filter = {"type": "", "params": {}}

        elif tok == eofd_tok:
            if current_filter is not None and current_filter["type"]:
                spec["filters"].append(current_filter)
            current_filter = None

        elif vocab.is_ft_token(tok):
            if current_filter is not None:
                current_filter["type"] = vocab.ft_type_id(tok)
            last_structural = tok

        elif vocab.is_cn_token(tok):
            last_structural = tok

    return spec
