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

from .encoding import decode_adsr, NOTE_MIN, WAVEFORMS
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


def accumulate_note_inference(
    note_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run a note-only model over a DataLoader and collect predictions vs targets.

    Args:
        note_model: Model that takes mel and returns note_logits (B, N_NOTES).
        loader:     DataLoader yielding (mel, note, adsr) batches.
        device:     Device to move mel tensors to before inference.

    Returns:
        Dict with a single "note" key containing:
          - "preds":   np.ndarray of predicted note values (class index + NOTE_MIN).
          - "targets": np.ndarray of ground truth note values.
    """
    note_model.eval()
    all_preds:   list[np.ndarray] = []
    all_targets: list[np.ndarray] = []

    with torch.no_grad():
        for mel, note, _adsr in loader:
            mel = mel.to(device)
            note_logits = note_model(mel)
            all_preds.append(note_logits.argmax(dim=1).cpu().numpy() + NOTE_MIN)
            all_targets.append(note.numpy())

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
    """Run an ADSR model over a DataLoader and collect predictions vs targets.

    The ADSR model is expected to take (mel, note) as inputs. The note can
    come from either ground-truth labels (training-time eval) or a separate
    note model (inference-time eval).

    Args:
        adsr_model:  Model that takes (mel, note) and returns adsr_pred (B, 4).
        loader:      DataLoader yielding (mel, note, adsr) batches.
        device:      Device to move tensors to before inference.
        note_source: If None, use ground-truth notes from the loader.
                     If an nn.Module, run note_source(mel) to get predicted notes
                     (argmax + NOTE_MIN, moved back to the adsr_model's device).

    Returns:
        Dict with a single "adsr" key containing:
          - "preds":     np.ndarray (N, 4) log-space predictions.
          - "targets":   np.ndarray (N, 4) log-space ground truth.
          - "preds_s":   np.ndarray (4, N) decoded to seconds.
          - "targets_s": np.ndarray (4, N) decoded to seconds.
    """
    adsr_model.eval()
    if note_source is not None:
        note_source.eval()

    all_preds:   list[np.ndarray] = []
    all_targets: list[np.ndarray] = []

    with torch.no_grad():
        for mel, note_gt, adsr in loader:
            mel = mel.to(device)
            if note_source is not None:
                note_logits = note_source(mel)
                note = note_logits.argmax(dim=1) + NOTE_MIN
            else:
                note = note_gt.to(device)

            adsr_pred = adsr_model(mel, note)
            all_preds.append(adsr_pred.cpu().numpy())
            all_targets.append(adsr.numpy())

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

    Equivalent to chaining accumulate_note_inference and
    accumulate_adsr_inference with note_source=note_model. The returned dict
    has the same shape as accumulate_inference(), so plot_accuracy() and
    display_comparison_table() work unchanged.

    Args:
        note_model: Model that takes mel and returns note_logits (B, N_NOTES).
        adsr_model: Model that takes (mel, note) and returns adsr_pred (B, 4).
        loader:     DataLoader yielding (mel, note, adsr) batches.
        device:     Device to move tensors to before inference.

    Returns:
        Combined dict with "note" and "adsr" keys — same structure as
        accumulate_inference().
    """
    note_model.eval()
    adsr_model.eval()

    all_note_preds:   list[np.ndarray] = []
    all_note_targets: list[np.ndarray] = []
    all_adsr_preds:   list[np.ndarray] = []
    all_adsr_targets: list[np.ndarray] = []

    with torch.no_grad():
        for mel, note_gt, adsr in loader:
            mel = mel.to(device)

            note_logits = note_model(mel)
            note_pred = note_logits.argmax(dim=1) + NOTE_MIN

            adsr_pred = adsr_model(mel, note_pred)

            all_note_preds.append(note_pred.cpu().numpy())
            all_note_targets.append(note_gt.numpy())
            all_adsr_preds.append(adsr_pred.cpu().numpy())
            all_adsr_targets.append(adsr.numpy())

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


def plot_adsr_accuracy(adsr_vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot ADSR scatter plots for a dict returned by accumulate_adsr_inference.

    Like plot_accuracy() but limited to the ADSR component — useful when
    evaluating the ADSR model in isolation (no note accuracy to report).

    Args:
        adsr_vals:      Output of accumulate_adsr_inference(), or any dict with
                        an "adsr" key of the same structure.
        log_to_mlflow:  If True, log the figure to the active MLflow run.
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
        mlflow.log_figure(fig, "adsr_scatter.png")

    plt.show()


def plot_note_accuracy(note_vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot per-note accuracy and print overall accuracy for a note-only model.

    Creates a bar chart with one bar per note class showing the fraction of
    correct predictions for that note, making pitch-specific weaknesses visible.

    Args:
        note_vals:      Output of accumulate_note_inference(), or any dict with
                        a "note" key of the same structure.
        log_to_mlflow:  If True, log the figure and overall accuracy to MLflow.
    """
    import matplotlib.pyplot as plt

    preds   = note_vals["note"]["preds"]
    targets = note_vals["note"]["targets"]

    note_acc = float((preds == targets).mean())
    print(f"Note accuracy: {note_acc:.3f}")

    # Per-note accuracy
    from .encoding import NOTE_MIN, NOTE_MAX
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
        mlflow.log_figure(fig, "note_accuracy.png")

    plt.show()


def accumulate_note_waveform_inference(
    model: nn.Module,
    loader: DataLoader,
    device: torch.device,
) -> dict:
    """Run a NoteWaveformPredictor over a DataLoader and collect predictions vs targets.

    Args:
        model:  Model that takes mel and returns (note_logits, waveform_logits).
        loader: DataLoader yielding (mel, note, adsr, waveform) batches.
        device: Device to move mel tensors to before inference.

    Returns:
        Dict with "note" and "waveform" keys, each containing:
          - "preds":   np.ndarray of predicted class indices.
          - "targets": np.ndarray of ground truth class indices.
        Note preds are shifted by NOTE_MIN to recover MIDI note values.
    """
    model.eval()
    all_note_preds:      list[np.ndarray] = []
    all_note_targets:    list[np.ndarray] = []
    all_waveform_preds:  list[np.ndarray] = []
    all_waveform_targets: list[np.ndarray] = []

    with torch.no_grad():
        for mel, note, _adsr, waveform in loader:
            mel = mel.to(device)
            note_logits, waveform_logits = model(mel)
            all_note_preds.append(note_logits.argmax(dim=1).cpu().numpy() + NOTE_MIN)
            all_note_targets.append(note.numpy() if isinstance(note, torch.Tensor) else np.array(note))
            all_waveform_preds.append(waveform_logits.argmax(dim=1).cpu().numpy())
            all_waveform_targets.append(waveform.numpy() if isinstance(waveform, torch.Tensor) else np.array(waveform))

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


def plot_waveform_accuracy(waveform_vals: dict, log_to_mlflow: bool = True) -> None:
    """Plot per-waveform accuracy and print overall accuracy.

    Creates a bar chart with one bar per waveform class showing the fraction of
    correct predictions.

    Args:
        waveform_vals:  Output of accumulate_note_waveform_inference(), or any
                        dict with a "waveform" key of the same structure.
        log_to_mlflow:  If True, log the figure and accuracy metric to MLflow.
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
        mlflow.log_figure(fig, "waveform_accuracy.png")

    plt.show()


def compare_audio_note_waveform(
    model: nn.Module,
    dataset,
    device: torch.device,
    sample_idx: int = 0,
    log_to_mlflow: bool = True,
) -> None:
    """Render and display ground truth vs predicted audio for a NoteWaveformPredictor.

    Picks a sample, predicts note and waveform, renders both using ground-truth
    ADSR (since this model does not predict ADSR), and displays mel spectrograms
    and playable audio widgets.

    Args:
        model:          NoteWaveformPredictor returning (note_logits, waveform_logits).
        dataset:        Dataset with items (mel, note, adsr, waveform).
        device:         Device to move the mel tensor to before inference.
        sample_idx:     Index of the sample to use. Defaults to 0.
        log_to_mlflow:  If True, log the mel comparison figure to MLflow.
    """
    import matplotlib.pyplot as plt
    from IPython.display import Audio, display
    from rustic_py.rustic_py import render  # type: ignore[import]

    mel_t, note_true, adsr_true, waveform_true = dataset[sample_idx]

    model.eval()
    with torch.no_grad():
        note_logits, waveform_logits = model(mel_t.unsqueeze(0).to(device))

    note_pred     = int(note_logits.argmax(dim=1).item()) + NOTE_MIN
    waveform_pred = int(waveform_logits.argmax(dim=1).item())
    waveform_pred_name = WAVEFORMS[waveform_pred]
    waveform_true_name = WAVEFORMS[int(waveform_true)]

    # Render predicted: use predicted note + waveform, ground-truth ADSR
    a, d, s, r = decode_adsr(adsr_true.numpy())
    pred_spec = random_spec(waveform=waveform_pred_name)
    pred_spec["note"] = note_pred
    pred_spec["source"]["attack"]  = a
    pred_spec["source"]["decay"]   = d
    pred_spec["source"]["sustain"] = s
    pred_spec["source"]["release"] = r
    pred_mel = render_mel(pred_spec)

    # Render ground truth
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
        mlflow.log_figure(fig, "audio_comparison.png")

    plt.show()

    true_mono = np.mean(render(true_spec), axis=1).astype(np.float32)
    pred_mono = np.mean(render(pred_spec), axis=1).astype(np.float32)

    print(f"Ground truth — note={int(note_true)}  waveform={waveform_true_name}")
    display(Audio(true_mono, rate=44100))
    print(f"Predicted    — note={note_pred}  waveform={waveform_pred_name}")
    display(Audio(pred_mono, rate=44100))


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


def compare_audio_dual(
    note_model: nn.Module,
    adsr_model: nn.Module,
    dataset: Dataset,
    device: torch.device,
    sample_idx: int = 0,
    log_to_mlflow: bool = True,
) -> None:
    """Render and display ground truth vs predicted audio using the two-model pipeline.

    Picks a sample from the dataset, runs note_model(mel) → note_pred, then
    adsr_model(mel, note_pred) → adsr_pred, decodes both, renders audio, and
    displays mel spectrograms and playable audio widgets side by side.

    Args:
        note_model:    Model that takes mel and returns note_logits.
        adsr_model:    Model that takes (mel, note) and returns adsr_pred.
        dataset:       Dataset to sample from. Items must be (mel, note, adsr).
        device:        Device to move tensors to before inference.
        sample_idx:    Index of the sample to use. Defaults to 0.
        log_to_mlflow: If True, log the mel comparison figure to MLflow.
    """
    import matplotlib.pyplot as plt
    from IPython.display import Audio, display
    from rustic_py.rustic_py import render  # type: ignore[import]

    mel_t, note_true, adsr_true = dataset[sample_idx]

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
        mlflow.log_figure(fig, "audio_comparison.png")

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
