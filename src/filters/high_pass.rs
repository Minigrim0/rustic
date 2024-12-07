use super::{Filter, SafeFilter, SafeSink, AudioGraphElement};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// High-pass filter using a first-order IIR filter
pub struct HighPassFilter {
    sources: [f32; 1],
    desc: [Option<(SafeFilter, usize)>; 1],
    sinks: [Option<(SafeSink, usize)>; 1],
    cutoff_frequency: f32,
    previous_output: f32,
    uuid: Uuid,
}

impl HighPassFilter {
    pub fn new(cutoff_frequency: f32) -> Self {
        Self {
            sources: [0.0],
            desc: [None],
            sinks: [None],
            cutoff_frequency,
            previous_output: 0.0,
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

impl Filter for HighPassFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    fn transform(&mut self) {
        let input = self.sources[0];
        let alpha = 1.0 / (self.cutoff_frequency + 1.0);
        let output = alpha * input + alpha * self.previous_output;
        self.previous_output = output;
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

impl AudioGraphElement for HighPassFilter {
    fn get_name(&self) -> &str {
        "High Pass Filter"
    }

    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

#[cfg(feature = "meta")]
impl Metadata for HighPassFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "HighPassFilter".to_string(),
            description: "High-pass filter using a first-order IIR filter".to_string(),
            inputs: 1,
            outputs: 1,
        }
    }
}
