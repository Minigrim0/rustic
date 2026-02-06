use std::sync::RwLock;

use log::info;
use tauri::State;

use crate::analysis::{
    FrequencyData, compute_fft, compute_spectrum, downsample_spectrogram, downsample_waveform,
    pick_top_frequencies,
};
use crate::error::AppError;
use crate::state::AudioState;
use crate::types::{SpectrogramData, SpectrumData, WaveformData};

/// Phase 2: Return waveform samples for a time window, downsampled for display.
#[tauri::command]
pub async fn get_waveform(
    start: f64,
    end: f64,
    target_points: u32,
    state: State<'_, RwLock<AudioState>>,
) -> Result<WaveformData, AppError> {
    info!(
        "get_waveform [{:.3}s, {:.3}s] target={}",
        start, end, target_points
    );

    // Scoped read-lock: copy the slice we need, then drop the lock.
    let (slice, sample_rate) = {
        let st = state.read()?;
        let buf = st.buffer.as_ref().ok_or(AppError::NoAudioLoaded)?;
        let (s, e) = st
            .time_to_sample_range(start, end)
            .ok_or(AppError::InvalidTimeRange)?;
        (buf.samples()[s..e].to_vec(), buf.sample_rate())
    };

    let (samples, downsampled) = downsample_waveform(&slice, target_points);

    Ok(WaveformData {
        samples,
        start_time: start,
        end_time: end,
        sample_rate,
        downsampled,
    })
}

/// Phase 2: Return FFT frequency data for a time window.
#[tauri::command]
pub async fn get_spectrum(
    start: f64,
    end: f64,
    top_count: usize,
    min_peak_distance: f32,
    state: State<'_, RwLock<AudioState>>,
) -> Result<SpectrumData, AppError> {
    info!(
        "get_spectrum [{:.3}s, {:.3}s] top_count={} min_dist={:.0}Hz",
        start, end, top_count, min_peak_distance
    );

    let (slice, sample_rate) = {
        let st = state.read()?;
        let buf = st.buffer.as_ref().ok_or(AppError::NoAudioLoaded)?;
        let (s, e) = st
            .time_to_sample_range(start, end)
            .ok_or(AppError::InvalidTimeRange)?;
        (buf.samples()[s..e].to_vec(), buf.sample_rate())
    };

    let frequencies = compute_fft(&slice, sample_rate);
    let top_frequencies = pick_top_frequencies(&frequencies, top_count, min_peak_distance);

    Ok(SpectrumData {
        frequencies,
        top_frequencies,
        start_time: start,
        end_time: end,
    })
}

/// Phase 2: Return top frequency peaks within a frequency range.
#[tauri::command]
pub async fn get_top_frequencies(
    start: f64,
    end: f64,
    freq_lo: f32,
    freq_hi: f32,
    top_count: usize,
    min_peak_distance: f32,
    state: State<'_, RwLock<AudioState>>,
) -> Result<Vec<FrequencyData>, AppError> {
    info!(
        "get_top_frequencies [{:.3}s, {:.3}s] freq=[{:.0}, {:.0}] Hz top_count={} min_dist={:.0}Hz",
        start, end, freq_lo, freq_hi, top_count, min_peak_distance
    );

    let (slice, sample_rate) = {
        let st = state.read()?;
        let buf = st.buffer.as_ref().ok_or(AppError::NoAudioLoaded)?;
        let (s, e) = st
            .time_to_sample_range(start, end)
            .ok_or(AppError::InvalidTimeRange)?;
        (buf.samples()[s..e].to_vec(), buf.sample_rate())
    };

    let frequencies = compute_fft(&slice, sample_rate);
    let filtered: Vec<FrequencyData> = frequencies
        .into_iter()
        .filter(|f| f.frequency >= freq_lo && f.frequency <= freq_hi)
        .collect();

    Ok(pick_top_frequencies(
        &filtered,
        top_count,
        min_peak_distance,
    ))
}

/// Phase 2: Return spectrogram (STFT) data for a time window.
#[tauri::command]
pub async fn get_spectrogram(
    start: f64,
    end: f64,
    state: State<'_, RwLock<AudioState>>,
) -> Result<SpectrogramData, AppError> {
    info!("get_spectrogram [{:.3}s, {:.3}s]", start, end);

    let (slice, sample_rate) = {
        let st = state.read()?;
        let buf = st.buffer.as_ref().ok_or(AppError::NoAudioLoaded)?;
        let (s, e) = st
            .time_to_sample_range(start, end)
            .ok_or(AppError::InvalidTimeRange)?;
        (buf.samples()[s..e].to_vec(), buf.sample_rate())
    };

    let raw = compute_spectrum(&slice, sample_rate);
    let data = downsample_spectrogram(raw, 500);
    let time_bins = data.len() as u32;
    let freq_bins = data.first().map_or(0, |v| v.len()) as u32;

    Ok(SpectrogramData {
        data,
        start_time: start,
        end_time: end,
        time_bins,
        freq_bins,
        sample_rate,
    })
}
