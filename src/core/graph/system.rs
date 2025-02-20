use log::trace;
use petgraph::prelude::NodeIndex;
use petgraph::Graph;
use petgraph::{algo::toposort, Direction};

use super::{Filter, Sink, Source};

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
    // Each weight represents the port into which the filter is connected
    graph: Graph<Box<dyn Filter>, (usize, usize)>,
    // Each layer represent filters that can be run concurrently.
    layers: Vec<Vec<usize>>,
    // The sources of the system and the filters they are connected to
    // The first usize is the index of the filter that the source is connected to
    // The second usize is the port of the filter that the source is connected to
    sources: Vec<(Box<dyn Source>, Vec<(usize, usize)>)>,
    // The sinks of the system. The first usize is the index of the filter that the sink is connected to
    // The second usize is the port of the filter that the sink is connected to
    // The Box<dyn Sink> is the sink itself
    sinks: Vec<(Vec<(usize, usize)>, Box<dyn Sink>)>,
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
        trace!("[Graph] Adding filter {}", filter.get_name());
        let nodeindex = self.graph.add_node(filter);
        self.graph[nodeindex].set_index(nodeindex.index());
        nodeindex
    }

    // Connects two filters together. This method connects the filter in the topologyu graph as well.
    // Do no use this function to close a feedback loop. Use the connect_feedback method instead.
    pub fn connect(
        &mut self,
        from: NodeIndex<u32>,
        to: NodeIndex<u32>,
        out_port: usize,
        in_port: usize,
    ) {
        trace!(
            "[Graph] Connecting {} (p: {}) to {} (p: {})",
            self.graph[from].get_name(),
            out_port,
            self.graph[to].get_name().to_string(),
            in_port
        );
        self.graph.add_edge(from, to, (out_port, in_port));
    }

    pub fn connect_source(&mut self, source: usize, to: NodeIndex<u32>, in_port: usize) {
        self.sources[source].1.push((to.index(), in_port));
    }

    pub fn connect_sink(&mut self, from: NodeIndex<u32>, sink: usize, out_port: usize) {
        self.sinks[sink].0.push((from.index(), out_port));
    }

    pub fn add_sink(&mut self, sink: Box<dyn Sink>) -> usize {
        trace!("[Graph] Setting Node {} as sink", sink.get_name());
        self.sinks.push((Vec::new(), sink));
        self.sinks.len() - 1
    }

    pub fn add_source(&mut self, source: Box<dyn Source>) -> usize {
        trace!("[Graph] Setting Node {} as source", source.get_name());
        self.sources.push((source, Vec::new()));
        self.sources.len() - 1
    }

    // Creates the execution layers by sorting the graph topologically.
    pub fn compute(&mut self) -> Result<(), ()> {
        if let Ok(topo) = toposort(&self.graph, None) {
            for node in topo {
                // TODO: Add same-layer ability (to run some filters in parallel)
                self.layers.push(vec![node.index()])
            }

            Ok(())
        } else {
            Err(())
        }
    }

    // Performs one full run of the system, running every filter once in an order such that data that entered the system this
    // run, can exit it this run as well.
    pub fn run(&mut self) {
        self.sources.iter_mut().for_each(|source| {
            let value = source.0.pull();
            for (filter, port) in source.1.iter() {
                let filter = &mut self.graph[NodeIndex::new(*filter)];
                filter.push(value, *port);
            }
        });

        for layer in self.layers.iter() {
            // TODO: Make this parallel
            layer.iter().for_each(|f| {
                let from_node_index = NodeIndex::new(*f);
                let outputs = {
                    let filter = &mut self.graph[from_node_index];
                    filter.transform()
                };

                let neighbours: Vec<NodeIndex> = {
                    self.graph
                        .neighbors_directed(from_node_index, Direction::Outgoing)
                        .collect()
                };

                for neighbour in neighbours {
                    let edges: Vec<(usize, usize)> = {
                        self.graph
                            .edges_connecting(from_node_index, neighbour)
                            .map(|e| *e.weight())
                            .collect()
                    };
                    let neighbour_node = &mut self.graph[neighbour];

                    edges.iter().for_each(|edge| {
                        neighbour_node.push(outputs[edge.0], edge.1);
                    });
                }
            });
        }
    }

    /// Returns a sink pipe from the system. If the index is out of bounds, returns an error.
    pub fn get_sink(&self, index: usize) -> Result<&Box<dyn Sink>, &str> {
        self.sinks
            .get(index)
            .and_then(|s| Some(&s.1))
            .ok_or("Index out of bounds")
    }
}
