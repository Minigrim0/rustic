use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::{warn, error};
use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};
#[cfg(feature = "meta")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "meta")]
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterMetadata {
    pub name: String,        // Name of the filter
    pub description: String, // Description of the filter
    pub inputs: usize,       // Number of input pipes
    pub outputs: usize,      // Number of output pipes
}

#[cfg(feature = "meta")]
pub trait Metadata {
    fn get_metadata() -> FilterMetadata;
}

/// A filter that can process data. Data should be pushed to the filter's input by either the preceding filter or a source.
pub trait Filter {
    fn push(&mut self, value: f32, port: usize);
    fn add_sink(&mut self, out_port: usize, sink: SafeFilter, in_port: usize);
    fn transform(&mut self);
    fn get_name(&self) -> &str;
    fn uuid(&self) -> uuid::Uuid;
}

// A pipe is defined as the filter and the port it is connected to
type Pipe = (NodeIndex<u32>, usize);

/// A Filter that is safe to share (same-thread)
pub type SafeFilter = Rc<RefCell<Box<dyn Filter>>>;
/// A Layer that contains safe filters
type FilterLayer = Vec<SafeFilter>;

pub type SafeSink = Rc<RefCell<Box<Sink>>>;

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
    graph: Graph<SafeFilter, ()>,
    // A map of filters uuids to their graph nodeindex
    filters: HashMap<uuid::Uuid, NodeIndex<u32>>,
    // Each layer represent filters that can be run concurrently.
    layers: Vec<FilterLayer>,
    // The sources of the system,
    sources: Vec<Pipe>,
    sinks: Vec<SafeSink>,
}

impl System {
    pub fn new() -> Self {
        System {
            graph: Graph::new(),
            filters: HashMap::new(),
            layers: Vec::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
        }
    }

    // Adds a filter to the system. Further references to this filter should be done using the returned uuid
    pub fn add_filter(&mut self, filter: Box<dyn Filter>) -> NodeIndex<u32> {
        let nodeindex = self.graph.add_node(Rc::from(RefCell::from(filter)));
        let uuid = if let Ok(filter) = self.graph[nodeindex].try_borrow() {
            filter.uuid()
        } else {
            error!("Unable to borrow filter to get uuid, using random one instead");
            uuid::Uuid::new_v4()
        };
        self.filters.insert(uuid, nodeindex);
        nodeindex
    }

    // Connects two filters together. This method connects the filter in the topologyu graph as well.
    // Do no use this function to close a feedback loop. Use the connect_feedback method instead.
    pub fn connect(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>, out_port: usize, in_port: usize) {
        self.graph.add_edge(from, to, ());
        if let Ok(mut filter) = self.graph[from].try_borrow_mut() {
            filter.add_sink(out_port, self.graph[to].clone(), in_port);
        } else {
            error!("Unable to borrow filter to set sink");
        }
    }

    // Connects two filters creating a feedback loop. This method does not connect the filter in the topology graph to allow for topological sorting.
    pub fn connect_feedback(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>, out_port: usize, in_port: usize) {
        if let Ok(mut filter) = self.graph[from].try_borrow_mut() {
            filter.add_sink(out_port, self.graph[to].clone(), in_port);
        } else {
            error!("Unable to borrow filter to set sink");
        }
    }

    pub fn connect_sink(&mut self, from: NodeIndex<u32>, sink: usize, out_port: usize, in_port: usize) {
        if let Ok(mut filter) = self.graph[from].try_borrow_mut() {
            let sink_filter: SafeFilter = Rc::new(RefCell::new(Box::new(Rc::clone(self.sinks.get(sink).expect("Index out of bounds for sink")))));
            filter.add_sink(out_port, sink_filter, in_port);
        }
    }

    pub fn add_source(&mut self, source: NodeIndex<u32>, in_port: usize) {
        self.sources.push((source, in_port));
    }

    pub fn add_sink(&mut self, sink: SafeSink) -> usize {
        self.sinks.push(sink);
        self.sinks.len() - 1
    }

    // Creates the execution layers by sorting the graph topologically.
    pub fn compute(&mut self) -> Result<(), ()> {
        if let Ok(topo) = toposort(&self.graph, None) {
            for node in topo {
                // TODO: Add same-layer ability (to run some filters in parallel)
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
    pub fn get_sink(&self, index: usize) -> Result<SafeSink, &str> {
        self.sinks.get(index)
            .and_then(|f| Some(Rc::clone(f)))
            .ok_or("Index out of bounds")
    }

    /// Tries to push a value to a source pipe. Returns an error if the index is out of bounds.
    pub fn push(&self, index: usize, value: f32) -> Result<(), ()> {
        match self.sources.get(index) {
            Some((index, port)) => {
                self.graph[*index].borrow_mut().push(value, *port);
                Ok(())
            }
            None => Err(()),
        }
    }
}

mod amplifier;
mod combinator;
mod delay;
mod high_pass;
mod low_pass;
mod structural;
mod sink;

pub use amplifier::*;
pub use combinator::*;
pub use delay::*;
pub use high_pass::*;
pub use low_pass::*;
pub use structural::*;
pub use sink::*;
