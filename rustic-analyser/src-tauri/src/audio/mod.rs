//! Audio processing and loading module
//!
//! This module handles loading audio files in various formats
//! and provides utilities for audio processing.

mod loader;

// Re-export public items
pub use loader::{AudioBuffer, AudioLoader};

/// Applies a window function to a slice of audio samples
///
/// Windowing is important for spectral analysis to reduce spectral leakage.
pub fn apply_window(samples: &[f32], window_type: WindowType) -> Vec<f32> {
    let len = samples.len();
    let mut windowed = Vec::with_capacity(len);

    for i in 0..len {
        let window_value = match window_type {
            WindowType::Rectangular => 1.0,
            WindowType::Hann => {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (len as f32 - 1.0)).cos())
            }
            WindowType::Hamming => {
                0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (len as f32 - 1.0)).cos()
            }
            WindowType::Blackman => {
                let x = 2.0 * std::f32::consts::PI * i as f32 / (len as f32 - 1.0);
                0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos()
            }
        };

        windowed.push(samples[i] * window_value);
    }

    windowed
}

/// Supported window function types for spectral analysis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowType {
    /// No windowing (rectangular window)
    Rectangular,

    /// Hann window - good general purpose window with good frequency resolution
    Hann,

    /// Hamming window - similar to Hann but doesn't go to zero at the edges
    Hamming,

    /// Blackman window - better sidelobe suppression than Hann/Hamming but worse resolution
    Blackman,
}

/// Normalizes audio samples to the range [-1.0, 1.0]
pub fn normalize_samples(samples: &[f32]) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let max_abs = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    if max_abs > 0.0 {
        samples.iter().map(|s| s / max_abs).collect()
    } else {
        samples.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_functions() {
        let samples = vec![1.0; 10];

        // Rectangular window should not modify samples
        let rectangular = apply_window(&samples, WindowType::Rectangular);
        assert_eq!(rectangular, samples);

        // Hann window should modify samples with first and last being 0
        let hann = apply_window(&samples, WindowType::Hann);
        assert!(hann[0].abs() < 1e-6);
        assert!(hann[9].abs() < 1e-6);

        // Middle of Hann window should be close to 1.0
        assert!((hann[5] - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_normalize_samples() {
        // Test empty input
        let empty: Vec<f32> = Vec::new();
        assert_eq!(normalize_samples(&empty), empty);

        // Test already normalized input
        let normalized = vec![-0.5, 0.0, 1.0, -0.3];
        let result = normalize_samples(&normalized);
        assert_eq!(result[2], 1.0); // Max value should still be 1.0

        // Test scaling
        let unnormalized = vec![-5.0, 0.0, 10.0, -3.0];
        let result = normalize_samples(&unnormalized);
        assert_eq!(result[2], 1.0); // Max value should be 1.0
        assert_eq!(result[0], -0.5); // -5.0 should become -0.5
    }
}
