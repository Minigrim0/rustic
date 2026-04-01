# Re-exports for backward compatibility. Use rustic_ml.data directly.
from rustic_ml.legacy.data.generation import random_spec, render_mel, generate_dataset  # noqa: F401
from rustic_ml.legacy.data.dataset import NpzDataset, prepare_dataloaders  # noqa: F401
