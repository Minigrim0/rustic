use super::{Filter, SafeFilter};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// A filter that take input from two sources and combines them into a single
/// output by adding them together.
pub struct CombinatorFilter {
    sources: [f32; 2],
    sinks: [Option<(SafeFilter, usize)>; 1],
    uuid: Uuid,
}

impl CombinatorFilter {
    pub fn new() -> Self {
        Self {
            sources: [0.0; 2],
            sinks: [None],
            uuid: Uuid::new_v4(),
        }
    }

    /// Set the sink of the filter
    pub fn with_sink(mut self, position: usize, sink: SafeFilter, sink_port: usize) -> Self {
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
        if let Some((sink, port)) = &self.sinks[0] {
            sink.borrow_mut().push(output, *port);
        }
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeFilter, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }

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
