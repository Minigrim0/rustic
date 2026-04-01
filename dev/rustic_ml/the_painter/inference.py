"""
ThePainter inference and evaluation utilities.

evaluate() is called by ThePainter.ipynb to compare a production model
against a candidate from a recent training run.
"""
# TODO: implement ThePainter evaluation
from __future__ import annotations


def evaluate(model, loader, device) -> dict[str, float]:
    """Run ThePainter on a validation DataLoader and return metrics.

    Metrics returned:
      mel_l1          mean L1 distance between predicted and rendered log-mel
      stft_distance   mean multi-scale STFT L1 distance

    Args:
        model:   ThePainter instance (nn.Module), already on device.
        loader:  DataLoader yielding {"token_ids", "cont_values", "cat_values", "mel"}.
        device:  torch.device

    Returns:
        Dict with keys: mel_l1, stft_distance
    """
    raise NotImplementedError("ThePainter evaluation is not yet implemented")
