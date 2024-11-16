import numpy as np


class Generator:
    def __init__(self, freq: float, shape: str = "sine"):
        self.frequency = freq
        self.time_index = 0.0
        self.shape = shape

    def tick(self, time: float, warp: float = 1.0) -> float:
        self.time_index += warp * time
        match self.shape:
            case "sine":
                return np.sin(2 * np.pi * self.frequency * self.time_index)
            case "square":
                return np.sign(np.sin(2 * np.pi * self.frequency * self.time_index))
            case "sawtooth":
                return 2 * (self.frequency * self.time_index - np.floor(self.frequency * self.time_index + 0.5))
            case "triangle":
                return 2 * np.abs(2 * (self.frequency * self.time_index - np.floor(self.frequency * self.time_index + 0.5))) - 1
            case _:
                raise ValueError(f"Invalid shape: {self.shape}")
