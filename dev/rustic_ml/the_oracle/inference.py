"""
TheOracle inference and evaluation utilities.

evaluate() is called by TheOracle.ipynb to compare a production model
against a candidate from a recent training run.

Also provides sample_sequences() for best-of-N reranking at inference time:
  sequences = sample_sequences(model, mel, vocab, k=16, temperature=1.0)
  ranked    = rerank(sequences, painter, device)   # ThePainter fast pre-filter
  best      = render_and_pick(ranked[:4])           # real Rust renderer final pick
"""
# TODO: implement TheOracle evaluation and sampling
from __future__ import annotations


def evaluate(model, loader, device) -> dict[str, float]:
    """Run TheOracle on a validation DataLoader and return metrics.

    Greedy decoding is used; each predicted sequence is rendered with the Rust
    renderer and compared to the input mel via multi-scale STFT and mel-L1 distance.

    Metrics returned:
      mel_l1          mean L1 distance between rendered output and input log-mel
      stft_distance   mean multi-scale STFT L1 distance
      valid_fraction  fraction of sequences that decoded to a valid GraphSpec

    Args:
        model:   TheOracle instance (nn.Module), already on device.
        loader:  DataLoader yielding {"mel"} batches (no token targets needed).
        device:  torch.device

    Returns:
        Dict with keys: mel_l1, stft_distance, valid_fraction
    """
    raise NotImplementedError("TheOracle evaluation is not yet implemented")


def sample_sequences(model, mel, vocab, k: int = 16, temperature: float = 1.0) -> list:
    """Sample K token sequences from TheOracle for a single mel input.

    Args:
        model:       TheOracle instance.
        mel:         Log-mel tensor (1, 1, MEL_BINS, T).
        vocab:       Vocabulary instance.
        k:           Number of sequences to sample.
        temperature: Sampling temperature (1.0 = no scaling).

    Returns:
        List of (token_ids, cont_values, cat_values) tuples, length k.
    """
    raise NotImplementedError
