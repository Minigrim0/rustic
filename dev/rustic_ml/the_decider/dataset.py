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
    """

    def __init__(
        self,
        n_samples: int,
        cache_dir: str | Path | None = None,
    ) -> None:
        self.n_samples = n_samples
        self.cache_dir = Path(cache_dir) if cache_dir is not None else None

    def __len__(self) -> int:
        return self.n_samples

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
            "mel":  torch.from_numpy(mel),
            "note": torch.tensor(note, dtype=torch.int64),
        }

        if self.cache_dir is not None:
            self.cache_dir.mkdir(parents=True, exist_ok=True)
            path = self.cache_dir / f"decider_{idx:06d}.npz"
            np.savez_compressed(path, mel=mel, note=np.int64(note))

        return sample

    @staticmethod
    def _load(path: Path) -> dict[str, torch.Tensor]:
        data = np.load(path)
        return {
            "mel":  torch.from_numpy(data["mel"]),
            "note": torch.tensor(int(data["note"]), dtype=torch.int64),
        }
