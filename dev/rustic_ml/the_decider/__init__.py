"""
TheDecider — Note classifier.

Maps a mel spectrogram to a MIDI note (0–127).
Trained on synthetic data from the full GraphSpec pipeline.
Its prediction is injected as a hard prefix NOTE token in TheOracle's decoder,
removing uncertainty over the most impactful structural decision.

Exports:
  TheDecider   nn.Module subclass
  train        Training entry point (callable from CLI or notebook)
  evaluate     Inference + metrics for the evaluation notebook
"""
from rustic_ml.the_decider.model import TheDecider
from rustic_ml.the_decider.train import train
from rustic_ml.the_decider.inference import evaluate

__all__ = ["TheDecider", "train", "evaluate"]
