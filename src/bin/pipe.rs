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
use rustic::filters::{CombinatorFilter, DelayFilter, DuplicateFilter, GainFilter, Pipe, System};
use rustic::generator::GENERATORS;

fn main() {
    colog::init();

    let duration = 5.0; // 0.25 seconds
    let sample_rate = 44100.0; // 44100 Hz

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
        (0.2 * sample_rate) as usize,
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
            values.push(system.get_sink(0).unwrap().borrow_mut().pop());
        }

        sink.append(SamplesBuffer::new(
            1 as u16,
            sample_rate as u32,
            values.iter().map(|n| *n).collect::<Vec<f32>>(),
        ));
    }
}
