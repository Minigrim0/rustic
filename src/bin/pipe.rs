//! Pipe and Filter Architecture test
//!! Run with `cargo run --bin pipe | uv too run pipeplot` to plot the output

use std::cell::RefCell;
use std::rc::Rc;

use log::error;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use rustic::core::note::Note;
use rustic::core::tones::{NOTES, TONES_FREQ};
use rustic::envelope::Envelope;
use rustic::filters::{CombinatorFilter, DelayFilter, DuplicateFilter, Filter, GainFilter, SafeSink, Sink as SysSink, System};
use rustic::generator::GENERATORS;

fn main() {
    colog::init();

    let duration = 5.0; // 0.25 seconds
    let sample_rate = 44100.0; // 44100 Hz

    let dupe_filter: Box<dyn Filter> = Box::from(DuplicateFilter::new());
    let sum_filter: Box<dyn Filter> = Box::from(CombinatorFilter::new());

    // Delay of half a second
    let delay_filter: Box<dyn Filter> = Box::from(DelayFilter::new((0.5 * sample_rate) as usize));

    // Diminish gain in feedback loop
    let gain_filter: Box<dyn Filter> = Box::from(GainFilter::new(0.6));

    let system_sink: SafeSink = Rc::new(RefCell::new(SysSink::new()));

    let mut system = System::new();
    let sum_filter = system.add_filter(sum_filter);
    let dupe_filter = system.add_filter(dupe_filter);
    let delay_filter = system.add_filter(delay_filter);
    let gain_filter = system.add_filter(gain_filter);

    let system_sink_id = system.add_sink(Rc::clone(&system_sink));

    system.add_source(sum_filter, 0);

    system.connect(sum_filter, dupe_filter, 0, 0);
    system.connect(dupe_filter, delay_filter, 1, 0);
    system.connect(delay_filter, gain_filter, 0, 0);

    // Do not connect those in the graph to avoid cycles
    system.connect_feedback(gain_filter, sum_filter, 0, 1);

    system.connect_sink(dupe_filter, system_sink_id, 0, 0);

    if let Err(_) = system.compute() {
        error!("An error occured while computing the filter graph's layers");
    }

    let envelope = Envelope::new()
        .with_attack(0.01, 0.4, None)
        .with_decay(0.01, 0.3, None)
        .with_release(0.4, 0.0, None);

    let mut initial_note = Note::new(TONES_FREQ[NOTES::A as usize][4], 0.0, 0.05)
        .with_generator(GENERATORS::SINE)
        .with_envelope(&envelope);
    let mut second_note = Note::new(TONES_FREQ[NOTES::C as usize][4], 0.25, 0.05)
        .with_generator(GENERATORS::SINE)
        .with_envelope(&envelope);
    let mut third_note = Note::new(TONES_FREQ[NOTES::E as usize][4], 0.5, 0.05)
        .with_generator(GENERATORS::SINE)
        .with_envelope(&envelope);

    // Create a `duration` second(s) long impulse
    for i in 0..(duration * sample_rate) as usize {
        system
            .push(
                0,
                initial_note.tick(i as i32, sample_rate as i32)
                    + second_note.tick(i as i32, sample_rate as i32)
                    + third_note.tick(i as i32, sample_rate as i32),
            )
            .unwrap();
    }

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // TODO: Move system to score
    let mut values = vec![];
    loop {
        values.clear();
        for _ in 0..sample_rate as usize {
            system.run();
            let sink: SafeSink = system.get_sink(0).unwrap();
            values.append(sink.borrow_mut().get_values());
        }

        sink.append(SamplesBuffer::new(
            1 as u16,
            sample_rate as u32,
            values.iter().map(|n| *n).collect::<Vec<f32>>(),
        ));
    }
}
