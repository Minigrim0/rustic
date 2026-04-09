"""
Lightweight (mel, note) dataset for TheDecider.

No tokenization or vocabulary needed — note is read directly from the spec.
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
import torch
from torch.utils.data import Dataset

from rustic_ml.legacy.data.generation import render_mel
from rustic_py import GraphSpec


class DeciderDataset(Dataset):
    """On-the-fly or cached dataset of (mel, note) pairs.

    Args:
        n_samples:  Number of samples.
        cache_dir:  Optional directory for pre-generated .npz files.
        max_frames: Fixed time-axis length; mels are padded (zeros) or
                    truncated to this size so batches can be stacked.
    """

    def __init__(
        self,
        n_samples: int,
        cache_dir: str | Path | None = None,
        max_frames: int = 256,
    ) -> None:
        self.n_samples = n_samples
        self.cache_dir = Path(cache_dir) if cache_dir is not None else None
        self.max_frames = max_frames

    def __len__(self) -> int:
        return self.n_samples

    def _fix_length(self, mel: np.ndarray) -> np.ndarray:
        """Pad or truncate mel to self.max_frames along the time axis."""
        t = mel.shape[1]
        if t >= self.max_frames:
            return mel[:, : self.max_frames]
        pad = np.zeros((mel.shape[0], self.max_frames - t), dtype=mel.dtype)
        return np.concatenate([mel, pad], axis=1)

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        if self.cache_dir is not None:
            path = self.cache_dir / f"decider_{idx:06d}.npz"
            if path.exists():
                return self._load(path)

        complexity = float(np.random.uniform(0.0, 0.5))
        spec = GraphSpec.random(complexity=complexity).to_spec()
        note = int(spec["note"])
        mel  = render_mel(spec)

        sample = {
            "mel":  torch.from_numpy(self._fix_length(mel)),
            "note": torch.tensor(note, dtype=torch.int64),
        }

        if self.cache_dir is not None:
            self.cache_dir.mkdir(parents=True, exist_ok=True)
            path = self.cache_dir / f"decider_{idx:06d}.npz"
            np.savez_compressed(path, mel=mel, note=np.int64(note))

        return sample

    def _load(self, path: Path) -> dict[str, torch.Tensor]:
        data = np.load(path)
        return {
            "mel":  torch.from_numpy(self._fix_length(data["mel"])),
            "note": torch.tensor(int(data["note"]), dtype=torch.int64),
        }

