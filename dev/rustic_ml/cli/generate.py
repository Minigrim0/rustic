"""
rustic-generate CLI entrypoint.

Generates a synthetic dataset of .npz batch files using the rustic_py renderer.

Usage:
    rustic-generate --config configs/phase2_waveform.toml --output-dir /data/datasets
    rustic-generate --config configs/base.toml --output-dir /data/datasets --n-samples 10000
"""
from __future__ import annotations

import argparse
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate a Rustic ML synthetic dataset.")
    parser.add_argument("--config", required=True, help="Path to experiment TOML config")
    parser.add_argument(
        "--output-dir",
        required=True,
        help="Directory to write .npz batch files into",
    )
    parser.add_argument(
        "--n-samples",
        type=int,
        default=None,
        help="Override n_samples from config",
    )
    parser.add_argument(
        "--n-workers",
        type=int,
        default=1,
        help="Number of parallel worker processes (default: 1)",
    )
    parser.add_argument(
        "--start-batch",
        type=int,
        default=0,
        help="Starting batch index for file naming (to append without overwriting)",
    )
    args = parser.parse_args()

    from rustic_ml.config import load_config
    from rustic_ml.data.generation import generate_dataset

    config = load_config(args.config)

    n_samples = args.n_samples if args.n_samples is not None else config.data.n_samples
    output_dir = Path(args.output_dir)

    print(f"Generating {n_samples:,} samples → {output_dir}")
    print(f"Batch size: {config.data.batch_size_gen}  Workers: {args.n_workers}")

    generate_dataset(
        n_samples=n_samples,
        output_dir=output_dir,
        batch_size=config.data.batch_size_gen,
        waveform=None,
        start_batch=args.start_batch,
        n_workers=args.n_workers,
    )

    print(f"Done. Dataset written to {output_dir}")


if __name__ == "__main__":
    main()
