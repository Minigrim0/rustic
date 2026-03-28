# ML Plan: Audio → GraphSpec

## Goal

Given an audio sample, predict the `GraphSpec` (synthesis parameters) that would reproduce it.
This is *analysis-by-synthesis*: the Rust renderer is a free oracle — every random spec we generate
can be rendered for free, giving infinite `(audio, spec)` training pairs with no external dataset.

---

## Infrastructure

- **Python env:** `dev/` with uv, Python 3.12, ROCm torch
- **Renderer:** `rustic-py` PyO3 bindings — `GraphSpec.render()` → `np.ndarray`
- **Tracking:** MLflow on localhost:5000 via `docker-compose.yml` (PostgreSQL backend, artifacts in `/data/mlflow`)
- **Audio features:** librosa mel-spectrograms (128 bins, 2048 FFT, 512 hop)
- **Loss:** multi-scale STFT + log-mel L1 (perceptual, not raw waveform MSE)

---

## Phases

### Phase 0 — Pipeline & data generation (current)

**Goal:** working training loop, rendering pipeline, MLflow tracking — before any real model.

**Files to create:**
- `dev/rustic_dev/encoding.py` — flat param encoding/decoding
- `dev/rustic_dev/dataset.py` — random spec generator + renderer + mel extraction
- `dev/rustic_dev/loss.py` — perceptual loss functions

**Encoding (Phase 1 target vector):**
- `note` — int 0–127 (classification target, kept separate)
- `log_attack`, `log_decay`, `sustain`, `log_release` — 4 floats (regression targets)
- ADSR encoded in log-space (spans multiple orders of magnitude: 0.001s–2.0s)

**Dataset format:** batched `.npz` files: `{'mel': (N, 128, T), 'note': (N,), 'adsr': (N, 4)}`

**Fixed render parameters:** `note_on=0.05`, `note_off=0.6`, `duration=1.0`, `sr=44100`

**Note range:** MIDI 36–84 (C2–C6), musically reasonable, avoids extreme frequencies

---

### Phase 1 — Supervised regression, fixed structure (sine, no filters)

**Goal:** prove the pipeline end-to-end. Simplest possible model.

**Spec:** waveform fixed to `"sine"`, no filters. Predict note + ADSR only.

**Model:** CNN encoder on log-mel → two heads:
- Classification head: softmax over 49 notes (MIDI 36–84)
- Regression head: 4 outputs for log-ADSR (decoded back to seconds at eval)

**Loss:** `CrossEntropy(note) + λ * MSE(adsr)`

**Bindings needed:** none (single source, no filters — already supported)

**MLflow logging:** loss curves, predicted vs ground-truth audio artifacts, ADSR scatter plots

**Dataset size:** 10k samples for first run, scale up if underfitting

---

### Phase 2 — Add waveform classification

**Goal:** mixed discrete/continuous output.

**Adds:** softmax head over 7 waveform types: `sine`, `square`, `sawtooth`, `triangle`,
`whitenoise`, `pinknoise`, `blank`

**Change:** training data now samples waveform uniformly, not fixed to sine

**Bindings needed:** none

---

### Phase 3 — Single optional filter

**Goal:** handle one optional filter at a fixed position.

**Spec adds:**
- Which filter (or none) — `n_filters + 1` classes
- Filter params — padded/masked regression head for unused filter slot

**Model upgrade:** larger regression head, masked loss on the filter param slot

**Bindings needed:** none (linear chain already supported)

---

### Phase 4 — Evolutionary search (parallel track, generates hard training data)

**Goal:** evolutionary solver for matching a target audio. Simultaneously generates
near-miss training examples that pure random sampling won't produce.

**Algorithm:**
1. Random population of GraphSpecs
2. Render each → compute perceptual distance to target
3. Select top-k → mutate (param jitter, swap filter type, add/remove filter)
4. Repeat for N generations

