//! Audio analysis module
//!
//! This module contains functionality for analyzing audio samples,
//! including FFT, spectrum analysis, pitch detection, and harmonic analysis.

mod fft;
mod harmonics;
mod pitch;
mod spectrum;

// Re-export public items
pub use fft::{compute_fft, FrequencyData};
pub use pitch::{estimate_pitch, frequency_to_note};
pub use spectrum::compute_spectrum;
