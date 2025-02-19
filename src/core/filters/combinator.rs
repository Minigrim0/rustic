use crate::core::graph::{AudioGraphElement, Entry, Filter};

use log::{error, trace};

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// A filter that take input from two sources and combines them into a single
/// output by adding them together.
pub struct CombinatorFilter<const INPUTS: usize, const OUTPUTS: usize> {
    sources: [f32; INPUTS],
    weights: [f32; INPUTS],
    index: usize,
}

impl<const INPUTS: usize, const OUTPUTS: usize> CombinatorFilter<INPUTS, OUTPUTS> {
    pub fn new() -> Self {
        Self {
            sources: [0.0; INPUTS],
            weights: [1.0; INPUTS],
            index: 0,
        }
    }
}

impl<const INPUTS: usize, const OUTPUTS: usize> Entry for CombinatorFilter<INPUTS, OUTPUTS> {
    fn push(&mut self, value: f32, port: usize) {
        if port >= INPUTS {
            error!("Port {} out of bounds for CombinatorFilter", port);
        }
        self.sources[port] = value;
    }
}

impl<const INPUTS: usize, const OUTPUTS: usize> Filter for CombinatorFilter<INPUTS, OUTPUTS> {
    fn transform(&mut self) -> Vec<f32> {
        let output = self
            .sources
            .iter()
            .zip(self.weights)
            .map(|(source, weight)| source * weight)
            .sum();

        trace!(
            "Combinator filter running [{}, {}] -> {}",
            self.sources[0],
            self.sources[1],
            output
        );

        Vec::from([output; OUTPUTS])
    }
}

impl<const INPUTS: usize, const OUTPUTS: usize> AudioGraphElement
    for CombinatorFilter<INPUTS, OUTPUTS>
{
    fn get_name(&self) -> &str {
        "Combinator"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

#[cfg(feature = "meta")]
impl Metadata for CombinatorFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "CombinatorFilter".to_string(),
            description: "Combines multiple inputs by adding them together".to_string(),
            inputs: 2,
            outputs: 1,
        }
    }
}
