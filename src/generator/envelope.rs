use std::default::Default;

use super::{segment::Segment, ToneGenerator};

// ADSR
// This is a simple ADSR envelope generator that can be used to control the amplitude of a sound over time.
#[derive(Debug, Clone)]
pub struct Envelope {
    pub attack: Segment,
    pub decay: Segment,
    pub release: Segment,
    sample_rate: f64,
}

impl Envelope {
    /// Creates a new Envelope with
    pub fn new(sample_rate: f64) -> Self {
        Self {
            attack: Segment::default(),
            decay: Segment::default(),
            release: Segment::default(),
            sample_rate,
        }
    }

    pub fn attach<'a>(&self, generator: &'a Box<dyn ToneGenerator>) -> Generator<'a> {
        Generator::new(self.clone(), generator)
    }

    pub fn sustain(&self) -> f64 {
        self.decay.end_value()
    }

    pub fn set_attack(&mut self, duration: f64, to: f64, control: Option<(f64, f64)>) -> bool {
        self.attack = Segment::new(0.0, to, duration, 0.0, control);
        true
    }

    pub fn set_decay(&mut self, duration: f64, to: f64, control: Option<(f64, f64)>) -> bool {
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
    pub fn set_release(&mut self, duration: f64, to: f64, control: Option<(f64, f64)>) -> bool {
        // TODO: Check release is correct
        self.release = Segment::new(self.decay.end_value(), to, duration, 0.0, control);
        true
    }
}

pub struct Generator<'a> {
    envelope: Envelope,
    generator: &'a Box<dyn ToneGenerator>,
    last: (bool, bool, f64)  // note on ? - note off ? - last_value
}

impl<'a> Generator<'a> {
    pub fn new(envelope: Envelope, generator: &'a Box<dyn ToneGenerator>) -> Generator<'a> {
        Self {
            envelope,
            generator,
            last: (false, false, 0.0),
        }
    }

    /// Returns the note value at a point in time, given the note_on, note_off and current time.
    pub fn get_at(&mut self, time: f64, note_on_time: Option<f64>, note_off_time: Option<f64>) -> (u8, f64) {
        let ampl = match note_on_time {
            Some(on_time) => {
                let on_elapsed = time - on_time;
                match note_off_time {
                    Some(off_time) => {
                        if !self.last.1 { // The note was just released
                            if on_elapsed < self.envelope.decay.end() {  // We haven't finished the decay cycle
                                println!("Note off before end of decay ! - Last value: {}", self.last.2);
                                self.envelope.release.change_from(self.last.2);
                            }
                        }
                        let off_elapsed = time - off_time;
                        if off_elapsed < self.envelope.release.end() {
                            (4, self.envelope.release.at(off_elapsed))
                        } else {
                            (0, 0.0)
                        }
                    },
                    None if on_elapsed < self.envelope.attack.end() => (1, self.envelope.attack.at(on_elapsed)),
                    None if on_elapsed < self.envelope.decay.end() => (2, self.envelope.decay.at(on_elapsed)),
                    None => (3, self.envelope.sustain())
                }
            },
            None => (0, 0.0)
        };

        self.last = (note_on_time.is_some(), note_off_time.is_some(), ampl.1);
        (ampl.0,  ampl.1 * self.generator.generate(time))

        // // After decay
        // let amplitude = if note_on_time.is_none() {
        //     // No note yet
        //     print!("NNY - ");
        //     0.0
        // } else if let Some(on_time) = note_on_time {
        //     print!("BDY - ");
        //     // Note is on
        //     let elapsed = time - on_time;

        //     if elapsed < self.envelope.attack.end() {
        //         // During attack
        //         self.envelope.attack.at(elapsed) * current
        //     } else if elapsed < self.envelope.decay.end() {
        //         // During decay
        //         self.envelope.decay.at(elapsed) * current
        //     } else {
        //         // Shouldn't happen
        //         0.0
        //     } } else if time > self.envelope.decay.end() {
        //     // Note is off
        //     if let Some(off_time) = note_off_time {
        //         print!("ADN - ");
        //         let elapsed = time - off_time;

        //         if elapsed < self.envelope.release.end() {
        //             // During release
        //             self.envelope.release.at(elapsed) * current
        //         } else {
        //             // After release is done
        //             0.0
        //         }
        //     // Note is still on, sustain
        //     } else {
        //         print!("ADY - ");
        //         self.envelope.sustain() * current
        //     }
        // // Before decay & note is on
        // }
        // // Before decay no note
        // } else {
        //     print!("BDN - ");
        //     0.0
        // };

        // amplitude
    }
}
