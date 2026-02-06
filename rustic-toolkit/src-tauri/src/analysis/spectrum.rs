use log::info;
use rustfft::{FftPlanner, num_complex::Complex};
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
    let num_frames = samples.len().saturating_sub(window_size) / hop_size + 1;

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

        for complex in fft_output.iter().take(nyquist) {
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

/// Downsample a spectrogram by averaging adjacent time frames into buckets.
/// Returns the data unchanged if it already has fewer frames than `max_time_bins`.
pub fn downsample_spectrogram(data: Vec<Vec<f32>>, max_time_bins: usize) -> Vec<Vec<f32>> {
    let n = data.len();
    if n <= max_time_bins || n == 0 {
        return data;
    }

    let freq_bins = data[0].len();
    let bucket_size = n.div_ceil(max_time_bins); // ceil division

    info!(
        "Downsampling spectrogram: {} frames -> ~{} frames (bucket_size={})",
        n, max_time_bins, bucket_size
    );

    let mut result = Vec::with_capacity(max_time_bins);
    for chunk in data.chunks(bucket_size) {
        let mut avg = vec![0.0f32; freq_bins];
        for frame in chunk {
            for (i, &val) in frame.iter().enumerate() {
                avg[i] += val;
            }
        }
        let scale = 1.0 / chunk.len() as f32;
        for v in &mut avg {
            *v *= scale;
        }
        result.push(avg);
    }

    result
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

    #[test]
    fn test_downsample_no_op_when_small() {
        let data = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let result = downsample_spectrogram(data.clone(), 10);
        assert_eq!(result, data);
    }

    #[test]
    fn test_downsample_averages_buckets() {
        // 4 frames -> 2 buckets of size 2
        let data = vec![
            vec![2.0, 4.0],
            vec![4.0, 6.0],
            vec![10.0, 20.0],
            vec![30.0, 40.0],
        ];
        let result = downsample_spectrogram(data, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![3.0, 5.0]); // avg of [2,4] and [4,6]
        assert_eq!(result[1], vec![20.0, 30.0]); // avg of [10,20] and [30,40]
    }

    #[test]
    fn test_downsample_empty_input() {
        let data: Vec<Vec<f32>> = vec![];
        let result = downsample_spectrogram(data, 10);
        assert!(result.is_empty());
    }
}
