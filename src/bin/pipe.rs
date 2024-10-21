//! Pipe and Filter Architecture with Combinator
//!
//! This module implements a flexible pipe and filter system that allows multiple filters
//! to connect to the same Pipe using a `Combinator`. The Combinator can have an arbitrary
//! number of pipes as input and output, performing a certain action (defined by a struct
//! implementing its trait) to produce output(s).

use log::info;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Represents a pipe that can store and transfer data.
pub struct Pipe {
    buff: VecDeque<f32>,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            buff: VecDeque::new(),
        }
    }

    pub fn push(&mut self, item: f32) {
        self.buff.push_back(item);
    }

    pub fn pop(&mut self) -> Option<f32> {
        self.buff.pop_front()
    }

    pub fn take(&mut self, amount: usize) -> Vec<f32> {
        self.buff.drain(0..amount).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.buff.is_empty()
    }
}

/// Trait defining the behavior of a filter operation.
pub trait FilterOperation: Sizeable {
    type Size = usize;

    fn apply(&self, sources: [&Vec<Pipe>; Self::Size]) -> Vec<f32>;
    fn name(&self) -> &str;

    /// Check if the filter is ready to process data.
    /// This can be used to check if the filter has enough data to process.
    /// For example, a filter that requires 10 samples to process can return false
    /// if it has less than 10 samples in the source buffer(s).
    fn ready(&self, filter: &Filter) -> bool;
}

/// A filter that can process data from source pipes and send to sink pipes.
pub struct Filter {
    sources: Vec<Rc<RefCell<Pipe>>>,
    sinks: Vec<Rc<RefCell<Pipe>>>,
    operation: Box<dyn FilterOperation>,
}

impl Filter {
    pub fn new(operation: Box<dyn FilterOperation>) -> Self {
        Self {
            sources: Vec::new(),
            sinks: Vec::new(),
            operation,
        }
    }

    pub fn add_source(&mut self, source: Rc<RefCell<Pipe>>) {
        self.sources.push(source);
    }

    pub fn add_sink(&mut self, sink: Rc<RefCell<Pipe>>) {
        self.sinks.push(sink);
    }

    pub fn transform(&self) {
        let input: Vec<f32> = self
            .sources
            .iter()
            .flat_map(|source| {
                let len = source.borrow().buff.len();
                source.borrow_mut().take(len)
            })
            .collect();

        let output = self.operation.apply(&self.sources);

        for (i, sink) in self.sinks.iter().enumerate() {
            let chunk_size = output.len() / self.sinks.len();
            let start = i * chunk_size;
            let end = if i == self.sinks.len() - 1 {
                output.len()
            } else {
                (i + 1) * chunk_size
            };
            output[start..end].iter().for_each(|&x| {
                sink.borrow_mut().buff.push_back(x);
            });
        }
    }
}

struct PFSystem {
    filters: Vec<Filter>,
    sources: Vec<Rc<RefCell<Pipe>>>,
    sinks: Vec<Rc<RefCell<Pipe>>>,
}

impl PFSystem {
    pub fn run(&self) {
        for filter in &self.filters {
            filter.transform();
        }
    }

    pub fn get_sink(&self, index: usize) -> Rc<RefCell<Pipe>> {
        self.sinks[index].clone()
    }
}

// /// A combinator is a superstructure that can combine multiple filters to create a complex
// /// data processing pipeline. It can have multiple sources and sinks, and can run the filters
// pub struct Combinator {
//     inputs: Vec<Rc<RefCell<Pipe>>>,
//     filters: Vec<Filter>,
//     exits: Vec<Rc<RefCell<Pipe>>>,
// }

// impl Combinator {
//     pub fn new() -> Self {
//         Self {
//             inputs: Vec::new(),
//             filters: Vec::new(),
//             exits: Vec::new(),
//         }
//     }

//     pub fn set_entry(&mut self, entry: Filter) {
//         self.filters.insert(0, entry);
//     }

//     pub fn add_filter(&mut self, filter: Filter) {
//         self.filters.push(filter);
//     }

//     pub fn run(&self) {
//         for filter in &self.filters {
//             filter.transform();
//         }
//     }
// }

// Example filter operation
struct DoubleFilter;

impl FilterOperation for DoubleFilter {
    fn apply(&self, input: &[f32]) -> Vec<f32> {
        input.iter().map(|&x| x * 2.0).collect()
    }

    fn name(&self) -> &str {
        "DoubleFilter"
    }

    fn ready(&self, filter: &Filter) -> bool {
        filter
            .sources
            .iter()
            .all(|source| source.borrow().buff.len() >= 1)
    }
}

/// Takes two sources and produces one output with the sum of the two sources
struct SumCombinationFilter;

impl FilterOperation for SumCombinationFilter {
    fn apply(&self, sources: )
}

fn main() {
    colog::init();

    let mut filters = vec![];

    info!("Creating a source pipe");
    let source = Rc::new(RefCell::new(Pipe::new()));
    info!("Pushing data to source pipe...");
    source.borrow_mut().push(1.0);
    source.borrow_mut().push(2.0);
    source.borrow_mut().push(3.0);

    let second_source = Rc::new(RefCell::new(Pipe::new()));
    second_source.borrow_mut().push(4.0);
    second_source.borrow_mut().push(5.0);
    second_source.borrow_mut().push(6.0);

    info!("Creating a sink pipe");
    let sink = Rc::new(RefCell::new(Pipe::new()));

    info!("Creating a filter and connecting to source and sink pipes");
    let mut filter = Filter::new(Box::new(DoubleFilter));
    filter.add_source(Rc::clone(&source));
    filter.add_source(Rc::clone(&second_source));
    filter.add_sink(Rc::clone(&sink));

    filters.push(filter);

    // info!("Creating a combinator and adding the filter");
    // let mut combinator = Combinator::new();
    // combinator.add_filter(filter);

    // info!("Running the combinator");
    // combinator.run();

    // loop {
    //     combinator.run();
    //     if let Some(value) = sink.borrow_mut().pop() {
    //         println!("{:?}", value);
    //     }
    // }

    let system = PFSystem {
        filters,
        sources: vec![source, second_source],
        sinks: vec![sink],
    };

    loop {
        system.run();
        if let Some(value) = system.get_sink(0).borrow_mut().pop() {
            println!("{:?}", value);
        }
    }
}
