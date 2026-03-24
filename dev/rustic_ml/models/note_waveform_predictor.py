import torch
from torch import nn

from rustic_ml.encoding import N_NOTES, N_WAVEFORMS


class NoteWaveformPredictor(nn.Module):
    """
    A model to predict Note & Waveform
    """

    def __init__(self, n_notes: int = N_NOTES, n_waveforms: int = N_WAVEFORMS):
        super().__init__()
        self.backbone = nn.Sequential(
            # Block 1: 1 → 32
            nn.Conv2d(1, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            # Block 2: 32 → 64
            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.BatchNorm2d(64), nn.ReLU(), nn.MaxPool2d(2),
            # Block 3: 64 → 128
            nn.Conv2d(64, 128, kernel_size=3, padding=1),
            nn.BatchNorm2d(128), nn.ReLU(), nn.MaxPool2d(2),
            # Block 4: 128 → 256
            nn.Conv2d(128, 256, kernel_size=3, padding=1),
            nn.BatchNorm2d(256), nn.ReLU(),
            nn.AdaptiveAvgPool2d((1, 1)),
        )
        self.note_head     = nn.Linear(256, n_notes)
        self.waveform_head = nn.Linear(256, n_waveforms)

        n_params = sum(p.numel() for p in self.parameters() if p.requires_grad)
        print(f"{self.__class__.__name__} - {n_params:,} trainable parameters")

    def forward(self, x: torch.Tensor):
        features = self.backbone(x).flatten(1)
        return self.note_head(features), self.waveform_head(features)

