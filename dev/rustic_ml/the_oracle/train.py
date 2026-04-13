"""
TheOracle training entry point.

Phase 1: supervised pretraining on canonical synthetic (mel, token_sequence) pairs.

Callable from both CLI (rustic-train-oracle) and notebook:

    from rustic_ml.the_oracle.train import train
    train(config, run_name="my_run")
"""
from __future__ import annotations

import argparse
import logging
import random
import time

import numpy as np
import torch
import torch.nn.functional as F
from torch.utils.data import DataLoader, Subset
from tqdm import tqdm

from rustic_ml.autoregressive import Vocabulary, ARDataset, ar_collate_fn
from rustic_ml.autoregressive.tokenizer import sequence_to_spec
from rustic_ml.legacy.config import Config
from rustic_ml.the_oracle.model import TheOracle
from rustic_ml.the_oracle.inference import evaluate
from rustic_ml.training.setup import (
    setup_mlflow, setup_device,
    mlflow_run, mlflow_log_params, mlflow_log_metrics, mlflow_log_model,
    mlflow_log_text, mlflow_log_image,
)

log = logging.getLogger(__name__)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s  %(levelname)s  %(message)s",
    datefmt="%H:%M:%S",
)

# ── Checkpoint artifact helpers ────────────────────────────────────────────────

def _token_sequence_text(token_ids: list[int], vocab: Vocabulary) -> str:
    """Convert a token ID list to a human-readable string."""
    return " ".join(vocab.id_to_token.get(t, f"?{t}") for t in token_ids)


def _log_checkpoint_artifacts(
    use_mlflow: bool,
    model:   TheOracle,
    batch:   dict,
    vocab:   Vocabulary,
    device:  torch.device,
    step:    int,
    n_examples: int = 4,
) -> None:
    """Decode a few examples from a batch and log predicted vs ground-truth
    token sequences (and mel) to MLflow as text artifacts.

    Args:
        model:       TheOracle (eval mode expected).
        batch:       Collated batch dict from ar_collate_fn.
        vocab:       Vocabulary instance.
        device:      Compute device.
        step:        Current epoch (used in artifact names).
        n_examples:  How many examples from the batch to log.
    """
    import io
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    model.eval()
    with torch.no_grad():
        mel = batch["mel"].unsqueeze(1).to(device)  # (B, 1, MEL_BINS, T)

        lines: list[str] = []
        for i in range(min(n_examples, mel.size(0))):
            single_mel = mel[i : i + 1]             # (1, 1, MEL_BINS, T)
            pred_ids, pred_cont, pred_cat = model.greedy_decode(
                single_mel, vocab, max_len=256
            )

            gt_ids = batch["token_ids"][i].tolist()
            # Strip padding from ground truth
            try:
                gt_ids = gt_ids[: gt_ids.index(vocab.eos) + 1]
            except ValueError:
                pass

            lines.append(f"=== Example {i} (epoch {step}) ===")
            lines.append(f"GT  ({len(gt_ids)} tokens): {_token_sequence_text(gt_ids, vocab)}")
            lines.append(
                f"PRED ({len(pred_ids)} tokens): {_token_sequence_text(pred_ids, vocab)}"
            )

            # Try to decode predicted sequence to a spec and note if it's valid
            try:
                sequence_to_spec(
                    pred_ids,
                    pred_cont.numpy(),
                    pred_cat.numpy(),
                    vocab,
                )
                lines.append("  → valid spec")
            except Exception as exc:
                lines.append(f"  → INVALID spec: {exc}")

            lines.append("")

            # ── mel comparison figure ────────────────────────────────────────
            fig, axes = plt.subplots(1, 1, figsize=(8, 3))
            gt_mel = batch["mel"][i].numpy()  # (MEL_BINS, T)
            axes.imshow(gt_mel, origin="lower", aspect="auto", cmap="magma")
            axes.set_title(f"Input mel — example {i}, epoch {step}")
            axes.set_xlabel("Time frame")
            axes.set_ylabel("Mel bin")
            fig.tight_layout()

            buf = io.BytesIO()
            fig.savefig(buf, format="png", dpi=80)
            buf.seek(0)
            plt.close(fig)
            mlflow_log_image(use_mlflow, buf.read(), f"mel_epoch{step:04d}_ex{i}.png")

        artifact_text = "\n".join(lines)
        mlflow_log_text(use_mlflow, artifact_text, f"sequences_epoch{step:04d}.txt")

    model.train()


