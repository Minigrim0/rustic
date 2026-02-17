use serde::ser::{Serialize, Serializer};
use std::{io, sync::PoisonError};
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

    #[error("Unknown render mode {0}")]
    UnknownRenderMode(String),

    #[error("communication channel closed unexpectedly")]
    ChannelClosed,

    #[error("Configuration error: {0}")]
    ConfigError(String),
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
