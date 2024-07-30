use std::f32::consts::PI;
use crate::tone_generator::ToneGenerator;
use crate::adsr::ADSR;

pub struct SineWave {
    frequency: f32,
    amplitude: f32,
    adsr: ADSR,
    note_on_time: f32,
    note_off_time: f32,
}

impl SineWave {
    pub fn new(frequency: f32, amplitude: f32, adsr: ADSR) -> Self {
        Self {
            frequency,
            amplitude,
            adsr,
            note_on_time: 0.0,
            note_off_time: -1.0, // Indicates note is not released yet
        }
    }

    pub fn note_on(&mut self, time: f32) {
        self.note_on_time = time;
        self.note_off_time = -1.0;
    }

    pub fn note_off(&mut self, time: f32) {
        self.note_off_time = time;
    }
}

impl ToneGenerator for SineWave {
    fn generate(&self, time: f32) -> f32 {
        let adsr_amplitude = self.adsr.get_amplitude(time, self.note_on_time, self.note_off_time);
        adsr_amplitude * self.amplitude * (2.0 * PI * self.frequency * time).sin()
    }
}