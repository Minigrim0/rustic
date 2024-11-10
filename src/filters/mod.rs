use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use log::warn;
use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};
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
    fn get_name(&self) -> &str;
}

/// A Filter that is safe to share (same-thread)
type SafeFilter = Rc<RefCell<Box<dyn Filter>>>;
/// A Layer that contains safe filters
type FilterLayer = Vec<SafeFilter>;

/// A Pipe & Filter system
/// The system is composed of filters, sources, sinks and pipes.
/// The system is represented as a directed graph where the filters are the nodes
/// The Sources & Sinks are special nodes that have respectivilly only outgoing (source) or incoming (sink) edges
/// The edges represent the pipes between the filters
/// Some filters have special output properties. E.g. the delay filter's input pipe is ignored when
/// the topology sorting is done, in order to avoid cycles. A system with cycles must include a delay or similar filter
/// to break the cycle.
pub struct System {
    // The actual filter graph, from which the execution order is derived
    graph: Graph<SafeFilter, SafePipe>,
    // Each layer represent filters that can be run concurrently.
    layers: Vec<FilterLayer>,
    // The sources of the system,
    sources: Vec<SafePipe>,
    sinks: Vec<SafePipe>,
}

impl System {
    pub fn new() -> Self {
        System {
            graph: Graph::new(),
            layers: Vec::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
        }
    }

    // Adds a filter to the system. Further references to this filter should be done using the returned uuid
    pub fn add_filter(&mut self, filter: Box<dyn Filter>) -> NodeIndex<u32> {
        self.graph.add_node(Rc::from(RefCell::from(filter)))
    }

    // Connects two filters together
    pub fn connect(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>, pipe: SafePipe) {
        self.graph.add_edge(from, to, pipe);
    }

    pub fn add_source(&mut self, source: SafePipe) {
        self.sources.push(source);
    }

    pub fn add_sink(&mut self, sink: SafePipe) {
        self.sinks.push(sink);
    }

    // Creates the execution layers by sorting the graph topologically.
    pub fn compute(&mut self) -> Result<(), ()> {
        if let Ok(topo) = toposort(&self.graph, None) {
            for node in topo {
                self.layers.push(vec![self.graph[node].clone()])
            }

            Ok(())
        } else {
            Err(())
        }
    }

    // Performs one full run of the system, running every filter once in an order such that data that entered the system this
    // run, can exit it this run as well.
    pub fn run(&mut self) {
        for layer in self.layers.iter() {
            // TODO: Make this parallel
            layer.iter().for_each(|f| {
                if let Ok(mut filter) = f.try_borrow_mut() {
                    filter.transform();
                } else {
                    warn!(
                        "Unable to borrow filter {} for transformation",
                        if let Ok(filter) = f.try_borrow() {
                            filter.get_name().to_string()
                        } else {
                            "ERR".to_string()
                        }
                    )
                }
            });
        }
    }

    /// Returns a sink pipe from the system. If the index is out of bounds, returns an error.
    pub fn get_sink(&self, index: usize) -> Result<SafePipe, ()> {
        self.sinks.get(index).cloned().ok_or(())
    }

    /// Tries to push a value to a source pipe. Returns an error if the index is out of bounds.
    pub fn push(&self, index: usize, value: f32) -> Result<(), ()> {
        match self.sources.get(index) {
            Some(source) => {
                source.borrow_mut().push(value);
                Ok(())
            }
            None => Err(()),
        }
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
