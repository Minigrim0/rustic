use super::staff::Staff;

/// A simple time signature denoted with its numerator and denominator.
/// ```rust
/// use rustic::prelude::TimeSignature;
///
/// // A simple 4/4 time signature
/// let time_signature = TimeSignature(4, 4);
/// ```
pub struct TimeSignature(usize, usize);

/// A music score. Has a defined time signature, tempo,
/// staves associated with their instruments, name, ...
pub struct Score<const INSTRUMENTS: usize, const LENGTH: usize> {
    pub name: String,                         // Name of the score
    pub signature: TimeSignature,             // Time signature of the score
    pub tempo: usize,                         // Tempo in bpm
    pub staves: [Staff<LENGTH>; INSTRUMENTS], // Staves of the score, contains the instruments
}
