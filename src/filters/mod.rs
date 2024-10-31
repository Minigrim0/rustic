use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use petgraph::graph::Graph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterMetadata {
    pub name: String,        // Name of the filter
    pub description: String, // Description of the filter
    pub inputs: usize,       // Number of input pipes
    pub outputs: usize,      // Number of output pipes
}

pub trait Metadata {
    fn get_metadata() -> FilterMetadata;
}

/// A filter that can process data from source pipes and send to sink pipes.
pub trait Filter {
    fn transform(&mut self);
}

pub struct PFSystem {
    filters: Vec<Box<dyn Filter>>,
    sources: Vec<SafePipe>,
    sinks: Vec<SafePipe>,
    graph: Graph<Rc<RefCell<Box<dyn Filter>>>, ()>,
}

impl PFSystem {
    pub fn new(
        filters: Vec<Box<dyn Filter>>,
        sources: Vec<SafePipe>,
        sinks: Vec<SafePipe>,
    ) -> Self {
        PFSystem {
            filters,
            sources,
            sinks,
            graph: Graph::new(),
        }
    }

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

    pub fn save(&mut self) -> Result<(), ()> {
        for filter in self.filters.iter() {
            self.graph.add_node(Rc::new(RefCell::from(filter)));
        }
        Ok(())
    }
}

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

    /// Pops the first element from the buffer and returns it.
    /// Defaults to 0.0. Because we're working with sound, no
    /// sample means silence.
    pub fn pop(&mut self) -> f32 {
        self.buff.pop_front().unwrap_or(0.0)
    }

    pub fn take(&mut self, amount: usize) -> Vec<f32> {
        self.buff.drain(0..amount).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.buff.is_empty()
    }
}

mod amplifier;
mod combinator;
mod delay;
mod high_pass;
mod low_pass;
mod structural;

pub type SafePipe = Rc<RefCell<Pipe>>;

pub use amplifier::*;
pub use combinator::*;
pub use delay::*;
pub use high_pass::*;
pub use low_pass::*;
pub use structural::*;
