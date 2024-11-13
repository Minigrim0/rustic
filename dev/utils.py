from typing_extensions import Optional
import numpy as np

class Segment:
    def __init__(self, _from: tuple[float, float], to: tuple[float, float], control: Optional[tuple[float, float]] = None):
        pass


class Envelope:
    def __init__(self, attack: Segment, decay: Segment, release: Segment):
        pass


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

    # Calculate the current value of the bezier curve
    return (1 - time) ** 3 * from_point[1] + 3 * (1 - time) ** 2 * time * control[1] + 3 * (1 - time) * time ** 2 * to_point[1] + time ** 3 * to_point[1]
