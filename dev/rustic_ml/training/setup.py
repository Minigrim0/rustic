"""
Training infrastructure: device detection, seed management, MLflow setup.
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


def setup_mlflow(tracking_uri: str, experiment_name: str) -> None:
    """Set the MLflow tracking URI and active experiment.

    Args:
        tracking_uri:    MLflow server URI, e.g. "http://192.168.1.254:5000".
        experiment_name: Name of the experiment to log runs under.
    """
    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment(experiment_name)
    print(f"MLflow connected: {tracking_uri}  (experiment: {experiment_name})")
