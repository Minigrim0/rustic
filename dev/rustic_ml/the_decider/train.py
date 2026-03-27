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
"""
# TODO: implement TheDecider training loop
from __future__ import annotations
import argparse


def train(config: dict, run_name: str | None = None) -> dict[str, float]:
    """Train TheDecider and log the run to MLflow.

    Args:
        config:   Flat or nested config dict (mirrors the TOML structure).
        run_name: Optional MLflow run name override.

    Returns:
        Final validation metrics dict.
    """
    raise NotImplementedError("TheDecider training is not yet implemented")


def main() -> None:
    """CLI entry point: rustic-train-decider."""
    parser = argparse.ArgumentParser(description="Train TheDecider (note classifier)")
    parser.add_argument("--config", required=True, help="Path to TOML config file")
    parser.add_argument("--run-name", default=None)
    args = parser.parse_args()

    from rustic_ml.legacy.config import load_config
    config = load_config(args.config)
    train(config, run_name=args.run_name)
