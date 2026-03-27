"""
Synthetic dataset generation: random spec sampling and audio rendering.

Contains the data generation side of the pipeline. Loading/batching lives
in dataset.py. These functions are safe to call from subprocesses.
"""
from __future__ import annotations

from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed
from multiprocessing.queues import Queue as MPQueue
import threading

import numpy as np
import librosa
from tqdm import tqdm

from rustic_ml.legacy.data.encoding import (
    encode_adsr, encode_waveform,
    NOTE_MIN, NOTE_MAX, ADSR_MIN, ADSR_MAX, WAVEFORMS,
)

from rustic_py import GraphSpec

# Fixed audio clip constants
DURATION = 1.0
SAMPLE_RATE = 44100
MEL_BINS = 128
N_FFT = 2048
HOP_LENGTH = 512

# Note timing sampling ranges (seconds)
NOTE_ON_MIN = 0.0
NOTE_ON_MAX = 0.3
NOTE_DURATION_MIN = 0.1
NOTE_DURATION_MAX = 0.9


def random_spec(complexity: float) -> dict:
    """Generate a random GraphSpec-compatible dict.

    Samples:
        note:         uniform integer from [NOTE_MIN, NOTE_MAX]
        attack/decay/release: log-uniform from [ADSR_MIN, ADSR_MAX]
        sustain:      uniform from [0.0, 1.0]
        waveform:     uniform from WAVEFORMS if None, otherwise fixed
        note_on:      uniform from [NOTE_ON_MIN, NOTE_ON_MAX]
        note_off:     note_on + uniform note_duration from
                      [NOTE_DURATION_MIN, NOTE_DURATION_MAX], capped at DURATION

    Returns a dict compatible with rustic_py.render().
    """
    return GraphSpec.random(complexity).canonical().to_spec()


def render_mel(
    spec_dict: dict,
    sample_rate: int = SAMPLE_RATE,
    n_mels: int = MEL_BINS,
    n_fft: int = N_FFT,
    hop_length: int = HOP_LENGTH,
) -> np.ndarray:
    """Render a spec dict to a log-mel spectrogram.

    Steps:
        1. Render via rustic_py.render() → shape (N, 2) stereo
        2. Mix to mono: mean over channels
        3. Compute mel spectrogram with librosa
        4. Convert to dB scale

    Returns:
        np.ndarray of shape (n_mels, T)
    """
    from rustic_py.rustic_py import render  # type: ignore[import]

    audio = render(spec_dict)  # shape (N, 2)
    mono = np.mean(audio, axis=1).astype(np.float32)

    mel = librosa.feature.melspectrogram(
        y=mono,
        sr=sample_rate,
        n_mels=n_mels,
        n_fft=n_fft,
        hop_length=hop_length,
    )
    log_mel = librosa.power_to_db(mel, ref=np.max)
    return log_mel.astype(np.float32)


def _generate_batch(
    batch_size: int,
    output_path: Path,
    waveform: str | None,
    progress_queue: MPQueue | None = None,
    slot_id: int = 0,
) -> Path:
    """Generate a single batch of samples and save it as a .npz file.

    This function is designed to run in a subprocess — it imports rustic_py
    and numpy independently, so it is safe to call from ProcessPoolExecutor.

    Each .npz contains arrays:
        mel:      (B, MEL_BINS, T) float32
        note:     (B,)             int32
        adsr:     (B, 4)           float32   [log(A), log(D), S, log(R)]
        waveform: (B,)             int32
        timing:   (B, 2)           float32   [note_on, note_off] in seconds

    Args:
        batch_size:      Number of samples to generate.
        output_path:     Full path for the output .npz file.
        waveform:        Source waveform type, or None for uniform random sampling.
        progress_queue:  Optional multiprocessing.Queue.
        slot_id:         Index of this worker's display slot.

    Returns:
        The output_path, so callers can track which batches completed.
    """
    mel_batch, note_batch, adsr_batch, waveform_batch, timing_batch = [], [], [], [], []

    for _ in range(batch_size):
        spec = random_spec(waveform=waveform)
        mel = render_mel(spec)
        src = spec["source"]
        mel_batch.append(mel)
        note_batch.append(spec["note"])
        adsr_batch.append(
            encode_adsr(src["attack"], src["decay"], src["sustain"], src["release"])
        )
        waveform_batch.append(encode_waveform(src["waveform"]))
        timing_batch.append([spec["note_on"], spec["note_off"]])
        if progress_queue is not None:
            progress_queue.put(slot_id)

    max_t = max(m.shape[1] for m in mel_batch)
    padded = np.zeros((len(mel_batch), MEL_BINS, max_t), dtype=np.float32)
    for j, m in enumerate(mel_batch):
        padded[j, :, : m.shape[1]] = m

    np.savez_compressed(
        output_path,
        mel=padded,
        note=np.array(note_batch, dtype=np.int32),
        adsr=np.stack(adsr_batch).astype(np.float32),
        waveform=np.array(waveform_batch, dtype=np.int32),
        timing=np.array(timing_batch, dtype=np.float32),
    )
    return output_path


