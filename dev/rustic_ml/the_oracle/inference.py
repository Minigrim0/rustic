"""
TheOracle inference and evaluation utilities.

evaluate() is called during training and by TheOracle.ipynb to compare models.

sample_sequences() is used for best-of-N reranking at inference time:
  sequences = sample_sequences(model, mel, vocab, k=16, temperature=1.0)
  ranked    = rerank(sequences, painter, device)   # ThePainter fast pre-filter
  best      = render_and_pick(ranked[:4])           # real Rust renderer final pick
"""
from __future__ import annotations

import numpy as np
import torch
from torch.utils.data import DataLoader

from rustic_ml.autoregressive.tokenizer import sequence_to_spec
from rustic_ml.autoregressive.vocab import Vocabulary
from rustic_ml.legacy.data.generation import render_mel


def evaluate(
    model,
    loader: DataLoader,
    device: torch.device,
    vocab:  Vocabulary,
    max_batches: int = 32,
) -> dict[str, float]:
    """Run TheOracle on a validation DataLoader and return metrics.

    Greedy decoding is used; each predicted sequence is rendered with the Rust
    renderer and compared to the input mel via mel-L1 distance.

    Metrics returned:
      mel_l1          mean L1 distance between rendered output and input log-mel
      valid_fraction  fraction of sequences that decoded to a renderable GraphSpec

    Args:
        model:       TheOracle instance (nn.Module), already on device.
        loader:      DataLoader yielding ar_collate_fn batches.
        device:      torch.device
        vocab:       Vocabulary instance.
        max_batches: Cap evaluation batches to keep it fast during training.

    Returns:
        Dict with keys: mel_l1, valid_fraction
    """
    model.eval()
    mel_l1_sum    = 0.0
    valid_count   = 0
    total_count   = 0

    with torch.no_grad():
        for batch_idx, batch in enumerate(loader):
            if batch_idx >= max_batches:
                break

            mel_batch = batch["mel"]                           # (B, MEL_BINS, T)
            B = mel_batch.size(0)
            total_count += B

            for i in range(B):
                single_mel = mel_batch[i : i + 1].unsqueeze(1).to(device)  # (1,1,MEL,T)
                try:
                    pred_ids, pred_cont, pred_cat = model.greedy_decode(
                        single_mel, vocab, max_len=256
                    )
                    spec = sequence_to_spec(
                        pred_ids,
                        pred_cont.numpy(),
                        pred_cat.numpy(),
                        vocab,
                    )
                    pred_mel = render_mel(spec)                # (MEL_BINS, T')
                    gt_mel   = mel_batch[i].numpy()            # (MEL_BINS, T)

                    # Align time axes for comparison
                    min_t = min(pred_mel.shape[1], gt_mel.shape[1])
                    l1 = float(np.abs(pred_mel[:, :min_t] - gt_mel[:, :min_t]).mean())
                    mel_l1_sum += l1
                    valid_count += 1
                except Exception:
                    pass  # invalid spec or render failure → counted as invalid

    valid_fraction = valid_count / max(total_count, 1)
    mel_l1 = mel_l1_sum / max(valid_count, 1)

    model.train()
    return {
        "mel_l1":         mel_l1,
        "valid_fraction": valid_fraction,
    }


def sample_sequences(
    model,
    mel:         torch.Tensor,
    vocab:       Vocabulary,
    k:           int   = 16,
    temperature: float = 1.0,
    max_len:     int   = 256,
) -> list[tuple[list[int], np.ndarray, np.ndarray]]:
    """Sample K token sequences from TheOracle for a single mel input.

    Args:
        model:       TheOracle instance (eval mode, on device).
        mel:         Log-mel tensor (1, 1, MEL_BINS, T).
        vocab:       Vocabulary instance.
        k:           Number of sequences to sample.
        temperature: Sampling temperature (1.0 = greedy argmax).
        max_len:     Maximum decode steps per sequence.

    Returns:
        List of (token_ids, cont_values, cat_values) tuples, length k.
        cont_values and cat_values are numpy arrays.
    """
    results = []
    for _ in range(k):
        tok_ids, cont, cat = model.greedy_decode(
            mel, vocab, max_len=max_len, temperature=temperature
        )
        results.append((tok_ids, cont.numpy(), cat.numpy()))
    return results
