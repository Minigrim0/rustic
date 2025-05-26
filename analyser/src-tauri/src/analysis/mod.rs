//! Audio analysis module
//! 
//! This module contains functionality for analyzing audio samples,
//! including FFT, spectrum analysis, pitch detection, and harmonic analysis.

mod fft;
mod spectrum;
mod harmonics;
mod pitch;

// Re-export public items
pub use fft::{FrequencyData, compute_fft};
pub use spectrum::compute_spectrum;
pub use harmonics::{identify_peaks, analyze_harmonics};
pub use pitch::{estimate_pitch, frequency_to_note};

/// Runs a complete analysis on the provided audio samples
pub fn analyze_audio(samples: &[f32], sample_rate: u32) -> AudioAnalysisResult {
    // Perform FFT analysis
    let frequencies = compute_fft(samples, sample_rate);
    
    // Identify peaks in the frequency data
    let peaks = identify_peaks(&frequencies, 0.1, 20.0);
    
    // Analyze harmonic relationships
    let harmonic_series = analyze_harmonics(&peaks, 0.05);
    
    // Compute spectrogram
    let spectrogram = compute_spectrum(samples, sample_rate);
    
    // Estimate pitch
    let pitch = estimate_pitch(samples, sample_rate);
    let note = pitch.map(frequency_to_note);
    
    AudioAnalysisResult {
        frequencies,
        peaks,
        harmonic_series,
        spectrogram,
        pitch,
        note,
    }
}

/// Contains all analysis results for an audio sample
#[derive(Debug)]
pub struct AudioAnalysisResult {
    /// Frequency data from FFT analysis
    pub frequencies: Vec<FrequencyData>,
    
    /// Identified peak frequencies
    pub peaks: Vec<FrequencyData>,
    
    /// Harmonic series identified in the signal
    pub harmonic_series: Vec<Vec<FrequencyData>>,
    
    /// Time-frequency spectrogram data
    pub spectrogram: Vec<Vec<f32>>,
    
    /// Estimated fundamental pitch in Hz (if detectable)
    pub pitch: Option<f32>,
    
    /// Musical note representation of the pitch (if detectable)
    pub note: Option<String>,
}