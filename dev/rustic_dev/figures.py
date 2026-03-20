import matplotlib.pyplot as plt
import numpy as np


def spectrogram(signal: np.ndarray, sample_rate: int, NFFT: int = None, title=None):
    if signal.ndim == 2:
        signal = signal.mean(axis=1)
    NFFT = min(
        1024 if NFFT is None else NFFT,
        len(signal)
    )  # the length of the windowing segments
    t = np.arange(0, len(signal) / sample_rate, 1 / sample_rate)

    fig, (ax1, ax2) = plt.subplots(nrows=2, sharex=True, figsize=(8, 4))
    ax1.plot(t, signal)
    ax1.set_ylabel('Signal')

    Pxx, freqs, bins, im = ax2.specgram(signal, NFFT=NFFT, Fs=sample_rate)
    # The `specgram` method returns 4 objects. They are:
    # - Pxx: the periodogram
    # - freqs: the frequency vector
    # - bins: the centers of the time bins
    # - im: the .image.AxesImage instance representing the data in the plot
    ax2.set_xlabel('Time (s)')
    ax2.set_ylabel('Frequency (Hz)')
    # ax2.set_xlim(0, 20)

    if title:
        fig.suptitle(title)

    plt.show()

def freq_compare(sig1: list, sig2: list, sr: int, sig1_name="Signal 1", sig2_name="Signal 2", focus_window=(0, 20e3)):
    """
    Displays two signals in frequency domain to be compared
    """

    fig, (ax1, ax2) = plt.subplots(nrows=2, sharex=True, figsize=(8, 4))

    ax1.magnitude_spectrum(sig1, Fs=sr)
    ax1.set_xlabel("Frequency (Hz)")
    ax1.set_ylabel(f"Magnitude ({sig1_name})")
    ax1.set_xlim(focus_window[0], focus_window[1])

    ax2.magnitude_spectrum(sig2, Fs=sr)
    ax2.set_xlabel("Frequency (Hz)")
    ax2.set_ylabel(f"Magnitude ({sig2_name})")
    ax2.set_xlim(focus_window[0], focus_window[1])

    fig.suptitle(f"Comparison of frequencies [{focus_window[0]};{focus_window[1]}]")
    plt.show()


def plot_waveform(audio: np.ndarray, sr: int, title: str = None):
    """Time-domain plot. Handles stereo (N, 2) or mono (N,)."""
    mono = audio.mean(axis=1) if audio.ndim == 2 else audio
    t = np.arange(len(mono)) / sr

    fig, ax = plt.subplots(figsize=(10, 3))
    ax.plot(t, mono)
    ax.set_xlabel("Time (s)")
    ax.set_ylabel("Amplitude")
    if title:
        ax.set_title(title)
    plt.tight_layout()
    plt.show()


def plot_diff(a: np.ndarray, b: np.ndarray, sr: int, a_name="A", b_name="B"):
    """Overlay two signals and show their difference in a second panel."""
    from .utils import mix_to_mono, diff

    a_mono = mix_to_mono(a)
    b_mono = mix_to_mono(b)
    d = mix_to_mono(diff(a, b))

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 5), sharex=True)

    ax1.plot(np.arange(len(a_mono)) / sr, a_mono, label=a_name)
    ax1.plot(np.arange(len(b_mono)) / sr, b_mono, label=b_name, alpha=0.7)
    ax1.set_ylabel("Amplitude")
    ax1.legend()

    ax2.plot(np.arange(len(d)) / sr, d, color="red")
    ax2.set_xlabel("Time (s)")
    ax2.set_ylabel("Difference")

    plt.tight_layout()
    plt.show()


def play(audio: np.ndarray, sr: int):
    """Play audio inline in a Jupyter notebook."""
    from IPython.display import Audio, display
    # Audio expects (channels, samples) for stereo
    display(Audio(audio.T if audio.ndim == 2 else audio, rate=sr))


def freq_display(sig: list, sr: int, sig_name="Signal 1", focus_window=(0, 20e3), draw_line_at=None):
    """
    Displays a signal in frequency domain
    """

    ax1 = plt.gca()

    ax1.magnitude_spectrum(sig, Fs=sr)
    ax1.set_xlabel("Frequency (Hz)")
    ax1.set_ylabel("Magnitude")
    ax1.set_xlim(focus_window[0], focus_window[1])

    if draw_line_at:
        ax1.axvline(draw_line_at, color='r', linestyle='--')

    plt.title(f"Frequency domain of {sig_name}")
    plt.show()
