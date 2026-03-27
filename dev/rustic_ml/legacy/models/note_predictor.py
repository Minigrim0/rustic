from __future__ import annotations

import torch
import torch.nn.functional as F
from torch import nn

from rustic_ml.legacy.data.encoding import N_NOTES, NOTE_MIN
from rustic_ml.legacy.models.base import RusticModel, PreprocessingConfig


class NotePredictor(RusticModel):
    """2-layer CNN for MIDI note classification from a mel spectrogram."""

    _preprocessing = PreprocessingConfig()
    _required_labels = ["note"]

    @property
    def preprocessing(self) -> PreprocessingConfig:
        return self._preprocessing

    @property
    def required_labels(self) -> list[str]:
        return self._required_labels

    def __init__(self, n_notes: int = N_NOTES):
        super().__init__()

        self.conv1 = nn.Sequential(
            nn.Conv2d(1, 16, kernel_size=3, padding=1),
            nn.BatchNorm2d(16),
            nn.ReLU(),
            nn.MaxPool2d(2),
        )
        self.conv2 = nn.Sequential(
            nn.Conv2d(16, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2),
        )
        self.pool = nn.AdaptiveAvgPool2d(1)
        self.dropout = nn.Dropout(0.3)
        self.head = nn.Linear(32, n_notes)

        n_params = sum(p.numel() for p in self.parameters() if p.requires_grad)
        print(f"{self.__class__.__name__} - {n_params:,} trainable parameters")

    def forward(self, mel: torch.Tensor) -> torch.Tensor:
        x = self.conv1(mel)
        x = self.conv2(x)
        x = self.pool(x).flatten(start_dim=1)
        return self.head(self.dropout(x))

    def compute_loss(self, batch: dict, config) -> dict[str, torch.Tensor]:
        note_gt = batch["note"]
        if not isinstance(note_gt, torch.Tensor):
            note_gt = torch.tensor(note_gt, device=batch["mel"].device)
        note_gt = note_gt - NOTE_MIN  # shift to 0-based class index
        note_logits = self(batch["mel"])
        loss = F.cross_entropy(note_logits, note_gt)
        return {"total": loss, "note": loss}
