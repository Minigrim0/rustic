"""
Configuration system for Rustic ML training runs.

Uses Python dataclasses for type-safe config objects, and TOML files for
experiment configuration. Experiment configs deep-merge on top of base.toml,
so only changed values need to be specified.

Usage:
    config = load_config("configs/phase2_waveform.toml")
    # config.run.model == "NoteWaveformPredictor"
    # config.training.n_epochs == 120 (overridden)
    # config.data.n_samples == 80_000 (from base)
"""
from __future__ import annotations

import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any


@dataclass
class RunConfig:
    """MLflow run configuration."""

    experiment: str = "Default"
    model: str = "NoteWaveformPredictor"
    mlflow_uri: str = "http://192.168.1.254:5000"


@dataclass
class DataConfig:
    """Dataset generation and loading configuration."""

    data_dir: str = "./data/datasets"
    n_samples: int = 80_000
    batch_size_gen: int = 1_000
    val_split: float = 0.1
    max_frames: int = 256


@dataclass
class TrainingConfig:
    """Model training hyperparameters."""

    batch_size: int = 64
    n_epochs: int = 80
    lr: float = 1e-3
    seed: int = 42
    lambda_waveform: float = 1.0


@dataclass
class ModelConfig:
    """Model architecture hyperparameters."""

    channels: list = field(default_factory=lambda: [32, 64, 128, 256])
    dropout: float = 0.3


@dataclass
class Config:
    """Top-level configuration container."""

    run: RunConfig = field(default_factory=RunConfig)
    data: DataConfig = field(default_factory=DataConfig)
    training: TrainingConfig = field(default_factory=TrainingConfig)
    model: ModelConfig = field(default_factory=ModelConfig)

    def to_flat_dict(self) -> dict:
        """Return a flat dict of all config values for MLflow logging."""
        result = {}
        for section_name, section in asdict(self).items():
            for k, v in section.items():
                result[f"{section_name}.{k}"] = v
        return result


def _load_toml(path: Path) -> dict:
    """Load a TOML file using stdlib tomllib (Python >=3.11) or tomli fallback."""
    if sys.version_info >= (3, 11):
        import tomllib
        with open(path, "rb") as f:
            return tomllib.load(f)
    else:
        try:
            import tomli
            with open(path, "rb") as f:
                return tomli.load(f)
        except ImportError:
            raise ImportError(
                "Python < 3.11 requires the 'tomli' package: pip install tomli"
            )


def _deep_merge(base: dict, override: dict) -> dict:
    """Recursively merge override into base, returning a new dict."""
    result = dict(base)
    for key, val in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(val, dict):
            result[key] = _deep_merge(result[key], val)
        else:
            result[key] = val
    return result


def _dict_to_config(data: dict) -> Config:
    """Convert a nested dict to a Config object."""
    return Config(
        run=RunConfig(**data.get("run", {})),
        data=DataConfig(**data.get("data", {})),
        training=TrainingConfig(**data.get("training", {})),
        model=ModelConfig(**data.get("model", {})),
    )


# Path to the base.toml relative to this file (dev/configs/base.toml)
_BASE_TOML = Path(__file__).parent.parent.parent / "configs" / "base.toml"


def load_config(path: str | Path) -> Config:
    """Load a config by deep-merging base.toml with an experiment TOML.

    Args:
        path: Path to the experiment TOML file. Can be absolute or relative
              to the current working directory.

    Returns:
        A Config instance with all defaults from base.toml overridden by
        values in the experiment config.
    """
    path = Path(path)

    base: dict[str, Any] = {}
    if _BASE_TOML.exists():
        base = _load_toml(_BASE_TOML)

    experiment = _load_toml(path) if path.exists() else {}
    merged = _deep_merge(base, experiment)
    return _dict_to_config(merged)
