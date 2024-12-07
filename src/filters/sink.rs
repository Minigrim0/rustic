use super::{AudioGraphElement, Sink};
use uuid::Uuid;

use log::trace;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Low-pass filter using a first-order IIR filter
pub struct AudioSink {
    values: Vec<f32>,
}

impl AudioSink {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }
}

impl Sink for AudioSink {
    fn get_values(&mut self) -> &mut Vec<f32> {
        &mut self.values
    }

    fn push(&mut self, value: f32, _port: usize) {
        // trace!("{}", value);
        self.values.push(value);
    }
}

impl AudioGraphElement for AudioSink {
    fn get_name(&self) -> &str {
        "Audio Sink"
    }

    fn uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}
