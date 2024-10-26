use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A filter that can process data from source pipes and send to sink pipes.
pub trait Filter {
    fn transform(&mut self);
}

pub struct PFSystem {
    filters: Vec<Box<dyn Filter>>,
    sources: Vec<SafePipe>,
    sinks: Vec<SafePipe>,
}

impl PFSystem {
    pub fn new(filters: Vec<Box<dyn Filter>>, sources: Vec<SafePipe>, sinks: Vec<SafePipe>) -> Self {
        PFSystem {
            filters,
            sources,
            sinks,
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
mod low_pass;
mod structural;

pub type SafePipe = Rc<RefCell<Pipe>>;

pub use amplifier::*;
pub use combinator::*;
pub use delay::*;
pub use low_pass::*;
pub use structural::*;
