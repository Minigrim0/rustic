use std::cmp::Ordering;
use std::collections::BinaryHeap;

use log::info;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use crate::generator::saw_tooth::SawTooth;
use crate::generator::sine_wave::SineWave;
use crate::generator::square_wave::SquareWave;
use crate::generator::white_noise::WhiteNoise;
use crate::generator::{Segment, GENERATORS};
use crate::generator::{Envelope, Generator, ToneGenerator};

#[cfg(feature = "plotting")]
use crate::plotting::plot_data;

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

    /// Sets the pitch bend for the note and returns the modified note
    /// This method allows you to attach a pitch bend to the note. This pitch bend will
    /// affect the frequency of the note.
    /// # Arguments
    /// * `pitch_bend` - A reference to the pitch bend to attach to the note. The instance
    /// that this reference points to is cloned then attached to the note.
    /// # Returns
    /// The modified `Note` instance with the specified pitch bend.
    /// # Example
    /// ```
    /// use rustic::score::Note;
    /// use rustic::generator::Segment;
    ///
    /// let note = Note::new(440.0, 0.0, 1.0)
    ///    .with_pitch_bend(&Segment::new(0.0, 1.0, 440.0, 880.0));
    /// ```
    pub fn with_pitch_bend(mut self, pitch_bend: &Segment) -> Self {
        self.generator.set_pitch_bend(pitch_bend.clone());
        self
    }

    pub fn tick(&mut self, sample: i32, sample_rate: i32) -> f32 {
        self.generator
            .tick(sample, sample_rate, self.start_time, self.start_time + self.duration)
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
    notes: BinaryHeap<Note>,
    playing: bool,
    current_sample: i32,
    sample_rate: i32,
    name: String
}

impl Score {
    pub fn new(name: String, sample_rate: i32) -> Self {
        Self {
            notes: BinaryHeap::new(),
            playing: false,
            current_sample: 0,
            sample_rate,
            name
        }
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn play(&mut self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let mut current_notes = vec![];
        let mut vals = vec![];

        #[cfg(feature = "plotting")]
        let mut plotting_vals = vec![];

        'builder: loop {
            self.current_sample += 1;
            let current_time = self.current_sample as f32 / self.sample_rate as f32;

            // Find currently playing notes in the binary heap and push themm to the current_notes vector
            'fetcher: loop {
                if let Some(note) = self.notes.peek() {
                    if note.start_time <= current_time {
                        current_notes.push(self.notes.pop().unwrap());
                    } else {
                        break 'fetcher;
                    }
                }
                break 'fetcher;
            }

            // Push to sink every second
            if vals.len() == self.sample_rate as usize {
                sink.append(SamplesBuffer::new(
                    1 as u16,
                    self.sample_rate as u32,
                    vals.iter().map(|(_, n)| *n).collect::<Vec<f32>>(),
                ));
                #[cfg(feature = "plotting")]
                plotting_vals.append(&mut vals);
            }

            current_notes.retain(|n| !n.is_completed(current_time));

            // If there are no notes to play, break the loop
            if current_notes.is_empty() {
                if self.notes.is_empty() {
                    sink.append(SamplesBuffer::new(
                        1 as u16,
                        self.sample_rate as u32,
                        vals.iter().map(|(_, n)| *n).collect::<Vec<f32>>(),
                    ));

                    #[cfg(feature = "plotting")]
                    plotting_vals.append(&mut vals);

                    info!(
                        "Event queue is empty ! Breaking the loop - {}",
                        current_time
                    );

                    break 'builder;
                }

                vals.push((current_time, 0.0));
                continue;
            }

            // Generate the current sample
            vals.push((
                current_time,
                current_notes
                    .iter_mut()
                    .map(|note| note.tick(self.current_sample, self.sample_rate as i32))
                    .sum(),
            ));
        }

        #[cfg(feature = "plotting")]
        {
            let duration = plotting_vals.len() as f32 / self.sample_rate as f32 + 0.1;
            if let Err(e) = plot_data(
                plotting_vals,
                format!("Score {} (SR={}Hz)", self.name, self.sample_rate).as_str(),
                (0.0, duration),
                (-1.1, 1.1),
                format!("score_{}_{}.png", self.name, self.sample_rate).as_str(),
            ) {
                println!("Error: {}", e.to_string());
            }
        }

        sink.sleep_until_end();
    }
}
