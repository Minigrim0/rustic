use log::info;
use music::generator::sine_wave::SineWave;
use music::generator::{Envelope, ToneGenerator};
use std::time::Instant;

#[cfg(feature = "plotting")]
use music::plotting::plot_data;

fn main() {
    // Tone Generator
    let sine_440: Box<dyn ToneGenerator> = Box::from(SineWave::new(440.0, 1.0));
    let sine_20: Box<dyn ToneGenerator> = Box::from(SineWave::new(20.0, 1.0));

    let mut envelope = Envelope::new(44100.0);
    let mut envelope2 = Envelope::new(44100.0);

    envelope.set_attack(0.2, 1.0, Some((0.0, 1.0)));
    envelope.set_decay(0.2, 0.6, Some((0.0, 0.6)));
    envelope.set_release(0.4, 0.0, Some((0.4, 0.6)));

    envelope2.set_attack(0.1, 1.0, Some((0.0, 1.0)));
    envelope2.set_decay(0.3, 0.8, Some((0.0, 0.6)));
    envelope2.set_release(0.4, 0.0, Some((0.4, 0.6)));

    let generator = envelope.attach(&sine_440);
    let generator2 = envelope2.attach(&sine_20);

    let mut results: Vec<(f32, f32)> = Vec::new();
    let mut results2: Vec<(f32, f32)> = Vec::new();
    let mut results_combined: Vec<(f32, f32)> = Vec::new();

    let sample_rate = 44100.0; // Hertz
    let duration = 1.0; // Seconds

    info!("Generating one second sample");
    let now = Instant::now();
    for sample in 0..(duration * sample_rate) as i32 {
        // Generate over one second at 1000Hz
        let current_time = sample as f32 / sample_rate;

        let val = if current_time < 0.02 * duration {
            // Note is not on yet
            generator.get_at(current_time, None, None)
        } else if current_time < 0.5 * duration {
            // Note is on for the moment
            generator.get_at(current_time, Some(0.02), None)
        } else {
            // Note has turned off
            generator.get_at(current_time, Some(0.02), Some(0.5))
        };

        let val2 = if current_time < 0.05 * duration {
            // Note is not on yet
            generator2.get_at(current_time, None, None)
        } else if current_time < 0.6 * duration {
            // Note is on for the moment
            generator2.get_at(current_time, Some(0.02), None)
        } else {
            // Note has turned off
            generator2.get_at(current_time, Some(0.02), Some(0.5))
        };

        results.push((current_time, val));
        results2.push((current_time, val2));
        results_combined.push((current_time, val + val2));
    }
    let elapsed = now.elapsed();

    info!("Completed");
    println!("Elapsed: {:.4?}", elapsed);

    #[cfg(feature = "plotting")]
    {
        if let Err(e) = plot_data(results, "sine_440", (-0.1, 1.1), (-1.1, 1.1), "Sine_440.png") {
            println!("Error: {}", e.to_string());
        }

        if let Err(e) = plot_data(results2, "sine_20", (-0.1, 1.1), (-1.1, 1.1), "Sine_20.png") {
            println!("Error: {}", e.to_string());
        }

        if let Err(e) = plot_data(results_combined, "sine_20 + 440", (-0.1, 1.1), (-1.1, 1.1), "Sine_20_plus_440.png") {
            println!("Error: {}", e.to_string());
        }
    }
}
