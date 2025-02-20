use crate::core::graph::{AudioGraphElement, Entry, Sink};

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Low-pass filter using a first-order IIR filter
pub struct AudioSink {
    values: Vec<f32>,
    index: usize,
}

impl AudioSink {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            index: 0,
        }
    }
}

impl Entry for AudioSink {
    fn push(&mut self, value: f32, _port: usize) {
        // trace!("{}", value);
        self.values.push(value);
    }
}

impl Sink for AudioSink {
    fn get_values(&self) -> Vec<f32> {
        self.values.clone()
    }

    fn as_entry(self) -> Box<dyn Entry> {
        Box::new(self)
    }
}

impl AudioGraphElement for AudioSink {
    fn get_name(&self) -> &str {
        "Audio Sink"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}
