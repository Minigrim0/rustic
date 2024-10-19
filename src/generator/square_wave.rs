use crate::generator::ToneGenerator;

#[derive(Debug)]
pub struct SquareWave {
    frequency: f64,
    amplitude: f64,
}

impl SquareWave {
    pub fn new(frequency: f64, amplitude: f64) -> Self {
        Self {
            frequency,
            amplitude,
        }
    }
}

impl ToneGenerator for SquareWave {
    fn generate(&self, time: f64) -> f64 {
        self.amplitude * (
            2.0 * ( 2.0 * ( time * self.frequency ).floor() - (2.0 * time * self.frequency).floor()) + 1.0
        )
    }
}
