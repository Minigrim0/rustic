"""
Model comparison utilities for evaluating a freshly trained model against
registered versions in the MLflow Model Registry.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import matplotlib.pyplot as plt
import mlflow
import mlflow.pytorch
import numpy as np
from mlflow.tracking import MlflowClient

if TYPE_CHECKING:
    import torch
    from torch import nn
    from torch.utils.data import DataLoader


def load_registered_model(
    model_name: str,
    device: torch.device,
    version: int | None = None,
    tracking_uri: str | None = None,
) -> nn.Module:
    """Load a model from the MLflow registry and return it in eval mode.

    Args:
        model_name:   Registered model name, e.g. "NotePredictor".
        device:       Device to load the model onto.
        version:      Specific version number to load. If None, loads the latest.
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.

    Returns:
        The loaded model moved to *device* and set to eval mode.

    Raises:
        RuntimeError: If no registered versions of *model_name* exist.
    """
    versions = fetch_model_versions(model_name, tracking_uri)
    if not versions:
        raise RuntimeError(
            f"No registered model '{model_name}' found in the registry."
        )
    if version is None:
        v = max(versions, key=lambda v: int(v.version))
    else:
        matches = [v for v in versions if int(v.version) == version]
        if not matches:
            available = sorted(int(v.version) for v in versions)
            raise RuntimeError(
                f"Version {version} of '{model_name}' not found. "
                f"Available: {available}"
            )
        v = matches[0]
    uri = f"models:/{model_name}/{v.version}"
    print(f"Loading '{model_name}' v{v.version} from registry ...")
    model = mlflow.pytorch.load_model(uri, map_location=device)
    model.to(device).eval()
    print(f"Loaded '{model_name}' v{v.version}")
    return model


def fetch_model_versions(model_name: str, tracking_uri: str | None = None) -> list:
    """Return all registered versions for *model_name* from the MLflow registry.

    Args:
        model_name:   Registered model name in the MLflow registry.
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.

    Returns:
        List of ModelVersion objects, empty if the model does not exist.
    """
    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)
    client = MlflowClient()
    try:
        return client.search_model_versions(f"name='{model_name}'")
    except Exception:
        return []


def compute_comparison_metrics(vals: dict) -> dict:
    """Compute summary metrics from ``accumulate_inference`` output.

    Args:
        vals: Dict returned by ``rustic_ml.evaluation.accumulate_inference``,
              with keys ``"note"`` and ``"adsr"``.

    Returns:
        Dict with keys: ``note_acc``, ``adsr_mae``, ``Attack``, ``Decay``,
        ``Sustain``, ``Release``.
    """
    note_acc = float((vals["note"]["preds"] == vals["note"]["targets"]).mean())
    per_param = np.abs(vals["adsr"]["preds_s"] - vals["adsr"]["targets_s"])  # (4, N)
    return {
        "note_acc": note_acc,
        "adsr_mae": float(per_param.mean()),
        "Attack":   float(per_param[0].mean()),
        "Decay":    float(per_param[1].mean()),
        "Sustain":  float(per_param[2].mean()),
        "Release":  float(per_param[3].mean()),
    }


def display_comparison_table(prev_m: dict, curr_m: dict, prev_version: str) -> None:
    """Print a side-by-side metric table comparing two models.

    Args:
        prev_m:       Metrics dict for the previous (registered) model.
        curr_m:       Metrics dict for the current (freshly trained) model.
        prev_version: Version label shown in the header (e.g. ``"3"``).
    """
    rows = [
        ("note_acc", "Note accuracy",     True),
        ("adsr_mae", "ADSR MAE (s)",      False),
        ("Attack",   "  Attack MAE (s)",  False),
        ("Decay",    "  Decay MAE (s)",   False),
        ("Sustain",  "  Sustain MAE",     False),
        ("Release",  "  Release MAE (s)", False),
    ]
    header = (
        f"{'Metric':<22}  {'Prev (v' + prev_version + ')':<18}"
        f"  {'Current':<18}  {'Delta':>10}  Win"
    )
    print(f"\n{header}")
    print("-" * len(header))
    current_wins = 0
    for key, label, higher_better in rows:
        p, c = prev_m[key], curr_m[key]
        delta = c - p
        win = (delta > 0) == higher_better
        if delta == 0:
            marker = "="
        elif win:
            marker = "NEW"
            current_wins += 1
        else:
            marker = "OLD"
        sign = "+" if delta >= 0 else ""
        print(f"  {label:<20}  {p:<18.4f}  {c:<18.4f}  {sign}{delta:>8.4f}  {marker}")

    total = len(rows)
    print(f"\nCurrent model wins {current_wins}/{total} metrics vs v{prev_version}.")


def display_adsr_scatter(
    prev_vals: dict,
    curr_vals: dict,
    model_name: str,
    prev_version: str,
) -> None:
    """Render a 2×4 grid of ADSR scatter plots: previous (top) vs current (bottom).

    Args:
        prev_vals:    ``accumulate_inference`` output for the previous model.
        curr_vals:    ``accumulate_inference`` output for the current model.
        model_name:   Model name shown in the figure title.
        prev_version: Version label for the previous model row title.
    """
    param_labels = ["Attack", "Decay", "Sustain", "Release"]
    fig, axes = plt.subplots(2, 4, figsize=(20, 8))
    for row_idx, (vals, title) in enumerate([
        (prev_vals, f"Previous v{prev_version}"),
        (curr_vals, "Current (this run)"),
    ]):
        for col, label in enumerate(param_labels):
            ax = axes[row_idx][col]
            t = vals["adsr"]["targets_s"][col]
            p = vals["adsr"]["preds_s"][col]
            ax.scatter(t, p, alpha=0.2, s=8)
            lo, hi = float(t.min()), float(t.max())
            ax.plot([lo, hi], [lo, hi], "r--", linewidth=1)
            ax.set_title(f"{title} — {label}")
            ax.set_xlabel("True (s)")
            ax.set_ylabel("Pred (s)")

    plt.suptitle(
        f"ADSR scatter: '{model_name}' v{prev_version} vs current", fontsize=13
    )
    plt.tight_layout()
    plt.show()


def compare_models(
    model_name: str,
    current_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
    tracking_uri: str | None = None,
) -> None:
    """Compare a freshly trained model against the latest registered version.

    Loads the latest registered version from the MLflow Model Registry,
    runs both models on ``loader``, prints a side-by-side metric table,
    and renders ADSR scatter plots for both.

    Args:
        model_name:    Registered model name in the MLflow registry.
        current_model: The freshly trained in-memory model.
        loader:        Validation DataLoader yielding (mel, note, adsr) batches.
        device:        Device to load and run the previous model on.
        tracking_uri:  MLflow tracking server URI. Uses the active URI if None.
    """
    from rustic_ml.evaluation import accumulate_inference
    from rustic_ml.mlflow_ui import show_registered_models

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    versions = fetch_model_versions(model_name, tracking_uri)
    if not versions:
        print(f"No registered model '{model_name}' found — nothing to compare against.")
        print("Registered models:")
        show_registered_models()
        return

    latest = max(versions, key=lambda v: int(v.version))
    prev_uri = f"models:/{model_name}/{latest.version}"
    print(f"Loading '{model_name}' v{latest.version} from {prev_uri} ...")
    prev_model = mlflow.pytorch.load_model(prev_uri, map_location=device)
    prev_model.to(device).eval()

    print("Accumulating inference on validation set ...")
    curr_vals = accumulate_inference(current_model, loader, device)
    prev_vals = accumulate_inference(prev_model, loader, device)

    prev_m = compute_comparison_metrics(prev_vals)
    curr_m = compute_comparison_metrics(curr_vals)

    display_comparison_table(prev_m, curr_m, str(latest.version))
    display_adsr_scatter(prev_vals, curr_vals, model_name, str(latest.version))


def compare_note_models(
    model_name: str,
    current_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
    tracking_uri: str | None = None,
) -> None:
    """Compare a freshly trained note model against the latest registered version.

    Loads the latest registered version of *model_name*, runs both models on
    *loader* using note-only inference, and prints a note-accuracy comparison
    table (no ADSR metrics, no scatter plots).

    Args:
        model_name:    Registered model name in the MLflow registry.
        current_model: The freshly trained note model.
        loader:        Validation DataLoader yielding (mel, note, adsr) batches.
        device:        Device to load and run the previous model on.
        tracking_uri:  MLflow tracking server URI. Uses the active URI if None.
    """
    from rustic_ml.evaluation import accumulate_note_inference
    from rustic_ml.mlflow_ui import show_registered_models

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    versions = fetch_model_versions(model_name, tracking_uri)
    if not versions:
        print(f"No registered model '{model_name}' found — nothing to compare against.")
        print("Registered models:")
        show_registered_models()
        return

    latest = max(versions, key=lambda v: int(v.version))
    prev_uri = f"models:/{model_name}/{latest.version}"
    print(f"Loading '{model_name}' v{latest.version} from {prev_uri} ...")
    prev_model = mlflow.pytorch.load_model(prev_uri, map_location=device)
    prev_model.to(device).eval()

    print("Accumulating note inference on validation set ...")
    curr_vals = accumulate_note_inference(current_model, loader, device)
    prev_vals = accumulate_note_inference(prev_model, loader, device)

    import numpy as np

    prev_acc = float((prev_vals["note"]["preds"] == prev_vals["note"]["targets"]).mean())
    curr_acc = float((curr_vals["note"]["preds"] == curr_vals["note"]["targets"]).mean())
    delta = curr_acc - prev_acc
    win = "NEW" if delta > 0 else ("=" if delta == 0 else "OLD")
    sign = "+" if delta >= 0 else ""

    header = f"{'Metric':<22}  {'Prev (v' + str(latest.version) + ')':<18}  {'Current':<18}  {'Delta':>10}  Win"
    print(f"\n{header}")
    print("-" * len(header))
    print(f"  {'Note accuracy':<20}  {prev_acc:<18.4f}  {curr_acc:<18.4f}  {sign}{delta:>8.4f}  {win}")
    print(f"\nCurrent model {'wins' if win == 'NEW' else 'loses'} vs v{latest.version}.")


def compare_note_waveform_models(
    model_name: str,
    current_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
    tracking_uri: str | None = None,
) -> None:
    """Compare a freshly trained NoteWaveformPredictor against the latest registered version.

    Loads the latest registered version of *model_name*, runs both models on
    *loader* using note+waveform inference, and prints a comparison table with
    note accuracy and waveform accuracy.

    Args:
        model_name:    Registered model name in the MLflow registry.
        current_model: The freshly trained NoteWaveformPredictor.
        loader:        Validation DataLoader yielding (mel, note, adsr, waveform) batches.
        device:        Device to load and run the previous model on.
        tracking_uri:  MLflow tracking server URI. Uses the active URI if None.
    """
    from rustic_ml.evaluation import accumulate_note_waveform_inference
    from rustic_ml.mlflow_ui import show_registered_models

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    versions = fetch_model_versions(model_name, tracking_uri)
    if not versions:
        print(f"No registered model '{model_name}' found — nothing to compare against.")
        print("Registered models:")
        show_registered_models()
        return

    latest = max(versions, key=lambda v: int(v.version))
    prev_uri = f"models:/{model_name}/{latest.version}"
    print(f"Loading '{model_name}' v{latest.version} from {prev_uri} ...")
    prev_model = mlflow.pytorch.load_model(prev_uri, map_location=device)
    prev_model.to(device).eval()

    print("Accumulating note+waveform inference on validation set ...")
    curr_vals = accumulate_note_waveform_inference(current_model, loader, device)
    prev_vals = accumulate_note_waveform_inference(prev_model, loader, device)

    prev_note_acc     = float((prev_vals["note"]["preds"] == prev_vals["note"]["targets"]).mean())
    curr_note_acc     = float((curr_vals["note"]["preds"] == curr_vals["note"]["targets"]).mean())
    prev_waveform_acc = float((prev_vals["waveform"]["preds"] == prev_vals["waveform"]["targets"]).mean())
    curr_waveform_acc = float((curr_vals["waveform"]["preds"] == curr_vals["waveform"]["targets"]).mean())

    rows = [
        ("Note accuracy",     prev_note_acc,     curr_note_acc,     True),
        ("Waveform accuracy", prev_waveform_acc, curr_waveform_acc, True),
    ]
    header = f"{'Metric':<22}  {'Prev (v' + str(latest.version) + ')':<18}  {'Current':<18}  {'Delta':>10}  Win"
    print(f"\n{header}")
    print("-" * len(header))
    wins = 0
    for label, p, c, higher_better in rows:
        delta = c - p
        win = (delta > 0) == higher_better
        marker = "NEW" if (win and delta != 0) else ("=" if delta == 0 else "OLD")
        if marker == "NEW":
            wins += 1
        sign = "+" if delta >= 0 else ""
        print(f"  {label:<20}  {p:<18.4f}  {c:<18.4f}  {sign}{delta:>8.4f}  {marker}")
    print(f"\nCurrent model wins {wins}/{len(rows)} metrics vs v{latest.version}.")


def compare_adsr_models(
    model_name: str,
    current_model: nn.Module,
    loader: DataLoader,
    device: torch.device,
    note_source: nn.Module | None = None,
    tracking_uri: str | None = None,
) -> None:
    """Compare a freshly trained ADSR model against the latest registered version.

    Loads the latest registered version of *model_name*, runs both models on
    *loader* using the same *note_source* (ensuring a fair comparison), then
    displays an ADSR metric table and scatter plots.

    Args:
        model_name:    Registered model name in the MLflow registry.
        current_model: The freshly trained ADSR model.
        loader:        Validation DataLoader yielding (mel, note, adsr) batches.
        device:        Device to load and run the previous model on.
        note_source:   Note model used to produce predicted notes for both models.
                       Pass None to use ground-truth notes from the loader.
        tracking_uri:  MLflow tracking server URI. Uses the active URI if None.
    """
    from rustic_ml.evaluation import accumulate_adsr_inference
    from rustic_ml.mlflow_ui import show_registered_models

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    versions = fetch_model_versions(model_name, tracking_uri)
    if not versions:
        print(f"No registered model '{model_name}' found — nothing to compare against.")
        print("Registered models:")
        show_registered_models()
        return

    latest = max(versions, key=lambda v: int(v.version))
    prev_uri = f"models:/{model_name}/{latest.version}"
    print(f"Loading '{model_name}' v{latest.version} from {prev_uri} ...")
    prev_model = mlflow.pytorch.load_model(prev_uri, map_location=device)
    prev_model.to(device).eval()

    note_label = "predicted" if note_source is not None else "ground-truth"
    print(f"Accumulating ADSR inference on validation set (notes: {note_label}) ...")
    curr_vals = accumulate_adsr_inference(current_model, loader, device, note_source)
    prev_vals = accumulate_adsr_inference(prev_model,    loader, device, note_source)

    # Build metrics dicts with only ADSR keys
    import numpy as np

    def _adsr_metrics(vals: dict) -> dict:
        per_param = np.abs(vals["adsr"]["preds_s"] - vals["adsr"]["targets_s"])
        return {
            "adsr_mae": float(per_param.mean()),
            "Attack":   float(per_param[0].mean()),
            "Decay":    float(per_param[1].mean()),
            "Sustain":  float(per_param[2].mean()),
            "Release":  float(per_param[3].mean()),
        }

    prev_m = _adsr_metrics(prev_vals)
    curr_m = _adsr_metrics(curr_vals)

    rows = [
        ("adsr_mae", "ADSR MAE (s)",      False),
        ("Attack",   "  Attack MAE (s)",  False),
        ("Decay",    "  Decay MAE (s)",   False),
        ("Sustain",  "  Sustain MAE",     False),
        ("Release",  "  Release MAE (s)", False),
    ]
    header = (
        f"{'Metric':<22}  {'Prev (v' + str(latest.version) + ')':<18}"
        f"  {'Current':<18}  {'Delta':>10}  Win"
    )
    print(f"\n{header}")
    print("-" * len(header))
    current_wins = 0
    for key, label, higher_better in rows:
        p, c = prev_m[key], curr_m[key]
        delta = c - p
        win = (delta > 0) == higher_better
        if delta == 0:
            marker = "="
        elif win:
            marker = "NEW"
            current_wins += 1
        else:
            marker = "OLD"
        sign = "+" if delta >= 0 else ""
        print(f"  {label:<20}  {p:<18.4f}  {c:<18.4f}  {sign}{delta:>8.4f}  {marker}")

    total = len(rows)
    print(f"\nCurrent model wins {current_wins}/{total} metrics vs v{latest.version}.")

    display_adsr_scatter(prev_vals, curr_vals, model_name, str(latest.version))
