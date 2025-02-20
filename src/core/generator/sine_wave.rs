use super::ToneGenerator;
use std::f32::consts::PI;

#[derive(Debug)]
pub struct SineWave {
    frequency: f32,
    amplitude: f32,
    timer: f32,
}

impl SineWave {
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            frequency,
            amplitude,
            timer: 0.0,
        }
    }
}

impl ToneGenerator for SineWave {
    fn tick(&mut self, elapsed_time: f32) -> f32 {
        self.timer += elapsed_time;
        self.amplitude * (2.0 * PI * self.frequency * self.timer).sin()
    }
}
