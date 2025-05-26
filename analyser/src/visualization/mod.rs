//! Visualization module for audio data
//!
//! This module provides visualization tools for audio data analysis,
//! including waveforms, FFT results, and spectrograms.

mod color_maps;
mod frequency;
mod spectrogram;
mod waveform;

// Re-export visualization components
pub use color_maps::ColorMap;
pub use frequency::FrequencyVisualizer;
pub use spectrogram::SpectrogramVisualizer;
// pub use waveform::WaveformVisualizer;

/// Scale factor modes for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    /// Linear scaling (no transformation)
    Linear,

    /// Logarithmic scaling (better for frequencies)
    Logarithmic,

    /// Decibel scaling (better for audio magnitudes)
    Decibel,

    /// Mel scale (perceptual scale for frequencies)
    Mel,
}

/// Common visualization options
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualizationOptions {
    /// Width of the visualization in pixels
    pub width: u32,

    /// Height of the visualization in pixels
    pub height: u32,

    /// Background color (CSS color string)
    pub background_color: String,

    /// Foreground color (CSS color string)
    pub foreground_color: String,

    /// Whether to show grid lines
    pub show_grid: bool,

    /// Whether to show axis labels
    pub show_labels: bool,

    /// Scaling mode for the x-axis
    pub x_scale: ScaleMode,

    /// Scaling mode for the y-axis
    pub y_scale: ScaleMode,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 400,
            background_color: "#222222".to_string(),
            foreground_color: "#00AAFF".to_string(),
            show_grid: true,
            show_labels: true,
            x_scale: ScaleMode::Linear,
            y_scale: ScaleMode::Linear,
        }
    }
}
