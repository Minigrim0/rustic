use rustic::core::tones::{NOTES, TONES_FREQ};
use rustic::core::{note::Note, score::Score};
use rustic::envelope::{Envelope, Segment};
use rustic::filters::{
    AudioGraphElement, CombinatorFilter, DelayFilter, DuplicateFilter, GainFilter, SafeFilter,
    Source, System,
};
use rustic::generator::GENERATORS;

struct Player {
    notes: Vec<Note>,
    i: usize,
    desc: [Option<(SafeFilter, usize)>; 1],
    sample_rate: f32,
}

impl Player {
    fn new() -> Self {
        let envelope = Envelope::new()
            .with_attack(0.025, 1.0, None)
            .with_decay(0.0, 0.5, None)
            .with_release(0.5, 0.0, None);

        let initial_note = Note::new(TONES_FREQ[NOTES::A as usize][4], 0.0, 0.05)
            .with_generator(GENERATORS::SINE)
            .with_envelope(&envelope);
        let second_note = Note::new(TONES_FREQ[NOTES::C as usize][4], 1.0, 0.05)
            .with_generator(GENERATORS::SINE)
            .with_envelope(&envelope);
        let third_note = Note::new(TONES_FREQ[NOTES::E as usize][4], 2.0, 0.05)
            .with_generator(GENERATORS::SINE)
            .with_envelope(&envelope);

        let notes = vec![initial_note, second_note, third_note];

        Self {
            notes,
            i: 0,
            desc: [None],
            sample_rate: 44100.0,
        }
    }
}

impl Source for Player {
    fn push(&mut self) {
        self.i += 1;
        if let Some(desc) = &self.desc[0] {
            if let Ok(mut filter) = desc.0.try_borrow_mut() {
                let data = self.notes[0].tick(self.i as i32, self.sample_rate as i32)
                    + self.notes[1].tick(self.i as i32, self.sample_rate as i32)
                    + self.notes[2].tick(self.i as i32, self.sample_rate as i32);

                filter.push(data, desc.1)
            }
        }
    }

    fn connect_entry(&mut self, to: SafeFilter, in_port: usize) {
        self.desc[0] = Some((to, in_port));
    }
}

impl AudioGraphElement for Player {
    fn get_name(&self) -> &str {
        "Player"
    }

    fn uuid(&self) -> uuid::Uuid {
        unimplemented!()
    }
}

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

    let sum_filter = CombinatorFilter::new();
    let dupe_filter = DuplicateFilter::new();

    // Delay of half a second
    let delay_filter = DelayFilter::new((0.2 * sample_rate as f32) as usize);

    // Diminish gain in feedback loop
    let gain_filter = GainFilter::new(0.6);

    let mut system = System::new();
    let sum_filter = system.add_filter(Box::from(sum_filter));
    let dupe_filter = system.add_filter(Box::from(dupe_filter));
    let delay_filter = system.add_filter(Box::from(delay_filter));
    let gain_filter = system.add_filter(Box::from(gain_filter));

    system.connect(sum_filter, dupe_filter, 0, 0);
    system.connect(dupe_filter, delay_filter, 1, 0);
    system.connect(delay_filter, gain_filter, 0, 0);
    // Do not connect those in the graph to avoid cycles
    system.connect_feedback(gain_filter, sum_filter, 0, 1);

    // Single system source
    system.add_source(source1);

    system.add_sink(system_sink);

    score.play();
}
