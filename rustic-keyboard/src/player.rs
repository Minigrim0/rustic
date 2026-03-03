//! [`KeyboardPlayer`] — translates physical key events into rustic audio commands.
//!
//! Key layout (chromatic from C, 0 = C):
//!
//! ```text
//! Row 0 (QWERTY): Q  W  E  R  T  Y  U  I  O  P
//!                 C  C# D  D# E  F  F# G  G# A
//!
//! Row 1 (ASDF):   A  S  D  F  G  H  J  K  L
//!                 C  C# D  D# E  F  F# G  G#
//! ```
//!
//! Row 0 defaults to octave 5, row 1 to octave 4.

#[cfg(feature = "input")]
use rustic::app::commands::{AudioCommand, Command};
#[cfg(feature = "input")]
use rustic::prelude::App;

use crate::commands::live::LiveCommand;
use crate::error::KeyboardError;
use crate::row::Row;

/// Holds the live keyboard state and drives playback by translating key
/// events into [`rustic::app::commands::AudioCommand`]s sent through the app.
pub struct KeyboardPlayer {
    rows: [Row; 2],
    octaves_linked: bool,
    instruments_linked: bool,
}

impl Default for KeyboardPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardPlayer {
    pub fn new() -> Self {
        Self {
            rows: [
                Row { instrument: 0, octave: 5 }, // row 0 — upper (QWERTY)
                Row { instrument: 0, octave: 4 }, // row 1 — lower (ASDF)
            ],
            octaves_linked: false,
            instruments_linked: false,
        }
    }

    /// Apply a [`LiveCommand`] to mutate row state (octave, instrument, linking).
    pub fn apply(&mut self, cmd: LiveCommand) -> Result<(), KeyboardError> {
        cmd.validate()?;
        match cmd {
            LiveCommand::OctaveUp(row) => {
                let r = row as usize;
                self.rows[r].octave = self.rows[r].octave.saturating_add(1).min(8);
                if self.octaves_linked {
                    self.rows[1 - r].octave = self.rows[r].octave;
                }
            }
            LiveCommand::OctaveDown(row) => {
                let r = row as usize;
                self.rows[r].octave = self.rows[r].octave.saturating_sub(1);
                if self.octaves_linked {
                    self.rows[1 - r].octave = self.rows[r].octave;
                }
            }
            LiveCommand::SetOctave { octave, row } => {
                let r = row as usize;
                self.rows[r].octave = octave;
                if self.octaves_linked {
                    self.rows[1 - r].octave = octave;
                }
            }
            LiveCommand::LinkOctaves => {
                self.octaves_linked = true;
            }
            LiveCommand::UnlinkOctaves => {
                self.octaves_linked = false;
            }
            LiveCommand::SelectInstrument { index, row } => {
                let r = row as usize;
                if r >= 2 {
                    return Err(KeyboardError::RowOutOfBounds(row));
                }
                self.rows[r].instrument = index;
                if self.instruments_linked {
                    self.rows[1 - r].instrument = index;
                }
            }
            LiveCommand::NextInstrument(row) => {
                let r = row as usize;
                if r >= 2 {
                    return Err(KeyboardError::RowOutOfBounds(row));
                }
                self.rows[r].instrument = self.rows[r].instrument.saturating_add(1);
                if self.instruments_linked {
                    self.rows[1 - r].instrument = self.rows[r].instrument;
                }
            }
            LiveCommand::PreviousInstrument(row) => {
                let r = row as usize;
                if r >= 2 {
                    return Err(KeyboardError::RowOutOfBounds(row));
                }
                self.rows[r].instrument = self.rows[r].instrument.saturating_sub(1);
                if self.instruments_linked {
                    self.rows[1 - r].instrument = self.rows[r].instrument;
                }
            }
            LiveCommand::LinkInstruments => {
                self.instruments_linked = true;
            }
            LiveCommand::UnlinkInstruments => {
                self.instruments_linked = false;
            }
        }
        Ok(())
    }