**Log all candidates to MLflow** as a run group — best candidate + score per generation.

**Bindings gap:** none for single-source; multi-source would require `GraphSpec` to accept
a list of sources (not yet implemented in `rustic-py`).

---

### Phase 5 — Variable filter chain (autoregressive decoder)

**Goal:** full variable-length `GraphSpec` with 0–N filters.

**Model:** autoregressive decoder — at each step, predict `(filter_type | STOP, params)`.
Stop token ends the chain. Similar to sequence generation.

**Bindings gaps needed:**
- Multi-source support in `GraphSpec` (list of sources, not just one)
- Source-relative filter parameterization (e.g., cutoff as fraction of note frequency)

---

### Phase 6 — World model (differentiable surrogate renderer)

**Goal:** learn a neural net that predicts how a spec change affects audio features.

**Input:** `(current_spec_embedding, proposed_action)` → **Output:** predicted feature delta

**Trained on:** all `(spec, render)` pairs from Phases 1–5 + evolutionary search

**Enables:** gradient-based planning without calling the Rust renderer

---

### Phase 7 — Policy (full system)

**Goal:** policy network trained with world model as surrogate.
Input: target audio features. Output: sequence of graph-building actions.

**Architecture:** Dreamer / Dyna style — plan in neural space, validate with real renders.

---

## Key Design Decisions

| Decision | Choice |
|---|---|
| Audio representation | Log-mel spectrogram (128 bins, 2048 FFT, 512 hop, sr=44100) |
| Perceptual loss | Multi-scale STFT L1 + log-mel L1 |
| Gradient through renderer | Not attempted until Phase 6; evolution/supervised until then |
| Graph structure | Fixed + masking (Phase 1–3), autoregressive (Phase 5+) |
| Note range | MIDI 36–84 (49 classes) |
| ADSR encoding | Log-space for attack/decay/release, linear for sustain |
| Tracking | MLflow — params, losses, audio artifacts per run |
| Dataset | Synthetic only; renderer is the oracle |

---

## Binding Gaps (in order of need)

| Needed for | Gap |
|---|---|
| Phase 4+ (rich patches) | Multiple sources per `GraphSpec` |
| Phase 5 | Source-relative filter parameterization |
| Phase 6+ | Batch rendering (render N specs in one call) |

---

## Autoregressive Strategy (current track)

This supersedes Phases 5–7 above. The model is a full encoder-decoder transformer that maps a
mel spectrogram to a complete `GraphSpec` token sequence, using the hierarchical token vocabulary
defined in `dev/rustic_ml/autoregressive/vocab.py`.

### Why autoregressive over the phased approach

The phased approach requires a separate architecture at each phase. The autoregressive approach
handles variable structure natively: the token sequence encodes the full graph (multi-source,
ADSR, DAG topology, filter params) and the model generates it left-to-right conditioned on the
mel encoder output. Training data is purely synthetic.

---

### Overall architecture

```
Mel spectrogram
    ├── Note classifier  (standalone, trains independently)
    │       ↓ hard prefix token injected into decoder
    └── Mel encoder  (CNN + transformer, contrastively aligned to spec encoder)
              ↓ cross-attention
         AR Decoder → token sequence
              ↓
         sequence_to_spec() → GraphSpec → Rust renderer → audio
```

**Note classifier:** 128-class softmax over mel features. Easy to train to >95% accuracy.
The predicted note is injected as a hard prefix (the NOTE token) in the AR decoder, eliminating
uncertainty over the single most consequential decision in the sequence. Verifiable: render the
output and check the fundamental frequency.

**Mel encoder / spec encoder contrastive alignment (CLIP-style):** train the mel encoder and a
spec encoder (token sequence → embedding) to produce close embeddings for matching pairs and
distant embeddings for non-matching pairs. The mel encoder then implicitly learns timbre features
(waveform mix, ADSR shape, filter presence) without requiring explicit labels. Degrades
gracefully: a slightly off embedding makes generation less certain, not wrong. Hard auxiliary
predictions for waveform type, ADSR, filter presence are *not* used — error propagation from an
incorrect waveform label would poison the entire generation.

