use serde::{Deserialize, Serialize};

use crate::error::KeyboardError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveCommand {
    OctaveUp(u8),
    OctaveDown(u8),
    SetOctave { octave: u8, row: u8 },
    LinkOctaves,
    UnlinkOctaves,
    SelectInstrument { index: usize, row: u8 },
    NextInstrument(u8),
    PreviousInstrument(u8),
    LinkInstruments,
    UnlinkInstruments,
}

impl LiveCommand {
    pub fn validate(&self) -> Result<(), KeyboardError> {
        match self {
            LiveCommand::SetOctave { octave, row } => {
                if *row >= 2 {
                    return Err(KeyboardError::RowOutOfBounds(*row));
                }
                if *octave > 8 {
                    return Err(KeyboardError::InvalidOctave(*octave));
                }
                Ok(())
            }
            LiveCommand::OctaveUp(row) | LiveCommand::OctaveDown(row) => {
                if *row >= 2 {
                    return Err(KeyboardError::RowOutOfBounds(*row));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
