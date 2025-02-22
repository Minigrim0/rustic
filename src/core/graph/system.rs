use log::trace;
use petgraph::prelude::NodeIndex;
use petgraph::Graph;
use petgraph::{algo::toposort, Direction};

use crate::core::generator::prelude::NullGenerator;

use super::sink::simple_sink;
use super::{simple_source, Filter, Sink, Source};

/// A Pipe & Filter system
/// The system is composed of filters, sources, sinks and pipes.
/// The system is represented as a directed graph where the filters are the nodes
/// The Sources & Sinks are special nodes that have respectivilly only outgoing (source) or incoming (sink) edges
/// The edges represent the pipes between the filters
/// Some filters have special output properties. E.g. the delay filter's input pipe is ignored when
/// the topology sorting is done, in order to avoid cycles. A system with cycles must include a delay or similar filter
/// to break the cycle.
pub struct System<const INPUTS: usize, const OUTPUTS: usize> {
    // The actual filter graph, from which the execution order is derived
    // Each weight represents the port into which the filter is connected
    graph: Graph<Box<dyn Filter>, (usize, usize)>,
    // Each layer represent filters that can be run concurrently.
    layers: Vec<Vec<usize>>,
    // The sources of the system and the filters they are connected to
    // The first usize is the index of the filter that the source is connected to
    // The second usize is the port of the filter that the source is connected to
    sources: [(Box<dyn Source>, Vec<(usize, usize)>); INPUTS],
    // The sinks of the system. The first usize is the index of the filter that the sink is connected to
    // The second usize is the port of the filter that the sink is connected to
    // The Box<dyn Sink> is the sink itself
    sinks: [(Vec<(usize, usize)>, Box<dyn Sink>); OUTPUTS],
}

impl<const INPUTS: usize, const OUTPUTS: usize> System<INPUTS, OUTPUTS> {
    pub fn new() -> Self {
        let sources: [(Box<dyn Source>, Vec<(usize, usize)>); INPUTS] = core::array::from_fn(|_| (simple_source(NullGenerator::new()), vec![]));
        let sinks: [(Vec<(usize, usize)>, Box<dyn Sink>); OUTPUTS] = core::array::from_fn(|_| (vec![], simple_sink()));

        System {
            graph: Graph::new(),
            layers: Vec::new(),
            sources,
            sinks,
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

    pub fn set_sink(&mut self, index: usize, sink: Box<dyn Sink>) {
        trace!("[Graph] Setting Node {} as sink {}", sink.get_name(), index);
        self.sinks[index] = (Vec::new(), sink);
    }

    pub fn set_source(&mut self, index: usize, source: Box<dyn Source>) {
        trace!("[Graph] Setting Node {} as source", source.get_name());
        self.sources[index] = (source, Vec::new());
    }

    // Creates the execution layers by sorting the graph topologically.
    pub fn compute(&mut self) -> Result<(), ()> {
        let acyclic_graph = self.graph.filter_map(
            |_index, node| Some(node),
            |index, edge| {
                if self.graph.edge_endpoints(index).and_then(|(_, to)| Some(self.graph[to].postponable())) == Some(true) {
                    None
                } else {
                    Some(edge)
                }
            }
        );

        if let Ok(topo) = toposort(&acyclic_graph, None) {
            for node in topo {
                // TODO: Add same-layer ability (to run some filters in parallel)
                // For each node in the topological order, push the node's index into the layers vector
                // if the next node has no dependencies to the current node, push it to the current layer as well
                // else push it to the next layer
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

        // Makes the sinks pull from their connected nodes
        self.sinks.iter_mut().for_each(|(connection, sink)| {
            connection.iter().for_each(|(node, port)| {
                let values = {
                    let node = &mut self.graph[NodeIndex::new(*node)];
                    node.transform()
                };
                sink.push(values[*port], 0);
            });
        });
    }

    /// Returns a sink pipe from the system. If the index is out of bounds, returns an error.
    pub fn get_sink(&mut self, index: usize) -> Result<&mut Box<dyn Sink>, &str> {
        self.sinks
            .get_mut(index)
            .and_then(|s| Some(&mut s.1))
            .ok_or("Index out of bounds")
    }
}
