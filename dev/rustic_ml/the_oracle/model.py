"""
TheOracle model definition.

An encoder-decoder transformer.

Encoder:
  - CNN patch embedding over log-mel spectrogram (128 mel × T frames)
  - N× transformer encoder layers (bidirectional self-attention)
  - Weights optionally initialised from a contrastively aligned mel encoder

Decoder:
  - Autoregressive, causal self-attention + cross-attention to encoder output
  - Three output heads at each position:
      token_head   softmax over 48-token vocabulary
      cont_head    sigmoid-scaled continuous values, masked to cont_layout[token_id]
      cat_head     per-field softmax, masked to cat_layout[token_id]
  - At inference, TheDecider's predicted note is injected as a forced prefix

Training:
  - Teacher forcing; loss = CE(token) + λ_c * MSE(cont) + λ_k * CE(cat)
  - Inactive head slots are zero-masked

Input:  mel (B, 1, MEL_BINS, T), tgt_token_ids (B, S), tgt_cont (B, S, cont_width),
        tgt_cat (B, S, cat_width)
Output: token_logits (B, S, vocab_size), cont_pred (B, S, cont_width),
        cat_logits (B, S, cat_width, n_classes)
"""
# TODO: implement TheOracle architecture
import torch.nn as nn


class TheOracle(nn.Module):
    """Autoregressive graph decoder: mel spectrogram → GraphSpec token sequence."""

    def __init__(self) -> None:
        super().__init__()
        raise NotImplementedError("TheOracle is not yet implemented")

    def forward(self, mel, tgt_token_ids, tgt_cont, tgt_cat):
        raise NotImplementedError
