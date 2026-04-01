"""
ThePainter training entry point.

Trained on synthetic (canonical token sequence, rendered log-mel) pairs using
L2 / mel-L1 reconstruction loss. Also supports a contrastive alignment loss
between ThePainter's spec embeddings and a mel encoder's embeddings (CLIP-style).

Callable from both CLI (rustic-train-painter) and notebook:

    from rustic_ml.the_painter.train import train
    train(config, run_name="my_run")
"""
# TODO: implement ThePainter training loop
from __future__ import annotations
import argparse


def train(config: dict, run_name: str | None = None) -> dict[str, float]:
    """Train ThePainter and log the run to MLflow.

    Args:
        config:   Flat or nested config dict.
        run_name: Optional MLflow run name override.

    Returns:
        Final validation metrics dict.
    """
    raise NotImplementedError("ThePainter training is not yet implemented")


def main() -> None:
    """CLI entry point: rustic-train-painter."""
    parser = argparse.ArgumentParser(description="Train ThePainter (surrogate renderer)")
    parser.add_argument("--config", required=True, help="Path to TOML config file")
    parser.add_argument("--run-name", default=None)
    args = parser.parse_args()

    from rustic_ml.legacy.config import load_config
    config = load_config(args.config)
    train(config, run_name=args.run_name)
