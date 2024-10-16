use music::generator::sine_wave::SineWave;
use music::generator::Envelope;

fn main() {
    // Tone Generator
    let sine_gen = SineWave::new(440.0, 1.0);
    let envelope = Envelope::new(1000.0, Box::from(sine_gen));

    for sample in 0..1000 {
        // Generate over one second at 1000Hz
        let current_time = sample / 1000;
        if sample < 500 {
            // Consider note on
        } else {
            // Consider note off
        }
    }
}
