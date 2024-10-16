use std::default::Default;

use super::{segment::Segment, ToneGenerator};

// ADSR
// This is a simple ADSR envelope generator that can be used to control the amplitude of a sound over time.
pub struct Envelope {
    attack: Segment,
    decay: Segment,
    sustain: f32,
    release: Segment,
    sample_rate: f32,
    generator: Box<dyn ToneGenerator>,
}

impl Envelope {
    /// Creates a new Envelope with
    pub fn new(sample_rate: f32, generator: Box<dyn ToneGenerator>) -> Self {
        Self {
            attack: Segment::default(),
            decay: Segment::default(),
            sustain: 1.0,
            release: Segment::default(),
            sample_rate,
            generator,
        }
    }

    pub fn set_attack(&mut self, attack: Segment) -> bool {
        // TODO: Check attack is correct
        self.attack = attack;
        true
    }

    pub fn set_decay(&mut self, decay: Segment) -> bool {
        // TODO: Check decay is correct
        self.decay = decay;
        true
    }

    pub fn set_sustain(&mut self, sustain: f32) -> bool {
        // TODO: Check sustain is correct
        self.sustain = sustain;
        true
    }

    pub fn set_release(&mut self, release: Segment) -> bool {
        // TODO: Check release is correct
        self.release = release;
        true
    }

    /// Returns the note value at a point in time, given the note_on, note_off and current time.
    pub fn get_at(&self, time: f32, note_on_time: f32, note_off_time: Option<f32>) -> f32 {
        let amplitude = if let Some(off_time) = note_off_time {
            // Note is off
            let elapsed = time - off_time;
            if elapsed < self.release.end() {
                // During release
                self.release.at(elapsed)
            } else {
                // After release is done
                0.0
            }
        } else {
            // Note is on
            let elapsed = time - note_on_time;

            if elapsed < self.attack.end() {
                // During attack
                self.attack.at(elapsed)
            } else if elapsed < self.attack.end() + self.decay.end() {
                // During decay
                self.decay.at(elapsed)
            } else {
                // During sustain
                self.sustain
            }
        };

        amplitude.max(0.0)
    }
}
