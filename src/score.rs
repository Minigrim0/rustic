use std::f64::consts::PI;

use crate::generator::white_noise::WhiteNoise;
use crate::generator::ToneGenerator;
use crate::generator::sine_wave::SineWave;
use crate::generator::saw_tooth::SawTooth;
use crate::generator::square_wave::SquareWave;

// Purpose: Contains the Score struct, which is used to store a sequence of notes to be played.
#[derive(Debug)]
pub struct Note {
    pub frequency: f64,
    pub start_time: f64,
    pub duration: f64,
    pub generator: Box<dyn ToneGenerator>,
}

pub enum GENERATORS {
    SINE,
    SAW,
    SQUARE,
    NOISE
}

impl Note {
    // Generate a note with a Sine generator
    pub fn new(frequency: f64, start_time: f64, duration: f64) -> Self {
        Self { frequency, start_time, duration, generator: Box::new(SawTooth::new(frequency, 1.0)) }
    }

    pub fn with_generator(mut self, generator: GENERATORS) -> Self {
        match generator {
            GENERATORS::SAW => {
                self.generator = Box::from(SawTooth::new(self.frequency, 1.0));
                self
            },
            GENERATORS::SINE => {
                self.generator = Box::from(SineWave::new(self.frequency, 1.0));
                self
            },
            GENERATORS::SQUARE => {
                self.generator = Box::from(SquareWave::new(self.frequency, 1.0));
                self
            },
            GENERATORS::NOISE => {
                self.generator = Box::from(WhiteNoise::new(1.0));
                self
            }
        }
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
