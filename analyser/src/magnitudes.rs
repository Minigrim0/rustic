use rustfft::num_complex::Complex;

/// Returns a vector of tuples containing each frequency and its magnitude.
pub fn get_magnitudes(samples: Vec<Complex<f32>>, sample_rate: u32) -> Vec<(u32, f32)> {
    let data: Vec<(u32, f32)> = samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let freq = ((index as f32 * sample_rate as f32) / samples.len() as f32) as u32;
            (freq, sample.norm())
        })
        .collect();

    data
}
