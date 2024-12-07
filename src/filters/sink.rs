use super::{Filter, SafeFilter};
use uuid::Uuid;

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Low-pass filter using a first-order IIR filter
pub struct Sink {
    values: Vec<f32>,
}

impl Sink {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    pub fn get_values(&mut self) -> &mut Vec<f32> {
        &mut self.values
    }
}

impl Filter for Sink {
    fn push(&mut self, value: f32, _port: usize) {
        self.values.push(value);
    }

    fn add_sink(&mut self, _out_port: usize, _sink: SafeFilter, _in_port: usize) {
        /* Do nothing */
    }

    fn transform(&mut self) {
        /* Do nothing */
    }

    fn get_name(&self) -> &str {
        "Sink"
    }

    fn uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}
