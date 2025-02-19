use crate::core::graph::{AudioGraphElement, Entry, Filter};

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Low-pass filter using a first-order IIR filter
pub struct LowPassFilter {
    sources: [f32; 1],
    cutoff_frequency: f32,
    previous_output: f32,
    index: usize,
}

impl LowPassFilter {
    pub fn new(cutoff_frequency: f32) -> Self {
        Self {
            sources: [0.0],
            cutoff_frequency,
            previous_output: 0.0,
            index: 0,
        }
    }
}

impl Entry for LowPassFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }
}

impl Filter for LowPassFilter {
    fn transform(&mut self) -> Vec<f32> {
        let input = self.sources[0];
        let alpha = self.cutoff_frequency / (self.cutoff_frequency + 1.0);
        let output = alpha * input + (1.0 - alpha) * self.previous_output;
        self.previous_output = output;
        vec![output]
    }
}

impl AudioGraphElement for LowPassFilter {
    fn get_name(&self) -> &str {
        "Low Pass Filter"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
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
