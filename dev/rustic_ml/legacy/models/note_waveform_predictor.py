from __future__ import annotations

import torch
import torch.nn.functional as F
from torch import nn

from rustic_ml.data.encoding import N_NOTES, N_WAVEFORMS, NOTE_MIN, WAVEFORMS, decode_adsr
from rustic_ml.models.base import RusticModel, PreprocessingConfig


class NoteWaveformPredictor(RusticModel):
    """4-layer CNN with dual heads for joint note + waveform classification."""

    _preprocessing = PreprocessingConfig()
    _required_labels = ["note", "waveform"]

    @property
    def preprocessing(self) -> PreprocessingConfig:
        return self._preprocessing

    @property
    def required_labels(self) -> list[str]:
        return self._required_labels

    def __init__(self, n_notes: int = N_NOTES, n_waveforms: int = N_WAVEFORMS):
        super().__init__()
        self.backbone = nn.Sequential(
            nn.Conv2d(1, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.BatchNorm2d(64), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(64, 128, kernel_size=3, padding=1),
            nn.BatchNorm2d(128), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(128, 256, kernel_size=3, padding=1),
            nn.BatchNorm2d(256), nn.ReLU(),
            nn.AdaptiveAvgPool2d((1, 1)),
        )
        self.note_head     = nn.Linear(256, n_notes)
        self.waveform_head = nn.Linear(256, n_waveforms)

        n_params = sum(p.numel() for p in self.parameters() if p.requires_grad)
        print(f"{self.__class__.__name__} - {n_params:,} trainable parameters")

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        features = self.backbone(x).flatten(1)
        return self.note_head(features), self.waveform_head(features)

    def compute_loss(self, batch: dict, config) -> dict[str, torch.Tensor]:
        mel = batch["mel"]
        note_gt = batch["note"]
        waveform_gt = batch["waveform"]

        if not isinstance(note_gt, torch.Tensor):
            note_gt = torch.tensor(note_gt, device=mel.device)
        if not isinstance(waveform_gt, torch.Tensor):
            waveform_gt = torch.tensor(waveform_gt, device=mel.device)

        note_gt = note_gt.to(mel.device) - NOTE_MIN  # shift to 0-based
        waveform_gt = waveform_gt.to(mel.device)

        note_logits, wf_logits = self(mel)
        loss_note = F.cross_entropy(note_logits, note_gt)
        loss_wf = F.cross_entropy(wf_logits, waveform_gt)
        total = loss_note + config.lambda_waveform * loss_wf
        return {"total": total, "note": loss_note, "waveform": loss_wf}

    def build_comparison_spec(self, sample: dict, device: torch.device) -> dict | None:
        """Predict note + waveform and return a GraphSpec for re-synthesis.

        Uses ground-truth ADSR from the sample (this model does not predict ADSR).
        """
        from rustic_ml.data.generation import random_spec

        self.eval()
        with torch.no_grad():
            note_logits, wf_logits = self(sample["mel"].unsqueeze(0).to(device))

        note_pred = int(note_logits.argmax(dim=1).item()) + NOTE_MIN
        wf_pred   = WAVEFORMS[int(wf_logits.argmax(dim=1).item())]

        adsr = sample["adsr"].numpy()
        a, d, s, r = decode_adsr(adsr)

        spec = random_spec(waveform=wf_pred)
        spec["note"] = note_pred
        spec["source"]["attack"]  = a
        spec["source"]["decay"]   = d
        spec["source"]["sustain"] = s
        spec["source"]["release"] = r
        return spec
