"""
rustic_ml.legacy — Old-track code from phases 1–3.

This subpackage contains the original flat-encoding pipeline:
  - data/       flat ADSR/note encoding, single-source dataset generation
  - models/     NotePredictor, ADSRPredictor, NoteWaveformPredictor (simple CNNs)
  - evaluation/ inference accumulators, figures, model comparison
  - cli/        rustic-train / rustic-generate entry points (still functional)
  - config.py   TOML config loader for the above

These are superseded by the autoregressive track (rustic_ml.autoregressive,
rustic_ml.the_decider, rustic_ml.the_painter, rustic_ml.the_oracle) but are
kept here to avoid breaking existing notebooks and MLflow artefacts.

Do not import from this package in new code.
"""
