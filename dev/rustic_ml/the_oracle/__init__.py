"""
TheOracle — Autoregressive graph decoder (mel → GraphSpec).

An encoder-decoder transformer that maps a log-mel spectrogram to a complete
canonical GraphSpec token sequence. The sequence fully describes the synthesis
graph: note, multi-source structure, ADSR envelopes, filter chain, DAG connections.

Training pipeline:
  1. Supervised pretraining on canonical synthetic (mel, token_sequence) pairs.
  2. Optional contrastive alignment of the mel encoder with ThePainter's spec encoder.
  3. PPO fine-tuning with perceptual (multi-scale STFT) reward.

At inference, TheOracle samples K candidate sequences; ThePainter pre-filters them;
the best survivor is validated with the Rust renderer.

Exports:
  TheOracle    nn.Module subclass
  train        Training entry point (callable from CLI or notebook)
  evaluate     Inference + metrics for the evaluation notebook
"""
from rustic_ml.the_oracle.model import TheOracle
from rustic_ml.the_oracle.train import train
from rustic_ml.the_oracle.inference import evaluate

__all__ = ["TheOracle", "train", "evaluate"]
