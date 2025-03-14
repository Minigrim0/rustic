use serde::{Deserialize, Serialize};

use super::staff::Staff;

/// A simple time signature denoted with its numerator and denominator.
/// ```rust
/// use rustic::prelude::TimeSignature;
///
/// // A simple 4/4 time signature
/// let time_signature = TimeSignature(4, 4);
/// ```
#[derive(Serialize, Deserialize)]
pub struct TimeSignature(pub usize, pub usize);

/// A music score. Has a defined time signature, tempo,
/// staves associated with their instruments, name, ...
#[derive(Serialize, Deserialize)]
pub struct Score {
    pub name: String,             // Name of the score
    pub signature: TimeSignature, // Time signature of the score
    pub tempo: usize,             // Tempo in bpm
    pub staves: Vec<Staff>,       // Staves of the score, contains the instruments
}

impl Score {
    pub fn new<S: AsRef<str>>(
        name: S,
        signature: TimeSignature,
        tempo: usize,
        staves: usize,
        measures: usize,
    ) -> Self {
        let staves = Vec::from_iter(
            (0..staves).map(|_| Staff::default().with_measures(measures, &signature)),
        );
        Self {
            name: name.as_ref().to_string(),
            signature,
            tempo,
            staves,
        }
    }
}
