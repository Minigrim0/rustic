"""
TheOracle model definition.

An encoder-decoder transformer that maps a log-mel spectrogram to a complete
canonical GraphSpec token sequence.

Encoder:
  - Conv2d patch embedding over (B, 1, 128, 87) → (B, 672, d_model)
  - Learned positional embedding over 672 patch slots
  - N× TransformerEncoder layers (bidirectional self-attention)

Decoder:
  - Input embedding: token + continuous projection + categorical embedding
  - Learned positional embedding over sequence positions
  - N× TransformerDecoder layers (causal self-attention + cross-attention to encoder)
  - Three output heads at each position:
      token_head   Linear → vocab_size         (CrossEntropy loss)
      cont_head    Linear → cont_width + Sigmoid (masked MSE loss)
      cat_head     Linear → cat_width × 128     (per-field CrossEntropy, masked)

Buffers (moved to device automatically):
  cont_mask   (vocab_size, cont_width)  bool  — active continuous fields per token
  cat_n_cls   (vocab_size, cat_width)   long  — n_classes per categorical field (0 = inactive)

Input:  mel (B, 1, MEL_BINS, T), tgt_token_ids (B, S), tgt_cont (B, S, cont_width),
        tgt_cat (B, S, cat_width)
Output: token_logits (B, S, vocab_size), cont_pred (B, S, cont_width),
        cat_logits (B, S, cat_width, MAX_CAT_CLASSES)
"""
from __future__ import annotations

import torch
import torch.nn as nn

from rustic_ml.autoregressive.vocab import Vocabulary

# ── Mel spectrogram constants ──────────────────────────────────────────────────
_MEL_BINS  = 128
_T_FIXED   = 87    # floor(44100/512) + 1 — fixed for DURATION=1.0, SR=44100, hop=512
_PATCH_F   = 4
_PATCH_T   = 4
_N_F_PATCH = _MEL_BINS // _PATCH_F                         # 32
_N_T_PATCH = (_T_FIXED - _PATCH_T) // _PATCH_T + 1        # 21
_ENC_SEQ   = _N_F_PATCH * _N_T_PATCH                       # 672

MAX_CAT_CLASSES = 128   # MIDI note is the widest categorical field
MAX_DEC_SEQ     = 512   # upper bound for decoder positional embedding


