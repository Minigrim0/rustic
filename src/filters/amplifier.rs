use super::{Filter, SafeFilter};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// A filter that returns the input value multiplied by a constant factor.
/// Note: a factor < 1.0 will attenuate the input signal, while a factor > 1.0
/// will amplify it.
// #[cfg_attr(feature = "meta", derive(Metadata))]
pub struct GainFilter {
    sources: [f32; 1],
    sinks: [Option<(SafeFilter, usize)>; 1],
    factor: f32,
    uuid: Uuid,
}

impl GainFilter {
    pub fn new(factor: f32) -> Self {
        Self {
            sources: [0.0],
            sinks: [None],
            factor,
            uuid: Uuid::new_v4(),
        }
    }

    /// Set the sink of the filter
    pub fn with_sink(mut self, position: usize, sink: SafeFilter, sink_port: usize) -> Self {
        self.sinks[position] = Some((sink, sink_port));
        self
    }
}

impl Filter for GainFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }

    fn transform(&mut self) {
        let output = self.sources.map(|f| f * self.factor);
        if let Some((sink, port)) = &self.sinks[0] {
            sink.borrow_mut().push(output[0], *port);
        }
    }

    fn get_name(&self) -> &str {
        "Gain Filter"
    }

    fn add_sink(&mut self, out_port: usize, sink: SafeFilter, in_port: usize) {
        self.sinks[out_port] = Some((sink, in_port));
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
