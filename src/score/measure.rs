use serde::{Deserialize, Serialize};

use super::{notes::Note, score::TimeSignature};

/// A measure contains a given amount of notes
#[derive(Serialize, Deserialize)]
pub struct Measure {
    notes: Vec<Note>,
}

impl Measure {
    pub fn new(signature: &TimeSignature) -> Self {
        Self {
            notes: Vec::from_iter((0..signature.0).map(|_| Note::new_pause(1).unwrap())),
        }
    }
}
