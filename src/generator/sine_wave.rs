use crate::generator::ToneGenerator;
use std::f32::consts::PI;

#[derive(Debug)]
pub struct SineWave {
    frequency: f32,
    amplitude: f32,
}

impl SineWave {
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            frequency,
            amplitude,
        }
    }
}

impl ToneGenerator for SineWave {
    fn generate(&self, time: f32) -> f32 {
        self.amplitude * (2.0 * PI * self.frequency * time).sin()
    }
}
