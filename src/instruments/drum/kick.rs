use crate::core::envelope::prelude::ADSREnvelope;
use crate::core::envelope::prelude::BezierEnvelope;
use crate::core::envelope::Envelope;
use crate::core::generator::prelude::SineWave;
use crate::core::generator::BendableGenerator;
use crate::instruments::Instrument;
use crate::Note;

pub struct Kick {
    generator: Box<dyn BendableGenerator>,
    amplitude_envelope: Box<dyn Envelope>,
    pitch_curve: Box<dyn Envelope>,
    current_tick: u32,
    playing: bool,
    output: f32,
}

impl Kick {
    pub fn new() -> Self {
        Self {
            generator: Box::from(SineWave::new(120.0, 0.3)),
            amplitude_envelope: {
                let mut env = ADSREnvelope::new();
                env.set_attack(0.04, 1.0, Some((0.0, 1.0)));
                env.set_decay(0.0, 1.0, None);
                env.set_release(0.26, 0.0, Some((0.0, 0.0)));
                Box::from(env)
            },
            pitch_curve: Box::from(BezierEnvelope::new(1.0, 0.5, 0.3, (2.0, 0.2))),
            current_tick: 0,
            playing: false,
            output: 0.0,
        }
    }
}

impl Instrument for Kick {
    fn start_note(&mut self, _note: Note, _velocity: f32) {
        self.current_tick = 0;
        self.playing = true;
    }

    fn stop_note(&mut self, _note: crate::Note) {
        // The note will continue playing until completed
        self.playing = false;
    }

    fn get_output(&mut self) -> f32 {
        self.output
    }

    fn tick(&mut self) {
        self.current_tick += 1;
        let current_time: f32 = self.current_tick as f32 / 44100.0;
        let current_pitch = self.pitch_curve.at(current_time);
        self.generator.set_pitch_bend(current_pitch);
        self.output = self.generator.tick(1.0 / 44100.0) * self.amplitude_envelope.at(current_time);
    }
}
