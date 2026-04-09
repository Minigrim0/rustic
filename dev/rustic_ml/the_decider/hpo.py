from __future__ import annotations

import random
from dataclasses import dataclass, field

import mlflow
import numpy as np
import optuna
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Subset

from rustic_ml.the_decider.dataset import DeciderDataset
from rustic_ml.the_decider.inference import evaluate
from rustic_ml.the_decider.model import TheDecider
from rustic_ml.training.setup import setup_mlflow

HPO_EXPERIMENT = "TheDecider-HPO"

@dataclass
class HpoConfig:
    mlflow_uri: str = "http://192.168.1.254:5000"
    n_samples: int = 4_000
    val_split: float = 0.20
    max_frames: int = 256
    n_epochs: int = 20
    seed: int = 42
    data_dir: str | None = None
    num_workers: int = 2

def create_study(
        mlflow_uri: str = "http://192.168.1.254:5000",
        n_startup_trials: int = 5,
        n_warmup_steps: int = 5,
        direction: str = "maximize",
        ) -> optuna.Study:
    pruner = optuna.pruners.MedianPruner(
            n_startup_trials=n_startup_trials,
            n_warmup_steps=n_warmup_steps
            )
    study = optuna.create_study(
            study_name="TheDecider-HPO",
            direction=direction,
            pruner=pruner,
            load_if_exists=True,
            )
    return study

def make_objective(device: torch.device, cfg: HpoConfig):
    def objective(trial: optuna.Trial) -> float:
        # Hyperparameter suggestion
        n_layers = trial.suggest_int("n_layers", 3, 5)
        base_ch = trial.suggest_categorical("base_channels", [16, 32, 64])
        channels = [base_ch * (2 ** i) for i in range(n_layers)]
        dropout = trial.suggest_float("dropout", 0.10, 0.50)
        lr = trial.suggest_float("lr", 1e-4, 1e-2, log=True)
        batch_size = trial.suggest_categorical("batch_size", [32, 64, 128])

        # Reproducibility
        random.seed(cfg.seed)
        np.random.seed(cfg.seed)
        torch.manual_seed(cfg.seed)

        # Dataset
        dataset = DeciderDataset(
            n_samples=cfg.n_samples,
            cache_dir=cfg.data_dir,
            max_frames=cfg.max_frames
        )
        n_val = max(1, int(cfg.n_samples * cfg.val_split))
        n_train = cfg.n_samples - n_val
        indices = list(range(cfg.n_samples))
        rng = random.Random(cfg.seed)
        rng.shuffle(indices)

        train_loader = DataLoader(
            Subset(dataset, indices[:n_train]),
            batch_size=batch_size,
            shuffle=True,
            num_workers=cfg.num_workers,
            pin_memory=True,
        )
        val_loader = DataLoader(
            Subset(dataset, indices[n_train:]),
            batch_size=batch_size,
            shuffle=False,
            num_workers=cfg.num_workers,
        )

        # Model
        model = TheDecider(channels=channels, dropout=dropout).to(device)
        optimizer = torch.optim.Adam(model.parameters(), lr=lr)
        criterion = nn.CrossEntropyLoss()

        setup_mlflow(cfg.mlflow_uri, HPO_EXPERIMENT)
        run_name = f"trial-{trial.number}"

        with mlflow.start_run(run_name=run_name):
            mlflow.log_params({
                "trial":      trial.number,
                "n_layers":   n_layers,
                "base_ch":    base_ch,
                "channels":   str(channels),
                "dropout":    dropout,
                "lr":         lr,
                "batch_size": batch_size,
                "n_epochs":   cfg.n_epochs,
                "n_samples":  cfg.n_samples,
            })

            top1 = 0.0
            for epoch in range(1, cfg.n_epochs + 1):
                model.train()
                train_loss = 0.0
                for batch in train_loader:
                    mel = batch["mel"].unsqueeze(1).to(device)
                    note = batch["note"].to(device)
                    optimizer.zero_grad()
                    loss = criterion(model(mel), note)
                    loss.backward()
                    optimizer.step()
                    train_loss += loss.item() * note.size(0)
                train_loss /= n_train

                # Validation
                val_metrics = evaluate(model, val_loader, device)
                top1 = val_metrics.get("top1_accuracy", 0.0)
                top5 = val_metrics.get("top5_accuracy", 0.0)

                mlflow.log_metrics(
                    {
                        "train_loss": train_loss,
                        "top1_accuracy": top1,
                        "top5_accuracy": top5,
                    },
                    step=epoch
                )

                trial.report(top1, step=epoch)
                if trial.should_prune():
                    raise optuna.TrialPruned()

        return top1
    return objective
