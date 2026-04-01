"""
Inference accumulation utilities.

Functions for running models over DataLoaders and collecting
predictions vs ground-truth targets for evaluation.
"""
from __future__ import annotations

import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader

from rustic_ml.legacy.data.encoding import decode_adsr, NOTE_MIN, WAVEFORMS


def accumulate_inference(
    model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run a combined note+ADSR model over a DataLoader.

    Args:
        model:  Model returning (note_logits, adsr_pred).
        loader: DataLoader yielding batch dicts with keys mel, note, adsr.
        device: Device to move mel tensors to.

    Returns:
        Dict with "note" and "adsr" keys, each containing preds/targets arrays.
    """
    model.eval()
    all_note_preds:   list[np.ndarray] = []
    all_note_targets: list[np.ndarray] = []
    all_adsr_preds:   list[np.ndarray] = []
    all_adsr_targets: list[np.ndarray] = []

    with torch.no_grad():
        for raw_batch in loader:
            mel = raw_batch["mel"].to(device)
            note = raw_batch["note"]
            adsr = raw_batch["adsr"]
            note_logits, adsr_pred = model(mel)

            all_note_preds.append(note_logits.argmax(dim=1).cpu().numpy() + NOTE_MIN)
            all_note_targets.append(
                note.numpy() if isinstance(note, torch.Tensor) else np.array(note)
            )
            all_adsr_preds.append(adsr_pred.cpu().numpy())
            all_adsr_targets.append(
                adsr.numpy() if isinstance(adsr, torch.Tensor) else np.array(adsr)
            )

    adsr_preds   = np.concatenate(all_adsr_preds)
    adsr_targets = np.concatenate(all_adsr_targets)

    return {
        "note": {
            "preds":   np.concatenate(all_note_preds),
            "targets": np.concatenate(all_note_targets),
        },
        "adsr": {
            "preds":     adsr_preds,
            "targets":   adsr_targets,
            "preds_s":   np.column_stack([decode_adsr(row) for row in adsr_preds]),
            "targets_s": np.column_stack([decode_adsr(row) for row in adsr_targets]),
        },
    }


def accumulate_note_inference(
    note_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run a note-only model over a DataLoader.

    Args:
        note_model: Model returning note_logits (B, N_NOTES).
        loader:     DataLoader yielding batch dicts.
        device:     Device for inference.

    Returns:
        Dict with "note" key containing preds/targets.
    """
    note_model.eval()
    all_preds:   list[np.ndarray] = []
    all_targets: list[np.ndarray] = []

    with torch.no_grad():
        for raw_batch in loader:
            mel = raw_batch["mel"].to(device)
            note = raw_batch["note"]
            note_logits = note_model(mel)
            all_preds.append(note_logits.argmax(dim=1).cpu().numpy() + NOTE_MIN)
            all_targets.append(
                note.numpy() if isinstance(note, torch.Tensor) else np.array(note)
            )

    return {
        "note": {
            "preds":   np.concatenate(all_preds),
            "targets": np.concatenate(all_targets),
        }
    }


def accumulate_adsr_inference(
    adsr_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
    note_source: nn.Module | None = None,
) -> dict:
    """Run an ADSR model over a DataLoader.

    Args:
        adsr_model:  Model taking (mel, note) and returning adsr_pred (B, 4).
        loader:      DataLoader yielding batch dicts.
        device:      Device for inference.
        note_source: If None, use ground-truth notes. If an nn.Module, use
                     predicted notes from note_source(mel).

    Returns:
        Dict with "adsr" key containing preds/targets in log-space and seconds.
    """
    adsr_model.eval()
    if note_source is not None:
        note_source.eval()

    all_preds:   list[np.ndarray] = []
    all_targets: list[np.ndarray] = []

    with torch.no_grad():
        for raw_batch in loader:
            mel = raw_batch["mel"].to(device)
            note_gt = raw_batch["note"]
            adsr = raw_batch["adsr"]

            if note_source is not None:
                note_logits = note_source(mel)
                note = note_logits.argmax(dim=1) + NOTE_MIN
            else:
                note = note_gt.to(device) if isinstance(note_gt, torch.Tensor) else torch.tensor(note_gt).to(device)

            adsr_pred = adsr_model(mel, note)
            all_preds.append(adsr_pred.cpu().numpy())
            all_targets.append(
                adsr.numpy() if isinstance(adsr, torch.Tensor) else np.array(adsr)
            )

    adsr_preds   = np.concatenate(all_preds)
    adsr_targets = np.concatenate(all_targets)

    return {
        "adsr": {
            "preds":     adsr_preds,
            "targets":   adsr_targets,
            "preds_s":   np.column_stack([decode_adsr(row) for row in adsr_preds]),
            "targets_s": np.column_stack([decode_adsr(row) for row in adsr_targets]),
        }
    }


def accumulate_pipeline_inference(
    note_model: nn.Module,
    adsr_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run the full two-model pipeline over a DataLoader.

    Returns a dict with "note" and "adsr" keys, same shape as
    accumulate_inference().
    """
    note_model.eval()
    adsr_model.eval()

    all_note_preds:   list[np.ndarray] = []
    all_note_targets: list[np.ndarray] = []
    all_adsr_preds:   list[np.ndarray] = []
    all_adsr_targets: list[np.ndarray] = []

    with torch.no_grad():
        for raw_batch in loader:
            mel = raw_batch["mel"].to(device)
            note_gt = raw_batch["note"]
            adsr = raw_batch["adsr"]

            note_logits = note_model(mel)
            note_pred = note_logits.argmax(dim=1) + NOTE_MIN

            adsr_pred = adsr_model(mel, note_pred)

            all_note_preds.append(note_pred.cpu().numpy())
            all_note_targets.append(
                note_gt.numpy() if isinstance(note_gt, torch.Tensor) else np.array(note_gt)
            )
            all_adsr_preds.append(adsr_pred.cpu().numpy())
            all_adsr_targets.append(
                adsr.numpy() if isinstance(adsr, torch.Tensor) else np.array(adsr)
            )

    adsr_preds   = np.concatenate(all_adsr_preds)
    adsr_targets = np.concatenate(all_adsr_targets)

    return {
        "note": {
            "preds":   np.concatenate(all_note_preds),
            "targets": np.concatenate(all_note_targets),
        },
        "adsr": {
            "preds":     adsr_preds,
            "targets":   adsr_targets,
            "preds_s":   np.column_stack([decode_adsr(row) for row in adsr_preds]),
            "targets_s": np.column_stack([decode_adsr(row) for row in adsr_targets]),
        },
    }


def accumulate_note_waveform_inference(
    model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run a NoteWaveformPredictor over a DataLoader.

    Args:
        model:  Model returning (note_logits, waveform_logits).
        loader: DataLoader yielding batch dicts with note and waveform keys.
        device: Device for inference.

    Returns:
        Dict with "note" and "waveform" keys.
    """
    model.eval()
    all_note_preds:       list[np.ndarray] = []
    all_note_targets:     list[np.ndarray] = []
    all_waveform_preds:   list[np.ndarray] = []
    all_waveform_targets: list[np.ndarray] = []

    with torch.no_grad():
        for raw_batch in loader:
            mel = raw_batch["mel"].to(device)
            note = raw_batch["note"]
            waveform = raw_batch["waveform"]

            note_logits, waveform_logits = model(mel)
            all_note_preds.append(note_logits.argmax(dim=1).cpu().numpy() + NOTE_MIN)
            all_note_targets.append(
                note.numpy() if isinstance(note, torch.Tensor) else np.array(note)
            )
            all_waveform_preds.append(waveform_logits.argmax(dim=1).cpu().numpy())
            all_waveform_targets.append(
                waveform.numpy() if isinstance(waveform, torch.Tensor) else np.array(waveform)
            )

    return {
        "note": {
            "preds":   np.concatenate(all_note_preds),
            "targets": np.concatenate(all_note_targets),
        },
        "waveform": {
            "preds":   np.concatenate(all_waveform_preds),
            "targets": np.concatenate(all_waveform_targets),
        },
    }
