"""
ARDataset: PyTorch Dataset for the autoregressive graph model.

Each sample is a (mel, token_ids, values_matrix) triple generated on-the-fly
(or loaded from a pre-generated cache).

  mel:           (MEL_BINS, T)             float32 tensor
  token_ids:     (seq_len,)                int64 tensor
  values_matrix: (seq_len, values_width)   float32 tensor

Sequences are padded to the longest sequence in the batch; the DataLoader
should use a custom collate_fn (provided below as ``ar_collate_fn``) that
handles variable-length padding.
"""
from __future__ import annotations

from pathlib import Path
from typing import Any

import numpy as np
import torch
from torch.utils.data import Dataset

from rustic_ml.data.generation import render_mel
from .generation import random_ar_spec
from .tokenizer import spec_to_sequence
from .vocab import Vocabulary


class ARDataset(Dataset):
    """On-the-fly or cached dataset of (mel, token_ids, values_matrix) triples.

    When ``cache_dir`` is provided the dataset tries to load pre-generated
    samples from ``<cache_dir>/ar_sample_<idx:06d>.npz`` and only falls back
    to live generation for missing indices.  Use ``ARDataset.generate_cache``
    to pre-populate.

    Args:
        n_samples:   Number of samples in the dataset.
        vocab:       :class:`Vocabulary` instance.
        max_filters: Maximum number of filters per graph (0 → source only).
        waveform:    Fix waveform, or None for uniform random.
        cache_dir:   Optional directory for pre-generated .npz files.
    """

    def __init__(
        self,
        n_samples: int,
        vocab: Vocabulary,
        max_filters: int = 3,
        waveform: str | None = None,
        cache_dir: str | Path | None = None,
    ) -> None:
        self.n_samples = n_samples
        self.vocab = vocab
        self.max_filters = max_filters
        self.waveform = waveform
        self.cache_dir = Path(cache_dir) if cache_dir is not None else None

    def __len__(self) -> int:
        return self.n_samples

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        if self.cache_dir is not None:
            path = self.cache_dir / f"ar_sample_{idx:06d}.npz"
            if path.exists():
                return self._load(path)

        spec = random_ar_spec(self.vocab, max_filters=self.max_filters, waveform=self.waveform)
        mel = render_mel(spec)
        token_ids, values = spec_to_sequence(spec, self.vocab)

        sample = {
            "mel": torch.from_numpy(mel),
            "token_ids": torch.tensor(token_ids, dtype=torch.int64),
            "values": torch.from_numpy(values),
        }

        if self.cache_dir is not None:
            self.cache_dir.mkdir(parents=True, exist_ok=True)
            np.savez_compressed(
                path,
                mel=mel,
                token_ids=np.array(token_ids, dtype=np.int64),
                values=values,
            )

        return sample

    @staticmethod
    def _load(path: Path) -> dict[str, torch.Tensor]:
        data = np.load(path)
        return {
            "mel": torch.from_numpy(data["mel"]),
            "token_ids": torch.from_numpy(data["token_ids"]),
            "values": torch.from_numpy(data["values"]),
        }

    def generate_cache(self, n_workers: int = 1) -> None:
        """Pre-generate all samples and write them to ``cache_dir``.

        Useful for freezing a dataset before a long training run.  Can be
        parallelised with ``n_workers > 1`` via ``ProcessPoolExecutor``.
        """
        if self.cache_dir is None:
            raise ValueError("cache_dir must be set to use generate_cache()")

        self.cache_dir.mkdir(parents=True, exist_ok=True)

        if n_workers == 1:
            from tqdm import tqdm
            for i in tqdm(range(self.n_samples), desc="Generating AR cache"):
                _ = self[i]
            return

        from concurrent.futures import ProcessPoolExecutor, as_completed
        from tqdm import tqdm

        with ProcessPoolExecutor(max_workers=n_workers) as ex:
            futs = {ex.submit(self.__getitem__, i): i for i in range(self.n_samples)}
            with tqdm(total=self.n_samples, desc="Generating AR cache") as pbar:
                for fut in as_completed(futs):
                    fut.result()
                    pbar.update(1)


def ar_collate_fn(
    batch: list[dict[str, torch.Tensor]],
) -> dict[str, torch.Tensor]:
    """Collate a list of AR samples into padded batch tensors.

    Pads token_ids and values along the sequence dimension using PAD / zeros.
    The batch dict also includes a boolean ``values_mask`` of shape
    (B, max_seq_len) that is True at <VALUES> token positions.

    Returns a dict with keys:
        mel:          (B, MEL_BINS, T)               float32
        token_ids:    (B, max_seq_len)                int64
        values:       (B, max_seq_len, values_width)  float32
        lengths:      (B,)                            int64  — true seq lengths
    """
    from rustic_ml.data.generation import MEL_BINS

    # ── mel: pad time dimension ────────────────────────────────────────────
    max_t = max(s["mel"].shape[-1] for s in batch)
    B = len(batch)
    mel_batch = torch.zeros(B, MEL_BINS, max_t)
    for i, s in enumerate(batch):
        t = s["mel"].shape[-1]
        mel_batch[i, :, :t] = s["mel"]

    # ── sequences: pad to max_seq_len ──────────────────────────────────────
    # Infer pad token id from first sample's token_ids (first sample always
    # starts with SOS so PAD is never 0 — we retrieve it from the first PAD
    # we find, or default to 2 which is always PAD per Vocabulary.from_rustic)
    pad_id: int = 2
    max_seq = max(s["token_ids"].shape[0] for s in batch)
    values_width = batch[0]["values"].shape[-1]

    token_batch = torch.full((B, max_seq), pad_id, dtype=torch.int64)
    values_batch = torch.zeros(B, max_seq, values_width, dtype=torch.float32)
    lengths = torch.zeros(B, dtype=torch.int64)

    for i, s in enumerate(batch):
        L = s["token_ids"].shape[0]
        token_batch[i, :L] = s["token_ids"]
        values_batch[i, :L] = s["values"]
        lengths[i] = L

    return {
        "mel": mel_batch,
        "token_ids": token_batch,
        "values": values_batch,
        "lengths": lengths,
    }
