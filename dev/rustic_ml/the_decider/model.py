"""
TheDecider model definition.

A CNN mel encoder with a 128-class softmax head predicting the MIDI note
(0–127) from a log-mel spectrogram.

Architecture (placeholder — to be finalised before first training run):
  - 3× (Conv2d → BatchNorm → ReLU → MaxPool) blocks
  - AdaptiveAvgPool to (1, 1)
  - Dropout
  - Linear(channels, 128)

Input:  (B, 1, MEL_BINS, T) log-mel spectrogram
Output: (B, 128) note logits
"""
# TODO: implement TheDecider architecture
import torch.nn as nn


class TheDecider(nn.Module):
    """Note classifier: mel spectrogram → MIDI note logit vector (128 classes)."""

    def __init__(self) -> None:
        super().__init__()
        raise NotImplementedError("TheDecider is not yet implemented")

    def forward(self, mel):
        raise NotImplementedError
