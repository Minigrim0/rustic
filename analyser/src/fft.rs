use log::info;
use rustfft::{FftPlanner, num_complex::Complex};

use super::loader::Loader;

pub fn fft(loader: &mut Loader) -> Option<(Vec<Complex<f32>>, u32)> {
    let (buffer, sample_rate) = match loader.next() {
        Some(buffer) => buffer,
        None => return None,
    };

    info!("Creating FFT planner");
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer.samples().len());

    info!("Processing FFT");
    let mut buffer = buffer
        .samples()
        .iter()
        .map(|s| Complex { re: *s, im: 0.0f32 })
        .collect::<Vec<Complex<f32>>>();
    fft.process(&mut buffer);

    Some((buffer, sample_rate))
}
