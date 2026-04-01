"""
ThePainter model definition.

A transformer encoder over a GraphSpec token sequence followed by an upsampling
decoder that produces a log-mel spectrogram. Also acts as the spec encoder for
contrastive alignment with the mel encoder.

Architecture (placeholder — to be finalised before first training run):
  Encoder:
    - Token embedding (vocab_size → d_model) + continuous/categorical value projection
    - N× transformer encoder layers (self-attention, FFN)
    - [CLS] token pooling → spec embedding vector (used for contrastive training)
  Decoder:
    - Linear projection → (MEL_BINS, T_frames) via learned upsampling

Input:  token_ids (B, S), cont_values (B, S, cont_width), cat_values (B, S, cat_width)
Output: predicted log-mel (B, MEL_BINS, T)
        spec_embedding (B, d_model)  — for contrastive loss
"""
# TODO: implement ThePainter architecture
import torch.nn as nn


class ThePainter(nn.Module):
    """Surrogate renderer: token sequence → predicted log-mel spectrogram."""

    def __init__(self) -> None:
        super().__init__()
        raise NotImplementedError("ThePainter is not yet implemented")

    def forward(self, token_ids, cont_values, cat_values):
        raise NotImplementedError