The spec encoder also doubles as the surrogate renderer backbone (Step 3).

---

### Step 1 — Canonical representation (prerequisite for all training)

**Problem:** the same perceptual output maps to many token sequences (source ordering, connection
ordering, equivalent ADSR shapes, redundant filters). Cross-entropy training on non-canonical
data produces smeared distributions at ordering decision boundaries.

**Solution:** `GraphSpec.canonical()` — a deterministic normalization producing one token
sequence per perceptual equivalence class.

**Canonical ordering rules:**

*Sources within a MultiSourceSpec:*
- Primary key: waveform `type_id` lexicographic (discrete, always stable)
- Secondary key: peak envelope amplitude descending (sustain level × glob_ampl scale)
- Rationale: waveform type is always unambiguous from audio; amplitude only breaks ties within
  the same waveform type, so the model is never asked to predict a near-coin-flip ordering.

*MultiSourceSpec blocks:*
- Primary key: number of sub-sources descending
- Secondary key: dominant waveform type lexicographic (most common WF in the block)
- Tertiary: peak glob_ampl sustain level descending

*Filters:*
- Topological sort of the DAG first (Kahn's algorithm — earlier in signal path = earlier in
  sequence); ties broken by `type_id` lexicographic, then primary parameter value ascending.
- After sorting, renumber all filter indices in connections accordingly.

*Connections:*
- Sort by type: `SourceSink < SourceFilter < FilterFilter < FilterSink`
- Within each type: exit node index ascending, then entry node index ascending.

**Implementation:** `GraphSpec.canonical() -> GraphSpec` in
`rustic-py/python/rustic_py/_classes.py`. Called before tokenization in the dataset pipeline.

---

### Step 1b — Training distribution coverage

**Problem:** `GraphSpec.random(complexity=uniform(0, 0.5))` oversamples single-sine-with-ADSR
specs and undersamples interesting timbres (detuned pairs, noise+filter combos, multi-source
mixing).

**Solutions:**
- Biased waveform sampling: within a spec, already-chosen waveforms become less likely to be
  chosen again. `blank` waveforms excluded from random generation entirely (no audio output,
  only pollutes the distribution).
- Complexity sampling: upweight mid-range complexity (0.2–0.4) over near-zero, so the dataset
  has more multi-source and multi-filter examples than pure uniform sampling produces.

**Implementation:** modify `MultiSourceSpec.random()` to accept a waveform exclusion/weight
list, updated per source drawn.

---

### Step 2 — Supervised pretraining on canonicalized synthetic data

**Architecture:** encoder-decoder transformer.
- *Encoder:* CNN patch embedding over log-mel (128 bins × T frames) → transformer encoder
  layers with bidirectional attention. Weights shared with the contrastive mel encoder.
- *Decoder:* autoregressive, causal attention + cross-attention to encoder → 3 output heads:
  - Token head: softmax over vocabulary (48 tokens)
  - Continuous values head: active fields only, masked to `cont_layout[token_id]`
  - Categorical values head: per-field softmax, masked to `cat_layout[token_id]`

**Loss:**
```
L = CrossEntropy(token) + λ_cont * MSE(cont_values) + λ_cat * CrossEntropy(cat_values)
```
Inactive head slots are masked to zero loss. Note token predicted by the standalone note
classifier is injected as a forced prefix — no loss on it in the AR model.

**Training data:** `GraphSpec.random(complexity)` → `canonical()` → render → log-mel.
Complexity capped at 0.5 to keep sequences under ~120 tokens.

**Teacher forcing:** standard. Scheduled sampling (gradually replacing GT tokens with model
predictions) can be introduced in late training to reduce exposure bias.

---

