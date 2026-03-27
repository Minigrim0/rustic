"""
Plotting and figure logging utilities.

All plot functions accept a log_to_mlflow flag. When True, figures are logged
to the currently active MLflow run via mlflow.log_figure().

log_mel_comparisons() is the headless equivalent of compare_audio*() — it
generates mel comparison figures for N validation samples and logs them to
MLflow with names that include the model name and registered version.
"""
from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import torch
import torch.nn as nn

from rustic_ml.data.encoding import decode_adsr, NOTE_MIN, WAVEFORMS

if TYPE_CHECKING:
    from torch.utils.data import Dataset
    import torch


def plot_accuracy(vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot ADSR scatter plots and print note accuracy.

    Args:
        vals:          Output of accumulate_inference().
        log_to_mlflow: If True, log figure and accuracy metric to MLflow.
    """
    import matplotlib.pyplot as plt

    adsr_targets_s = vals["adsr"]["targets_s"]
    adsr_preds_s   = vals["adsr"]["preds_s"]
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
        mlflow.log_figure(fig, "figures/adsr_scatter.png")

    plt.show()

    note_acc = float((note_preds == note_targets).mean())
    print(f"Note accuracy: {note_acc:.3f}")

    if log_to_mlflow:
        import mlflow
        mlflow.log_metric("val/note_accuracy", note_acc)


def plot_adsr_accuracy(adsr_vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot ADSR scatter plots for accumulate_adsr_inference() output.

    Args:
        adsr_vals:     Dict with "adsr" key containing preds_s and targets_s.
        log_to_mlflow: If True, log the figure to the active MLflow run.
    """
    import matplotlib.pyplot as plt

    adsr_targets_s = adsr_vals["adsr"]["targets_s"]
    adsr_preds_s   = adsr_vals["adsr"]["preds_s"]

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
        mlflow.log_figure(fig, "figures/adsr_scatter.png")

    plt.show()


def plot_note_accuracy(note_vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot per-note accuracy bar chart.

    Args:
        note_vals:     Dict with "note" key containing preds and targets.
        log_to_mlflow: If True, log the figure and overall accuracy to MLflow.
    """
    import matplotlib.pyplot as plt
    from rustic_ml.data.encoding import NOTE_MIN, NOTE_MAX

    preds   = note_vals["note"]["preds"]
    targets = note_vals["note"]["targets"]

    note_acc = float((preds == targets).mean())
    print(f"Note accuracy: {note_acc:.3f}")

    notes = range(NOTE_MIN, NOTE_MAX + 1)
    per_note = [
        float((preds[targets == n] == n).mean()) if (targets == n).any() else float("nan")
        for n in notes
    ]

    fig, ax = plt.subplots(figsize=(14, 4))
    ax.bar(list(notes), per_note, width=0.8)
    ax.axhline(note_acc, color="r", linestyle="--", linewidth=1, label=f"mean {note_acc:.3f}")
    ax.set_xlabel("Note (MIDI)")
    ax.set_ylabel("Accuracy")
    ax.set_title("Per-note accuracy")
    ax.legend()
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_metric("val/note_accuracy", note_acc)
        mlflow.log_figure(fig, "figures/note_accuracy.png")

    plt.show()


def plot_waveform_accuracy(waveform_vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot per-waveform accuracy bar chart.

    Args:
        waveform_vals: Dict with "waveform" key containing preds and targets.
        log_to_mlflow: If True, log the figure and accuracy metric to MLflow.
    """
    import matplotlib.pyplot as plt

    preds   = waveform_vals["waveform"]["preds"]
    targets = waveform_vals["waveform"]["targets"]

    waveform_acc = float((preds == targets).mean())
    print(f"Waveform accuracy: {waveform_acc:.3f}  (random baseline: {1/len(WAVEFORMS):.3f})")

    per_waveform = [
        float((preds[targets == i] == i).mean()) if (targets == i).any() else float("nan")
        for i in range(len(WAVEFORMS))
    ]

    fig, ax = plt.subplots(figsize=(10, 4))
    ax.bar(WAVEFORMS, per_waveform, width=0.6)
    ax.axhline(waveform_acc, color="r", linestyle="--", linewidth=1, label=f"mean {waveform_acc:.3f}")
    ax.set_xlabel("Waveform")
    ax.set_ylabel("Accuracy")
    ax.set_title("Per-waveform accuracy")
    ax.legend()
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_metric("val/waveform_accuracy", waveform_acc)
        mlflow.log_figure(fig, "figures/waveform_accuracy.png")

    plt.show()


def compare_audio(
    model: nn.Module,
    dataset: "Dataset",
    device: "torch.device",
    sample_idx: int = 0,
    log_to_mlflow: bool = True,
) -> None:
    """Display ground truth vs predicted audio for a combined note+ADSR model.

    Args:
        model:         Model returning (note_logits, adsr_pred).
        dataset:       Dataset yielding batch dicts.
        device:        Device for inference.
        sample_idx:    Index of the sample to use.
        log_to_mlflow: If True, log the mel comparison figure to MLflow.
    """
    import matplotlib.pyplot as plt
    from IPython.display import Audio, display
    from rustic_py.rustic_py import render  # type: ignore[import]
    from rustic_ml.data.generation import random_spec, render_mel

    sample = dataset[sample_idx]
    mel_t, note_true, adsr_true = sample["mel"], sample["note"], sample["adsr"]

    model.eval()
    with torch.no_grad():
        note_logits, adsr_pred = model(mel_t.unsqueeze(0).to(device))

    note_pred = int(note_logits.argmax(dim=1).item()) + NOTE_MIN
    a, d, s, r = decode_adsr(adsr_pred.cpu().numpy()[0])

    pred_spec = random_spec(waveform="sine")
    pred_spec["note"] = note_pred
    pred_spec["source"]["attack"]  = a
    pred_spec["source"]["decay"]   = d
    pred_spec["source"]["sustain"] = s
    pred_spec["source"]["release"] = r
    pred_mel = render_mel(pred_spec)

    fig, axes = plt.subplots(1, 2, figsize=(12, 4))
    axes[0].imshow(mel_t.squeeze().numpy(), aspect="auto", origin="lower", cmap="magma")
    axes[0].set_title(f"Ground truth  note={note_true}")
    axes[1].imshow(pred_mel, aspect="auto", origin="lower", cmap="magma")
    axes[1].set_title(f"Predicted  note={note_pred}")
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_figure(fig, f"figures/mel_compare/sample_{sample_idx}.png")

    plt.show()

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


def compare_audio_dual(
    note_model: nn.Module,
    adsr_model: nn.Module,
    dataset: "Dataset",
    device: "torch.device",
    sample_idx: int = 0,
    log_to_mlflow: bool = True,
) -> None:
    """Display ground truth vs predicted audio for the two-model pipeline.

    Args:
        note_model:    Model returning note_logits.
        adsr_model:    Model returning adsr_pred from (mel, note).
        dataset:       Dataset yielding batch dicts.
        device:        Device for inference.
        sample_idx:    Index of the sample to use.
        log_to_mlflow: If True, log the mel comparison figure to MLflow.
    """
    import matplotlib.pyplot as plt
    from IPython.display import Audio, display
    from rustic_py.rustic_py import render  # type: ignore[import]
    from rustic_ml.data.generation import random_spec, render_mel

    sample = dataset[sample_idx]
    mel_t, note_true, adsr_true = sample["mel"], sample["note"], sample["adsr"]

    note_model.eval()
    adsr_model.eval()
    with torch.no_grad():
        mel_in = mel_t.unsqueeze(0).to(device)
        note_logits = note_model(mel_in)
        note_pred_t = note_logits.argmax(dim=1) + NOTE_MIN
        adsr_pred = adsr_model(mel_in, note_pred_t)

    note_pred = int(note_pred_t.item())
    a, d, s, r = decode_adsr(adsr_pred.cpu().numpy()[0])

    pred_spec = random_spec(waveform="sine")
    pred_spec["note"] = note_pred
    pred_spec["source"]["attack"]  = a
    pred_spec["source"]["decay"]   = d
    pred_spec["source"]["sustain"] = s
    pred_spec["source"]["release"] = r
    pred_mel = render_mel(pred_spec)

    fig, axes = plt.subplots(1, 2, figsize=(12, 4))
    axes[0].imshow(mel_t.squeeze().numpy(), aspect="auto", origin="lower", cmap="magma")
    axes[0].set_title(f"Ground truth  note={note_true}")
    axes[1].imshow(pred_mel, aspect="auto", origin="lower", cmap="magma")
    axes[1].set_title(f"Predicted  note={note_pred}")
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_figure(fig, f"figures/mel_compare/sample_{sample_idx}.png")

    plt.show()

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


def compare_audio_note_waveform(
    model: nn.Module,
    dataset: "Dataset",
    device: "torch.device",
    sample_idx: int = 0,
    log_to_mlflow: bool = True,
) -> None:
    """Display ground truth vs predicted audio for a NoteWaveformPredictor.

    Uses ground-truth ADSR (not predicted) for re-synthesis since this model
    does not predict ADSR.

    Args:
        model:         NoteWaveformPredictor returning (note_logits, waveform_logits).
        dataset:       Dataset yielding batch dicts.
        device:        Device for inference.
        sample_idx:    Index of the sample to use.
        log_to_mlflow: If True, log the mel comparison figure to MLflow.
    """
    import matplotlib.pyplot as plt
    from IPython.display import Audio, display
    from rustic_py.rustic_py import render  # type: ignore[import]
    from rustic_ml.data.generation import random_spec, render_mel

    sample = dataset[sample_idx]
    mel_t = sample["mel"]
    note_true = sample["note"]
    adsr_true = sample["adsr"]
    waveform_true = sample["waveform"]

    model.eval()
    with torch.no_grad():
        note_logits, waveform_logits = model(mel_t.unsqueeze(0).to(device))

    note_pred          = int(note_logits.argmax(dim=1).item()) + NOTE_MIN
    waveform_pred      = int(waveform_logits.argmax(dim=1).item())
    waveform_pred_name = WAVEFORMS[waveform_pred]
    waveform_true_name = WAVEFORMS[int(waveform_true)]

    a, d, s, r = decode_adsr(adsr_true.numpy())
    pred_spec = random_spec(waveform=waveform_pred_name)
    pred_spec["note"] = note_pred
    pred_spec["source"]["attack"]  = a
    pred_spec["source"]["decay"]   = d
    pred_spec["source"]["sustain"] = s
    pred_spec["source"]["release"] = r
    pred_mel = render_mel(pred_spec)

    true_spec = random_spec(waveform=waveform_true_name)
    true_spec["note"] = int(note_true)
    true_spec["source"]["attack"]  = a
    true_spec["source"]["decay"]   = d
    true_spec["source"]["sustain"] = s
    true_spec["source"]["release"] = r

    fig, axes = plt.subplots(1, 2, figsize=(12, 4))
    axes[0].imshow(mel_t.squeeze().numpy(), aspect="auto", origin="lower", cmap="magma")
    axes[0].set_title(f"Ground truth  note={int(note_true)}  wf={waveform_true_name}")
    axes[1].imshow(pred_mel, aspect="auto", origin="lower", cmap="magma")
    axes[1].set_title(f"Predicted  note={note_pred}  wf={waveform_pred_name}")
    plt.tight_layout()

    if log_to_mlflow:
        import mlflow
        mlflow.log_figure(fig, f"figures/mel_compare/sample_{sample_idx}.png")

    plt.show()

    true_mono = np.mean(render(true_spec), axis=1).astype(np.float32)
    pred_mono = np.mean(render(pred_spec), axis=1).astype(np.float32)

    print(f"Ground truth — note={int(note_true)}  waveform={waveform_true_name}")
    display(Audio(true_mono, rate=44100))
    print(f"Predicted    — note={note_pred}  waveform={waveform_pred_name}")
    display(Audio(pred_mono, rate=44100))


def log_mel_comparisons(
    model: nn.Module,
    dataset: "Dataset",
    device: "torch.device",
    model_name: str,
    registered_version: str | None,
    n_samples: int = 4,
) -> None:
    """Log mel comparison figures for N validation samples to MLflow.

    Calls model.build_comparison_spec() to reconstruct a GraphSpec from
    predictions, renders audio via rustic_py, and logs side-by-side mel
    figures. Skipped silently if build_comparison_spec() returns None.

    Artifact naming:
        figures/mel_compare/{ModelName}_current_vs_v{N}_sample_{i}.png
        figures/mel_compare/{ModelName}_current_sample_{i}.png  (no registry)

    Args:
        model:               A RusticModel subclass.
        dataset:             Validation dataset.
        device:              Device for inference.
        model_name:          Class name of the model (e.g. "NoteWaveformPredictor").
        registered_version:  Latest registered version string, or None.
        n_samples:           Number of samples to log.
    """
    import mlflow
    import matplotlib.pyplot as plt
    from rustic_ml.data.generation import render_mel

    version_tag = f"_vs_v{registered_version}" if registered_version else ""

    for i in range(min(n_samples, len(dataset))):
        sample = dataset[i]
        pred_spec = model.build_comparison_spec(sample, device)
        if pred_spec is None:
            return  # model doesn't support comparison

        mel_gt = sample["mel"].squeeze().numpy()  # (MEL_BINS, T)
        pred_mel = render_mel(pred_spec)

        fig, axes = plt.subplots(1, 2, figsize=(12, 4))
        axes[0].imshow(mel_gt, aspect="auto", origin="lower", cmap="magma")
        axes[0].set_title(f"Ground truth — sample {i}")
        axes[1].imshow(pred_mel, aspect="auto", origin="lower", cmap="magma")
        axes[1].set_title(f"{model_name} predicted")
        fig.suptitle(f"{model_name}{version_tag} — sample {i}")
        plt.tight_layout()

        artifact_path = (
            f"figures/mel_compare/"
            f"{model_name}_current{version_tag}_sample_{i}.png"
        )
        mlflow.log_figure(fig, artifact_path)
        plt.close(fig)
