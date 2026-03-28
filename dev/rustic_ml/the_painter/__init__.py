"""
ThePainter — Surrogate renderer (spec → mel).

Maps a canonical GraphSpec token sequence to a predicted log-mel spectrogram.
Trained purely on synthetic (token_sequence, rendered_mel) pairs.

Roles:
  1. Contrastive alignment: spec encoder embeddings are aligned with the mel
     encoder (CLIP-style) to give TheOracle's encoder timbre-aware conditioning.
  2. Fast reranking: at inference, ThePainter pre-filters TheOracle's K candidate
     sequences before the expensive Rust renderer is invoked.
  3. RL value baseline: ThePainter's predicted mel distance provides a cheap
     reward signal for PPO fine-tuning of TheOracle.

Exports:
  ThePainter   nn.Module subclass
  train        Training entry point (callable from CLI or notebook)
  evaluate     Inference + metrics for the evaluation notebook
"""
from rustic_ml.the_painter.model import ThePainter
from rustic_ml.the_painter.train import train
from rustic_ml.the_painter.inference import evaluate

__all__ = ["ThePainter", "train", "evaluate"]
