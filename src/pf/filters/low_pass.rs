/// Low-pass filter using a first-order IIR filter
use super::Filter;
use crate::pf::pipe::SafePipe;

pub struct LowPassFilter {
    source: SafePipe,
    sink: SafePipe,
    cutoff_frequency: f32,
    previous_output: f32,
}

impl LowPassFilter {
    pub fn new(source: SafePipe, sink: SafePipe, cutoff_frequency: f32) -> Self {
        Self {
            source,
            sink,
            cutoff_frequency,
            previous_output: 0.0,
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
}