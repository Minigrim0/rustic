from typing_extensions import Optional
import numpy as np

class Segment:
    def __init__(self, _from: tuple[float, float], to: tuple[float, float], control: Optional[tuple[float, float]] = None):
        pass


class Envelope:
    def __init__(self, attack: Segment, decay: Segment, release: Segment):
        pass


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