    #[cfg(feature = "input")]
    fn note_on(&self, app: &App, row: usize, note_idx: u8) {
        let note = self.rows[row].get_note(note_idx);
        let instrument_idx = self.rows[row].instrument;
        let _ = app.send(Command::Audio(AudioCommand::NoteStart {
            instrument_idx,
            note,
            velocity: 1.0,
        }));
    }

    #[cfg(feature = "input")]
    fn note_off(&self, app: &App, row: usize, note_idx: u8) {
        let note = self.rows[row].get_note(note_idx);
        let instrument_idx = self.rows[row].instrument;
        let _ = app.send(Command::Audio(AudioCommand::NoteStop {
            instrument_idx,
            note,
        }));
    }

    /// Block on keyboard events and translate them into audio commands.
    ///
    /// Uses [`crate::inputs::find_keyboard`] to pick the first available device.
    /// Prefer [`Self::run_with_device`] when you need explicit device selection.
    ///
    /// Requires the `input` feature. Call after [`App::start()`].
    #[cfg(feature = "input")]
    pub fn run(&mut self, app: &App) {
        use crate::inputs::keyboard::find_keyboard;

        let device = match find_keyboard() {
            Some(d) => d,
            None => {
                log::error!("No keyboard device found — keyboard player not started");
                return;
            }
        };

        self.run_with_device(device, app);
    }

    /// Block on keyboard events from `device` and translate them into audio commands.
    ///
    /// Requires the `input` feature. Call after [`App::start()`].
    #[cfg(feature = "input")]
    pub fn run_with_device(&mut self, mut device: evdev::Device, app: &App) {
        use evdev::InputEventKind;

        log::info!("Keyboard player running");

        loop {
            let events = match device.fetch_events() {
                Ok(events) => events,
                Err(e) => {
                    log::error!("Keyboard event error: {e}");
                    break;
                }
            };

            for event in events {
                if let InputEventKind::Key(key) = event.kind() {
                    match event.value() {
                        1 /* pressed */ => {
                            if let Some((row, note_idx)) = Self::key_to_note(key) {
                                self.note_on(app, row, note_idx);
                            }
                        }
                        0 /* released */ => {
                            if let Some((row, note_idx)) = Self::key_to_note(key) {
                                self.note_off(app, row, note_idx);
                            }
                        }
                        _ => {} // auto-repeat, ignore
                    }
                }
            }
        }
    }

    /// Map a physical key to `(row, note_index)`.
    ///
    /// `note_index` is a chromatic offset from C (0 = C, 1 = C#, …, 11 = B).
    #[cfg(feature = "input")]
    fn key_to_note(key: evdev::Key) -> Option<(usize, u8)> {
        use evdev::Key;
        match key {
            // Row 0 — QWERTY (octave 5 by default)
            Key::KEY_Q => Some((0, 0)),  // C
            Key::KEY_W => Some((0, 1)),  // C#
            Key::KEY_E => Some((0, 2)),  // D
            Key::KEY_R => Some((0, 3)),  // D#
            Key::KEY_T => Some((0, 4)),  // E
            Key::KEY_Y => Some((0, 5)),  // F
            Key::KEY_U => Some((0, 6)),  // F#
            Key::KEY_I => Some((0, 7)),  // G
            Key::KEY_O => Some((0, 8)),  // G#
            Key::KEY_P => Some((0, 9)),  // A

            // Row 1 — ASDF (octave 4 by default)
            Key::KEY_A => Some((1, 0)),  // C
            Key::KEY_S => Some((1, 1)),  // C#
            Key::KEY_D => Some((1, 2)),  // D
            Key::KEY_F => Some((1, 3)),  // D#
            Key::KEY_G => Some((1, 4)),  // E
            Key::KEY_H => Some((1, 5)),  // F
            Key::KEY_J => Some((1, 6)),  // F#
            Key::KEY_K => Some((1, 7)),  // G
            Key::KEY_L => Some((1, 8)),  // G#

            _ => None,
        }
    }
}
