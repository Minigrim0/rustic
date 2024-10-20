use crate::generator::ToneGenerator;

#[derive(Debug)]
pub struct SquareWave {
    frequency: f32,
    amplitude: f32,
}

impl SquareWave {
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            frequency,
            amplitude,
        }
    }
}

impl ToneGenerator for SquareWave {
    fn generate(&self, time: f32) -> f32 {
        self.amplitude * (
            2.0 * ( 2.0 * ( time * self.frequency ).floor() - (2.0 * time * self.frequency).floor()) + 1.0
        )
    }
}
