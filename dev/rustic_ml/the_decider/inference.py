"""
TheDecider inference and evaluation utilities.

evaluate() is called by TheDecider.ipynb to compare a production model
against a candidate from a recent training run.
"""
from __future__ import annotations

import torch


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
    if loader is None:
        return {}

    model.eval()
    top1_correct = 0
    top5_correct = 0
    total = 0

    with torch.no_grad():
        for batch in loader:
            mel  = batch["mel"].unsqueeze(1).to(device)   # (B, 1, MEL_BINS, T)
            note = batch["note"].to(device)                # (B,)

            logits = model(mel)                            # (B, 128)
            top5 = logits.topk(5, dim=1).indices          # (B, 5)

            top1_correct += (top5[:, 0] == note).sum().item()
            top5_correct += (top5 == note.unsqueeze(1)).any(dim=1).sum().item()
            total += note.size(0)

    if total == 0:
        return {"top1_accuracy": 0.0, "top5_accuracy": 0.0}

    return {
        "top1_accuracy": top1_correct / total,
        "top5_accuracy": top5_correct / total,
    }
