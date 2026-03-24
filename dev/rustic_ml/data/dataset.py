"""
Dataset loading utilities.

NpzDataset wraps .npz batch files as a torch Dataset, returning all available
fields as a dict. prepare_dataloaders handles dataset counts, generates missing
samples, and builds train/val DataLoaders.
"""
from __future__ import annotations

from pathlib import Path

import numpy as np


class NpzDataset:
    """torch.utils.data.Dataset backed by .npz files on disk.

    Returns each sample as a dict with keys:
        mel:      torch.Tensor (1, MEL_BINS, T) float32
        note:     int
        adsr:     torch.Tensor (4,) float32
        waveform: int  (-1 if the file pre-dates waveform labels)
        timing:   torch.Tensor (2,) float32  [note_on, note_off] in seconds
                  (-1, -1 if the file pre-dates timing labels)

    Args:
        source: Directory path or list of .npz file paths.
    """

    def __init__(self, source: str | Path | list):
        if isinstance(source, (str, Path)):
            source = Path(source)
            self._paths = sorted(source.glob("*.npz"))
        else:
            self._paths = [Path(p) for p in source]

        if not self._paths:
            raise ValueError(f"No .npz files found in {source!r}")

        # Build index: list of (file_idx, sample_idx_within_file)
        self._index: list[tuple[int, int]] = []
        self._cache: dict[int, dict] = {}

        for file_idx, path in enumerate(self._paths):
            data = np.load(path)
            n = data["note"].shape[0]
            self._index.extend((file_idx, j) for j in range(n))

    def __len__(self) -> int:
        return len(self._index)

    def __getitem__(self, idx: int) -> dict:
        import torch

        file_idx, sample_idx = self._index[idx]

        if file_idx not in self._cache:
            self._cache = {file_idx: dict(np.load(self._paths[file_idx]))}

        data = self._cache[file_idx]
        mel = data["mel"][sample_idx]    # (MEL_BINS, T)
        note = int(data["note"][sample_idx])
        adsr = data["adsr"][sample_idx]  # (4,)

        waveform = int(data["waveform"][sample_idx]) if "waveform" in data else -1
        timing = (
            data["timing"][sample_idx].astype(np.float32)
            if "timing" in data
            else np.array([-1.0, -1.0], dtype=np.float32)
        )

        return {
            "mel": torch.from_numpy(mel).float().unsqueeze(0),  # (1, MEL_BINS, T)
            "note": note,
            "adsr": torch.from_numpy(adsr).float(),
            "waveform": waveform,
            "timing": torch.from_numpy(timing).float(),
        }


def prepare_dataloaders(
    data_dir: str | Path,
    n_samples: int,
    batch_size_gen: int,
    batch_size: int,
    val_fraction: float = 0.1,
    seed: int | None = None,
) -> tuple:
    """Ensure a dataset exists, then build train/val DataLoaders.

    1. Counts samples already in *data_dir* (*.npz files).
    2. If fewer than *n_samples* exist, generates the missing ones via
       :func:`generate_dataset` (appends, never overwrites).
    3. Randomly selects ``round(n_samples / batch_size_gen)`` batch files,
       splits them into train / val at *val_fraction*, wraps in
       :class:`NpzDataset` and ``DataLoader``.

    Args:
        data_dir:       Directory containing (or to receive) .npz batch files.
        n_samples:      Target total number of samples.
        batch_size_gen: Samples per .npz file used during generation.
        batch_size:     Mini-batch size for the DataLoaders.
        val_fraction:   Fraction of files reserved for validation (default 0.1).
        seed:           Optional seed for the file-selection shuffle.

    Returns:
        (train_loader, val_loader, train_ds, val_ds)
    """
    import random as _random
    import torch
    from torch.utils.data import DataLoader
    from rustic_ml.data.generation import generate_dataset

    data_dir = Path(data_dir)
    data_dir.mkdir(parents=True, exist_ok=True)

    existing_files = sorted(data_dir.glob("*.npz"))
    existing_samples = sum(np.load(p)["note"].shape[0] for p in existing_files)

    if existing_samples < n_samples:
        missing = n_samples - existing_samples
        print(f"Found {existing_samples} samples, generating {missing} more...")
        generate_dataset(
            missing,
            data_dir,
            batch_size=batch_size_gen,
            waveform=None,
            start_batch=len(existing_files),
        )
        existing_files = sorted(data_dir.glob("*.npz"))

    n_files = round(n_samples / batch_size_gen)
    rng = _random.Random(seed)
    selected = rng.sample(existing_files, min(n_files, len(existing_files)))

    split = round(len(selected) * (1 - val_fraction))
    train_ds = NpzDataset(selected[:split])
    val_ds   = NpzDataset(selected[split:])

    train_loader = DataLoader(train_ds, batch_size=batch_size, shuffle=False, num_workers=0)
    val_loader   = DataLoader(val_ds,   batch_size=batch_size, shuffle=False, num_workers=0)

    print(f"Train: {len(train_ds)} samples ({split} files)  |  Val: {len(val_ds)} samples ({len(selected) - split} files)")
    return train_loader, val_loader, train_ds, val_ds
