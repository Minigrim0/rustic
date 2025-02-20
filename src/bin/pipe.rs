//! Pipe and Filter Architecture test
//!! Run with `cargo run --bin pipe | uv too run pipeplot` to plot the output

use log::error;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use rustic::core::envelope::prelude::Envelope;
use rustic::core::filters::{
    AudioSink, CombinatorFilter, DelayFilter, DuplicateFilter, GainFilter,
};
use rustic::core::generator::GENERATORS;
use rustic::core::graph::{AudioGraphElement, Filter, Sink as SystemSink, Source, System};
use rustic::core::note::Note;
use rustic::core::tones::{NOTES, TONES_FREQ};

struct Player {
    notes: Vec<Note>,
    i: usize,
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
            sample_rate: 44100.0,
        }
    }
}

impl Source for Player {
    fn pull(&mut self) -> f32 {
        self.i += 1;
        self.notes[0].tick(self.i as i32, self.sample_rate as i32)
            + self.notes[1].tick(self.i as i32, self.sample_rate as i32)
            + self.notes[2].tick(self.i as i32, self.sample_rate as i32)
    }
}

impl AudioGraphElement for Player {
    fn get_name(&self) -> &str {
        "Player"
    }

    fn set_index(&mut self, index: usize) {}

    fn get_index(&self) -> usize {
        0
    }
}

fn main() {
    colog::init();

    let sample_rate = 44100.0; // 44100 Hz

    let source = Box::new(Player::new());

    let dupe_filter: Box<dyn Filter> = Box::from(DuplicateFilter::new());
    let sum_filter: Box<dyn Filter> = Box::from(CombinatorFilter::<2, 1>::new());

    // Delay of half a second
    let delay_filter: Box<dyn Filter> = Box::from(DelayFilter::new((0.2 * sample_rate) as usize));

    // Diminish gain in feedback loop
    let gain_filter: Box<dyn Filter> = Box::from(GainFilter::new(0.6));

    let system_sink: Box<dyn SystemSink> = Box::from(AudioSink::new());

    let mut system = System::new();
    let sum_filter = system.add_filter(sum_filter);
    let dupe_filter = system.add_filter(dupe_filter);
    let delay_filter = system.add_filter(delay_filter);
    let gain_filter = system.add_filter(gain_filter);

    let system_sink_id = system.add_sink(system_sink);

    let system_source_id = system.add_source(source);

    system.connect(sum_filter, dupe_filter, 0, 0);
    system.connect(dupe_filter, delay_filter, 1, 0);
    system.connect(delay_filter, gain_filter, 0, 0);

    // Do not connect those in the graph to avoid cycles
    system.connect(gain_filter, sum_filter, 0, 1);

    system.connect_sink(dupe_filter, system_sink_id, 0);
    system.connect_source(system_source_id, sum_filter, 0);

    if let Err(_) = system.compute() {
        error!("An error occured while computing the filter graph's layers");
    }

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // TODO: Move system to score
    let mut values = vec![];
    loop {
        values.clear();
        for _ in 0..sample_rate as usize {
            system.run();
        }

        match system.get_sink(0) {
            Ok(s) => values.append(&mut s.get_values()),
            Err(e) => error!("Unable to get sink from system: {}", e),
        };

        sink.append(SamplesBuffer::new(
            1 as u16,
            sample_rate as u32,
            values.iter().map(|n| *n).collect::<Vec<f32>>(),
        ));
    }
}
