from scipy.signal import butter, filtfilt, sosfilt
import numpy as np


def resonant_bandpass(data: np.ndarray, f_center: float, Q: float, fs: float):
    """
    Resonant bandpass filter using a biquad design.

    Implemented from http://musicweb.ucsd.edu/~trsmyth/filters/Bi_quadratic_Resonant_Filte.html

    Parameters:
        data: ndarray
            Input signal to filter.
        f_center: float
            Center frequency in Hz.
        Q: float
            Quality factor for resonance.
        fs: float
            Sampling frequency in Hz.

    Returns:
        ndarray
            Filtered signal.
    """

    period = 1 / fs
    bandwidth = f_center / Q

    R = np.exp(-np.pi * bandwidth * period)

    B = [1, 0, -R]
    A = [1, -2 * R * np.cos(2 * np.pi * f_center * period), R ** 2]

    # Apply the filter
    sos = np.array([[*B, *A]])

    return sosfilt(sos, data)


def bandpass(data, lowcut, highcut, fs, order=5):
    """
    Band-pass filter the data between the given cutoff frequencies.

    Args:
        data: The data to filter.
        lowcut: The lower cutoff frequency.
        highcut: The upper cutoff frequency.
        fs: The sampling frequency.
        order: The filter order.
    """
    b, a = butter(order, [lowcut / (fs / 2), highcut / (fs / 2)], btype='band')
    return filtfilt(b, a, data)


def low_pass(data, cutoff, fs, order=5):
    """
    Low-pass filter the data at the given cutoff frequency.
    """
    b, a = butter(order, cutoff / (fs / 2))
    return filtfilt(b, a, data)


def high_pass(data, cutoff, fs, order=5):
    """
    High-pass filter the data at the given cutoff frequency.
    """
    b, a = butter(order, cutoff / (fs / 2), btype='high')
    return filtfilt(b, a, data)