### Step 3 — Spec encoder + contrastive alignment

**Spec encoder:** transformer encoder over the token sequence (continuous and categorical values
embedded alongside token IDs) → pooled embedding vector. Same architecture used as surrogate
renderer backbone.

**Contrastive training (CLIP-style):**
- Positive pairs: `(mel, token_sequence)` from the same spec
- Negative pairs: `(mel, token_sequence)` from different specs
- Loss: InfoNCE / NT-Xent over a batch

Trains in parallel with Step 2 (no dependency). The mel encoder from Step 2 and the spec encoder
are aligned in the same embedding space.

**Surrogate renderer use:** spec encoder → MLP head → predicted log-mel. Trained with L2 loss
against real rendered mel. Used in Step 4 as a fast pre-filter and in Step 5 as a value baseline.

---

### Step 4 — Best-of-N reranking at inference

No additional training required. After the supervised AR model is trained:

1. Sample K token sequences from the AR model (temperature sampling, K = 8–32)
2. Surrogate renderer fast-ranks all K: keep top M (e.g. M=4)
3. Real Rust renderer runs on the M survivors → log-mel
4. Multi-scale STFT distance to input mel → return best

**Why not beam search:** beam search maximises sequence likelihood, not perceptual quality.
Two sequences with similar likelihood can produce very different audio. Best-of-N with a
perceptual reranker directly optimises the right objective.

---

### Step 5 — RL fine-tuning

**Prerequisite:** stable supervised model from Step 2. Without it, RL on a 48-token vocabulary
over sequences of length ~100 will not explore productively.

**Algorithm:** PPO with KL penalty against the supervised model.
- *Policy:* AR decoder (frozen encoder, fine-tuned decoder)
- *Reward:* multi-scale STFT distance between rendered output and target mel (real renderer,
  not surrogate, for final reward signal)
- *Value baseline:* surrogate renderer distance (cheap, reduces PPO variance)
- *KL penalty:* prevents the policy from diverging into syntactically invalid sequences

**What RL fixes that supervision cannot:**
- Resolves perceptual equivalences canonical sorting cannot fully eliminate
- Discovers non-obvious combinations rare in the random training distribution

---

### Key design decisions

| Decision | Choice | Rationale |
|---|---|---|
| Note prediction | Standalone classifier, hard prefix | High accuracy, verifiable, removes biggest AR uncertainty |
| Other feature prediction | Contrastive alignment, not explicit | Graceful degradation; avoids error propagation |
| Training objective | CE + MSE + CE on heads, teacher forcing | Stable; perceptual loss deferred to fine-tuning |
| Canonical form | Topological filter sort, WF-primary source sort | Stable, model-predictable ordering |
| Sequence length cap | ~120 tokens (complexity ≤ 0.5) | Keeps quadratic attention tractable |
| Perceptual metric | Multi-scale STFT L1 | Standard; correlates well with human perception |
| RL algorithm | PPO + KL penalty | Prevents grammar collapse |
| Surrogate use | Reranking pre-filter + RL value baseline | Not used for direct gradients (discrete sampling) |
| Data | Synthetic only | Renderer is the oracle; no real-audio dataset needed |

---

## Implementation Notes

Concrete numbers and design choices not visible from the stub files, needed to implement
**ThePainter** and **TheOracle** without re-discussing.

---

### Vocabulary dimensions (runtime values from `Vocabulary.from_rustic()`)

| Name | Value | Source |
|---|---|---|
| `vocab_size` | ~49 (auto from rustic_py) | `len(vocab.tokens)` |
| `cont_width` | 4 (max = 4 ADSR fields: dur, peak, ct, cp) | `vocab.cont_width` |
| `cat_width` | 2 (max = 2 connection index fields) | `vocab.cat_width` |
| max cat classes | 128 (MIDI note) | `max(vocab.cat_n_classes.values())` |
| `MEL_BINS` | 128 | `rustic_ml.legacy.data.generation.MEL_BINS` |
| Mel T frames | 87 (DURATION=1.0, SR=44100, hop=512: 1+floor(44100/512)) | fixed by renderer |

