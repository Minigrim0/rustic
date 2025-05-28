use log::info;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

/// Computes a time-frequency spectrogram for the given samples
pub fn compute_spectrum(samples: &[f32], sample_rate: u32) -> Vec<Vec<f32>> {
    info!(
        "Computing spectrum for {} samples at {} Hz",
        samples.len(),
        sample_rate
    );

    // Parameters for the Short-Time Fourier Transform (STFT)
    let window_size = 1024;
    let hop_size = window_size / 4; // 75% overlap
    let num_frames = (samples.len() - window_size) / hop_size + 1;

    // Create the output spectrogram
    let mut spectrogram = Vec::with_capacity(num_frames);

    // Create FFT planner
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);

    // Process each frame
    for frame_index in 0..num_frames {
        let start_index = frame_index * hop_size;

        // Apply window function to the frame
        let mut fft_input: Vec<Complex<f32>> = Vec::with_capacity(window_size);
        for i in 0..window_size {
            if start_index + i < samples.len() {
                let window_val =
                    0.5 * (1.0 - (2.0 * PI * i as f32 / (window_size as f32 - 1.0)).cos());
                fft_input.push(Complex {
                    re: samples[start_index + i] * window_val,
                    im: 0.0,
                });
            } else {
                fft_input.push(Complex { re: 0.0, im: 0.0 });
            }
        }

        // Perform FFT
        let mut fft_output = fft_input;
        fft.process(&mut fft_output);

        // Convert FFT output to magnitudes (only using half due to Nyquist)
        let nyquist = window_size / 2;
        let mut frame_magnitudes = Vec::with_capacity(nyquist);

        for i in 0..nyquist {
            let complex = fft_output[i];
            let magnitude = (complex.re.powi(2) + complex.im.powi(2)).sqrt();
            frame_magnitudes.push(magnitude);
        }

        // Add frame to spectrogram
        spectrogram.push(frame_magnitudes);
    }

    info!(
        "Spectrogram completed: {} frames, {} frequency bins",
        spectrogram.len(),
        spectrogram.first().map_or(0, |v| v.len())
    );

    // Return the spectrogram
    spectrogram
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_spectrum() {
        // Create a test signal with two frequencies
        let sample_rate = 44100;
        let duration = 0.5; // 500ms
        let num_samples = (sample_rate as f32 * duration) as usize;

        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            // Mix 440Hz and 880Hz
            samples.push(0.5 * (2.0 * PI * 440.0 * t).sin() + 0.3 * (2.0 * PI * 880.0 * t).sin());
        }

        let spectrogram = compute_spectrum(&samples, sample_rate);

        // Basic verification
        assert!(!spectrogram.is_empty());
        assert!(!spectrogram[0].is_empty());
    }
}
