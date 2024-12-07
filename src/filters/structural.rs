/// This file contains structural filters i.e. filters that do not modify
/// values that pass through it but rather duplicate/merges its inputs
use super::{Filter, SafeFilter};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Duplicates the content of the input onto two outputs
pub struct DuplicateFilter {
    sources: [f32; 1],
    sinks: [Option<(SafeFilter, usize)>; 2],
    uuid: Uuid,
}

impl DuplicateFilter {
    pub fn new() -> Self {
        Self {
            sources: [0.0],
            sinks: [None, None],
            uuid: Uuid::new_v4(),
        }
    }

    /// Set the sink of the filter
    pub fn with_sink(mut self, position: usize, sink: SafeFilter, sink_port: usize) -> Self {
        self.sinks[position] = Some((sink, sink_port));
        self
    }
}

impl Filter for DuplicateFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    fn transform(&mut self) {
        let source_value = self.sources[0];
        self.sinks
            .iter()
            .for_each(|sink| match sink {
                Some((sink, port)) => sink.borrow_mut().push(source_value, *port),
                None => (),
            });
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeFilter, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }

    fn get_name(&self) -> &str {
        "Duplicate"
    }

    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

#[cfg(feature = "meta")]
impl Metadata for DuplicateFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "DuplicateFilter".to_string(),
            description: "Duplicates the content of the input onto two outputs".to_string(),
            inputs: 1,
            outputs: 2,
        }
    }
}