**`cont_layout[token_id]`** → `list[str]` of active field names (length 0–4).
Zero-padded to `cont_width` in tensors. Fields with no entry = inactive.

**`cat_layout[token_id]`** → `list[str]` of active field names (length 0–2).
**`cat_n_classes[field_name]`** → int, n_classes for that field.

Build dense mask tensors once at training start:
```python
# cont_mask[token_id, field_idx] = True if field is active for that token
cont_mask = torch.zeros(vocab_size, cont_width, dtype=torch.bool)
for tid, fields in vocab.cont_layout.items():
    cont_mask[tid, :len(fields)] = True

# cat_n_cls[token_id, field_idx] = n_classes (0 = inactive)
cat_n_cls = torch.zeros(vocab_size, cat_width, dtype=torch.long)
for tid, fields in vocab.cat_layout.items():
    for i, fname in enumerate(fields):
        cat_n_cls[tid, i] = vocab.cat_n_classes[fname]
```
Register both as `model.register_buffer(...)` so they move with `.to(device)`.

---

### ThePainter — concrete architecture

`d_model = 256`, `nhead = 8`, `ffn_dim = 1024`, `n_layers = 4`, `T_fixed = 87`

**Input embedding** (combined, added together):
- Token embedding: `nn.Embedding(vocab_size, d_model)` on `token_ids`
- Continuous projection: `nn.Linear(cont_width, d_model)` on `cont_values` (zero where inactive)
- Categorical projection: `nn.Embedding(max_cat_classes, d_model)` × `cat_width`, sum over fields
  (zero-embed inactive fields by treating class 0 as padding — or use a separate padding embedding)

**Encoder:**
- Prepend a learnable `[CLS]` token embedding
- `nn.TransformerEncoder` with `n_layers=4`, bidirectional (no causal mask)
- CLS output position → `spec_embedding` of shape `(B, d_model)` for contrastive loss

**Mel decoder:**
- `nn.Linear(d_model, d_model * 2)` → GELU → `nn.Linear(d_model * 2, MEL_BINS * T_fixed)`
- Reshape to `(B, MEL_BINS, T_fixed)` — this is the predicted log-mel
- No sigmoid/softmax; log-mel values are unbounded (loss is L1 or MSE against rendered log-mel)

**Forward signature:**
```python
def forward(self, token_ids, cont_values, cat_values):
    # returns: mel_pred (B, MEL_BINS, T_fixed), spec_emb (B, d_model)
```

**Loss:** `L1(mel_pred, rendered_mel)` + optional `multi_scale_stft_loss`. No contrastive loss
in the standalone surrogate training run; contrastive alignment is a separate training phase.

---

### TheOracle — concrete architecture

`d_model = 256`, `nhead = 8`, `ffn_dim = 1024`, `n_enc_layers = 4`, `n_dec_layers = 4`

**Mel encoder (CNN patch → transformer):**
```python
# Patch embedding: non-overlapping 4×4 patches over (1, 128, 87)
nn.Conv2d(1, d_model, kernel_size=(4, 4), stride=(4, 4))  # → (B, d_model, 32, 21)
# Flatten spatial dims → sequence: (B, 32*21, d_model) = (B, 672, d_model)
# Add sinusoidal 1D position embedding on the flattened sequence
# 4 TransformerEncoder layers (bidirectional)
```
Output: encoder memory `(B, 672, d_model)` for cross-attention.

**AR decoder input embedding** (same as ThePainter, without CLS):
- `nn.Embedding(vocab_size, d_model)` + `nn.Linear(cont_width, d_model)` + cat embeddings

**Decoder:**
- Causal `nn.TransformerDecoder` with `n_dec_layers=4`
- Cross-attends to encoder memory at every layer