# ── Loss ───────────────────────────────────────────────────────────────────────

def _compute_loss(
    token_logits: torch.Tensor,
    cont_pred:    torch.Tensor,
    cat_logits:   torch.Tensor,
    tgt_ids:      torch.Tensor,
    tgt_cont:     torch.Tensor,
    tgt_cat:      torch.Tensor,
    cont_mask:    torch.Tensor,
    cat_n_cls:    torch.Tensor,
    pad_id:       int,
    lambda_cont:  float,
    lambda_cat:   float,
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    """Compute the combined AR loss and return individual components.

    Returns:
        (total_loss, token_loss, cont_loss, cat_loss)
    """
    cat_width = cat_logits.size(2)
    pad_mask  = tgt_ids != pad_id                                  # (B, S)

    # Token loss — cross-entropy over non-PAD positions
    token_loss = F.cross_entropy(
        token_logits[pad_mask],
        tgt_ids[pad_mask],
    )

    # Continuous loss — MSE over active fields in non-PAD positions
    active_cont = cont_mask[tgt_ids] & pad_mask.unsqueeze(-1)      # (B, S, cont_width)
    if active_cont.any():
        cont_loss = F.mse_loss(cont_pred[active_cont], tgt_cont[active_cont])
    else:
        cont_loss = token_loss.new_zeros(())

    # Categorical loss — per-field cross-entropy over active, non-PAD positions
    cat_loss = token_loss.new_zeros(())
    for f in range(cat_width):
        active = (cat_n_cls[tgt_ids, f] > 0) & pad_mask            # (B, S)
        if not active.any():
            continue
        n_cls = int(cat_n_cls[tgt_ids[active], f].max())
        cat_loss = cat_loss + F.cross_entropy(
            cat_logits[active, f, :n_cls],
            tgt_cat[active, f],
        )

    total = token_loss + lambda_cont * cont_loss + lambda_cat * cat_loss
    return total, token_loss, cont_loss, cat_loss


# ── Main training loop ─────────────────────────────────────────────────────────

def train(config: Config, run_name: str | None = None) -> dict[str, float]:
    """Train TheOracle and log the run to MLflow.

    Args:
        config:   Config dataclass (from load_config).
        run_name: Optional MLflow run name override.

    Returns:
        Final validation metrics dict.
    """
    # ── Hyper-parameters ─────────────────────────────────────────────────────
    mlflow_uri   = config.run.mlflow_uri
    experiment   = config.run.experiment
    data_dir     = config.data.data_dir
    n_samples    = config.data.n_samples
    val_split    = config.data.val_split
    batch_size   = config.training.batch_size
    n_epochs     = config.training.n_epochs
    lr           = config.training.lr
    seed         = config.training.seed
    d_model      = config.training.d_model
    nhead        = config.training.nhead
    ffn_dim      = config.training.ffn_dim
    n_enc_layers = config.training.n_enc_layers
    n_dec_layers = config.training.n_dec_layers
    lambda_cont  = config.training.lambda_cont
    lambda_cat   = config.training.lambda_cat
    max_seq_len  = config.training.max_seq_len
    dropout      = config.training.dropout
    ckpt_every   = config.training.ckpt_every

    # ── Reproducibility ───────────────────────────────────────────────────────
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)

    log.info("Initialising compute device …")
    t0 = time.monotonic()
    device = setup_device()
    log.info("Device ready in %.1f s", time.monotonic() - t0)

    # ── Vocabulary ────────────────────────────────────────────────────────────
    log.info("Building vocabulary from rustic_py …")
    t0 = time.monotonic()
    vocab = Vocabulary.from_rustic()
    log.info("Vocabulary ready in %.1f s  (%d tokens, cont_width=%d, cat_width=%d)",
             time.monotonic() - t0, len(vocab), vocab.cont_width, vocab.cat_width)

    # ── Dataset ───────────────────────────────────────────────────────────────
    log.info("Building dataset (n_samples=%d, cache_dir=%s) …", n_samples, data_dir)
    t0 = time.monotonic()
    dataset = ARDataset(n_samples=n_samples, vocab=vocab, cache_dir=data_dir)
    log.info("Dataset ready in %.1f s", time.monotonic() - t0)

    n_val   = max(1, int(n_samples * val_split))
    n_train = n_samples - n_val
    indices = list(range(n_samples))
    random.shuffle(indices)
    train_ds = Subset(dataset, indices[:n_train])
    val_ds   = Subset(dataset, indices[n_train:])
    log.info("Split: %d train / %d val", n_train, n_val)

    log.info("Creating DataLoaders …")
    t0 = time.monotonic()
    train_loader = DataLoader(
        train_ds, batch_size=batch_size, shuffle=True,
        num_workers=2, pin_memory=True, collate_fn=ar_collate_fn,
    )
    val_loader = DataLoader(
        val_ds, batch_size=batch_size, shuffle=False,
        num_workers=2, collate_fn=ar_collate_fn,
    )
    log.info("DataLoaders ready in %.1f s", time.monotonic() - t0)

    # ── Model ─────────────────────────────────────────────────────────────────
    log.info("Instantiating TheOracle …")
    t0 = time.monotonic()
    model = TheOracle(
        vocab=vocab,
        d_model=d_model,
        nhead=nhead,
        ffn_dim=ffn_dim,
        n_enc_layers=n_enc_layers,
        n_dec_layers=n_dec_layers,
        dropout=dropout,
    ).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=lr)
    n_params = sum(p.numel() for p in model.parameters())
    log.info("Model ready in %.1f s  (%d params)", time.monotonic() - t0, n_params)

    # Grab a fixed validation batch for checkpoint artifacts
    val_iter    = iter(val_loader)
    ckpt_batch  = next(val_iter)

    # ── MLflow ────────────────────────────────────────────────────────────────
    log.info("Connecting to MLflow at %s …", mlflow_uri)
    t0 = time.monotonic()
    use_mlflow = setup_mlflow(mlflow_uri, experiment)
    log.info("MLflow ready in %.1f s", time.monotonic() - t0)

    log.info("Opening MLflow run …")
    t0 = time.monotonic()
    val_metrics: dict[str, float] = {}

    with mlflow_run(use_mlflow, run_name=run_name) as run:
        log.info("Run opened in %.1f s  (id=%s)", time.monotonic() - t0, run.info.run_id)
        log.info("Logging params …")
        t0 = time.monotonic()
        mlflow_log_params(use_mlflow, {
            "n_samples":    n_samples,
            "val_split":    val_split,
            "batch_size":   batch_size,
            "n_epochs":     n_epochs,
            "lr":           lr,
            "seed":         seed,
            "d_model":      d_model,
            "nhead":        nhead,
            "ffn_dim":      ffn_dim,
            "n_enc_layers": n_enc_layers,
            "n_dec_layers": n_dec_layers,
            "lambda_cont":  lambda_cont,
            "lambda_cat":   lambda_cat,
            "max_seq_len":  max_seq_len,
            "dropout":      dropout,
            "n_params":     n_params,
        })
        log.info("Params logged in %.1f s", time.monotonic() - t0)
        log.info("Starting training loop (%d epochs) …", n_epochs)

        for epoch in range(1, n_epochs + 1):
            model.train()
            total_loss = cont_loss_sum = cat_loss_sum = tok_loss_sum = 0.0

            with tqdm(train_loader, desc=f"Epoch {epoch:3d}/{n_epochs}", unit="batch", leave=False) as pbar:
                for batch in pbar:
                    mel       = batch["mel"].unsqueeze(1).to(device)   # (B, 1, MEL_BINS, T)
                    token_ids = batch["token_ids"].to(device)          # (B, S)
                    cont      = batch["cont_values"].to(device)        # (B, S, CW)
                    cat       = batch["cat_values"].to(device)         # (B, S, KW)

                    # Teacher forcing: feed tokens[:-1], predict tokens[1:]
                    tgt_in_ids  = token_ids[:, :-1]
                    tgt_in_cont = cont[:, :-1]
                    tgt_in_cat  = cat[:, :-1]
                    tgt_ids     = token_ids[:, 1:]
                    tgt_cont    = cont[:, 1:]
                    tgt_cat     = cat[:, 1:]

                    optimizer.zero_grad()
                    token_logits, cont_pred, cat_logits = model(
                        mel, tgt_in_ids, tgt_in_cont, tgt_in_cat
                    )

                    loss, tok_l, c_l, k_l = _compute_loss(
                        token_logits, cont_pred, cat_logits,
                        tgt_ids, tgt_cont, tgt_cat,
                        model.cont_mask, model.cat_n_cls,
                        vocab.pad, lambda_cont, lambda_cat,
                    )

                    loss.backward()
                    torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                    optimizer.step()

                    B = mel.size(0)
                    total_loss    += loss.item() * B
                    tok_loss_sum  += tok_l.item() * B
                    cont_loss_sum += c_l.item() * B
                    cat_loss_sum  += k_l.item() * B
                    pbar.set_postfix(loss=f"{loss.item():.4f}")

            total_loss    /= n_train
            tok_loss_sum  /= n_train
            cont_loss_sum /= n_train
            cat_loss_sum  /= n_train

            val_metrics = evaluate(model, val_loader, device, vocab)

            mlflow_log_metrics(use_mlflow, {
                "train_loss":       total_loss,
                "train_token_loss": tok_loss_sum,
                "train_cont_loss":  cont_loss_sum,
                "train_cat_loss":   cat_loss_sum,
                **val_metrics,
            }, step=epoch)
            print(
                f"Epoch {epoch:3d}/{n_epochs}  "
                f"loss={total_loss:.4f}  "
                f"tok={tok_loss_sum:.4f}  "
                f"cont={cont_loss_sum:.4f}  "
                f"cat={cat_loss_sum:.4f}  "
                f"val_mel_l1={val_metrics.get('mel_l1', 0):.4f}  "
                f"valid={val_metrics.get('valid_fraction', 0):.2%}"
            )

            # ── Checkpoint artifacts ─────────────────────────────────────────
            if epoch % ckpt_every == 0 or epoch == n_epochs:
                log.info("Logging checkpoint artifacts (epoch %d) …", epoch)
                _log_checkpoint_artifacts(use_mlflow, model, ckpt_batch, vocab, device, step=epoch)

        log.info("Saving model artifact …")
        t0 = time.monotonic()
        mlflow_log_model(use_mlflow, model, "model")
        log.info("Model artifact saved in %.1f s", time.monotonic() - t0)
        print(f"Run saved: {run.info.run_id}")

    return val_metrics


def main() -> None:
    """CLI entry point: rustic-train-oracle."""
    parser = argparse.ArgumentParser(description="Train TheOracle (AR graph decoder)")
    parser.add_argument("--config", required=True, help="Path to TOML config file")
    parser.add_argument("--run-name", default=None)
    args = parser.parse_args()

    from rustic_ml.legacy.config import load_config
    config = load_config(args.config)
    train(config, run_name=args.run_name)
