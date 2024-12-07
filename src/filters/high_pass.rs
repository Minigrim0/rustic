use super::{Filter, SafeFilter};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// High-pass filter using a first-order IIR filter
pub struct HighPassFilter {
    sources: [f32; 1],
    sinks: [Option<(SafeFilter, usize)>; 1],
    cutoff_frequency: f32,
    previous_output: f32,
    uuid: Uuid,
}

impl HighPassFilter {
    pub fn new(cutoff_frequency: f32) -> Self {
        Self {
            sources: [0.0],
            sinks: [None],
            cutoff_frequency,
            previous_output: 0.0,
            uuid: Uuid::new_v4(),
        }
    }

    /// Set the sink of the filter
    pub fn with_sink(mut self, position: usize, sink: SafeFilter, sink_port: usize) -> Self {
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
        if let Some((sink, port)) = &self.sinks[0] {
            sink.borrow_mut().push(output, *port);
        }
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeFilter, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }

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
