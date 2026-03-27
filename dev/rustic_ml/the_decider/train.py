"""
TheDecider training entry point.

Callable from both the CLI (via rustic-train-decider) and directly from a
notebook cell:

    from rustic_ml.the_decider.train import train
    train(config, run_name="my_run")

Config keys used (TOML or dict):
  run.mlflow_uri       MLflow tracking URI
  run.experiment       MLflow experiment name  (default: "TheDecider")
  data.data_dir        Directory for cached dataset .npz files
  data.n_samples       Number of training samples to generate
  data.val_split       Fraction held out for validation
  training.batch_size
  training.n_epochs
  training.lr
  training.seed
  training.dropout
"""
from __future__ import annotations

import argparse
import random

import mlflow
import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Subset
from tqdm import tqdm

from rustic_ml.the_decider.dataset import DeciderDataset
from rustic_ml.the_decider.model import TheDecider
from rustic_ml.the_decider.inference import evaluate
from rustic_ml.training.setup import setup_mlflow, setup_device


def _nested_get(cfg: dict, *keys, default=None):
    for k in keys:
        if not isinstance(cfg, dict):
            return default
        cfg = cfg.get(k, default)
    return cfg


def train(config: dict, run_name: str | None = None) -> dict[str, float]:
    """Train TheDecider and log the run to MLflow.

    Args:
        config:   Flat or nested config dict (mirrors the TOML structure).
        run_name: Optional MLflow run name override.

    Returns:
        Final validation metrics dict.
    """
    # ── Hyper-parameters ─────────────────────────────────────────────────────
    mlflow_uri = _nested_get(config, "run", "mlflow_uri", default="http://192.168.1.254:5000")
    experiment = _nested_get(config, "run", "experiment",  default="TheDecider")
    data_dir   = _nested_get(config, "data", "data_dir",   default=None)
    n_samples  = _nested_get(config, "data", "n_samples",  default=10_000)
    val_split  = _nested_get(config, "data", "val_split",  default=0.1)
    batch_size = _nested_get(config, "training", "batch_size", default=64)
    n_epochs   = _nested_get(config, "training", "n_epochs",   default=40)
    lr         = _nested_get(config, "training", "lr",          default=1e-3)
    seed       = _nested_get(config, "training", "seed",        default=42)
    dropout    = _nested_get(config, "training", "dropout",     default=0.3)

    # ── Reproducibility ───────────────────────────────────────────────────────
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)

    device = setup_device()

    # ── Dataset ───────────────────────────────────────────────────────────────
    dataset = DeciderDataset(n_samples=n_samples, cache_dir=data_dir)

    n_val   = max(1, int(n_samples * val_split))
    n_train = n_samples - n_val
    indices = list(range(n_samples))
    random.shuffle(indices)
    train_ds = Subset(dataset, indices[:n_train])
    val_ds   = Subset(dataset, indices[n_train:])

    train_loader = DataLoader(
        train_ds, batch_size=batch_size, shuffle=True, num_workers=2, pin_memory=True,
    )
    val_loader = DataLoader(
        val_ds, batch_size=batch_size, shuffle=False, num_workers=2,
    )

    # ── Model ─────────────────────────────────────────────────────────────────
    model = TheDecider(dropout=dropout).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=lr)
    criterion = nn.CrossEntropyLoss()

    # ── MLflow ────────────────────────────────────────────────────────────────
    setup_mlflow(mlflow_uri, experiment)
    val_metrics: dict[str, float] = {}
    with mlflow.start_run(run_name=run_name) as run:
        mlflow.log_params({
            "n_samples": n_samples,
            "val_split": val_split,
            "batch_size": batch_size,
            "n_epochs": n_epochs,
            "lr": lr,
            "seed": seed,
            "dropout": dropout,
        })

        for epoch in range(1, n_epochs + 1):
            model.train()
            train_loss = 0.0
            with tqdm(train_loader, desc=f"Epoch {epoch:3d}/{n_epochs}", unit="batch", leave=False) as pbar:
                for batch in pbar:
                    mel  = batch["mel"].unsqueeze(1).to(device)  # (B, 1, MEL_BINS, T)
                    note = batch["note"].to(device)               # (B,)

                    optimizer.zero_grad()
                    loss = criterion(model(mel), note)
                    loss.backward()
                    optimizer.step()

                    train_loss += loss.item() * note.size(0)
                    pbar.set_postfix(loss=f"{loss.item():.4f}")

            train_loss /= n_train
            val_metrics = evaluate(model, val_loader, device)

            mlflow.log_metrics(
                {"train_loss": train_loss, **val_metrics},
                step=epoch,
            )
            print(
                f"Epoch {epoch:3d}/{n_epochs}  "
                f"loss={train_loss:.4f}  "
                f"top1={val_metrics.get('top1_accuracy', 0):.3f}  "
                f"top5={val_metrics.get('top5_accuracy', 0):.3f}"
            )

        mlflow.pytorch.log_model(model, "model")
        print(f"Run saved: {run.info.run_id}")

    return val_metrics


def main() -> None:
    """CLI entry point: rustic-train-decider."""
    parser = argparse.ArgumentParser(description="Train TheDecider (note classifier)")
    parser.add_argument("--config", required=True, help="Path to TOML config file")
    parser.add_argument("--run-name", default=None)
    args = parser.parse_args()

    from rustic_ml.legacy.config import load_config
    config = load_config(args.config)
    train(config, run_name=args.run_name)
