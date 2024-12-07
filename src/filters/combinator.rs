use super::{AudioGraphElement, Filter, SafeFilter, SafeSink};
use uuid::Uuid;

use log::trace;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// A filter that take input from two sources and combines them into a single
/// output by adding them together.
pub struct CombinatorFilter {
    sources: [f32; 2],
    desc: [Option<(SafeFilter, usize)>; 1],
    sinks: [Option<(SafeSink, usize)>; 1],
    uuid: Uuid,
}

impl CombinatorFilter {
    pub fn new() -> Self {
        Self {
            sources: [0.0; 2],
            desc: [None],
            sinks: [None],
            uuid: Uuid::new_v4(),
        }
    }

    /// Set the sink of the filter
    pub fn with_connection(mut self, position: usize, to: SafeFilter, to_port: usize) -> Self {
        self.desc[position] = Some((to, to_port));
        self
    }

    /// Set the sink of the filter
    pub fn with_sink(mut self, position: usize, sink: SafeSink, sink_port: usize) -> Self {
        self.sinks[position] = Some((sink, sink_port));
        self
    }
}

impl Filter for CombinatorFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    fn transform(&mut self) {
        let output = self.sources.iter().sum();
        trace!("Combinator filter running [{}, {}] -> {}", self.sources[0], self.sources[1], output);

        self.desc.iter().for_each(|f| if let Some(filter) = f{
            filter.0.borrow_mut().push(output, filter.1);
        });
        self.sinks.iter().for_each(|s| if let Some(sink) = s {
            sink.0.borrow_mut().push(output, sink.1);
        });
    }

    fn connect(&mut self, out_port: usize, to: SafeFilter, to_port: usize) {
        self.desc[out_port] = Some((to, to_port));
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeSink, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }
}

impl AudioGraphElement for CombinatorFilter {
    fn get_name(&self) -> &str {
        "Combinator"
    }

    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

#[cfg(feature = "meta")]
impl Metadata for CombinatorFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "CombinatorFilter".to_string(),
            description: "Combines two inputs by adding them together".to_string(),
            inputs: 2,
            outputs: 1,
        }
    }
}
