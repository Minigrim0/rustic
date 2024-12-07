/// This file contains structural filters i.e. filters that do not modify
/// values that pass through it but rather duplicate/merges its inputs
use super::{Filter, SafeFilter, SafeSink, AudioGraphElement};
use uuid::Uuid;

use log::trace;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Duplicates the content of the input onto two outputs
pub struct DuplicateFilter {
    sources: [f32; 1],
    desc: [Option<(SafeFilter, usize)>; 2],
    sinks: [Option<(SafeSink, usize)>; 1],
    uuid: Uuid,
}

impl DuplicateFilter {
    pub fn new() -> Self {
        Self {
            sources: [0.0],
            desc: [None, None],
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

impl Filter for DuplicateFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    fn transform(&mut self) {
        let source_value = self.sources[0];

        trace!("Duplicate filter running {} -> {}, {}", source_value, source_value, source_value);
        self.desc.iter().for_each(|f| if let Some(filter) = f{
            filter.0.borrow_mut().push(source_value, filter.1);
        });
        self.sinks.iter().for_each(|s| if let Some(sink) = s {
            sink.0.borrow_mut().push(source_value, sink.1);
        });
    }

    fn connect(&mut self, from_port: usize, sink: SafeFilter, to_port: usize) {
        self.desc[from_port] = Some((sink, to_port));
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeSink, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }
}

impl AudioGraphElement for DuplicateFilter {
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
