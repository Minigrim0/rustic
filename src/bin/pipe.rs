//! Pipe and Filter Architecture test
//!! Run with `cargo run --bin pipe | uv too run pipeplot` to plot the output

use std::cell::RefCell;
use std::rc::Rc;

use log::{error, trace};

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use rustic::core::note::Note;
use rustic::core::tones::{NOTES, TONES_FREQ};
use rustic::envelope::Envelope;
use rustic::filters::{CombinatorFilter, DelayFilter, DuplicateFilter, Filter, GainFilter, SafeSink, SafeFilter, AudioSink, System, Source, AudioGraphElement};
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
            sample_rate: 44100.0
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

                filter.push(
                    data,
                    desc.1
                )
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
    colog::init();

    let duration = 5.0; // 0.25 seconds
    let sample_rate = 44100.0; // 44100 Hz

    let source = Player::new();

    let dupe_filter: Box<dyn Filter> = Box::from(DuplicateFilter::new());
    let sum_filter: Box<dyn Filter> = Box::from(CombinatorFilter::new());

    // Delay of half a second
    let delay_filter: Box<dyn Filter> = Box::from(DelayFilter::new((0.2 * sample_rate) as usize));

    // Diminish gain in feedback loop
    let gain_filter: Box<dyn Filter> = Box::from(GainFilter::new(0.6));

    let system_sink: SafeSink = Rc::new(RefCell::new(Box::from(AudioSink::new())));

    let mut system = System::new();
    let sum_filter = system.add_filter(sum_filter);
    let dupe_filter = system.add_filter(dupe_filter);
    let delay_filter = system.add_filter(delay_filter);
    let gain_filter = system.add_filter(gain_filter);

    let system_sink_id = system.add_sink(Rc::clone(&system_sink));

    let system_source_id = system.add_source(Rc::from(RefCell::from(Box::from(source) as Box<dyn Source>)));

    system.connect(sum_filter, dupe_filter, 0, 0);
    system.connect(dupe_filter, delay_filter, 1, 0);
    system.connect(delay_filter, gain_filter, 0, 0);

    // Do not connect those in the graph to avoid cycles
    system.connect_feedback(gain_filter, sum_filter, 0, 1);

    system.connect_sink(dupe_filter, system_sink_id, 0, 0);
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

        let sys_sink: SafeSink = system.get_sink(0).unwrap();
        values.append(sys_sink.borrow_mut().get_values());

        sink.append(SamplesBuffer::new(
            1 as u16,
            sample_rate as u32,
            values.iter().map(|n| *n).collect::<Vec<f32>>(),
        ));
    }
}
