use std::default::Default;
use log::info;

use super::{segment::Segment, ToneGenerator};

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

#[derive(Debug)]
pub struct Generator {
    envelope: Envelope,       // An envelope for the note amplitude
    pitch_curve: Segment, // An evelope for the note pitch
    tone_generator: Box<dyn ToneGenerator>,
    pub last: (bool, bool, f32), // note on ? - note off ? - last_value
}

impl Generator {
    pub fn new(envelope: Envelope, tone_generator: Box<dyn ToneGenerator>) -> Generator {
        Self {
            envelope,
            pitch_curve: Segment::default(),  // Segment default is a constant segment
            tone_generator,
            last: (false, false, 0.0),
        }
    }

    /// Returns the note value at a point in time, given the note_on, note_off and current time.
    pub fn tick(&mut self, sample: i32, sample_rate: i32, note_on_time: f32, note_off_time: f32) -> f32 {
        let time = sample as f32 / sample_rate as f32;

        let ampl = if note_on_time <= time {
            let on_elapsed = time - note_on_time;
            if note_off_time <= time {
                if !self.last.1 {
                    // The note was just released
                    if on_elapsed < self.envelope.decay.end() {
                        // We haven't finished the decay cycle
                        // println!("Note off before end of decay ! - Last value: {}", self.last.2);
                        self.envelope.release.change_from(self.last.2);
                    }
                }
                let off_elapsed = time - note_off_time;
                if off_elapsed < self.envelope.release.end() {
                    self.envelope.release.at(off_elapsed)
                } else {
                    0.0
                }
            } else if on_elapsed < self.envelope.attack.end() {
                self.envelope.attack.at(on_elapsed)
            } else if on_elapsed < self.envelope.decay.end() {
                self.envelope.decay.at(on_elapsed)
            } else {
                self.envelope.sustain()
            }
        } else {
            0.0
        };

        let warp = if time < note_on_time {
            self.pitch_curve.start_value()
        } else if time < note_off_time {
            self.pitch_curve.at(time - note_on_time)
        } else {
            self.pitch_curve.end_value()
        };

        self.last = (note_on_time >= time, note_off_time >= time, ampl);
        ampl * self.tone_generator.tick(1.0 / sample_rate as f32 * warp)
    }

    pub fn set_tone_generator(&mut self, tone_generator: Box<dyn ToneGenerator>) {
        self.tone_generator = tone_generator;
    }

    pub fn set_pitch_bend(&mut self, pitch_curve: Segment) {
        self.pitch_curve = pitch_curve
    }

    pub fn set_ampl_envelope(&mut self, ampl_envelope: Envelope) {
        self.envelope = ampl_envelope
    }

    pub fn covers(&self, time: f32, note_on_time: f32, note_off_time: f32) -> bool {
        time >= note_on_time && time <= note_off_time + self.envelope.release.end()
    }
}
