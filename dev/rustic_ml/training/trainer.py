"""
Model-agnostic Trainer.

The Trainer owns the training loop, MLflow metric logging, and end-of-run
figure logging. All model-specific logic lives in the RusticModel subclass
(compute_loss, build_comparison_spec). The Trainer never branches on model type.
"""
from __future__ import annotations

from collections import defaultdict
from typing import TYPE_CHECKING

import mlflow
import torch
import torch.nn as nn

if TYPE_CHECKING:
    from torch.utils.data import DataLoader, Dataset
    from rustic_ml.config import Config
    from rustic_ml.models.base import RusticModel

# Number of validation samples used for end-of-run comparison figures
_N_COMPARISON_SAMPLES = 4


class Trainer:
    """Trains a RusticModel with MLflow metric and figure logging.

    Assumes an active MLflow run has been started before calling fit().
    """

    def __init__(self, model: "RusticModel", config: "Config", device: torch.device):
        self.model = model
        self.config = config
        self.device = device

    def fit(
        self,
        train_loader: "DataLoader",
        val_loader: "DataLoader",
        val_ds: "Dataset | None" = None,
    ) -> dict[str, float]:
        """Run the full training loop and log metrics + figures to MLflow.

        Args:
            train_loader: DataLoader for training data.
            val_loader:   DataLoader for validation data.
            val_ds:       Optional Dataset (for single-sample comparison figures).
                          If None, comparison figures are skipped.

        Returns:
            Dict of final validation loss values (averaged over the last epoch).
        """
        cfg = self.config.training
        optimizer = torch.optim.Adam(self.model.parameters(), lr=cfg.lr)

        final_val: dict[str, float] = {}

        for epoch in range(cfg.n_epochs):
            train_totals = self._run_epoch(train_loader, optimizer, train=True)
            val_totals = self._run_epoch(val_loader, optimizer=None, train=False)

            n_train = len(train_loader)
            n_val = len(val_loader)

            metrics = (
                {f"train/{k}": v / n_train for k, v in train_totals.items()}
                | {f"val/{k}": v / n_val for k, v in val_totals.items()}
            )
            mlflow.log_metrics(metrics, step=epoch)

            if epoch % 10 == 0 or epoch == cfg.n_epochs - 1:
                train_loss = train_totals.get("total", 0.0) / n_train
                val_loss = val_totals.get("total", 0.0) / n_val
                print(f"Epoch {epoch + 1:>4}/{cfg.n_epochs}  "
                      f"train={train_loss:.4f}  val={val_loss:.4f}")

            final_val = {k: v / n_val for k, v in val_totals.items()}

        self._log_end_of_run_figures(val_loader, val_ds)
        return final_val

    def _run_epoch(
        self,
        loader: "DataLoader",
        optimizer: torch.optim.Optimizer | None,
        train: bool,
    ) -> dict[str, float]:
        """Run one epoch. Returns summed losses dict."""
        self.model.train(train)
        totals: dict[str, float] = defaultdict(float)

        ctx = torch.enable_grad() if train else torch.no_grad()
        with ctx:
            for raw_batch in loader:
                batch = self._to_device(raw_batch)
                losses = self.model.compute_loss(batch, self.config.training)

                if train and optimizer is not None:
                    optimizer.zero_grad()
                    losses["total"].backward()
                    optimizer.step()

                for k, v in losses.items():
                    totals[k] += v.item()

        return dict(totals)

    def _to_device(self, batch: dict) -> dict:
        """Move all tensor values in a batch dict to self.device."""
        return {
            k: v.to(self.device) if isinstance(v, torch.Tensor) else v
            for k, v in batch.items()
        }

    def _log_end_of_run_figures(
        self,
        val_loader: "DataLoader",
        val_ds: "Dataset | None",
    ) -> None:
        """Log evaluation figures to the active MLflow run."""
        from rustic_ml.evaluation.figures import log_mel_comparisons

        # Fetch latest registered version for naming
        model_name = type(self.model).__name__
        registered_version = _get_latest_registry_version(model_name)

        if val_ds is not None:
            log_mel_comparisons(
                model=self.model,
                dataset=val_ds,
                device=self.device,
                model_name=model_name,
                registered_version=registered_version,
                n_samples=_N_COMPARISON_SAMPLES,
            )


def _get_latest_registry_version(model_name: str) -> str | None:
    """Fetch the latest registered version number for a model, or None."""
    try:
        from mlflow.tracking import MlflowClient
        client = MlflowClient()
        versions = client.search_model_versions(f"name='{model_name}'")
        if not versions:
            return None
        latest = max(versions, key=lambda v: int(v.version))
        return str(latest.version)
    except Exception:
        return None
