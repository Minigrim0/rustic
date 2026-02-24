//! Keyboard instrument and live input handling for Rustic
//!
//! This crate provides keyboard-specific functionality that was extracted from the
//! core `rustic` crate:
//!
//! - [`row`]: The `Row` abstraction mapping keyboard rows to instruments + octaves
//! - [`commands`]: `LiveCommand` for real-time octave/instrument switching
//! - [`inputs`]: evdev-based keyboard input detection (requires `input` feature)
//! - [`instruments`]: The polyphonic `Keyboard` instrument and its builder
//! - [`voices`]: Voice allocation traits and strategies for polyphonic instruments
//! - [`error`]: Keyboard-specific error types

pub mod commands;
pub mod error;
pub mod inputs;
pub mod instruments;
pub mod row;
pub mod voices;
