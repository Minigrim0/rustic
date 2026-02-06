//! Audio analysis module
//!
//! This module contains functionality for analyzing audio samples,
//! including FFT, spectrum analysis, pitch detection, and waveform downsampling.

mod downsample;
mod fft;
mod peaks;
mod pitch;
mod spectrum;

// Re-export public items
pub use downsample::downsample_waveform;
pub use fft::{FrequencyData, compute_fft};
pub use peaks::pick_top_frequencies;
pub use pitch::{estimate_pitch, frequency_to_note};
pub use spectrum::{compute_spectrum, downsample_spectrogram};
