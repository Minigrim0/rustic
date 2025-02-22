use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use log::{info, trace};
use petgraph::dot::Dot;
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
    // The node index is the index of the filter that the source is connected to
    // The second usize is the port of the filter that the source is connected to
    sources: [(Box<dyn Source>, (NodeIndex<u32>, usize)); INPUTS],
    // The sinks of the system.
    // The node index is the index of the filter that the sink is connected to
    // The second usize is the port of the filter that the sink is connected to
    sinks: [((NodeIndex<u32>, usize), Box<dyn Sink>); OUTPUTS],
}

impl<const INPUTS: usize, const OUTPUTS: usize> System<INPUTS, OUTPUTS> {
    /// Creates a new system with simple null sources & simple sinks
    pub fn new() -> Self {
        let sources: [(Box<dyn Source>, (NodeIndex<u32>, usize)); INPUTS] = core::array::from_fn(|_| (simple_source(NullGenerator::new()), (NodeIndex::new(0), 0)));
        let sinks: [((NodeIndex<u32>, usize), Box<dyn Sink>); OUTPUTS] = core::array::from_fn(|_| ((NodeIndex::new(0), 0), simple_sink()));

        System {
            graph: Graph::new(),
            layers: Vec::new(),
            sources,
            sinks,
        }
    }

    /// Merges the two systems together to create a new one. The graphs are merged following the given mapping from sinks to sources.
    /// Sinks to sources links are replaced with a simple combinator filter
    pub fn merge<const T: usize>(mut self, other: System<OUTPUTS, T>, mapping: Vec<(usize, usize)>) -> System<INPUTS, T> {

        // Contains the mapping other graph -> new graph
        let mut new_edge_map: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        for (from, to) in mapping.iter() {
            let (graph_b_source_descendant_index, graph_b_node_port) = other.sources[*to].1;
            let source_descendant = dyn_clone::clone_box(&*other.graph[graph_b_source_descendant_index]);

            // Save the new index of the source descendant
            let new_index = if new_edge_map.contains_key(&graph_b_source_descendant_index) {
                match new_edge_map.get(&graph_b_source_descendant_index) {
                    Some(v) => *v,
                    None => panic!("What ?")
                }
            } else {
                let new_index = self.graph.add_node(source_descendant);
                self.graph[new_index].set_index(new_index.index());
                info!("idx {} -> idx {}", graph_b_source_descendant_index.index(), new_index.index());
                new_edge_map.insert(graph_b_source_descendant_index, new_index);
                new_index
            };

            // Connect the sink's predecessors to the source's successors
            let (sink_predecessor_id, sink_predecessor_port) = self.sinks[*from].0;
            info!("Node {} -> Sink {} & Source {} -> Node {} => Node {} -> Node {}", sink_predecessor_id.index(), from, to, graph_b_source_descendant_index.index(), sink_predecessor_id.index(), new_index.index());
            self.graph.add_edge(sink_predecessor_id, new_index, (sink_predecessor_port, graph_b_node_port));
        }

        // Go through all nodes in the other graph and add them to the new graph
        for node_index in other.graph.node_indices() {
            // Skip already added nodes
            if let Some(_) = new_edge_map.get(&node_index) {
                info!("Node {} already pushed to new graph", node_index.index());
                continue;
            }

            let node = dyn_clone::clone_box(&*other.graph[node_index]);
            let new_index = self.graph.add_node(node);
            new_edge_map.insert(node_index, new_index);
        }

        // Now that all nodes are added, connect the edges
        for edge in other.graph.edge_indices() {
            let (other_from, other_to) = other.graph.edge_endpoints(edge).unwrap();
            let (from, to) = (new_edge_map[&other_from], new_edge_map[&other_to]);
            info!("Edge ({}, {}) -> ({}, {})", other_from.index(), other_to.index(), from.index(), to.index());
            let weight = other.graph[edge];
            self.graph.add_edge(from, to, weight);
        }

        let new_sinks: [((NodeIndex<u32>, usize), Box<dyn Sink>); T] = core::array::from_fn(|index| ((new_edge_map[&other.sinks[index].0.0], other.sinks[index].0.1), dyn_clone::clone_box(&*other.sinks[index].1)));

        let new_system: System<INPUTS, T> = System {
            graph: self.graph,
            layers: self.layers,
            sources: self.sources,
            sinks: new_sinks,
        };

        new_system
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

    /// Connects a source to a filter of the graph
    pub fn connect_source(&mut self, source: usize, to: NodeIndex<u32>, in_port: usize) {
        self.sources[source].1 = (to, in_port);
    }

    /// Connects a filter from the graph to a sink
    pub fn connect_sink(&mut self, from: NodeIndex<u32>, sink: usize, out_port: usize) {
        info!("Node {} (p: {}) -> Sink {}", from.index(), out_port, sink);
        self.sinks[sink].0 = (from, out_port);
    }

    /// Sets the sink at index `index` to be the given sink object
    pub fn set_sink(&mut self, index: usize, sink: Box<dyn Sink>) {
        trace!("[Graph] Setting Node {} as sink {}", sink.get_name(), index);
        self.sinks[index] = ((NodeIndex::new(0), 0), sink);
    }

    /// Sets source at index `index` to be the given source object
    pub fn set_source(&mut self, index: usize, source: Box<dyn Source>) {
        trace!("[Graph] Setting Node {} as source", source.get_name());
        self.sources[index] = (source, (NodeIndex::new(0), 0));
    }

    // Creates the execution layers by sorting the graph topologically.
    pub fn compute(&mut self) -> Result<(), ()> {
        // Makes the graph acyclic to be able to create a topology sort
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
        self.sources.iter_mut().for_each(|(source, (desc, port))| {
            let value = source.pull();
            let filter = &mut self.graph[*desc];
            filter.push(value, *port);
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
        self.sinks.iter_mut().for_each(|((node, port), sink)| {
            let values = {
                let node = &mut self.graph[*node];
                node.transform()
            };
            sink.push(values[*port], 0);
        });
    }

    /// Returns a sink pipe from the system. If the index is out of bounds, returns an error.
    pub fn get_sink(&mut self, index: usize) -> Result<&mut Box<dyn Sink>, &str> {
        self.sinks
            .get_mut(index)
            .and_then(|s| Some(&mut s.1))
            .ok_or("Index out of bounds")
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let mut output = File::create(path).map_err(|e| e.to_string())?;
        write!(output, "{:?}", Dot::with_config(&self.graph, &[])).map_err(|e| e.to_string())
    }
}
