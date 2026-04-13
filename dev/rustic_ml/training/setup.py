"""
Training infrastructure: device detection, seed management, MLflow setup.
"""
import random
from contextlib import contextmanager

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
    import os

    use_cuda = os.environ.get("NO_CUDA", 0) == 0

    if torch.cuda.is_available() and use_cuda:
        name = torch.cuda.get_device_name(0)
        print(f"CUDA detected — using {name}")
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        print("MPS detected — using Apple Silicon GPU")
        return torch.device("mps")
    print("No GPU detected — using CPU")
    return torch.device("cpu")


def setup_mlflow(tracking_uri: str | None, experiment_name: str) -> bool:
    """Set the MLflow tracking URI and active experiment.

    Args:
        tracking_uri:    MLflow server URI, e.g. "http://100.101.108.22:5000".
                         Pass None to disable MLflow entirely.
        experiment_name: Name of the experiment to log runs under.

    Returns:
        True if MLflow was configured, False if disabled.
    """
    if not tracking_uri:
        print("MLflow disabled (no tracking URI configured)")
        return False
    import mlflow
    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment(experiment_name)
    print(f"MLflow connected: {tracking_uri}  (experiment: {experiment_name})")
    return True


@contextmanager
def mlflow_run(enabled: bool, run_name: str | None = None):
    """Context manager that opens an MLflow run when enabled, or is a no-op.

    Args:
        enabled:  Whether MLflow is active (return value of setup_mlflow).
        run_name: Optional run name passed to mlflow.start_run().

    Yields:
        The MLflow run object when enabled, or a plain namespace with a dummy
        run_id when disabled.
    """
    if enabled:
        import mlflow
        with mlflow.start_run(run_name=run_name) as run:
            yield run
    else:
        class _DummyRun:
            class info:
                run_id = "local"
        yield _DummyRun()


def mlflow_log_params(enabled: bool, params: dict) -> None:
    if enabled:
        import mlflow
        mlflow.log_params(params)


def mlflow_log_metrics(enabled: bool, metrics: dict, step: int | None = None) -> None:
    if enabled:
        import mlflow
        mlflow.log_metrics(metrics, step=step)


def mlflow_log_model(enabled: bool, model, artifact_path: str) -> None:
    if enabled:
        import mlflow
        mlflow.pytorch.log_model(model, artifact_path)


def mlflow_log_text(enabled: bool, text: str, artifact_path: str) -> None:
    if enabled:
        import mlflow
        mlflow.log_text(text, artifact_path)


def mlflow_log_image(enabled: bool, image, artifact_path: str) -> None:
    if enabled:
        import mlflow
        mlflow.log_image(image, artifact_path)
