//! Utility functions for the Sample Analyser
//!
//! This module provides utility functions used across the application,
//! including error handling, data conversion, and common calculations.

// General utility functions
pub mod common {

    /// Format a frequency in a human-readable way
    pub fn format_frequency(freq: f32) -> String {
        if freq >= 1000.0 {
            format!("{:.2} kHz", freq / 1000.0)
        } else {
            format!("{:.1} Hz", freq)
        }
    }

    /// Format a time value in a human-readable way
    pub fn format_time(seconds: f32) -> String {
        if seconds >= 60.0 {
            let minutes = (seconds / 60.0).floor();
            let remaining_seconds = seconds - (minutes * 60.0);
            format!("{}:{:02}", minutes as u32, remaining_seconds as u32)
        } else {
            format!("{:.2} s", seconds)
        }
    }

    /// Format a decibel value in a human-readable way
    pub fn format_db(db: f32) -> String {
        format!("{:.1} dB", db)
    }

    /// Convert a linear amplitude to decibels
    pub fn to_db(amplitude: f32) -> f32 {
        20.0 * (amplitude.max(1e-6)).log10()
    }

    /// Convert decibels to a linear amplitude
    pub fn from_db(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }
}

// Create a simple error type for the application
pub mod error {
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    pub enum AnalyserError {
        IoError(std::io::Error),
        AudioError(String),
        AnalysisError(String),
        TauriError(String),
    }

    impl fmt::Display for AnalyserError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                AnalyserError::IoError(err) => write!(f, "I/O error: {}", err),
                AnalyserError::AudioError(msg) => write!(f, "Audio error: {}", msg),
                AnalyserError::AnalysisError(msg) => write!(f, "Analysis error: {}", msg),
                AnalyserError::TauriError(msg) => write!(f, "Tauri error: {}", msg),
            }
        }
    }

    impl Error for AnalyserError {}

    impl From<std::io::Error> for AnalyserError {
        fn from(err: std::io::Error) -> Self {
            AnalyserError::IoError(err)
        }
    }

    impl From<&str> for AnalyserError {
        fn from(msg: &str) -> Self {
            AnalyserError::AnalysisError(msg.to_string())
        }
    }

    impl From<String> for AnalyserError {
        fn from(msg: String) -> Self {
            AnalyserError::AnalysisError(msg)
        }
    }
}

// Audio utility functions
pub mod audio {
    /// Resamples audio data to a target sample rate
    pub fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
        if source_rate == target_rate {
            return samples.to_vec();
        }

        let ratio = target_rate as f32 / source_rate as f32;
        let new_len = (samples.len() as f32 * ratio).floor() as usize;
        let mut resampled = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = i as f32 / ratio;
            let src_idx_floor = src_idx.floor() as usize;
            let src_idx_ceil = (src_idx_floor + 1).min(samples.len() - 1);
            let fract = src_idx - src_idx_floor as f32;

            // Linear interpolation
            let sample = samples[src_idx_floor] * (1.0 - fract) + samples[src_idx_ceil] * fract;
            resampled.push(sample);
        }

        resampled
    }

    /// Applies gain to audio samples
    pub fn apply_gain(samples: &mut [f32], gain_db: f32) {
        let gain_linear = 10.0_f32.powf(gain_db / 20.0);
        samples.iter_mut().for_each(|s| *s *= gain_linear);
    }

    /// Normalizes audio to peak amplitude of 1.0
    pub fn normalize(samples: &mut [f32]) {
        if samples.is_empty() {
            return;
        }

        let max_amp = samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, |a, b| a.max(b));

        if max_amp > 0.0 {
            let gain = 1.0 / max_amp;
            samples.iter_mut().for_each(|s| *s *= gain);
        }
    }

    /// Computes the RMS (Root Mean Square) level of a sample buffer
    pub fn compute_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares = samples.iter().map(|s| s * s).sum::<f32>();

        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Applies a fade in/out effect to a sample buffer
    pub fn apply_fade(samples: &mut [f32], fade_in_ms: u32, fade_out_ms: u32, sample_rate: u32) {
        let fade_in_samples = (fade_in_ms as f32 * sample_rate as f32 / 1000.0) as usize;
        let fade_out_samples = (fade_out_ms as f32 * sample_rate as f32 / 1000.0) as usize;

        // Apply fade in
        for i in 0..fade_in_samples.min(samples.len()) {
            let gain = i as f32 / fade_in_samples as f32;
            samples[i] *= gain;
        }

        // Apply fade out
        let fade_out_start = samples.len().saturating_sub(fade_out_samples);
        for i in fade_out_start..samples.len() {
            let gain = (samples.len() - i) as f32 / fade_out_samples as f32;
            samples[i] *= gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_frequency() {
        assert_eq!(common::format_frequency(100.0), "100.0 Hz");
        assert_eq!(common::format_frequency(1000.0), "1.00 kHz");
        assert_eq!(common::format_frequency(4500.0), "4.50 kHz");
    }

    #[test]
    fn test_format_time() {
        assert_eq!(common::format_time(30.5), "30.50 s");
        assert_eq!(common::format_time(65.0), "1:05");
        assert_eq!(common::format_time(125.0), "2:05");
    }

    #[test]
    fn test_db_conversion() {
        // 0 dB should be amplitude 1.0
        assert!((common::from_db(0.0) - 1.0).abs() < 1e-6);

        // -6 dB should be amplitude 0.5
        assert!((common::from_db(-6.0) - 0.5).abs() < 1e-6);

        // Test round trip
        let amplitude = 0.25;
        let db = common::to_db(amplitude);
        let result = common::from_db(db);
        assert!((result - amplitude).abs() < 1e-6);
    }

    #[test]
    fn test_audio_utils() {
        // Test normalize
        let mut samples = vec![-0.5, 0.2, 0.8, -0.3];
        audio::normalize(&mut samples);
        assert!((samples[2] - 1.0).abs() < 1e-6); // Highest value should be 1.0

        // Test RMS
        let samples = vec![0.0, 1.0, 0.0, -1.0];
        let rms = audio::compute_rms(&samples);
        assert!((rms - 0.5).abs() < 1e-6); // RMS should be 0.5
    }
}
