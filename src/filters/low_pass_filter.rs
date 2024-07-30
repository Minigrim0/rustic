use crate::filter::Filter;

pub struct LowPassFilter {
    cutoff_frequency: f32,
    previous_output: f32,
}

impl LowPassFilter {
    pub fn new(cutoff_frequency: f32) -> Self {
        Self {
            cutoff_frequency,
            previous_output: 0.0,
        }
    }
}

impl Filter for LowPassFilter {
    fn apply(&mut self, input: f32) -> f32 {
        let alpha = self.cutoff_frequency / (self.cutoff_frequency + 1.0);
        let output = alpha * input + (1.0 - alpha) * self.previous_output;
        self.previous_output = output;
        output
    }
}