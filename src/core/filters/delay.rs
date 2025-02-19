use std::collections::VecDeque;

use log::trace;

use crate::core::graph::{AudioGraphElement, Entry, Filter};

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Delays it input for x samples
pub struct DelayFilter {
    sources: [f32; 1],
    delay_for: usize,
    buffer: VecDeque<f32>,
    index: usize,
}

impl DelayFilter {
    pub fn new(delay: usize) -> Self {
        Self {
            sources: [0.0],
            delay_for: delay,
            buffer: VecDeque::from(vec![0.0; delay]),
            index: 0,
        }
    }
}

impl Entry for DelayFilter {
    fn push(&mut self, value: f32, port: usize) {
        self.sources[port] = value;
    }
}

impl Filter for DelayFilter {
    fn transform(&mut self) -> Vec<f32> {
        let input = self.sources[0];
        let output = self.buffer.pop_front().unwrap_or(0.0);

        trace!("Delay filter running {} -> {}", self.sources[0], output);

        self.buffer.push_back(input);
        vec![output]
    }
}

impl AudioGraphElement for DelayFilter {
    fn get_name(&self) -> &str {
        "Delay Filter"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
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
