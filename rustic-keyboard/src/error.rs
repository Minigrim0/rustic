use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyboardError {
    #[error("Row {0} out of bounds")]
    RowOutOfBounds(u8),
    #[error("Invalid octave: {0}")]
    InvalidOctave(u8),
}
