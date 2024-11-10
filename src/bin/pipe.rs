//! Pipe and Filter Architecture test
//!! Run with `cargo run --bin pipe | uv too run pipeplot` to plot the output

use std::cell::RefCell;
use std::rc::Rc;

use log::error;

use rustic::filters::{CombinatorFilter, DelayFilter, DuplicateFilter, GainFilter};
use rustic::filters::{Pipe, System};

fn main() {
    colog::init();

    let duration = 1.0; // 0.25 seconds
    let sample_rate = 100.0; // 100 Hz

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
        (0.5 * sample_rate) as usize,
    );

    // Diminish gain in feedback loop
    let gain_filter = GainFilter::new(Rc::clone(&feedback_delayed), Rc::clone(&feedback_end), 0.75);

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

    // Create a `duration` second(s) long impulse
    for i in 0..(duration * sample_rate) as usize {
        system
            .push(0, 100.0 - (i as f32 / (duration * sample_rate)) * 100.0)
            .unwrap();
    }

    loop {
        println!("{}", system.get_sink(0).unwrap().borrow_mut().pop());
        system.run();
    }
}
