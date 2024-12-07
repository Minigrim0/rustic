use std::collections::VecDeque;
use uuid::Uuid;

use log::trace;

use super::{Filter, SafeFilter, SafeSink, AudioGraphElement};
#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Delays it input for x samples
pub struct DelayFilter {
    sources: [f32; 1],
    desc: [Option<(SafeFilter, usize)>; 1],
    sinks: [Option<(SafeSink, usize)>; 1],
    delay_for: usize,
    buffer: VecDeque<f32>,
    uuid: Uuid,
}

impl DelayFilter {
    pub fn new(delay: usize) -> Self {
        Self {
            sources: [0.0],
            desc: [None],
            sinks: [None],
            delay_for: delay,
            buffer: VecDeque::from(vec![0.0; delay]),
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

impl Filter for DelayFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    fn transform(&mut self) {
        let input = self.sources[0];
        let output = self.buffer.pop_front().unwrap_or(0.0);

        trace!("Delay filter running {} -> {}", self.sources[0], output);

        self.buffer.push_back(input);
        self.desc.iter().for_each(|f| if let Some(filter) = f{
            filter.0.borrow_mut().push(output, filter.1);
        });
        self.sinks.iter().for_each(|s| if let Some(sink) = s {
            sink.0.borrow_mut().push(output, sink.1);
        });
    }

    fn connect(&mut self, from_port: usize, to: SafeFilter, to_port: usize) {
        self.desc[from_port] = Some((to, to_port));
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeSink, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }
}

impl AudioGraphElement for DelayFilter {
    fn get_name(&self) -> &str {
        "Delay Filter"
    }

    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

#[cfg(feature = "meta")]
impl Metadata for DelayFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "DelayFilter".to_string(),
            description: "Delays its input for x samples".to_string(),
            inputs: 1,
            outputs: 1,
        }
    }
}
