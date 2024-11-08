use super::{Filter, FilterMetadata, Metadata, SafePipe};
use uuid::Uuid;

/// High-pass filter using a first-order IIR filter
pub struct HighPassFilter {
    source: SafePipe,
    sink: SafePipe,
    cutoff_frequency: f32,
    previous_output: f32,
    uuid: Uuid,
}

impl HighPassFilter {
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

impl Filter for HighPassFilter {
    fn transform(&mut self) {
        let input = self.source.borrow_mut().pop();
        let alpha = 1.0 / (self.cutoff_frequency + 1.0);
        let output = alpha * input + alpha * self.previous_output;
        self.previous_output = output;
        self.sink.borrow_mut().push(output);
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

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
