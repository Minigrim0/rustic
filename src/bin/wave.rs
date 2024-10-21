use log::{error, info};
use std::time::Instant;

use rustic::generator::sine_wave::SineWave;
use rustic::generator::{Envelope, ToneGenerator};

#[cfg(feature = "plotting")]
use rustic::plotting::plot_data;

fn main() {
    // Tone Generator
    let sine_440: Box<dyn ToneGenerator> = Box::from(SineWave::new(440.0, 1.0));
    let sine_20: Box<dyn ToneGenerator> = Box::from(SineWave::new(20.0, 1.0));

    let mut envelope = Envelope::new();
    let mut envelope2 = Envelope::new();

    envelope.set_attack(0.2, 1.0, Some((0.0, 1.0)));
    envelope.set_decay(0.2, 0.6, Some((0.0, 0.6)));
    envelope.set_release(0.4, 0.0, Some((0.4, 0.6)));

    envelope2.set_attack(0.1, 1.0, Some((0.0, 1.0)));
    envelope2.set_decay(0.3, 0.8, Some((0.0, 0.6)));
    envelope2.set_release(0.4, 0.0, Some((0.4, 0.6)));

    let mut generator = envelope.attach(sine_440);
    let mut generator2 = envelope2.attach(sine_20);

    let mut results: Vec<(f32, f32)> = Vec::new();
    let mut results2: Vec<(f32, f32)> = Vec::new();
    let mut results_combined: Vec<(f32, f32)> = Vec::new();

    let sample_rate = 44100.0; // Hertz
    let duration = 1.0; // Seconds

    info!("Generating one second sample");
    let now = Instant::now();
    for sample in 0..(duration * sample_rate) as i32 {
        let current_time = sample as f32 / sample_rate;

        let val = generator.get_at(current_time, 0.02, 0.5);
        let val2 = generator2.get_at(current_time, 0.02, 0.5);

        results.push((current_time, val));
        results2.push((current_time, val2));
        results_combined.push((current_time, val + val2));
    }
    let elapsed = now.elapsed();

    info!("Completed");
    info!("Elapsed: {:.4?}", elapsed);

    #[cfg(feature = "plotting")]
    {
        if let Err(e) = plot_data(
            results,
            "sine_440",
            (-0.1, 1.1),
            (-1.1, 1.1),
            "Sine_440.png",
        ) {
            error!("Error: {}", e.to_string());
        }

        if let Err(e) = plot_data(results2, "sine_20", (-0.1, 1.1), (-1.1, 1.1), "Sine_20.png") {
            error!("Error: {}", e.to_string());
        }

        if let Err(e) = plot_data(
            results_combined,
            "sine_20 + 440",
            (-0.1, 1.1),
            (-1.1, 1.1),
            "Sine_20_plus_440.png",
        ) {
            error!("Error: {}", e.to_string());
        }
    }
}
