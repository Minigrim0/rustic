# Re-exports for backward compatibility. Use rustic_ml.data.encoding directly.
from rustic_ml.data.encoding import (  # noqa: F401
    NOTE_MIN, NOTE_MAX, N_NOTES,
    ADSR_MIN, ADSR_MAX,
    WAVEFORMS, N_WAVEFORMS,
    encode_adsr, decode_adsr,
    encode_note,
    encode_waveform, decode_waveform,
)
