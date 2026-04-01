"""
rustic_ml.autoregressive — vocabulary, tokeniser, and data generation
for the Phase-5 autoregressive graph decoder.
"""
from .vocab import Vocabulary
from .tokenizer import spec_to_sequence, sequence_to_spec
from .generation import random_ar_spec
from .dataset import ARDataset, ar_collate_fn

__all__ = [
    "Vocabulary",
    "spec_to_sequence",
    "sequence_to_spec",
    "random_ar_spec",
    "ARDataset",
    "ar_collate_fn",
]
