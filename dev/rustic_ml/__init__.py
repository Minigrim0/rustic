"""
rustic_ml — ML toolkit for audio graph synthesis prediction.

Active subpackages (new track):
  training        Trainer, PerceptualLoss, setup utilities
  autoregressive  Token vocabulary, tokenizer, AR dataset
  the_decider     Note classifier (mel → MIDI note)
  the_painter     Surrogate renderer (token sequence → mel)
  the_oracle      Autoregressive graph decoder (mel → GraphSpec token sequence)

Legacy subpackage (phases 1–3, do not use in new code):
  legacy          NotePredictor, ADSRPredictor, NoteWaveformPredictor,
                  flat encoding, old CLI entry points
"""
from rustic_ml import training, autoregressive

__all__ = ["training", "autoregressive"]
