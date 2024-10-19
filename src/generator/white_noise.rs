use crate::generator::ToneGenerator;
use rand;

#[derive(Debug)]
pub struct WhiteNoise {
    amplitude: f64,
}

impl WhiteNoise {
    pub fn new(amplitude: f64) -> Self {
        Self {
            amplitude,
        }
    }
}

impl ToneGenerator for WhiteNoise {
    fn generate(&self, _time: f64) -> f64 {
        self.amplitude * (rand::random::<f64>() * 2.0 - 1.0)
    }
}
