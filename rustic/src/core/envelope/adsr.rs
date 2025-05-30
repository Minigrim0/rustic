use std::{default::Default, fmt};

use super::{segment::Segment, Envelope};
use crate::core::generator::prelude::SimpleGenerator;
use crate::core::generator::VariableToneGenerator;

/// ADSR Envelope. Consists of four segments;
/// * Attack: The beginning of the envelope, where the amplitude increases from 0 to the maximum value.
/// * Decay: The transition from the maximum amplitude to the sustain level.
/// * Sustain: The constant amplitude during the duration of the note.
/// * Release: The transition from the sustain level to 0 amplitude once the note is released.
///
/// The ADSR envelope contains three segments for the attack, decay and release phases. This
/// allows the envelope to use either linear or exponential interpolation.
#[derive(Debug, Clone)]
pub struct ADSREnvelope {
    pub attack: Segment,
    pub decay: Segment,
    pub release: Segment,
}

impl ADSREnvelope {
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

    /// Creates a simple constant envelope.
    /// The note reaches the maximum amplitude as soon as it is played and
    /// the minimum as soon as it is released.
    pub fn constant() -> Self {
        Self {
            attack: Segment::new(0.0, 1.0, 0.0, 0.0, None),
            decay: Segment::new(1.0, 1.0, 0.0, 0.0, None),
            release: Segment::new(1.0, 0.0, 0.0, 0.0, None),
        }
    }

    /// Creates a generator from a tone generator and the envelope (cloned)
    pub fn attach_amplitude(&self, generator: Box<dyn VariableToneGenerator>) -> SimpleGenerator {
        SimpleGenerator::new(Box::from(self.clone()), generator)
    }

    /// Returns the sustain value of the envelope
    pub fn sustain(&self) -> f32 {
        self.decay.end_value()
    }

    /// Sets the attack segment of the envelope.
    pub fn set_attack(&mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> bool {
        self.attack = Segment::new(0.0, to, duration, 0.0, control);
        true
    }

    /// Sets the decay segment of the envelope.
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

    /// Sets the release section of the envelope.
    pub fn set_release(&mut self, duration: f32, to: f32, control: Option<(f32, f32)>) -> bool {
        self.release = Segment::new(self.decay.end_value(), to, duration, 0.0, control);
        true
    }
}

impl fmt::Display for ADSREnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ADSR: Attack: {}, Decay: {}, Sustain: {}, Release: {}",
            self.attack,
            self.decay,
            self.sustain(),
            self.release
        )
    }
}

impl Envelope for ADSREnvelope {
    fn at(&self, time: f32, note_off: f32) -> f32 {
        if self.attack.covers(time) {
            self.attack.at(time)
        } else if self.decay.covers(time) {
            self.decay.at(time)
        } else {
            if note_off > 0.0 {
                if self.release.covers(time - note_off) {
                    self.release.at(time - note_off)
                } else {
                    0.0
                }
            } else {
                self.decay.end_value()
            }
        }
    }

    fn completed(&self, time: f32, note_off: f32) -> bool {
        note_off > 0.0 && time - note_off > self.release.end()
    }
}
