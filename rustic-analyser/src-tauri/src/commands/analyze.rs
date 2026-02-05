use std::path::Path;
use std::sync::RwLock;

use log::info;
use tauri::State;

use crate::analysis::{compute_fft, estimate_pitch, frequency_to_note};
use crate::error::AppError;
use crate::state::AudioState;
use crate::types::AudioSummary;

/// Phase 1: Load an audio file and return a global summary.
///
/// The audio buffer is cached in application state for subsequent windowed queries.
#[tauri::command]
pub async fn analyze_audio_file(
    path: String,
    state: State<'_, RwLock<AudioState>>,
) -> Result<AudioSummary, AppError> {
    info!("Analyzing audio file: {}", path);

    if !Path::new(&path).exists() {
        return Err(AppError::FileNotFound);
    }

    // Write-lock for the full load+summarize cycle.
    // This is the only write command so contention is negligible.
    let mut st = state.write()?;

    let audio_buffer = st.loader.load_file(&path)?;

    info!(
        "Loaded {} samples at {} Hz",
        audio_buffer.samples().len(),
        audio_buffer.sample_rate()
    );

    let samples = audio_buffer.samples();
    let sample_rate = audio_buffer.sample_rate();

    // Global FFT for peak frequency
    let frequencies = compute_fft(samples, sample_rate);
    let peak_frequency = frequencies
        .iter()
        .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
        .map(|f| f.frequency)
        .unwrap_or(0.0);

    // RMS level
    let rms_level = if samples.is_empty() {
        0.0
    } else {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    };

    // Pitch estimation
    let pitch = estimate_pitch(samples, sample_rate);
    let note = pitch.map(frequency_to_note);

    let summary = AudioSummary {
        sample_rate,
        duration: audio_buffer.duration() as f64,
        channels: audio_buffer.channels(),
        total_samples: samples.len() as u64,
        peak_frequency,
        rms_level,
        pitch,
        note,
    };

    st.summary = Some(summary.clone());
    st.buffer = Some(audio_buffer);

    Ok(summary)
}