def _drain_progress(
    queue: MPQueue,
    worker_bars: list,
    overall_bar,
    stop_event: threading.Event,
) -> None:
    """Background thread: drain the progress queue and update ipywidgets bars."""
    while not stop_event.is_set() or not queue.empty():
        try:
            slot_id = queue.get(timeout=0.05)
            worker_bars[slot_id].value += 1
            overall_bar.value += 1
        except Exception:
            pass


def _create_widget_bars(n_slots: int, n_samples: int, batch_size: int) -> tuple:
    """Create and display ipywidgets progress bars for parallel generation."""
    import ipywidgets as widgets
    from IPython.display import display

    overall_bar = widgets.IntProgress(
        value=0, min=0, max=n_samples,
        description="Total:",
        layout=widgets.Layout(width="500px"),
    )
    worker_bars = [
        widgets.IntProgress(
            value=0, min=0, max=batch_size,
            description=f"Worker {i}:",
            layout=widgets.Layout(width="500px"),
        )
        for i in range(n_slots)
    ]
    display(widgets.VBox([overall_bar] + worker_bars))
    return overall_bar, worker_bars


def generate_dataset(
    n_samples: int,
    output_dir: str | Path,
    batch_size: int = 1000,
    waveform: str | None = None,
    start_batch: int = 0,
    n_workers: int = 1,
) -> None:
    """Generate a dataset of random specs and save as batched .npz files.

    Each .npz file contains:
        'mel':      (B, MEL_BINS, T)   float32
        'note':     (B,)               int32
        'adsr':     (B, 4)             float32   [log(A), log(D), S, log(R)]
        'waveform': (B,)               int32
        'timing':   (B, 2)             float32   [note_on, note_off] in seconds

    When n_workers > 1, batches are generated in parallel using
    ProcessPoolExecutor. Progress is displayed as ipywidgets bars when running
    inside a Jupyter notebook, falling back to tqdm otherwise.

    Args:
        n_samples:   Total number of samples to generate.
        output_dir:  Directory where .npz files will be written.
        batch_size:  Samples per .npz file.
        waveform:    Source waveform type (None = random per sample).
        start_batch: Index to start naming batch files from.
        n_workers:   Number of parallel worker processes.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    n_full, remainder = divmod(n_samples, batch_size)
    batch_sizes = [batch_size] * n_full + ([remainder] if remainder else [])
    jobs = [
        (sz, output_dir / f"batch_{start_batch + i:04d}.npz", waveform)
        for i, sz in enumerate(batch_sizes)
    ]

    if n_workers == 1:
        with tqdm(total=n_samples, desc="Generating dataset") as pbar:
            for sz, path, wf in jobs:
                _generate_batch(sz, path, wf)
                pbar.update(sz)
        return

    use_widgets = False

    n_slots = min(n_workers, len(jobs))
    import multiprocessing
    manager = multiprocessing.Manager()
    progress_queue: MPQueue = manager.Queue()

    overall_pbar: tqdm | None = None
    stop_event: threading.Event | None = None
    drain_thread: threading.Thread | None = None

    if use_widgets:
        overall_bar, worker_bars = _create_widget_bars(n_slots, n_samples, batch_size)
        stop_event = threading.Event()
        drain_thread = threading.Thread(
            target=_drain_progress,
            args=(progress_queue, worker_bars, overall_bar, stop_event),
            daemon=True,
        )
        drain_thread.start()
    else:
        overall_pbar = tqdm(total=n_samples, desc="Generating dataset")

    with ProcessPoolExecutor(max_workers=n_workers) as executor:
        futures = {
            executor.submit(
                _generate_batch,
                sz, path, wf,
                progress_queue if use_widgets else None,
                i % n_slots,
            ): sz
            for i, (sz, path, wf) in enumerate(jobs)
        }
        for future in as_completed(futures):
            future.result()
            if overall_pbar is not None:
                overall_pbar.update(futures[future])

    if stop_event is not None and drain_thread is not None:
        stop_event.set()
        drain_thread.join()
    if overall_pbar is not None:
        overall_pbar.close()
    manager.shutdown()
