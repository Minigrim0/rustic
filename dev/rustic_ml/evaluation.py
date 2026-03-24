# Re-exports for backward compatibility. Use rustic_ml.evaluation directly.
from rustic_ml.evaluation.inference import (  # noqa: F401
    accumulate_inference,
    accumulate_note_inference,
    accumulate_adsr_inference,
    accumulate_pipeline_inference,
    accumulate_note_waveform_inference,
)
from rustic_ml.evaluation.figures import (  # noqa: F401
    plot_accuracy,
    plot_adsr_accuracy,
    plot_note_accuracy,
    plot_waveform_accuracy,
    compare_audio,
    compare_audio_dual,
    compare_audio_note_waveform,
)
