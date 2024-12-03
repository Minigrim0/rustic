use rustic::core::{note::Note, score::Score};
use rustic::envelope::{Envelope, Segment};
use rustic::generator::GENERATORS;

#[cfg(feature = "plotting")]
use rustic::plotting::plot_data;

fn main() {
    let scale = 0.2; // Master volume
    let sample_rate = 44100; // Sample rate

    let envelope = Envelope::new()
        .with_attack(0.5, scale * 1.0, Some((0.0, 1.0)))
        .with_decay(0.1, scale * 0.8, None)
        .with_release(2.0, scale * 0.0, Some((0.0, 0.0)));

    let pitch_bend = Segment::new(1.0, 0.1, 1.0, 5.0, None);

    let mut score = Score::new("Pitch bending".to_string(), sample_rate);
    score.add_note(
        Note::new(60.0, 0.0, 5.0)
            .with_generator(GENERATORS::SINE)
            .with_envelope(&envelope)
            .with_pitch_bend(&pitch_bend),
    );

    score.play();
}
