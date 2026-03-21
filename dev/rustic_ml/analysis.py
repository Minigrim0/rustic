"""
Training run analysis utilities.

Fetches metrics from MLflow, plots loss curves, and prints diagnostic tips
with concrete hyperparameter suggestions where possible.
"""
from dataclasses import dataclass


# Default mapping from local hyperparameter names to MLflow param keys.
# Override any key by passing a custom param_keys dict to analyze_run().
DEFAULT_PARAM_KEYS: dict[str, str] = {
    "n_epochs":    "epochs",
    "n_samples":   "samples",
    "lambda_adsr": "lambda ADSR",
    "lr":          "lr",
    "batch_size":  "batch_size",
}


@dataclass
class MetricSpec:
    """Descriptor for a single MLflow metric.

    Attributes:
        name:        MLflow metric key, e.g. "train/loss_note".
        split:       Which data split produced this metric ("train" or "val").
        metric_type: Semantic type shared across splits, e.g. "loss_note",
                     "loss_adsr", "accuracy_note". Metrics with the same
                     metric_type are plotted on the same subplot.
    """

    name: str
    split: str
    metric_type: str


def _fetch_run_data(
    metrics: list[MetricSpec],
    run_id: str,
    tracking_uri: str | None,
    param_keys: dict[str, str],
) -> tuple[dict[str, list[tuple[int, float]]], dict[str, float | int | None]]:
    """Fetch metric histories and hyperparameters from MLflow for a single run.

    Args:
        metrics:      List of MetricSpec descriptors to fetch.
        run_id:       MLflow run ID.
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.
        param_keys:   Mapping from local param name to MLflow param key.

    Returns:
        A tuple of:
          - history: dict mapping metric name → list of (step, value) pairs,
            sorted by step.
          - params: dict mapping local param name → parsed value (float or int),
            or None if the key was not found in the run.
    """
    import mlflow

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    client = mlflow.tracking.MlflowClient()

    # Fetch metric histories
    history: dict[str, list[tuple[int, float]]] = {}
    for spec in metrics:
        raw = client.get_metric_history(run_id, spec.name)
        history[spec.name] = sorted((m.step, m.value) for m in raw)

    # Fetch hyperparameters logged to the run
    run = client.get_run(run_id)
    logged = run.data.params  # dict[str, str] — all values are strings in MLflow

    params: dict[str, float | int | None] = {}
    for local_name, mlflow_key in param_keys.items():
        raw_val = logged.get(mlflow_key)
        if raw_val is None:
            params[local_name] = None
            continue
        # Attempt int first, then float
        try:
            params[local_name] = int(raw_val)
        except ValueError:
            try:
                params[local_name] = float(raw_val)
            except ValueError:
                params[local_name] = None

    return history, params


def _plot_loss_curves(
    metrics: list[MetricSpec],
    history: dict[str, list[tuple[int, float]]],
) -> None:
    """Plot loss curves grouped by metric_type, one subplot per type.

    Each subplot shows all splits (train/val) for that metric type overlaid,
    making it easy to spot train/val gaps visually.

    Args:
        metrics: List of MetricSpec descriptors.
        history: Metric histories as returned by _fetch_run_data.
    """
    import matplotlib.pyplot as plt

    # Group metric specs by metric_type
    by_type: dict[str, list[MetricSpec]] = {}
    for spec in metrics:
        by_type.setdefault(spec.metric_type, []).append(spec)

    n_types = len(by_type)
    fig, axes = plt.subplots(1, n_types, figsize=(5 * n_types, 4), squeeze=False)

    for ax, (metric_type, specs) in zip(axes[0], by_type.items()):
        for spec in specs:
            pairs = history.get(spec.name, [])
            if not pairs:
                continue
            steps, values = zip(*pairs)
            ax.plot(steps, values, label=spec.split)

        ax.set_title(metric_type)
        ax.set_xlabel("Epoch")
        ax.set_ylabel("Value")
        ax.legend()

    plt.suptitle("Training run — metric curves")
    plt.tight_layout()
    plt.show()


def _last_value(history: dict[str, list[tuple[int, float]]], name: str) -> float | None:
    """Return the last recorded value for a metric, or None if not present."""
    pairs = history.get(name)
    if not pairs:
        return None
    return pairs[-1][1]


