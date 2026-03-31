"""
TheDecider model definition.

A CNN mel encoder with a 128-class softmax head predicting the MIDI note
(0–127) from a log-mel spectrogram.

Architecture:
  N× (Conv2d → BatchNorm → ReLU → MaxPool) blocks, channel widths from `channels`
  AdaptiveAvgPool2d(1, 1)
  Dropout(dropout)
  Linear(channels[-1], 128)

Input:  (B, 1, MEL_BINS, T) log-mel spectrogram
Output: (B, 128) note logits
"""
from __future__ import annotations

import torch.nn as nn


def _conv_block(in_ch: int, out_ch: int) -> nn.Sequential:
    return nn.Sequential(
        nn.Conv2d(in_ch, out_ch, kernel_size=3, padding=1, bias=False),
        nn.BatchNorm2d(out_ch),
        nn.ReLU(inplace=True),
        nn.MaxPool2d(2),
    )


class TheDecider(nn.Module):
    """Note classifier: mel spectrogram → MIDI note logit vector (128 classes)."""

    def __init__(self, channels: list[int] = [32, 64, 128, 256], dropout: float = 0.3) -> None:
        super().__init__()
        blocks = []
        in_ch = 1
        for out_ch in channels:
            blocks.append(_conv_block(in_ch, out_ch))
            in_ch = out_ch
        self.encoder = nn.Sequential(*blocks)
        self.pool = nn.AdaptiveAvgPool2d((1, 1))
        self.head = nn.Sequential(
            nn.Dropout(dropout),
            nn.Linear(channels[-1], 128),
        )

    def forward(self, mel):
        # mel: (B, 1, MEL_BINS, T)
        x = self.encoder(mel)   # (B, 256, H', W')
        x = self.pool(x)        # (B, 256, 1, 1)
        x = x.flatten(1)        # (B, 256)
        return self.head(x)     # (B, 128)
