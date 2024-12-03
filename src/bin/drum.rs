use std::cell::RefCell;
use std::rc::Rc;

use rustic::core::tones::{NOTES, TONES_FREQ};
use rustic::core::{note::Note, score::Score};
use rustic::envelope::{Envelope, Segment};
use rustic::filters::{CombinatorFilter, DelayFilter, DuplicateFilter, GainFilter, Pipe, System};
use rustic::generator::GENERATORS;

fn main() {
    let scale = 0.4; // Master volume
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
            Note::new(
                TONES_FREQ[NOTES::A as usize][2],
                x as f32 * 1.0 + 0.75,
                0.05,
            )
            .with_generator(GENERATORS::SINE)
            .with_envelope(&envelope)
            .with_pitch_bend(&pitch_bend),
        );

        notes.push(
            Note::new(TONES_FREQ[NOTES::E as usize][4], x as f32, 0.05)
                .with_generator(GENERATORS::NOISE)
                .with_envelope(&envelope)
                .with_pitch_bend(&pitch_bend),
        );

        for i in 0..4 {
            notes.push(
                Note::new(
                    TONES_FREQ[NOTES::A as usize][6 - i],
                    x as f32 * 1.0 + 0.25 * i as f32,
                    0.05,
                )
                .with_generator(GENERATORS::SINE)
                .with_envelope(&envelope)
                .with_pitch_bend(&pitch_bend),
            );
        }
    }

    let mut score = Score::new("Drum".to_string(), sample_rate);
    for note in notes {
        score.add_note(note);
    }

    let source1 = Rc::new(RefCell::new(Pipe::new()));

    let sum_result = Rc::new(RefCell::new(Pipe::new()));

    let feedback_source = Rc::new(RefCell::new(Pipe::new())); // Source for the feedback loop
    let feedback_delayed = Rc::new(RefCell::new(Pipe::new())); // Delayed feedback
    let feedback_end = Rc::new(RefCell::new(Pipe::new()));

    let system_sink = Rc::new(RefCell::new(Pipe::new()));

    let sum_filter = CombinatorFilter::new(
        [Rc::clone(&source1), Rc::clone(&feedback_end)],
        Rc::clone(&sum_result),
    );
    let dupe_filter = DuplicateFilter::new(
        Rc::clone(&sum_result),
        [Rc::clone(&feedback_source), Rc::clone(&system_sink)],
    );

    // Delay of half a second
    let delay_filter = DelayFilter::new(
        Rc::clone(&feedback_source),
        Rc::clone(&feedback_delayed),
        (0.2 * sample_rate as f32) as usize,
    );

    // Diminish gain in feedback loop
    let gain_filter = GainFilter::new(Rc::clone(&feedback_delayed), Rc::clone(&feedback_end), 0.6);

    let mut system = System::new();
    let sum_filter = system.add_filter(Box::from(sum_filter));
    let dupe_filter = system.add_filter(Box::from(dupe_filter));
    let delay_filter = system.add_filter(Box::from(delay_filter));
    let gain_filter = system.add_filter(Box::from(gain_filter));

    system.connect(sum_filter, dupe_filter, sum_result);
    system.connect(dupe_filter, delay_filter, feedback_source);
    system.connect(delay_filter, gain_filter, feedback_delayed);
    // Do not connect those in the graph to avoid cycles
    // system.connect(gain_filter, sum_filter, feedback_end);

    // Single system source
    system.add_source(source1);

    system.add_sink(system_sink);

    score.play();
}
