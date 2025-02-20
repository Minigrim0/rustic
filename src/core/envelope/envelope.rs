use std::default::Default;

use super::segment::Segment;
use crate::core::generator::{Generator, ToneGenerator};

// ADSR
// This is a simple ADSR envelope generator that can be used to control the amplitude of a sound over time.
#[derive(Debug, Clone)]
pub struct Envelope {
    pub attack: Segment,
    pub decay: Segment,
    pub release: Segment,
}

impl Envelope {
    /// Creates a new Envelope with
    pub fn new() -> Self {
        Self {
            attack: Segment::default(),
            decay: Segment::default(),
            release: Segment::default(),
        }
    }

    pub fn with_attack(mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> Self {
        self.set_attack(duration, to, control);
        self
    }

    pub fn with_decay(mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> Self {
        self.set_decay(duration, to, control);
        self
    }

    pub fn with_release(mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> Self {
        self.set_release(duration, to, control);
        self
    }

    /// A simple envelope, where the note is a maximum as soon as it is played and
    /// at minimum as soon as it is released
    pub fn constant() -> Self {
        Self {
            attack: Segment::new(0.0, 1.0, 0.0, 0.0, None),
            decay: Segment::new(1.0, 1.0, 0.0, 0.0, None),
            release: Segment::new(1.0, 0.0, 0.0, 0.0, None),
        }
    }

    pub fn attach(&self, generator: Box<dyn ToneGenerator>) -> Generator {
        Generator::new(self.clone(), generator)
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
}
