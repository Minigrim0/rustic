use super::{Filter, SafePipe};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Low-pass filter using a first-order IIR filter
pub struct LowPassFilter {
    source: SafePipe,
    sink: SafePipe,
    cutoff_frequency: f32,
    previous_output: f32,
    uuid: Uuid,
}

impl LowPassFilter {
    pub fn new(source: SafePipe, sink: SafePipe, cutoff_frequency: f32) -> Self {
        Self {
            source,
            sink,
            cutoff_frequency,
            previous_output: 0.0,
            uuid: Uuid::new_v4(),
        }
    }
}

impl Filter for LowPassFilter {
    fn transform(&mut self) {
        let input = self.source.borrow_mut().pop();
        let alpha = self.cutoff_frequency / (self.cutoff_frequency + 1.0);
        let output = alpha * input + (1.0 - alpha) * self.previous_output;
        self.previous_output = output;
        self.sink.borrow_mut().push(output);
    }

    fn get_name(&self) -> &str {
        "Low Pass Filter"
    }
}

#[cfg(feature = "meta")]
impl Metadata for LowPassFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "LowPassFilter".to_string(),
            description: "Low-pass filter using a first-order IIR filter".to_string(),
            inputs: 1,
            outputs: 1,
        }
    }
}
