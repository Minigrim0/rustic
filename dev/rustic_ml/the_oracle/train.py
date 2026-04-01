"""
TheOracle training entry point.

Phase 1: supervised pretraining on canonical synthetic (mel, token_sequence) pairs.
Phase 2: optional PPO fine-tuning with perceptual reward (Rust renderer + mel distance).

Callable from both CLI (rustic-train-oracle) and notebook:

    from rustic_ml.the_oracle.train import train
    train(config, run_name="my_run")
"""
# TODO: implement TheOracle training loop
from __future__ import annotations
import argparse


def train(config: dict, run_name: str | None = None) -> dict[str, float]:
    """Train TheOracle and log the run to MLflow.

    Args:
        config:   Flat or nested config dict.
        run_name: Optional MLflow run name override.

    Returns:
        Final validation metrics dict.
    """
    raise NotImplementedError("TheOracle training is not yet implemented")


def main() -> None:
    """CLI entry point: rustic-train-oracle."""
    parser = argparse.ArgumentParser(description="Train TheOracle (AR graph decoder)")
    parser.add_argument("--config", required=True, help="Path to TOML config file")
    parser.add_argument("--run-name", default=None)
    args = parser.parse_args()

    from rustic_ml.legacy.config import load_config
    config = load_config(args.config)
    train(config, run_name=args.run_name)
