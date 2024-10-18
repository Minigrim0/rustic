use crate::generator::ToneGenerator;
use std::f64::consts::PI;

#[derive(Debug)]
pub struct SineWave {
    frequency: f64,
    amplitude: f64,
}

impl SineWave {
    pub fn new(frequency: f64, amplitude: f64) -> Self {
        Self {
            frequency,
            amplitude,
        }
    }
}

impl ToneGenerator for SineWave {
    fn generate(&self, time: f64) -> f64 {
        self.amplitude * (2.0 * PI * self.frequency * time).sin()
    }
}
