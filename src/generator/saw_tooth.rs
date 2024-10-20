use crate::generator::ToneGenerator;

#[derive(Debug)]
pub struct SawTooth {
    period: f32,
    amplitude: f32,
}

impl SawTooth {
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            period: 1.0 / frequency,
            amplitude,
        }
    }
}

impl ToneGenerator for SawTooth {
    fn generate(&self, time: f32) -> f32 {
        self.amplitude * (
            2.0 * (( time / self.period ) - ((1.0 / 2.0) + (time / self.period)).floor())
        )
    }
}
