use crate::generator::ToneGenerator;

#[derive(Debug)]
pub struct SquareWave {
    frequency: f32,
    amplitude: f32,
    timer: f32,
}

impl SquareWave {
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            frequency,
            amplitude,
            timer: 0.0,
        }
    }
}

impl ToneGenerator for SquareWave {
    fn tick(&mut self, elapsed_time: f32) -> f32 {
        self.timer += elapsed_time;

        self.amplitude * (
            2.0 * ( 2.0 * ( self.timer * self.frequency ).floor() - (2.0 * self.timer * self.frequency).floor()) + 1.0
        )
    }
}
