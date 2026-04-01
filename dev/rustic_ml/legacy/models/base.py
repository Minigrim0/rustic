"""
Base class and preprocessing configuration for all Rustic ML models.

Each model inherits RusticModel and declares:
  - preprocessing:    mel/audio parameters for data generation
  - required_labels:  which label columns to load from .npz files
  - compute_loss:     forward pass + loss, returns a dict of named losses
  - build_comparison_spec: (optional) reconstruct a GraphSpec for audio re-synthesis
"""
from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import TYPE_CHECKING

import torch.nn as nn

if TYPE_CHECKING:
    import torch
    from rustic_ml.config import TrainingConfig


@dataclass
class PreprocessingConfig:
    """Audio and mel spectrogram parameters used during data generation and loading."""

    sample_rate: int = 44100
    n_mels: int = 128
    n_fft: int = 2048
    hop_length: int = 512
    note_on: float = 0.05
    note_off: float = 0.6
    duration: float = 1.0


class RusticModel(nn.Module, ABC):
    """Abstract base class for all Rustic synthesis-analysis models.

    Subclasses must implement:
      - preprocessing:    return a PreprocessingConfig describing mel/audio params
      - required_labels:  return the list of label keys needed from the dataset
      - compute_loss:     run forward pass and return a dict of named loss tensors

    Subclasses may optionally override:
      - build_comparison_spec: return a GraphSpec dict for audio re-synthesis
        comparisons (used by Trainer to log mel figures to MLflow). Return None
        to skip mel comparison figures for this model.
    """

    @property
    @abstractmethod
    def preprocessing(self) -> PreprocessingConfig:
        """Return the PreprocessingConfig used during data generation and loading."""
        ...

    @property
    @abstractmethod
    def required_labels(self) -> list[str]:
        """Return the label columns needed from the dataset.

        Example: ["note", "waveform"] or ["note", "adsr"]
        The data pipeline uses this to ensure all required columns are loaded.
        """
        ...

    @abstractmethod
    def compute_loss(self, batch: dict, config: "TrainingConfig") -> "dict[str, torch.Tensor]":
        """Run forward pass on a batch and return a dict of named losses.

        The dict must contain a "total" key with the scalar loss to backprop.
        Additional keys (e.g. "note", "adsr", "waveform") are logged to MLflow
        individually.

        Args:
            batch:  Dict with keys "mel", "note", "adsr", "waveform" (all tensors
                    already moved to the correct device).
            config: TrainingConfig with hyperparameters like lambda_waveform.

        Returns:
            Dict mapping loss name → scalar Tensor. Must include "total".
        """
        ...

    def build_comparison_spec(self, sample: dict, device: "torch.device") -> dict | None:
        """Build a GraphSpec dict for audio re-synthesis from a single sample.

        Called by Trainer at end of training to generate mel comparison figures.
        Override in subclasses that can reconstruct a synthesis spec from their
        predictions. Return None (default) to skip mel comparison for this model.

        Args:
            sample: Single-item dict from the dataset (tensors, not batched).
            device: Device to run inference on.

        Returns:
            GraphSpec-compatible dict passable to rustic_py.render(), or None.
        """
        return None
