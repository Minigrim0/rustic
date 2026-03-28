"""
Flat parameter encode/decode utilities for ADSR and note parameters.
No torch dependency — pure numpy.
"""
import numpy as np

NOTE_MIN = 36
NOTE_MAX = 84
N_NOTES = 49  # NOTE_MAX - NOTE_MIN + 1

ADSR_MIN = 0.001
ADSR_MAX = 2.0

WAVEFORMS = ["sine", "square", "saw", "triangle", "whitenoise", "pinknoise", "blank"]
N_WAVEFORMS = len(WAVEFORMS)


def encode_waveform(waveform: str) -> int:
    """Return the integer index for a waveform name."""
    return WAVEFORMS.index(waveform)


def decode_waveform(idx: int) -> str:
    """Return the waveform name for an integer index."""
    return WAVEFORMS[idx]


def encode_adsr(attack: float, decay: float, sustain: float, release: float) -> np.ndarray:
    """Encode ADSR parameters to a 4-element numpy array.

    Encoding:
        [log(attack), log(decay), sustain, log(release)]

    attack/decay/release are clipped to [ADSR_MIN, ADSR_MAX] before log.
    sustain is kept linear in [0.0, 1.0].
    """
    a = np.log(np.clip(attack, ADSR_MIN, ADSR_MAX))
    d = np.log(np.clip(decay, ADSR_MIN, ADSR_MAX))
    s = float(np.clip(sustain, 0.0, 1.0))
    r = np.log(np.clip(release, ADSR_MIN, ADSR_MAX))
    return np.array([a, d, s, r], dtype=np.float32)


def decode_adsr(encoded: np.ndarray) -> tuple[float, float, float, float]:
    """Decode a 4-element numpy array back to (attack, decay, sustain, release).

    exp() is applied to the log columns; sustain is clipped to [0, 1].
    """
    attack = float(np.exp(encoded[0]))
    decay = float(np.exp(encoded[1]))
    sustain = float(np.clip(encoded[2], 0.0, 1.0))
    release = float(np.exp(encoded[3]))
    return attack, decay, sustain, release


def encode_note(note: int) -> int:
    """Validate and return the note as-is (identity encoding).

    Note must be in range [NOTE_MIN, NOTE_MAX] (inclusive).
    """
    if not (NOTE_MIN <= note <= NOTE_MAX):
        raise ValueError(f"note={note!r} must be in [{NOTE_MIN}, {NOTE_MAX}]")
    return note
