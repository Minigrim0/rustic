use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri::State;

use crate::analysis::{
    compute_fft, compute_spectrum, estimate_pitch, frequency_to_note, FrequencyData,
};
use crate::audio::{AudioBuffer, AudioLoader};
use crate::utils::chunks_with_padding;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisBin {
    /// The first sample index of the bin
    pub sample_index: usize,
    /// The amount of samples analysed in the bin
    pub bin_size: usize,

    pub peak_frequency: f32,
    pub frequencies: Vec<FrequencyData>,
    pub rms_level: f32,
    pub pitch: Option<f32>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub sample_rate: u32,
    pub duration: f32,
    pub channels: u16,
    pub bins: Vec<AnalysisBin>,
}

/// Holds the currently loaded audio file
pub struct AudioState {
    pub buffer: Option<AudioBuffer>,
    pub loader: AudioLoader,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            buffer: None,
            loader: AudioLoader::new(),
        }
    }
}

/// Loads and analyzes an audio file
#[tauri::command]
pub async fn analyze_audio_file(
    path: String,
    state: State<'_, Arc<std::sync::Mutex<AudioState>>>,
) -> Result<AnalysisResult, String> {
    info!("Analyzing audio file: {}", path);

    let audio_path = Path::new(&path);
    if !audio_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    let mut state = state
        .lock()
        .map_err(|_| "Failed to lock audio state".to_string())?;

    match state.loader.load_file(&path) {
        Ok(audio_buffer) => {
            info!(
                "Successfully loaded audio file: {} samples at {} Hz",
                audio_buffer.samples().len(),
                audio_buffer.sample_rate()
            );

            // Store the buffer in the state for later use
            let samples = audio_buffer.samples().to_vec();
            let sample_rate = audio_buffer.sample_rate();
            let channels = audio_buffer.channels();
            let duration = audio_buffer.duration();

            state.buffer = Some(audio_buffer);

            let analysis_bin_size = 1024;
            let bins = chunks_with_padding(&samples, analysis_bin_size)
                .map(|(sample_index, samples)| {
                    // Compute FFT
                    let frequencies = compute_fft(&samples, sample_rate);

                    // Find peak frequencies
                    let peak_frequency = frequencies
                        .iter()
                        .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
                        .unwrap();

                    // Calculate RMS level
                    let rms_level =
                        samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
                    let rms_level = rms_level.sqrt();

                    // Estimate pitch
                    let pitch = estimate_pitch(&samples, sample_rate);
                    let note = pitch.map(frequency_to_note);

                    AnalysisBin {
                        sample_index,
                        bin_size: analysis_bin_size,

                        peak_frequency: peak_frequency.frequency,
                        frequencies,
                        rms_level,
                        pitch,
                        note,
                    }
                })
                .collect();

            Ok(AnalysisResult {
                sample_rate,
                duration,
                channels,
                bins,
            })
        }
        Err(e) => {
            error!("Error loading audio file: {}", e);
            Err(format!("Error loading audio file: {}", e))
        }
    }
}

/// Returns the sample rate of the loaded audio file
#[tauri::command]
pub fn get_sample_rate(state: State<'_, Arc<std::sync::Mutex<AudioState>>>) -> Result<u32, String> {
    let state = state
        .lock()
        .map_err(|_| "Failed to lock audio state".to_string())?;

    match &state.buffer {
        Some(buffer) => Ok(buffer.sample_rate()),
        None => Err("No audio file loaded".to_string()),
    }
}

/// Returns the samples from the loaded audio file
#[tauri::command]
pub fn get_samples(
    state: State<'_, Arc<std::sync::Mutex<AudioState>>>,
) -> Result<Vec<f32>, String> {
    let state = state
        .lock()
        .map_err(|_| "Failed to lock audio state".to_string())?;

    match &state.buffer {
        Some(buffer) => Ok(buffer.samples().to_vec()),
        None => Err("No audio file loaded".to_string()),
    }
}

/// Computes the FFT of the loaded audio file
#[tauri::command]
pub fn compute_fft_command(
    state: State<'_, Arc<std::sync::Mutex<AudioState>>>,
) -> Result<Vec<(usize, Vec<FrequencyData>)>, String> {
    info!("Computing FFT");

    let state = state
        .lock()
        .map_err(|_| "Failed to lock audio state".to_string())?;

    let analysis_bin_size = 1024;
    match &state.buffer {
        Some(buffer) => Ok(chunks_with_padding(&buffer.samples(), analysis_bin_size)
            .map(|(sample_index, samples)| {
                let frequencies = compute_fft(&samples, buffer.sample_rate());
                (sample_index, frequencies)
            })
            .collect()),
        None => Err("No audio file loaded".to_string()),
    }
}

/// Returns a list of frequencies and their magnitudes
#[tauri::command]
pub fn list_frequencies(
    state: State<'_, Arc<std::sync::Mutex<AudioState>>>,
) -> Result<Vec<(usize, Vec<FrequencyData>)>, String> {
    compute_fft_command(state)
}

/// Computes the spectrum analysis of the loaded audio file
#[tauri::command]
pub fn compute_spectrum_command(
    state: State<'_, Arc<std::sync::Mutex<AudioState>>>,
) -> Result<Vec<Vec<f32>>, String> {
    info!("Computing spectrum");

    let state = state
        .lock()
        .map_err(|_| "Failed to lock audio state".to_string())?;

    match &state.buffer {
        Some(buffer) => {
            let spectrum = compute_spectrum(buffer.samples(), buffer.sample_rate());
            Ok(spectrum)
        }
        None => Err("No audio file loaded".to_string()),
    }
}

/// Saves the analysis results to a file
#[tauri::command]
pub async fn save_analysis(path: String, result: AnalysisResult) -> Result<(), String> {
    info!("Saving analysis results to: {}", path);

    match serde_json::to_string_pretty(&result) {
        Ok(json) => match std::fs::write(&path, json) {
            Ok(_) => {
                info!("Analysis results saved successfully");
                Ok(())
            }
            Err(e) => {
                error!("Error writing analysis results: {}", e);
                Err(format!("Error writing analysis results: {}", e))
            }
        },
        Err(e) => {
            error!("Error serializing analysis results: {}", e);
            Err(format!("Error serializing analysis results: {}", e))
        }
    }
}

/// Estimates the pitch of the loaded audio file
#[tauri::command]
pub fn estimate_pitch_command(
    state: State<'_, Arc<std::sync::Mutex<AudioState>>>,
) -> Result<Option<f32>, String> {
    info!("Estimating pitch");

    let state = state
        .lock()
        .map_err(|_| "Failed to lock audio state".to_string())?;

    match &state.buffer {
        Some(buffer) => {
            let pitch = estimate_pitch(buffer.samples(), buffer.sample_rate());
            Ok(pitch)
        }
        None => Err("No audio file loaded".to_string()),
    }
}

/// Converts a frequency to a musical note
#[tauri::command]
pub fn frequency_to_note_command(frequency: f32) -> Result<String, String> {
    Ok(frequency_to_note(frequency))
}
