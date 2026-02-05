use std::{io, sync::PoisonError};

use serde::ser::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("File not found")]
    FileNotFound,

    #[error("No audio file loaded")]
    NoAudioLoaded,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Unable to acquire lock, lock poisoned")]
    LockPoisoned,

    #[error("Invalid time rande")]
    InvalidTimeRange,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(_: PoisonError<T>) -> Self {
        AppError::LockPoisoned
    }
}
