use std::default::Default;

use super::{segment::Segment, ToneGenerator};

// ADSR
// This is a simple ADSR envelope generator that can be used to control the amplitude of a sound over time.
pub struct Envelope {
    pub attack: Segment,
    pub decay: Segment,
    pub release: Segment,
    sample_rate: f32,
    generator: Box<dyn ToneGenerator>,
}

impl Envelope {
    /// Creates a new Envelope with
    pub fn new(sample_rate: f32, generator: Box<dyn ToneGenerator>) -> Self {
        Self {
            attack: Segment::default(),
            decay: Segment::default(),
            release: Segment::default(),
            sample_rate,
            generator,
        }
    }

    pub fn sustain(&self) -> f32 {
        self.decay.end_value()
    }

    pub fn set_attack(&mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> bool {
        self.attack = Segment::new(0.0, to, duration, 0.0, control);
        true
    }

    pub fn set_decay(&mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> bool {
        // TODO: Check decay is correct
        self.decay = Segment::new(
            self.attack.end_value(),
            to,
            duration,
            self.attack.end(),
            control,
        );
        true
    }

    // For release, elapsed is from 0 to release duration
    pub fn set_release(&mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> bool {
        // TODO: Check release is correct
        self.release = Segment::new(self.decay.end_value(), to, duration, 0.0, control);
        true
    }

    /// Returns the note value at a point in time, given the note_on, note_off and current time.
    pub fn get_at(&self, time: f32, note_on_time: Option<f32>, note_off_time: Option<f32>) -> f32 {
        let current = self.generator.generate(time);

        let amplitude = if let Some(off_time) = note_off_time {
            // Note is off
            let elapsed = time - off_time;
            if elapsed < self.release.end() {
                // During release
                self.release.at(elapsed) * current
            } else {
                // After release is done
                0.0
            }
        } else if let Some(on_time) = note_on_time {
            // Note is on
            let elapsed = time - on_time;

            if elapsed < self.attack.end() {
                // During attack
                self.attack.at(elapsed) * current
            } else if elapsed < self.decay.end() {
                // During decay
                self.decay.at(elapsed) * current
            } else {
                // During sustain
                self.sustain() * current
            }
        } else {
            0.0
        };

        amplitude
    }
}