def _print_tips(
    metrics: list[MetricSpec],
    history: dict[str, list[tuple[int, float]]],
    params: dict[str, float | int | None],
) -> None:
    """Analyse metric histories and print diagnostic tips to stdout.

    Checks performed:
      - Overfitting: large gap between train and val for the same metric_type.
      - Still learning: train loss still decreasing at the last epoch.
      - Loss imbalance: one loss type dominates the other in training,
        suggesting LAMBDA_ADSR should be adjusted.

    Where hyperparameters are available, concrete suggested values are computed
    and included in the tip.

    Args:
        metrics: List of MetricSpec descriptors.
        history: Metric histories as returned by _fetch_run_data.
        params:  Hyperparameter values as returned by _fetch_run_data.
    """
    # Index specs by (split, metric_type) for quick lookup
    by_split_type: dict[tuple[str, str], MetricSpec] = {
        (s.split, s.metric_type): s for s in metrics
    }

    # Collect all metric_types that have both a train and val counterpart
    metric_types = {s.metric_type for s in metrics}
    splits = {s.split for s in metrics}

    # --- Overfitting check ---
    # Flag when val is more than 2x train for a loss metric, or train accuracy
    # is significantly higher than val accuracy.
    if "train" in splits and "val" in splits:
        for mt in metric_types:
            train_spec = by_split_type.get(("train", mt))
            val_spec   = by_split_type.get(("val",   mt))
            if train_spec is None or val_spec is None:
                continue

            train_val = _last_value(history, train_spec.name)
            val_val   = _last_value(history, val_spec.name)
            if train_val is None or val_val is None:
                continue

            is_loss     = "loss" in mt
            is_accuracy = "accuracy" in mt

            # For losses: val >> train signals overfitting
            if is_loss and train_val > 0 and val_val > 2 * train_val:
                gap = val_val - train_val
                suggestions = "more data, dropout, or weight decay"
                extra = ""
                n_samples = params.get("n_samples")
                if n_samples is not None:
                    extra = f" (try n_samples={int(n_samples) * 2})"
                print(
                    f"[OVERFITTING] {train_spec.name}={train_val:.4f} vs "
                    f"{val_spec.name}={val_val:.4f} (gap={gap:.4f}) — "
                    f"try: {suggestions}{extra}"
                )

            # For accuracy: train >> val signals overfitting
            if is_accuracy and val_val < train_val * 0.8:
                print(
                    f"[OVERFITTING] {train_spec.name}={train_val:.4f} vs "
                    f"{val_spec.name}={val_val:.4f} — "
                    f"try: more data, dropout, or weight decay"
                )

    # --- Still learning check ---
    # Look at the last 20% of epochs for each train loss metric.
    # If the slope is still negative, training should continue.
    for spec in metrics:
        if spec.split != "train" or "loss" not in spec.metric_type:
            continue
        pairs = history.get(spec.name, [])
        if len(pairs) < 5:
            continue

        tail = pairs[int(len(pairs) * 0.8):]
        steps  = [p[0] for p in tail]
        values = [p[1] for p in tail]

        # Simple linear slope over the tail
        n = len(steps)
        mean_x = sum(steps) / n
        mean_y = sum(values) / n
        slope_num = sum((steps[i] - mean_x) * (values[i] - mean_y) for i in range(n))
        slope_den = sum((steps[i] - mean_x) ** 2 for i in range(n))
        slope = slope_num / slope_den if slope_den != 0 else 0.0

        if slope < -1e-4:
            extra = ""
            n_epochs = params.get("n_epochs")
            if n_epochs is not None:
                # Extrapolate: how many more epochs until slope flattens?
                # Rough estimate: current value / |slope per epoch|
                last_val = values[-1]
                epochs_to_plateau = int(last_val / abs(slope)) if slope != 0 else 0
                suggested = round(float(n_epochs)) + epochs_to_plateau
                extra = f" (try n_epochs={suggested})"
            print(
                f"[STILL LEARNING] {spec.name} slope over last 20% of epochs = "
                f"{slope:.5f} — training should continue{extra}"
            )

    # --- Loss imbalance check ---
    # Compare the last train loss_note and train loss_adsr values.
    # If one is more than 10x the other, LAMBDA_ADSR is likely off.
    train_note_spec = by_split_type.get(("train", "loss_note"))
    train_adsr_spec = by_split_type.get(("train", "loss_adsr"))
    if train_note_spec is not None and train_adsr_spec is not None:
        note_val = _last_value(history, train_note_spec.name)
        adsr_val = _last_value(history, train_adsr_spec.name)
        if note_val is not None and adsr_val is not None and adsr_val > 0 and note_val > 0:
            ratio = note_val / adsr_val
            lam = params.get("lambda_adsr")

            if ratio < 0.1:
                # note loss << adsr loss → ADSR dominates gradient updates
                # decrease lambda to rebalance gradient magnitudes
                extra = ""
                if lam is not None:
                    suggested_lam = round(float(lam) * ratio, 4)
                    extra = f" (try lambda_adsr={suggested_lam})"
                print(
                    f"[LOSS IMBALANCE] {train_note_spec.name}={note_val:.4f} << "
                    f"{train_adsr_spec.name}={adsr_val:.4f} (ratio={ratio:.4f}) — "
                    f"ADSR loss dominates gradient updates; try decreasing LAMBDA_ADSR to rebalance{extra}"
                )
            elif ratio > 10:
                # note loss >> adsr loss → note dominates gradient updates
                # increase lambda to give ADSR more weight
                extra = ""
                if lam is not None:
                    suggested_lam = round(float(lam) * ratio, 4)
                    extra = f" (try lambda_adsr={suggested_lam})"
                print(
                    f"[LOSS IMBALANCE] {train_note_spec.name}={note_val:.4f} >> "
                    f"{train_adsr_spec.name}={adsr_val:.4f} (ratio={ratio:.4f}) — "
                    f"note loss dominates gradient updates; try increasing LAMBDA_ADSR{extra}"
                )


def analyze_run(
    metrics: list[MetricSpec],
    run_id: str | None = None,
    tracking_uri: str | None = None,
    param_keys: dict[str, str] | None = None,
) -> None:
    """Fetch metrics from MLflow, plot loss curves, and print diagnostic tips.

    Args:
        metrics:      List of MetricSpec descriptors defining which metrics to
                      fetch and how to group them.
        run_id:       MLflow run ID to analyze. If None, uses the last active run.
        tracking_uri: MLflow tracking server URI. If None, uses the active URI.
        param_keys:   Mapping from local param name to MLflow param key.
                      Merged with DEFAULT_PARAM_KEYS — only override the keys
                      that differ from the defaults.
    """
    import mlflow

    # Resolve run_id
    if run_id is None:
        last = mlflow.last_active_run()
        if last is None:
            raise ValueError("No active MLflow run found. Pass run_id explicitly.")
        run_id = last.info.run_id
    assert run_id is not None

    # Merge param key overrides with defaults
    resolved_keys = {**DEFAULT_PARAM_KEYS, **(param_keys or {})}

    history, params = _fetch_run_data(metrics, run_id, tracking_uri, resolved_keys)
    _plot_loss_curves(metrics, history)
    _print_tips(metrics, history, params)