**Output heads (applied to decoder output `(B, S, d_model)`):**
```python
token_head : nn.Linear(d_model, vocab_size)
             # → (B, S, vocab_size), standard CE loss
cont_head  : nn.Sequential(nn.Linear(d_model, cont_width), nn.Sigmoid())
             # → (B, S, cont_width), masked MSE loss
cat_head   : nn.Linear(d_model, cat_width * MAX_CAT_CLASSES)
             # → reshape (B, S, cat_width, MAX_CAT_CLASSES)
             # mask logits beyond cat_n_cls[token_id, field] with float('-inf')
             # CE loss per active field
```
`MAX_CAT_CLASSES = 128` (MIDI note is the largest categorical field).

**Loss computation:**
```python
# Token loss (skip PAD positions)
pad_mask = (tgt_ids != vocab.pad)  # (B, S)
token_loss = F.cross_entropy(token_logits[pad_mask], tgt_ids[pad_mask])

# Continuous loss (skip inactive fields)
active_cont = cont_mask[tgt_ids]  # (B, S, cont_width) bool
cont_loss = F.mse_loss(cont_pred[active_cont], tgt_cont[active_cont])

# Categorical loss (per field, skip inactive)
cat_loss = 0
for f in range(cat_width):
    active = cat_n_cls[tgt_ids, f] > 0   # (B, S)
    if not active.any():
        continue
    n_cls = cat_n_cls[tgt_ids[active], f]  # varies per position
    logits_f = cat_logits[active, f, :]    # (n_active, MAX_CAT_CLASSES)
    # mask logits beyond n_cls — tricky with variable n_cls; simplest: use max_n_cls per token
    # in practice all fields sharing the same field index have the same n_cls, so it's constant
    cat_loss = cat_loss + F.cross_entropy(logits_f[:, :n_cls.max()], tgt_cat[active, f])

loss = token_loss + λ_cont * cont_loss + λ_cat * cat_loss
# λ_cont = 1.0, λ_cat = 0.5 (starting point, tune by magnitude)
```

**Inference (greedy decode with TheDecider prefix):**
1. Encode mel → encoder memory
2. Feed TheDecider note prediction as forced `NOTE` token (cat field 0 = note class, cont = note_on/note_off defaults)
3. Generate tokens autoregressively until `<EOS>` or max_len
4. Pass sequence to `sequence_to_spec(token_ids, cont_values, cat_values, vocab)` → GraphSpec dict → Rust render

**Forward signature:**
```python
def forward(self, mel, tgt_token_ids, tgt_cont, tgt_cat):
    # mel: (B, 1, MEL_BINS, T)
    # tgt_*: teacher-forced targets shifted right (SOS prepended)
    # returns: token_logits (B, S, vocab_size),
    #          cont_pred    (B, S, cont_width),
    #          cat_logits   (B, S, cat_width, MAX_CAT_CLASSES)
```

---

### ThePainter dataset

`ARDataset` from `rustic_ml.autoregressive.dataset` already produces the right keys:
`token_ids`, `cont_values`, `cat_values`, `mel`. Use it directly with `ar_collate_fn`.
The mel from `ar_collate_fn` is `(B, MEL_BINS, max_T)` — crop/pad to `T_fixed=87` before loss.

### TheOracle dataset

Same `ARDataset` + `ar_collate_fn`. All required keys are present.
Call `GraphSpec.canonical()` on the spec before tokenizing to get canonical sequences
(canonical sorting is implemented in `rustic-py/python/rustic_py/_classes.py`).

**Note:** `ARDataset.__getitem__` currently calls `random_ar_spec()` which calls
`GraphSpec.random().to_spec()` but does NOT call `.canonical()` before tokenizing.
Add the canonical call in `generation.py:random_ar_spec()` before returning, so all
generated sequences are already canonical — TheOracle training depends on this.
