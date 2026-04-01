from rustic_ml.legacy.evaluation.inference import (
    accumulate_inference,
    accumulate_note_inference,
    accumulate_adsr_inference,
    accumulate_pipeline_inference,
    accumulate_note_waveform_inference,
)
from rustic_ml.legacy.evaluation.figures import (
    plot_accuracy,
    plot_adsr_accuracy,
    plot_note_accuracy,
    plot_waveform_accuracy,
    compare_audio,
    compare_audio_dual,
    compare_audio_note_waveform,
    log_mel_comparisons,
)
from rustic_ml.legacy.evaluation.comparison import (
    load_registered_model,
    fetch_model_versions,
    compare_models,
    compare_note_models,
    compare_adsr_models,
    compare_note_waveform_models,
)
from rustic_ml.legacy.evaluation.analysis import MetricSpec, analyze_run

__all__ = [
    "accumulate_inference",
    "accumulate_note_inference",
    "accumulate_adsr_inference",
    "accumulate_pipeline_inference",
    "accumulate_note_waveform_inference",
    "plot_accuracy",
    "plot_adsr_accuracy",
    "plot_note_accuracy",
    "plot_waveform_accuracy",
    "compare_audio",
    "compare_audio_dual",
    "compare_audio_note_waveform",
    "log_mel_comparisons",
    "load_registered_model",
    "fetch_model_versions",
    "compare_models",
    "compare_note_models",
    "compare_adsr_models",
    "compare_note_waveform_models",
    "MetricSpec",
    "analyze_run",
]
