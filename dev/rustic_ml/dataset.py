"""
Dataset generation and loading utilities.

Dual-mode: saves .npz batches to disk AND exposes a torch.utils.data.Dataset.
"""
import os
from pathlib import Path

import numpy as np
import librosa
from tqdm import tqdm

from .encoding import encode_adsr, NOTE_MIN, NOTE_MAX, ADSR_MIN, ADSR_MAX

# Fixed render constants (match ML_PLAN)
NOTE_ON = 0.05
NOTE_OFF = 0.6
DURATION = 1.0
SAMPLE_RATE = 44100
MEL_BINS = 128
N_FFT = 2048
HOP_LENGTH = 512


def random_spec(waveform: str = "sine") -> dict:
    """Generate a random GraphSpec-compatible dict.

    Samples:
        note: uniform integer from [NOTE_MIN, NOTE_MAX]
        attack/decay/release: log-uniform from [ADSR_MIN, ADSR_MAX]
        sustain: uniform from [0.0, 1.0]

    Returns a dict compatible with rustic_py.render().
    """
    note = int(np.random.randint(NOTE_MIN, NOTE_MAX + 1))
    log_min = np.log(ADSR_MIN)
    log_max = np.log(ADSR_MAX)
    attack = float(np.exp(np.random.uniform(log_min, log_max)))
    decay = float(np.exp(np.random.uniform(log_min, log_max)))
    sustain = float(np.random.uniform(0.0, 1.0))
    release = float(np.exp(np.random.uniform(log_min, log_max)))

    return {
        "note": note,
        "note_on": NOTE_ON,
        "note_off": NOTE_OFF,
        "duration": DURATION,
        "sample_rate": float(SAMPLE_RATE),
        "block_size": 512,
        "source": {
            "waveform": waveform,
            "frequency_relation": "identity",
            "attack": attack,
            "decay": decay,
            "sustain": sustain,
            "release": release,
        },
        "filters": [],
    }


def render_mel(spec_dict: dict) -> np.ndarray:
    """Render a spec dict to a log-mel spectrogram.

    Steps:
        1. Render via rustic_py.render() → shape (N, 2) stereo
        2. Mix to mono: mean over channels
        3. Compute mel spectrogram with librosa
        4. Convert to dB scale

    Returns:
        np.ndarray of shape (MEL_BINS, T)
    """
    from rustic_py.rustic_py import render

    audio = render(spec_dict)  # shape (N, 2)
    mono = np.mean(audio, axis=1).astype(np.float32)

    mel = librosa.feature.melspectrogram(
        y=mono,
        sr=SAMPLE_RATE,
        n_mels=MEL_BINS,
        n_fft=N_FFT,
        hop_length=HOP_LENGTH,
    )
    log_mel = librosa.power_to_db(mel, ref=np.max)
    return log_mel.astype(np.float32)


def generate_dataset(
    n_samples: int,
    output_dir: str | Path,
    batch_size: int = 1000,
    waveform: str = "sine",
) -> None:
    """Generate a dataset of random specs and save as batched .npz files.

    Each .npz file contains:
        'mel':  (B, MEL_BINS, T)   float32
        'note': (B,)               int32
        'adsr': (B, 4)             float32

    Args:
        n_samples:  Total number of samples to generate.
        output_dir: Directory where .npz files will be written.
        batch_size: Samples per .npz file.
        waveform:   Source waveform type.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    batch_idx = 0
    mel_batch, note_batch, adsr_batch = [], [], []

    with tqdm(total=n_samples, desc="Generating dataset") as pbar:
        for i in range(n_samples):
            spec = random_spec(waveform=waveform)
            mel = render_mel(spec)
            src = spec["source"]

            mel_batch.append(mel)
            note_batch.append(spec["note"])
            adsr_batch.append(
                encode_adsr(src["attack"], src["decay"], src["sustain"], src["release"])
            )

            pbar.update(1)

            if len(mel_batch) == batch_size or i == n_samples - 1:
                # Pad mels to same T dimension within batch
                max_t = max(m.shape[1] for m in mel_batch)
                padded = np.zeros((len(mel_batch), MEL_BINS, max_t), dtype=np.float32)
                for j, m in enumerate(mel_batch):
                    padded[j, :, : m.shape[1]] = m

                np.savez_compressed(
                    output_dir / f"batch_{batch_idx:04d}.npz",
                    mel=padded,
                    note=np.array(note_batch, dtype=np.int32),
                    adsr=np.stack(adsr_batch).astype(np.float32),
                )
                batch_idx += 1
                mel_batch, note_batch, adsr_batch = [], [], []


class NpzDataset:
    """torch.utils.data.Dataset backed by .npz files on disk.

    Args:
        source: Directory path or list of .npz file paths.

    Each item returned by __getitem__ is a tuple:
        (mel_tensor, note_int, adsr_tensor)

        mel_tensor:  float32 torch.Tensor of shape (1, MEL_BINS, T)
        note_int:    int
        adsr_tensor: float32 torch.Tensor of shape (4,)
    """

    def __init__(self, source: str | Path | list):
        import torch  # noqa: F401 — validate torch is available at init

        if isinstance(source, (str, Path)):
            source = Path(source)
            self._paths = sorted(source.glob("*.npz"))
        else:
            self._paths = [Path(p) for p in source]

        if not self._paths:
            raise ValueError(f"No .npz files found in {source!r}")

        # Build index: list of (file_idx, sample_idx_within_file)
        self._index: list[tuple[int, int]] = []
        self._cache: dict[int, dict] = {}  # simple LRU would be overkill here

        for file_idx, path in enumerate(self._paths):
            data = np.load(path)
            n = data["note"].shape[0]
            self._index.extend((file_idx, j) for j in range(n))

    def __len__(self) -> int:
        return len(self._index)

    def __getitem__(self, idx: int):
        import torch

        file_idx, sample_idx = self._index[idx]

        if file_idx not in self._cache:
            self._cache = {file_idx: dict(np.load(self._paths[file_idx]))}

        data = self._cache[file_idx]
        mel = data["mel"][sample_idx]          # (MEL_BINS, T)
        note = int(data["note"][sample_idx])
        adsr = data["adsr"][sample_idx]        # (4,)

        mel_tensor = torch.from_numpy(mel).float().unsqueeze(0)   # (1, MEL_BINS, T)
        adsr_tensor = torch.from_numpy(adsr).float()

        return mel_tensor, note, adsr_tensor
