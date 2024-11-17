use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use rustic::generator::{Envelope, Segment, GENERATORS};
use rustic::score::{Note, Score};
use rustic::tones::{NOTES, TONES_FREQ};

#[cfg(feature = "plotting")]
use rustic::plotting::plot_data;

fn main() {
    let scale = 0.2; // Master volume
    let duration = 20.0; // Duration of the song
    let sample_rate = 44100; // Sample rate

    let envelope = {
        let mut env = Envelope::new();
        env.set_attack(0.04, scale * 1.0, Some((0.0, 1.0)));
        env.set_decay(0.0, scale * 1.0, None);
        env.set_release(0.2, scale * 0.0, Some((0.0, 0.0)));
        env
    };

    let pitch_bend = Segment::new(1.0, 0.5, 0.2, 0.0, Some((2.0, 0.2)));
    let mut notes = vec![];

    for x in 0..100 {
        notes.push(
            Note::new(TONES_FREQ[NOTES::A as usize][2], x as f32 * 1.0 + 0.5, 0.05)
                .with_generator(GENERATORS::SINE)
                .with_envelope(&envelope)
                .with_pitch_bend(&pitch_bend),
        );

        notes.push(
            Note::new(TONES_FREQ[NOTES::E as usize][4], x as f32, 0.05)
                .with_generator(GENERATORS::SINE)
                .with_envelope(&envelope)
                .with_pitch_bend(&pitch_bend),
        );
    }

    let mut score = Score::new("Drum".to_string(), sample_rate);
    for note in notes {
        score.add_note(note);
    }

    score.play();
}
