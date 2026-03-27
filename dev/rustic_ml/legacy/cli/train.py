"""
rustic-train CLI entrypoint.

Runs a full training cycle for any RusticModel defined in rustic_ml.models,
driven entirely by a TOML configuration file.

Usage:
    rustic-train --config configs/phase2_waveform.toml
    rustic-train --config configs/phase2_waveform.toml --run-name my_experiment
"""
from __future__ import annotations

import argparse
import sys

import mlflow


def _build_model(model_name: str):
    """Instantiate a model by class name."""
    from rustic_ml import models as m
    cls = getattr(m, model_name, None)
    if cls is None:
        print(f"Unknown model '{model_name}'. Available: {m.__all__}", file=sys.stderr)
        sys.exit(1)
    return cls()


def main() -> None:
    parser = argparse.ArgumentParser(description="Train a Rustic ML model headlessly.")
    parser.add_argument("--config", required=True, help="Path to experiment TOML config")
    parser.add_argument("--run-name", default=None, help="Optional MLflow run name")
    args = parser.parse_args()

    from rustic_ml.config import load_config
    from rustic_ml.training.setup import setup_device, set_seeds, setup_mlflow
    from rustic_ml.training.trainer import Trainer
    from rustic_ml.data.dataset import prepare_dataloaders

    config = load_config(args.config)

    setup_mlflow(config.run.mlflow_uri, config.run.experiment)
    set_seeds(config.training.seed)
    device = setup_device()

    print(f"Model:      {config.run.model}")
    print(f"Experiment: {config.run.experiment}")
    print(f"Epochs:     {config.training.n_epochs}")
    print(f"Data dir:   {config.data.data_dir}")

    train_loader, val_loader, _train_ds, val_ds = prepare_dataloaders(
        data_dir=config.data.data_dir,
        n_samples=config.data.n_samples,
        batch_size_gen=config.data.batch_size_gen,
        batch_size=config.training.batch_size,
        val_fraction=config.data.val_split,
        seed=config.training.seed,
    )

    model = _build_model(config.run.model).to(device)

    with mlflow.start_run(run_name=args.run_name):
        mlflow.log_params(config.to_flat_dict())
        trainer = Trainer(model, config, device)
        final_metrics = trainer.fit(train_loader, val_loader, val_ds=val_ds)
        mlflow.pytorch.log_model(model, config.run.model)

    print("\nTraining complete.")
    for k, v in final_metrics.items():
        print(f"  val/{k}: {v:.4f}")


if __name__ == "__main__":
    main()
