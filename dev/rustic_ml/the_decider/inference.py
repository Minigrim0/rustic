"""
TheDecider inference and evaluation utilities.

evaluate() is called by TheDecider.ipynb to compare a production model
against a candidate from a recent training run.
"""
from __future__ import annotations

import numpy as np
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


def evaluate_detailed(model, loader, device) -> dict:
    """Run TheDecider and return per-note accuracy and confusion matrix.

    Returns:
        Dict with keys:
          per_note_top1  np.ndarray (128,) top-1 accuracy per MIDI note
          per_note_top5  np.ndarray (128,) top-5 accuracy per MIDI note
          note_counts    np.ndarray (128,) validation samples per note
          confusion      np.ndarray (128, 128) raw counts, confusion[true, pred]
    """
    if loader is None:
        return {}

    model.eval()
    note_top1   = np.zeros(128, dtype=np.int64)
    note_top5   = np.zeros(128, dtype=np.int64)
    note_counts = np.zeros(128, dtype=np.int64)
    confusion   = np.zeros((128, 128), dtype=np.int64)

    with torch.no_grad():
        for batch in loader:
            mel  = batch["mel"].unsqueeze(1).to(device)
            note = batch["note"].to(device)

            logits = model(mel)
            top5   = logits.topk(5, dim=1).indices

            note_np = note.cpu().numpy()
            top5_np = top5.cpu().numpy()
            pred_np = top5_np[:, 0]

            np.add.at(note_counts, note_np, 1)
            np.add.at(note_top1, note_np, (pred_np == note_np).astype(np.int64))
            np.add.at(note_top5, note_np, (top5_np == note_np[:, None]).any(axis=1).astype(np.int64))
            np.add.at(confusion, (note_np, pred_np), 1)

    safe = np.where(note_counts > 0, note_counts, 1)
    return {
        "per_note_top1": note_top1 / safe,
        "per_note_top5": note_top5 / safe,
        "note_counts":   note_counts,
        "confusion":     confusion,
    }
