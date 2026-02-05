use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::analysis::FrequencyData;

/// Summary of a fully analyzed audio file (Phase 1 response).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct AudioSummary {
    pub sample_rate: u32,
    pub duration: f64,
    pub channels: u16,
    #[ts(type = "number")]
    pub total_samples: u64,
    pub peak_frequency: f32,
    pub rms_level: f32,
    pub pitch: Option<f32>,
    pub note: Option<String>,
}

/// Waveform data for a time window (Phase 2 response).
/// If `downsampled` is true, `samples` contains interleaved [min, max, min, max, ...] pairs.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct WaveformData {
    pub samples: Vec<f32>,
    pub start_time: f64,
    pub end_time: f64,
    pub sample_rate: u32,
    pub downsampled: bool,
}

/// Frequency spectrum (FFT) for a time window (Phase 2 response).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct SpectrumData {
    pub frequencies: Vec<FrequencyData>,
    /// Global top frequency peaks (sorted by magnitude descending, min 50 Hz separation).
    pub top_frequencies: Vec<FrequencyData>,
    pub start_time: f64,
    pub end_time: f64,
}

/// Spectrogram (STFT) for a time window (Phase 2 response).
/// `data[time_index][freq_bin]` = magnitude.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct SpectrogramData {
    pub data: Vec<Vec<f32>>,
    pub start_time: f64,
    pub end_time: f64,
    pub time_bins: u32,
    pub freq_bins: u32,
}