"""
Evaluation utilities for audio synthesis models.

Functions for accumulating model predictions over a DataLoader, plotting
ADSR scatter plots, and comparing rendered audio between ground truth and
model predictions.
"""
import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Dataset

from .encoding import decode_adsr, NOTE_MIN
from .dataset import random_spec, render_mel


def accumulate_inference(
    model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run the model over a DataLoader and collect predictions vs targets.

    Sets the model to eval mode and disables gradient computation for the
    duration of the loop.

    Args:
        model:  The model to evaluate. Must return (note_logits, adsr_pred).
        loader: DataLoader yielding (mel, note, adsr) batches.
        device: Device to move mel tensors to before inference.

    Returns:
        A dict with two keys, "note" and "adsr", each containing:
          - "preds":     np.ndarray of raw model outputs (note: class indices
                         shifted back by NOTE_MIN; adsr: log-space floats).
          - "targets":   np.ndarray of ground truth values.
          - "preds_s":   (adsr only) np.ndarray shape (4, N) decoded to seconds.
          - "targets_s": (adsr only) np.ndarray shape (4, N) decoded to seconds.
    """
    model.eval()
    all_note_preds:   list[np.ndarray] = []
    all_note_targets: list[np.ndarray] = []
    all_adsr_preds:   list[np.ndarray] = []
    all_adsr_targets: list[np.ndarray] = []

    with torch.no_grad():
        for mel, note, adsr in loader:
            mel = mel.to(device)
            note_logits, adsr_pred = model(mel)

            all_note_preds.append(note_logits.argmax(dim=1).cpu().numpy() + NOTE_MIN)
            all_note_targets.append(note.numpy())
            all_adsr_preds.append(adsr_pred.cpu().numpy())
            all_adsr_targets.append(adsr.numpy())

    adsr_preds   = np.concatenate(all_adsr_preds)    # (N, 4) log-space
    adsr_targets = np.concatenate(all_adsr_targets)  # (N, 4) log-space

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


def plot_accuracy(vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot ADSR scatter plots and print note accuracy.

    Creates a 1×4 figure with predicted vs true scatter plots for each ADSR
    parameter (in seconds). Optionally logs the figure and note accuracy to
    the currently active MLflow run.

    Args:
        vals:           Output of accumulate_inference().
        log_to_mlflow:  If True, log the figure and accuracy metric to MLflow.
                        Requires an active MLflow run.
    """
    import matplotlib.pyplot as plt

    adsr_targets_s = vals["adsr"]["targets_s"]  # (4, N)
    adsr_preds_s   = vals["adsr"]["preds_s"]    # (4, N)
    note_targets   = vals["note"]["targets"]
    note_preds     = vals["note"]["preds"]

    fig, axes = plt.subplots(1, 4, figsize=(16, 4))
    labels = ["Attack", "Decay", "Sustain", "Release"]

    for i, (ax, label) in enumerate(zip(axes, labels)):
        ax.scatter(adsr_targets_s[i], adsr_preds_s[i], alpha=0.3, s=10)
        lim = [float(adsr_targets_s[i].min()), float(adsr_targets_s[i].max())]
        ax.plot(lim, lim, "r--", linewidth=1)
        ax.set_xlabel(f"True {label} (s)")
        ax.set_ylabel(f"Predicted {label} (s)")
        ax.set_title(label)

    plt.suptitle("ADSR predictions vs ground truth")
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_figure(fig, "adsr_scatter.png")

    plt.show()

    note_acc = float((note_preds == note_targets).mean())
    print(f"Note accuracy: {note_acc:.3f}")

    if log_to_mlflow:
        import mlflow
        mlflow.log_metric("val/note_accuracy", note_acc)


def compare_audio(
    model: nn.Module,
    dataset: Dataset,
    device: torch.device,
    sample_idx: int = 0,
    log_to_mlflow: bool = True,
) -> None:
    """Render and display ground truth vs predicted audio for one sample.

    Picks a sample from the dataset, runs the model, decodes the predicted
    note and ADSR back to a GraphSpec, renders both specs, and displays the
    mel spectrograms and playable audio widgets side by side.

    Args:
        model:          The model to evaluate. Must return (note_logits, adsr_pred).
        dataset:        Dataset to sample from. Items must be (mel, note, adsr).
        device:         Device to move the mel tensor to before inference.
        sample_idx:     Index of the sample to use. Defaults to 0.
        log_to_mlflow:  If True, log the mel comparison figure to MLflow.
    """
    import matplotlib.pyplot as plt
    from IPython.display import Audio, display
    from rustic_py.rustic_py import render  # type: ignore[import]

    mel_t, note_true, adsr_true = dataset[sample_idx]

    model.eval()
    with torch.no_grad():
        note_logits, adsr_pred = model(mel_t.unsqueeze(0).to(device))

    note_pred = int(note_logits.argmax(dim=1).item()) + NOTE_MIN
    a, d, s, r = decode_adsr(adsr_pred.cpu().numpy()[0])

    # Build and render predicted spec
    pred_spec = random_spec(waveform="sine")
    pred_spec["note"] = note_pred
    pred_spec["source"]["attack"]  = a
    pred_spec["source"]["decay"]   = d
    pred_spec["source"]["sustain"] = s
    pred_spec["source"]["release"] = r
    pred_mel = render_mel(pred_spec)

    # Plot mel comparison
    fig, axes = plt.subplots(1, 2, figsize=(12, 4))
    axes[0].imshow(mel_t.squeeze().numpy(), aspect="auto", origin="lower", cmap="magma")
    axes[0].set_title(f"Ground truth  note={note_true}")
    axes[1].imshow(pred_mel, aspect="auto", origin="lower", cmap="magma")
    axes[1].set_title(f"Predicted  note={note_pred}")
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_figure(fig, "audio_comparison.png")

    plt.show()

    # Build and render ground truth spec
    a_t, d_t, s_t, r_t = decode_adsr(adsr_true.numpy())
    true_spec = random_spec(waveform="sine")
    true_spec["note"] = int(note_true)
    true_spec["source"]["attack"]  = a_t
    true_spec["source"]["decay"]   = d_t
    true_spec["source"]["sustain"] = s_t
    true_spec["source"]["release"] = r_t

    true_mono = np.mean(render(true_spec), axis=1).astype(np.float32)
    pred_mono = np.mean(render(pred_spec), axis=1).astype(np.float32)

    print(f"Ground truth — note={int(note_true)}  A={a_t:.3f} D={d_t:.3f} S={s_t:.3f} R={r_t:.3f}")
    display(Audio(true_mono, rate=44100))
    print(f"Predicted    — note={note_pred}  A={a:.3f} D={d:.3f} S={s:.3f} R={r:.3f}")
    display(Audio(pred_mono, rate=44100))
