use crate::core::graph::{AudioGraphElement, Entry, Filter};

use log::trace;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// A filter that returns the input value multiplied by a constant factor.
/// Note: a factor < 1.0 will attenuate the input signal, while a factor > 1.0
/// will amplify it.
#[cfg_attr(feature = "meta2", derive(derive::MetaData))]
pub struct GainFilter {
    sources: [f32; 1],
    factor: f32,
    index: usize,
}

impl GainFilter {
    pub fn new(factor: f32) -> Self {
        Self {
            sources: [0.0],
            factor,
            index: 0,
        }
    }
}

impl Entry for GainFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }
}

impl Filter for GainFilter {
    /// Transforms the input value by multiplying it by the factor and sends it to the sink.
    /// If multiple sources are connected to the filter, the output will be the sum of all
    /// the sources multiplied by the factor.
    fn transform(&mut self) -> Vec<f32> {
        let output: f32 = self.sources.map(|f| f * self.factor).iter().sum();
        trace!("Gain filter running {} -> {}", self.sources[0], output);
        vec![output]
    }
}

impl AudioGraphElement for GainFilter {
    fn get_name(&self) -> &str {
        "Gain Filter"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
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
