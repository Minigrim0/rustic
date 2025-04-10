use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::notes::Note;
use super::staff::Staff;
use crate::instruments::Instrument;

/// A simple time signature denoted with its numerator and denominator.
/// ```rust
/// use rustic::prelude::score::TimeSignature;
///
/// // A simple 4/4 time signature
/// let time_signature = TimeSignature(4, 4);
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct TimeSignature(pub usize, pub usize);

impl Default for TimeSignature {
    fn default() -> Self {
        Self(4, 4)
    }
}

impl TimeSignature {
    pub const C: TimeSignature = TimeSignature(4, 4);
}

/// A music score. Has a defined time signature, tempo,
/// staves associated with their instruments, name, ...
#[derive(Serialize, Deserialize)]
pub struct Score {
    pub name: String,             // Name of the score
    pub signature: TimeSignature, // Time signature of the score
    pub tempo: usize,             // Tempo in bpm
    pub staves: Vec<Staff>,       // Staves of the score, contains the instruments
    #[serde(skip)]
    pub instruments: Vec<Box<dyn Instrument>>,
}

impl Score {
    pub fn new<S: AsRef<str>>(name: S, signature: TimeSignature, tempo: usize) -> Self {
        Self {
            name: name.as_ref().to_string(),
            signature,
            tempo,
            staves: Vec::new(),
            instruments: Vec::new(),
        }
    }

    /// Loads a score from a toml file. Returns either a score or an error message.
    pub fn load_toml(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let score: Self =
            toml::from_str(&content).map_err(|e| format!("Failed to parse toml: {}", e))?;
        Ok(score)
    }

    /// Dumps the score to a toml string.
    pub fn dump_toml(&self) -> Result<String, String> {
        toml::to_string(self).map_err(|e| format!("Failed to dump toml: {}", e))
    }

    /// Saves the score to a toml file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = self.dump_toml()?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Adds an instrument to the score and returns its index.
    /// The instrument's index is also added to the corresponding staff.
    /// If the number of staves is less than the number of instruments, a new staff is created.
    /// ```rust
    /// use rustic::prelude::score::{Score, TimeSignature};
    /// use rustic::instruments::prelude::{HiHat, Kick, Snare};
    ///
    /// let mut score = Score::new("Test", TimeSignature(4, 4), 120, 1, 20);
    /// let kick_index = score.add_instrument(Box::new(Kick::new()));
    /// ```
    pub fn add_instrument(&mut self, instrument: Box<dyn Instrument>) -> usize {
        self.instruments.push(instrument);
        let instrument_index = self.instruments.len() - 1;
        self.staves.push(Staff::new(&self.signature));
        self.staves
            .last_mut()
            .unwrap()
            .set_instrument(instrument_index);
        instrument_index
    }

    /// Adds a note to the score for the specified instrument at the first available space.
    pub fn add_note(&mut self, instrument: usize, note: Note) -> Result<(), String> {
        if instrument >= self.instruments.len() {
            return Err("Instrument index out of bounds".to_string());
        }

        let staff = if let Some(staff) = self
            .staves
            .iter_mut()
            .find(|staff| staff.get_instrument() == instrument)
        {
            staff
        } else {
            return Err("No staff found for the instrument".to_string());
        };

        staff.add_note(note)
    }
}