class TheOracle(nn.Module):
    """Autoregressive graph decoder: log-mel spectrogram → GraphSpec token sequence."""

    def __init__(
        self,
        vocab: Vocabulary,
        d_model: int = 256,
        nhead: int = 8,
        ffn_dim: int = 1024,
        n_enc_layers: int = 4,
        n_dec_layers: int = 4,
        dropout: float = 0.1,
    ) -> None:
        super().__init__()

        vocab_size  = len(vocab)
        cont_width  = vocab.cont_width
        cat_width   = vocab.cat_width

        self.vocab_size = vocab_size
        self.cont_width = cont_width
        self.cat_width  = cat_width
        self.d_model    = d_model
        self.pad_id     = vocab.pad
        self.eos_id     = vocab.eos

        # ── mel encoder ───────────────────────────────────────────────────────
        self.patch_embed = nn.Conv2d(
            1, d_model, kernel_size=(_PATCH_F, _PATCH_T), stride=(_PATCH_F, _PATCH_T)
        )
        self.enc_pos = nn.Embedding(_ENC_SEQ, d_model)
        enc_layer = nn.TransformerEncoderLayer(
            d_model=d_model, nhead=nhead, dim_feedforward=ffn_dim,
            dropout=dropout, batch_first=True,
        )
        self.encoder = nn.TransformerEncoder(enc_layer, num_layers=n_enc_layers)

        # ── decoder input embedding ───────────────────────────────────────────
        self.tok_embed  = nn.Embedding(vocab_size, d_model, padding_idx=vocab.pad)
        self.cont_proj  = nn.Linear(cont_width, d_model, bias=False)
        # +1 so that class-0 can serve as a no-op padding embedding for inactive fields
        self.cat_embed  = nn.Embedding(MAX_CAT_CLASSES + 1, d_model, padding_idx=0)
        self.dec_pos    = nn.Embedding(MAX_DEC_SEQ, d_model)
        self.emb_drop   = nn.Dropout(dropout)

        # ── decoder ───────────────────────────────────────────────────────────
        dec_layer = nn.TransformerDecoderLayer(
            d_model=d_model, nhead=nhead, dim_feedforward=ffn_dim,
            dropout=dropout, batch_first=True,
        )
        self.decoder = nn.TransformerDecoder(dec_layer, num_layers=n_dec_layers)

        # ── output heads ──────────────────────────────────────────────────────
        self.token_head = nn.Linear(d_model, vocab_size)
        self.cont_head  = nn.Sequential(
            nn.Linear(d_model, cont_width),
            nn.Sigmoid(),
        )
        self.cat_head   = nn.Linear(d_model, cat_width * MAX_CAT_CLASSES)

        # ── vocabulary masks (registered as buffers → move with .to(device)) ─
        cont_mask = torch.zeros(vocab_size, cont_width, dtype=torch.bool)
        for tid, fields in vocab.cont_layout.items():
            cont_mask[tid, :len(fields)] = True

        cat_n_cls = torch.zeros(vocab_size, cat_width, dtype=torch.long)
        for tid, fields in vocab.cat_layout.items():
            for i, fname in enumerate(fields):
                cat_n_cls[tid, i] = vocab.cat_n_classes[fname]

        self.register_buffer("cont_mask", cont_mask)
        self.register_buffer("cat_n_cls", cat_n_cls)

        self._init_weights()

    def _init_weights(self) -> None:
        for m in self.modules():
            if isinstance(m, nn.Linear):
                nn.init.xavier_uniform_(m.weight)
                if m.bias is not None:
                    nn.init.zeros_(m.bias)
            elif isinstance(m, nn.Embedding):
                nn.init.normal_(m.weight, std=0.02)
                if m.padding_idx is not None:
                    m.weight.data[m.padding_idx].zero_()

    def _encode_mel(self, mel: torch.Tensor) -> torch.Tensor:
        """Encode log-mel spectrogram to encoder memory.

        Args:
            mel: (B, 1, MEL_BINS, T) — T is cropped/padded to _T_FIXED internally.

        Returns:
            memory: (B, _ENC_SEQ, d_model)
        """
        B, _, n_freq, T = mel.shape
        # Crop or zero-pad time axis to _T_FIXED
        if T > _T_FIXED:
            mel = mel[:, :, :, :_T_FIXED]
        elif T < _T_FIXED:
            pad = torch.zeros(B, 1, n_freq, _T_FIXED - T, device=mel.device, dtype=mel.dtype)
            mel = torch.cat([mel, pad], dim=-1)

        x = self.patch_embed(mel)               # (B, d_model, N_F, N_T)
        x = x.flatten(2).transpose(1, 2)        # (B, NF*NT, d_model)

        pos = torch.arange(x.size(1), device=x.device)
        x = x + self.enc_pos(pos)
        return self.encoder(x)                  # (B, ENC_SEQ, d_model)

    def _embed_decoder_input(
        self,
        token_ids: torch.Tensor,
        cont:      torch.Tensor,
        cat:       torch.Tensor,
    ) -> torch.Tensor:
        """Build decoder input embedding from token IDs + values.

        Args:
            token_ids: (B, S) int64
            cont:      (B, S, cont_width) float32
            cat:       (B, S, cat_width)  int64

        Returns:
            (B, S, d_model)
        """
        _, S = token_ids.shape

        emb = self.tok_embed(token_ids)                 # (B, S, d_model)
        emb = emb + self.cont_proj(cont)                # continuous params

        # Sum categorical field embeddings; cat values are 0-indexed class IDs,
        # shift by +1 so that 0 maps to the padding embedding (no-op).
        for f in range(self.cat_width):
            emb = emb + self.cat_embed(cat[:, :, f] + 1)

        pos = torch.arange(S, device=token_ids.device)
        emb = emb + self.dec_pos(pos)
        return self.emb_drop(emb)

    def forward(
        self,
        mel:           torch.Tensor,
        tgt_token_ids: torch.Tensor,
        tgt_cont:      torch.Tensor,
        tgt_cat:       torch.Tensor,
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """Teacher-forced forward pass.

        Args:
            mel:           (B, 1, MEL_BINS, T)
            tgt_token_ids: (B, S) — decoder input tokens (SOS-prefixed, shifted right)
            tgt_cont:      (B, S, cont_width) — decoder input continuous values
            tgt_cat:       (B, S, cat_width)  — decoder input categorical values

        Returns:
            token_logits: (B, S, vocab_size)
            cont_pred:    (B, S, cont_width)
            cat_logits:   (B, S, cat_width, MAX_CAT_CLASSES)
        """
        B, S = tgt_token_ids.shape

        memory = self._encode_mel(mel)                  # (B, ENC_SEQ, d_model)

        tgt_emb = self._embed_decoder_input(tgt_token_ids, tgt_cont, tgt_cat)

        # Causal mask — upper-triangular, -inf above diagonal
        causal_mask = nn.Transformer.generate_square_subsequent_mask(
            S, device=tgt_token_ids.device
        )
        # Padding key mask — True where token is PAD (ignored positions)
        tgt_key_mask = (tgt_token_ids == self.pad_id)   # (B, S)

        dec_out = self.decoder(
            tgt_emb, memory,
            tgt_mask=causal_mask,
            tgt_key_padding_mask=tgt_key_mask,
        )                                                # (B, S, d_model)

        token_logits = self.token_head(dec_out)          # (B, S, vocab_size)
        cont_pred    = self.cont_head(dec_out)           # (B, S, cont_width)
        cat_raw      = self.cat_head(dec_out)            # (B, S, cat_width * MAX_CAT_CLASSES)
        cat_logits   = cat_raw.view(B, S, self.cat_width, MAX_CAT_CLASSES)

        return token_logits, cont_pred, cat_logits

    @torch.no_grad()
    def greedy_decode(
        self,
        mel:      torch.Tensor,
        vocab:    Vocabulary,
        max_len:  int = 256,
        temperature: float = 1.0,
    ) -> tuple[list[int], torch.Tensor, torch.Tensor]:
        """Greedy (or temperature-sampled) autoregressive decoding for a single mel.

        Args:
            mel:         (1, 1, MEL_BINS, T)
            vocab:       Vocabulary instance.
            max_len:     Maximum sequence length before forced stop.
            temperature: >1 = more random, 1.0 = argmax (greedy).

        Returns:
            token_ids:   list[int] of predicted token IDs (includes SOS, EOS)
            cont_values: (seq_len, cont_width) float32 tensor
            cat_values:  (seq_len, cat_width)  int64 tensor
        """
        self.eval()
        device = next(self.parameters()).device
        memory = self._encode_mel(mel.to(device))       # (1, ENC_SEQ, d_model)

        token_ids:   list[int]         = [vocab.sos]
        cont_values: list[torch.Tensor] = [torch.zeros(self.cont_width)]
        cat_values:  list[torch.Tensor] = [torch.zeros(self.cat_width, dtype=torch.long)]

        for _ in range(max_len):
            ids_t  = torch.tensor([token_ids],  device=device)            # (1, S)
            cont_t = torch.stack(cont_values).unsqueeze(0).to(device)     # (1, S, CW)
            cat_t  = torch.stack(cat_values).unsqueeze(0).to(device)      # (1, S, KW)

            tgt_emb = self._embed_decoder_input(ids_t, cont_t, cat_t)
            S = ids_t.size(1)
            causal_mask = nn.Transformer.generate_square_subsequent_mask(S, device=device)
            dec_out = self.decoder(tgt_emb, memory, tgt_mask=causal_mask) # (1, S, d_model)

            last = dec_out[:, -1, :]                    # (1, d_model)

            tok_logits = self.token_head(last)           # (1, vocab_size)
            if temperature != 1.0:
                tok_logits = tok_logits / temperature
                next_tok = int(torch.multinomial(tok_logits.softmax(-1), 1).item())
            else:
                next_tok = int(tok_logits.argmax(-1).item())

            next_cont = self.cont_head(last).squeeze(0).cpu()  # (cont_width,)
            cat_raw   = self.cat_head(last).view(self.cat_width, MAX_CAT_CLASSES)
            next_cat  = cat_raw.argmax(-1).cpu()               # (cat_width,)

            token_ids.append(next_tok)
            cont_values.append(next_cont)
            cat_values.append(next_cat)

            if next_tok == vocab.eos:
                break

        return (
            token_ids,
            torch.stack(cont_values),                   # (seq_len, cont_width)
            torch.stack(cat_values),                    # (seq_len, cat_width)
        )
