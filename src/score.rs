use std::cmp::Ordering;

use crate::generator::saw_tooth::SawTooth;
use crate::generator::sine_wave::SineWave;
use crate::generator::square_wave::SquareWave;
use crate::generator::white_noise::WhiteNoise;
use crate::generator::GENERATORS;
use crate::generator::{Envelope, Generator, ToneGenerator};

/// Represents a musical note that can be part of a score. It has an associated generator,
/// that can generate the tone of the note in any of the `GENERATORS` shapes.
///
/// # Fields
///
/// * `frequency` - The frequency of the note in Hertz (Hz).
/// * `start_time` - The start time of the note in seconds.
/// * `duration` - The duration of the note in seconds.
/// * `generator` - A boxed trait object that generates the tone for the note.
#[derive(Debug)]
pub struct Note {
    pub frequency: f32,
    pub start_time: f32,
    pub duration: f32,
    pub generator: Box<Generator>,
}

impl Eq for Note {}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time && self.frequency == other.frequency
    }
}

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.start_time.partial_cmp(&self.start_time)
    }
}

impl Ord for Note {
    fn cmp(&self, other: &Self) -> Ordering {
        other.partial_cmp(&self).unwrap_or(Ordering::Equal)
    }
}

impl Note {
    /// Creates a new note instance with the given parameters. The default generator is a Sine generator.
    pub fn new(frequency: f32, start_time: f32, duration: f32) -> Self {
        let env: Envelope = Envelope::constant();
        let tone_generator: Box<dyn ToneGenerator> = Box::from(SineWave::new(frequency, 1.0));

        Self {
            frequency,
            start_time,
            duration,
            generator: Box::from(env.attach(tone_generator)),
        }
    }

    /// Sets the tone generator for the note and returns the modified note.
    ///
    /// This method allows you to specify the type of tone generator to be used for the note.
    /// It takes a `GENERATORS` enum value and sets the corresponding generator.
    ///
    /// # Arguments
    ///
    /// * `generator` - The type of tone generator to be used for the note.
    ///
    /// # Returns
    ///
    /// The modified `Note` instance with the specified tone generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustic::score::Note;
    /// use rustic::generator::GENERATORS;
    ///
    /// let note = Note::new(440.0, 0.0, 1.0)
    ///     .with_generator(GENERATORS::SINE);
    /// ```
    pub fn with_generator(mut self, generator: GENERATORS) -> Self {
        let new_tone_generator: Box<dyn ToneGenerator> = match generator {
            GENERATORS::SAW => Box::from(SawTooth::new(self.frequency, 1.0)),
            GENERATORS::SINE => Box::from(SineWave::new(self.frequency, 1.0)),
            GENERATORS::SQUARE => Box::from(SquareWave::new(self.frequency, 1.0)),
            GENERATORS::NOISE => Box::from(WhiteNoise::new(1.0)),
        };

        self.generator.set_tone_generator(new_tone_generator);
        self
    }

    /// Sets the envelope for the note and returns the modified note
    ///
    /// This method allows you to attach an envelope to the note. This envelope will
    /// affect the amplitude of the note (the volume).
    ///
    /// # Arguments
    ///
    /// * `envelope` - A reference to the envelope to attach to the note. The instance
    /// that this reference points to is cloned then attached to the note.
    ///
    /// # Returns
    ///
    /// The modified `Note` instance with the specified envelope.
    ///
    /// # Example
    ///
    /// ```
    /// use rustic::score::Note;
    /// use rustic::generator::Envelope;
    ///
    /// let note = Note::new(440.0, 0.0, 1.0)
    ///     .with_envelope(&Envelope::constant());
    /// ```
    pub fn with_envelope(mut self, envelope: &Envelope) -> Self {
        self.generator.set_ampl_envelope(envelope.clone());
        self
    }

    pub fn get_at(&mut self, time: f32) -> f32 {
        self.generator
            .get_at(time, self.start_time, self.start_time + self.duration)
    }

    /// Returns true when to note is completed (amplitude envelope release has finished)
    /// This depends on the `off_time` of the note.
    pub fn is_completed(&self, time: f32) -> bool {
        !self
            .generator
            .covers(time, self.start_time, self.start_time + self.duration)
            && time > self.start_time
    }
}

pub struct Score {
    pub notes: Vec<Note>,
}

// impl Score {
//     pub fn new(notes: Vec<Note>) -> Self {
//         Self { notes }
//     }

//     pub fn play(&self, player: &mut Player) {
//         let mut time = 0.0;

//         for note in &self.notes {
//             time = note.start_time;
//             player.sound_system.generator.generate(time);
//             player.play(note.duration);
//         }
//     }
// }
