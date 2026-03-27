"""
TheDecider inference and evaluation utilities.

evaluate() is called by TheDecider.ipynb to compare a production model
against a candidate from a recent training run.
"""
# TODO: implement TheDecider evaluation
from __future__ import annotations


def evaluate(model, loader, device) -> dict[str, float]:
    """Run TheDecider on a validation DataLoader and return metrics.

    Metrics returned:
      top1_accuracy   fraction of samples where argmax == true note
      top5_accuracy   fraction of samples where true note is in top-5 predictions

    Args:
        model:   TheDecider instance (nn.Module), already on device.
        loader:  DataLoader yielding batches with at least {"mel", "note"} keys.
        device:  torch.device

    Returns:
        Dict with keys: top1_accuracy, top5_accuracy
    """
    raise NotImplementedError("TheDecider evaluation is not yet implemented")
