use crate::generator::ToneGenerator;

#[derive(Debug)]
pub struct SawTooth {
    period: f64,
    amplitude: f64,
}

impl SawTooth {
    pub fn new(frequency: f64, amplitude: f64) -> Self {
        Self {
            period: 1.0 / frequency,
            amplitude,
        }
    }
}

impl ToneGenerator for SawTooth {
    fn generate(&self, time: f64) -> f64 {
        2.0 * (( time / self.period ) - ((1.0 / 2.0) + (time / self.period)).floor())
    }
}
