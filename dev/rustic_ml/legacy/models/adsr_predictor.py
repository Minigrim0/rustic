from __future__ import annotations

import torch
import torch.nn.functional as F
from torch import nn

from rustic_ml.legacy.data.encoding import NOTE_MIN, N_NOTES
from rustic_ml.legacy.models.base import RusticModel, PreprocessingConfig


class ADSRPredictor(RusticModel):
    """3-layer CNN for ADSR regression, conditioned on note via embedding."""

    _preprocessing = PreprocessingConfig()
    _required_labels = ["note", "adsr"]

    @property
    def preprocessing(self) -> PreprocessingConfig:
        return self._preprocessing

    @property
    def required_labels(self) -> list[str]:
        return self._required_labels

    def __init__(self, n_notes: int = N_NOTES, note_embed_dim: int = 16):
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
        self.conv3 = nn.Sequential(
            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.MaxPool2d(2),
        )
        self.pool = nn.AdaptiveAvgPool2d(1)
        self.note_embed = nn.Embedding(n_notes, note_embed_dim)
        self.dropout = nn.Dropout(0.3)
        self.head = nn.Linear(64 + note_embed_dim, 4)

        n_params = sum(p.numel() for p in self.parameters() if p.requires_grad)
        print(f"{self.__class__.__name__} - {n_params:,} trainable parameters")

    def forward(self, mel: torch.Tensor, note: torch.Tensor, note_min: int = NOTE_MIN) -> torch.Tensor:
        x = self.conv1(mel)
        x = self.conv2(x)
        x = self.conv3(x)
        x = self.pool(x).flatten(start_dim=1)

        e = self.note_embed(note - note_min)
        x = torch.cat([x, e], dim=1)
        x = self.dropout(x)

        out = self.head(x)
        out = out.clone()
        out[:, 2] = torch.sigmoid(out[:, 2])
        return out

    def compute_loss(self, batch: dict, config) -> dict[str, torch.Tensor]:
        mel = batch["mel"]
        note = batch["note"]
        adsr_gt = batch["adsr"]

        if not isinstance(note, torch.Tensor):
            note = torch.tensor(note, device=mel.device)
        note = note.to(mel.device)

        adsr_pred = self(mel, note)
        loss = F.mse_loss(adsr_pred, adsr_gt)
        return {"total": loss, "adsr": loss}
