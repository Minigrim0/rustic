import numpy as np


def time_scale(nsample: int, sample_rate: int, offset: int = 0) -> np.ndarray:
    """Converts a number of sample to their seconds equivalent."""
    return np.arange(offset / sample_rate, (offset + nsample) / sample_rate, 1 / sample_rate)


def generate_wave(freq: int = 440, duration: float = 1, sr: int = 44100, shape: str = 'sine'):
    match shape:
        case 'sine':
            return np.sin(2 * np.pi * freq * np.arange(0, duration, 1 / sr))
        case 'square':
            return np.sign(np.sin(2 * np.pi * freq * np.arange(0, duration, 1 / sr)))
        case 'sawtooth':
            return 2 * (freq * np.arange(0, duration, 1 / sr) - np.floor(freq * np.arange(0, duration, 1 / sr) + 0.5))
        case 'triangle':
            return 2 * np.abs(2 * (freq * np.arange(0, duration, 1 / sr) - np.floor(freq * np.arange(0, duration, 1 / sr) + 0.5))) - 1
        case _:
            raise ValueError(f"Invalid shape: {shape}")


def generate_bezier(time: float, _from: float = 1.0, to: float = 0.0, duration: float = 1.0, control: tuple[float, float] = (0.0, 0.0)) -> float:
    """
    Returns the current value of the bezier curve at time `time` with the given control points.
    """
    if time < 0:
       return _from
    elif time > duration:
        return to

    from_point = (0.0, _from)
    to_point = (duration, to)

    progress = time / duration

    # Calculate the current value of the bezier curve
    return (1 - progress) ** 2 * from_point[1] + 2 * (1 - progress) * progress * control[1] + progress ** 2 * to_point[1]

def mix_to_mono(audio: np.ndarray) -> np.ndarray:
    """Mix a stereo audio array into a mono array using mean"""
    if audio.ndim == 2:
        return audio.mean(axis=1)
    return audio

def normalize(audio: np.ndarray) -> np.ndarray:
    """Peak-normalize to [-1; 1]. Returns 0 unchanged"""
    peak = np.abs(audio).max()
    if peak == 0:
        return audio
    return audio / peak


def trim_silence(audio: np.ndarray, threshold: float = 1e-4) -> np.ndarray:
    """Strip leading and trailing samples below threshold"""
    mono = mix_to_mono(audio)
    mask = np.abs(mono) > threshold
    if not mask.any():
        return audio[:0]
    start = mask.argmax()
    end = len(mask) - mask[::-1].argmax()
    return audio[start:end]

def diff(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """Returns the sample to sample difference, padding the shorter signal with 0"""
    if len(a) > len(b):
        b = np.pad(b, ((0, len(a) - len(b)),) + ((0, 0),) * (b.ndim - 1))
    elif len(b) > len(a):
        a = np.pad(a, ((0, len(b) - len(a)),) + ((0, 0),) * (a.ndim - 1))
    return a - b

def rms(audio: np.ndarray) -> float:
    """RMS energy of the signal"""
    return float(np.sqrt(np.mean(mix_to_mono(audio) ** 2)))

def rms_compare(a: np.ndarray, b: np.ndarray) -> tuple[float, float, float]:
    """Returns (rms_a, rms_b, ratio a / b). ration > 1 means a is louder"""
    ra, rb = rms(a), rms(b)
    ratio = ra / rb if rb != 0 else float("inf")
    return ra, rb, ratio

