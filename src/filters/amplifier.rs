use super::{AudioGraphElement, Filter, SafeFilter, SafeSink};
use uuid::Uuid;

use log::trace;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// A filter that returns the input value multiplied by a constant factor.
/// Note: a factor < 1.0 will attenuate the input signal, while a factor > 1.0
/// will amplify it.
#[cfg_attr(feature = "meta2", derive(derive::MetaData))]
pub struct GainFilter {
    sources: [f32; 1],
    desc: [Option<(SafeFilter, usize)>; 1],
    sinks: [Option<(SafeSink, usize)>; 1],
    factor: f32,
    uuid: Uuid,
}

impl GainFilter {
    pub fn new(factor: f32) -> Self {
        Self {
            sources: [0.0],
            desc: [None],
            sinks: [None],
            factor,
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

impl Filter for GainFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    /// Transforms the input value by multiplying it by the factor and sends it to the sink.
    /// If multiple sources are connected to the filter, the output will be the sum of all
    /// the sources multiplied by the factor.
    fn transform(&mut self) {
        let output: f32 = self.sources.map(|f| f * self.factor).iter().sum();
        trace!("Gain filter running {} -> {}", self.sources[0], output);

        self.desc.iter().for_each(|f| {
            if let Some(filter) = f {
                filter.0.borrow_mut().push(output, filter.1);
            }
        });
        self.sinks.iter().for_each(|s| {
            if let Some(sink) = s {
                sink.0.borrow_mut().push(output, sink.1);
            }
        });
    }

    fn connect(&mut self, out_port: usize, sink: SafeFilter, in_port: usize) {
        self.desc[out_port] = Some((sink, in_port));
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeSink, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
    }
}

impl AudioGraphElement for GainFilter {
    fn get_name(&self) -> &str {
        "Gain Filter"
    }

    fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

#[cfg(feature = "meta")]
impl Metadata for GainFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "GainFilter".to_string(),
            description: "A filter that returns the input value multiplied by a constant factor."
                .to_string(),
            inputs: 1,
            outputs: 1,
        }
    }
}
