//! Pipe and Filter Architecture test

use log::info;
use std::cell::RefCell;
use std::rc::Rc;

use rustic::pf::filters::Filter;
use rustic::pf::filters::{AmplifierFilter, CombinatorFilter, DelayFilter, DuplicateFilter};
use rustic::pf::pipe::{Pipe, SafePipe};

struct PFSystem {
    filters: Vec<Box<dyn Filter>>,
    sources: Vec<SafePipe>,
    sinks: Vec<SafePipe>,
}

impl PFSystem {
    pub fn run(&mut self) {
        for filter in self.filters.iter_mut() {
            filter.transform();
        }
    }

    pub fn get_sink(&self, index: usize) -> SafePipe {
        self.sinks[index].clone()
    }

    pub fn push(&self, index: usize, value: f32) {
        self.sources[index].borrow_mut().push(value * 2.0);
    }
}

fn main() {
    colog::init();

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
        22050,
    );

    let ampl_filter = AmplifierFilter::new(Rc::clone(&feedback_delayed), Rc::clone(&source2), 0.75);

    filters.push(Box::from(sum_filter));
    filters.push(Box::from(dupe_filter));
    filters.push(Box::from(delay_filter));
    filters.push(Box::from(ampl_filter));

    let mut system = PFSystem {
        filters,
        sources: vec![source1, source2],
        sinks: vec![system_sink],
    };

    system.push(0, 100.0);

    loop {
        println!("{}", system.get_sink(0).borrow_mut().pop());
        system.run();
    }
}
