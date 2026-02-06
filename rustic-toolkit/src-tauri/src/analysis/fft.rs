use log::info;
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct FrequencyData {
    pub frequency: f32,
    pub magnitude: f32,
    pub phase: f32,
}

/// Computes the Fast Fourier Transform for the given samples
pub fn compute_fft(samples: &[f32], sample_rate: u32) -> Vec<FrequencyData> {
    info!(
        "Computing FFT for {} samples at {} Hz",
        samples.len(),
        sample_rate
    );

    // We need a power of 2 for the FFT size
    let mut fft_size = 1;
    while fft_size < samples.len() {
        fft_size *= 2;
    }

    // Prepare input data for FFT
    let mut fft_input: Vec<Complex<f32>> = Vec::with_capacity(fft_size);

    // Apply Hann window function to reduce spectral leakage
    for (i, &sample) in samples.iter().take(fft_size).enumerate() {
        let window_val = 0.5 * (1.0 - (2.0 * PI * i as f32 / (fft_size as f32 - 1.0)).cos());
        fft_input.push(Complex {
            re: sample * window_val,
            im: 0.0,
        });
    }

    // Pad with zeros if needed
    fft_input.resize(fft_size, Complex { re: 0.0, im: 0.0 });

    // Create FFT planner
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    // Perform FFT
    let mut fft_output = fft_input;
    fft.process(&mut fft_output);

    // Convert FFT output to frequency data
    let nyquist = fft_size / 2;
    let mut result = Vec::with_capacity(nyquist);

    for (i, complex) in fft_output.iter().take(nyquist).enumerate() {
        let frequency = i as f32 * sample_rate as f32 / fft_size as f32;
        let magnitude = (complex.re.powi(2) + complex.im.powi(2)).sqrt();
        let phase = complex.im.atan2(complex.re);

        result.push(FrequencyData {
            frequency,
            magnitude,
            phase,
        });
    }

    // Normalize magnitudes
    if let Some(max_magnitude) = result.iter().map(|f| f.magnitude).reduce(f32::max)
        && max_magnitude > 0.0
    {
        for freq_data in &mut result {
            freq_data.magnitude /= max_magnitude;
        }
    }

    info!("FFT completed: {} frequency bins generated", result.len());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_fft_sine_wave() {
        // Create a sine wave at 440 Hz
        let sample_rate = 44100;
        let frequency = 440.0;
        let duration = 0.1; // 100ms
        let num_samples = (sample_rate as f32 * duration) as usize;

        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            samples.push((2.0 * PI * frequency * t).sin());
        }

        let frequencies = compute_fft(&samples, sample_rate);

        // Find the peak frequency
        let peak = frequencies
            .iter()
            .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
            .unwrap();

        // The peak frequency should be close to 440 Hz
        assert!((peak.frequency - 440.0).abs() < 10.0);
    }
}
