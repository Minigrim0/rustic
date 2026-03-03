//! Physical keyboard input handling and live-play commands for Rustic.
//!
//! This crate provides OS-level keyboard interfacing extracted from the core `rustic` crate:
//!
//! - [`row`]: The `Row` abstraction mapping physical keyboard rows to instruments + octaves
//! - [`commands`]: `LiveCommand` for real-time octave/instrument switching
//! - [`inputs`]: evdev-based keyboard input detection (requires `input` feature)
//! - [`player`]: [`KeyboardPlayer`] — holds row state, maps key events to audio commands
//! - [`error`]: Keyboard-specific error types

pub mod commands;
pub mod error;
pub mod inputs;
pub mod player;
pub mod row;

pub use player::KeyboardPlayer;
