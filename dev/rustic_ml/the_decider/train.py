"""
TheDecider training entry point.

Callable from both the CLI (via rustic-train-decider) and directly from a
notebook cell:

    from rustic_ml.the_decider.train import train
    train(config, run_name="my_run")

Config fields used (from rustic_ml.legacy.config.Config):
  run.mlflow_uri       MLflow tracking URI
  run.experiment       MLflow experiment name
  data.data_dir        Directory for cached dataset .npz files
  data.n_samples       Number of training samples to generate
  data.val_split       Fraction held out for validation
  data.max_frames      Time-axis length for mel spectrograms (default 256)
  training.batch_size
  training.n_epochs
  training.lr
  training.seed
  model.channels       CNN channel widths, e.g. [32, 64, 128, 256]
  model.dropout        Dropout rate before the linear head (default 0.3)
"""
from __future__ import annotations

import argparse
import logging
import random
import time

import mlflow
import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Subset
from tqdm import tqdm

from rustic_ml.legacy.config import Config
from rustic_ml.the_decider.dataset import DeciderDataset
from rustic_ml.the_decider.model import TheDecider
from rustic_ml.the_decider.inference import evaluate
from rustic_ml.training.setup import setup_mlflow, setup_device

log = logging.getLogger(__name__)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s  %(levelname)s  %(message)s",
    datefmt="%H:%M:%S",
)


def train(config: Config, run_name: str | None = None) -> dict[str, float]:
    """Train TheDecider and log the run to MLflow.

    Args:
        config:   Config dataclass (from load_config).
        run_name: Optional MLflow run name override.

    Returns:
        Final validation metrics dict.
    """
    # Hyper-parameters
    mlflow_uri = config.run.mlflow_uri
    experiment = config.run.experiment
    data_dir   = config.data.data_dir
    n_samples  = config.data.n_samples
    val_split  = config.data.val_split
    batch_size = config.training.batch_size
    n_epochs   = config.training.n_epochs
    lr         = config.training.lr
    seed       = config.training.seed
    channels   = config.model.channels
    dropout    = config.model.dropout

    # Reproducibility
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)

    log.info("Initialising compute device …")
    t0 = time.monotonic()
    device = setup_device()
    log.info("Device ready in %.1f s", time.monotonic() - t0)

    # Dataset
    log.info("Building dataset (n_samples=%d, cache_dir=%s) …", n_samples, data_dir)
    t0 = time.monotonic()
    dataset = DeciderDataset(n_samples=n_samples, cache_dir=data_dir, max_frames=config.data.max_frames)
    log.info("Dataset ready in %.1f s", time.monotonic() - t0)

    n_val   = max(1, int(n_samples * val_split))
    n_train = n_samples - n_val
    indices = list(range(n_samples))
    random.shuffle(indices)
    train_ds = Subset(dataset, indices[:n_train])
    val_ds   = Subset(dataset, indices[n_train:])
    log.info("Split: %d train / %d val", n_train, n_val)

    log.info("Creating DataLoaders (num_workers=2) …")
    t0 = time.monotonic()
    train_loader = DataLoader(
        train_ds, batch_size=batch_size, shuffle=True, num_workers=2, pin_memory=True,
    )
    val_loader = DataLoader(
        val_ds, batch_size=batch_size, shuffle=False, num_workers=2,
    )
    log.info("DataLoaders ready in %.1f s", time.monotonic() - t0)

    # Model
    log.info("Instantiating model …")
    t0 = time.monotonic()
    model = TheDecider(channels=channels, dropout=dropout).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=lr)
    criterion = nn.CrossEntropyLoss()
    log.info("Model ready in %.1f s  (%d params)", time.monotonic() - t0,
             sum(p.numel() for p in model.parameters()))

    # MLflow
    log.info("Connecting to MLflow at %s …", mlflow_uri)
    t0 = time.monotonic()
    setup_mlflow(mlflow_uri, experiment)
    log.info("MLflow connected in %.1f s", time.monotonic() - t0)

    log.info("Opening MLflow run …")
    t0 = time.monotonic()
    val_metrics: dict[str, float] = {}
    with mlflow.start_run(run_name=run_name) as run:
        log.info("Run opened in %.1f s  (id=%s)", time.monotonic() - t0, run.info.run_id)
        log.info("Logging params …")
        t0 = time.monotonic()
        mlflow.log_params({
            "n_samples": n_samples,
            "val_split": val_split,
            "batch_size": batch_size,
            "n_epochs": n_epochs,
            "lr": lr,
            "seed": seed,
            "channels": channels,
            "dropout": dropout,
        })
        log.info("Params logged in %.1f s", time.monotonic() - t0)
        log.info("Starting training loop (%d epochs) …", n_epochs)

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

        log.info("Saving model artifact …")
        t0 = time.monotonic()
        mlflow.pytorch.log_model(model, "model")
        log.info("Model artifact saved in %.1f s", time.monotonic() - t0)
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

