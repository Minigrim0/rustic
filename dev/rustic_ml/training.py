"""
Training utilities: seed management, hyperparameter logging, and metric logging.
"""

import random

import mlflow
import numpy as np
import torch


def set_seeds(seed: int) -> None:
    """Set random seeds for Python, NumPy, and PyTorch (CPU + CUDA).

    Args:
        seed: Integer seed value.
    """
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)
    if torch.cuda.is_available():
        torch.cuda.manual_seed(seed)


def setup_device() -> torch.device:
    """Detect and return the best available compute device.

    Checks for CUDA, then MPS (Apple Silicon), then falls back to CPU.
    Prints which device was selected.

    Returns:
        torch.device ready to pass to model.to() and tensor.to().
    """
    if torch.cuda.is_available():
        name = torch.cuda.get_device_name(0)
        print(f"CUDA detected — using {name}")
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        print("MPS detected — using Apple Silicon GPU")
        return torch.device("mps")
    print("No GPU detected — using CPU")
    return torch.device("cpu")


def log_hyperparams(hyperparams: dict) -> None:
    """Log all entries of a hyperparameter dict to the active MLflow run.

    Call this once at the start of a run, after mlflow.start_run().

    Args:
        hyperparams: Dict mapping param name to value. All values are
                     converted to strings by MLflow automatically.
    """
    mlflow.log_params(hyperparams)


def compute_and_log_note_metrics(
    train_losses: dict,
    train_n_batches: int,
    val_losses: dict,
    val_n_batches: int,
    val_accuracy: float,
    epoch: int,
) -> None:
    """Normalize and log note-model-specific metrics to the active MLflow run.

    Args:
        train_losses:    Dict with at least a "note" key — summed over batches.
        train_n_batches: Number of training batches (divisor for normalization).
        val_losses:      Dict with at least a "note" key — summed over batches.
        val_n_batches:   Number of validation batches.
        val_accuracy:    Note classification accuracy on the validation set (0–1).
        epoch:           Current epoch index, used as the MLflow step.
    """
    mlflow.log_metrics(
        {
            "train/loss_note":   train_losses["note"] / train_n_batches,
            "val/loss_note":     val_losses["note"]   / val_n_batches,
            "val/accuracy_note": val_accuracy,
        },
        step=epoch,
    )


def compute_and_log_adsr_metrics(
    train_losses: dict,
    train_n_batches: int,
    val_losses: dict,
    val_n_batches: int,
    epoch: int,
) -> None:
    """Normalize and log ADSR-model-specific metrics to the active MLflow run.

    Args:
        train_losses:    Dict with at least an "adsr" key — summed over batches.
        train_n_batches: Number of training batches (divisor for normalization).
        val_losses:      Dict with at least an "adsr" key — summed over batches.
        val_n_batches:   Number of validation batches.
        epoch:           Current epoch index, used as the MLflow step.
    """
    mlflow.log_metrics(
        {
            "train/loss_adsr": train_losses["adsr"] / train_n_batches,
            "val/loss_adsr":   val_losses["adsr"]   / val_n_batches,
        },
        step=epoch,
    )


def compute_and_log_metrics(
    train_losses: dict,
    train_n_batches: int,
    val_losses: dict,
    val_n_batches: int,
    val_accuracy: float,
    epoch: int,
) -> None:
    """Normalize accumulated losses and log metrics to the active MLflow run.

    Args:
        train_losses:    Dict with keys "loss", "note", "adsr" — summed over batches.
        train_n_batches: Number of training batches (divisor for normalization).
        val_losses:      Dict with keys "loss", "note", "adsr" — summed over batches.
        val_n_batches:   Number of validation batches.
        val_accuracy:    Note classification accuracy on the validation set (0–1).
        epoch:           Current epoch index, used as the MLflow step.
    """
    mlflow.log_metrics(
        {
            "train/loss":         train_losses["loss"] / train_n_batches,
            "train/loss_note":    train_losses["note"] / train_n_batches,
            "train/loss_adsr":    train_losses["adsr"] / train_n_batches,
            "val/loss":           val_losses["loss"]   / val_n_batches,
            "val/loss_note":      val_losses["note"]   / val_n_batches,
            "val/loss_adsr":      val_losses["adsr"]   / val_n_batches,
            "val/accuracy_note":  val_accuracy,
        },
        step=epoch,
    )
