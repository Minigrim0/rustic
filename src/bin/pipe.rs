//! Pipe and Filter Architecture test
//!! Run with `cargo run --bin pipe | uv too run pipeplot` to plot the output

use std::cell::RefCell;
use std::rc::Rc;

use rustic::filters::{AmplifierFilter, CombinatorFilter, DelayFilter, DuplicateFilter};
use rustic::filters::{Filter, Pipe, PFSystem};


fn main() {
    colog::init();

    let duration = 1.0;  // 0.25 seconds
    let sample_rate = 100.0;  // 100 Hz

    let mut filters: Vec<Box<dyn Filter>> = vec![];

    let source1 = Rc::new(RefCell::new(Pipe::new()));
    let source2 = Rc::new(RefCell::new(Pipe::new()));

    let sum_result = Rc::new(RefCell::new(Pipe::new()));

    let feedback_source = Rc::new(RefCell::new(Pipe::new())); // Source for the feedback loop
    let feedback_delayed = Rc::new(RefCell::new(Pipe::new())); // Delayed feedback

    let system_sink = Rc::new(RefCell::new(Pipe::new()));

    let sum_filter = CombinatorFilter::new(
        [Rc::clone(&source1), Rc::clone(&source2)],
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

    let ampl_filter = AmplifierFilter::new(Rc::clone(&feedback_delayed), Rc::clone(&source2), 0.75);

    filters.push(Box::from(sum_filter));
    filters.push(Box::from(dupe_filter));
    filters.push(Box::from(delay_filter));
    filters.push(Box::from(ampl_filter));

    let mut system = PFSystem::new(
        filters,
        vec![source1, source2],
        vec![system_sink],
    );

    // Create a `duration` second(s) long impulse
    for i in 0..(duration * sample_rate) as usize {
        system.push(0, 100.0 - (i as f32 / (duration * sample_rate)) * 100.0);
    }

    loop {
        println!("{}", system.get_sink(0).borrow_mut().pop());
        system.run();
    }
}
