"""
Dataset generation and loading utilities.

Dual-mode: saves .npz batches to disk AND exposes a torch.utils.data.Dataset.
"""
from __future__ import annotations

from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed
from multiprocessing.queues import Queue as MPQueue
import threading

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
    from rustic_py.rustic_py import render  # type: ignore[import]

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


def _generate_batch(
    batch_size: int,
    output_path: Path,
    waveform: str,
    progress_queue: MPQueue | None = None,
    slot_id: int = 0,
) -> Path:
    """Generate a single batch of samples and save it as a .npz file.

    This function is designed to run in a subprocess — it imports rustic_py
    and numpy independently, so it is safe to call from ProcessPoolExecutor.

    Args:
        batch_size:      Number of samples to generate.
        output_path:     Full path for the output .npz file.
        waveform:        Source waveform type.
        progress_queue:  Optional multiprocessing.Queue. If provided, sends
                         slot_id after each sample so the main process can
                         update progress bars.
        slot_id:         Index of this worker's display slot (0..n_workers-1).
                         Only used when progress_queue is provided.

    Returns:
        The output_path, so callers can track which batches completed.
    """
    mel_batch, note_batch, adsr_batch = [], [], []

    for _ in range(batch_size):
        spec = random_spec(waveform=waveform)
        mel = render_mel(spec)
        src = spec["source"]
        mel_batch.append(mel)
        note_batch.append(spec["note"])
        adsr_batch.append(
            encode_adsr(src["attack"], src["decay"], src["sustain"], src["release"])
        )
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
    )
    return output_path


def _drain_progress(
    queue: MPQueue,
    worker_bars: list,
    overall_bar,
    stop_event: threading.Event,
) -> None:
    """Background thread: drain the progress queue and update ipywidgets bars.

    Reads slot_id values from the queue. Each value increments the
    corresponding worker bar and the overall bar by one sample.

    Runs until stop_event is set AND the queue is empty.

    Args:
        queue:       multiprocessing.Queue fed by worker processes.
        worker_bars: List of ipywidgets.IntProgress, one per worker slot.
        overall_bar: ipywidgets.IntProgress for the total sample count.
        stop_event:  Threading event set by the main thread when all futures
                     have completed.
    """
    while not stop_event.is_set() or not queue.empty():
        try:
            slot_id = queue.get(timeout=0.05)
            worker_bars[slot_id].value += 1
            overall_bar.value += 1
        except Exception:
            pass  # queue.Empty or transient error — keep polling


def _create_widget_bars(n_slots: int, n_samples: int, batch_size: int) -> tuple:
    """Create and display ipywidgets progress bars for parallel generation.

    Returns a tuple of (overall_bar, worker_bars) where overall_bar is an
    IntProgress for total samples and worker_bars is a list of IntProgress,
    one per worker slot.

    Args:
        n_slots:   Number of concurrent worker slots.
        n_samples: Total number of samples to generate (sets bar maximum).
    """
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
    waveform: str = "sine",
    start_batch: int = 0,
    n_workers: int = 1,
) -> None:
    """Generate a dataset of random specs and save as batched .npz files.

    Each .npz file contains:
        'mel':  (B, MEL_BINS, T)   float32
        'note': (B,)               int32
        'adsr': (B, 4)             float32

    When n_workers > 1, batches are generated in parallel using
    ProcessPoolExecutor. Progress is displayed as ipywidgets bars when running
    inside a Jupyter notebook, falling back to tqdm otherwise.

    Args:
        n_samples:   Total number of samples to generate.
        output_dir:  Directory where .npz files will be written.
        batch_size:  Samples per .npz file.
        waveform:    Source waveform type.
        start_batch: Index to start naming batch files from. Set this to the
                     number of existing batches to append without overwriting.
        n_workers:   Number of parallel worker processes. Defaults to 1
                     (sequential). Set to os.cpu_count() for full parallelism.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Build job list — last batch may be smaller if n_samples % batch_size != 0
    n_full, remainder = divmod(n_samples, batch_size)
    batch_sizes = [batch_size] * n_full + ([remainder] if remainder else [])
    jobs = [
        (sz, output_dir / f"batch_{start_batch + i:04d}.npz", waveform)
        for i, sz in enumerate(batch_sizes)
    ]

    if n_workers == 1:
        # Sequential — simple tqdm over individual samples
        with tqdm(total=n_samples, desc="Generating dataset") as pbar:
            for sz, path, wf in jobs:
                _generate_batch(sz, path, wf)
                pbar.update(sz)
        return

    # Parallel — try ipywidgets, fall back to tqdm
    try:
        import ipywidgets as widgets
        from IPython.display import display
        use_widgets = True
    except ImportError:
        use_widgets = False

    n_slots = min(n_workers, len(jobs))
    import multiprocessing
    manager = multiprocessing.Manager()
    progress_queue: MPQueue = manager.Queue()

    # Initialise to None so Pyright knows these are always bound below
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
            future.result()  # re-raises any exception from the worker
            if overall_pbar is not None:
                overall_pbar.update(futures[future])

    if stop_event is not None and drain_thread is not None:
        stop_event.set()
        drain_thread.join()
    if overall_pbar is not None:
        overall_pbar.close()
    manager.shutdown()


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
        if isinstance(source, (str, Path)):
            source = Path(source)
            self._paths = sorted(source.glob("*.npz"))
        else:
            self._paths = [Path(p) for p in source]

        if not self._paths:
            raise ValueError(f"No .npz files found in {source!r}")

        # Build index: list of (file_idx, sample_idx_within_file)
        self._index: list[tuple[int, int]] = []
        self._cache: dict[int, dict] = {}

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
