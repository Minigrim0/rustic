use device_query::Keycode;
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

/// KeyMapper for translating keyboard key codes to Commands
pub struct KeyMapper {
    app_sender: Sender<Commands>,
}

impl KeyMapper {
    /// Create a new KeyMapper
    pub fn new(app_sender: Sender<Commands>) -> Self {
        KeyMapper { app_sender }
    }

    /// Send a command to the rustic audio engine
    pub fn send_command(&self, command: Commands) {
        let _ = self.app_sender.send(command);
    }

    /// Map a keyboard key to a Command
    pub fn map_key(
        &self,
        key: &Keycode,
        is_release: bool,
        shift_pressed: bool,
    ) -> Option<Commands> {
        if is_release {
            self.map_key_release(key)
        } else if shift_pressed {
            self.map_key_with_shift(key)
        } else {
            self.map_key_press(key)
        }
    }

    /// Map a key press event to a Command
    fn map_key_press(&self, key: &Keycode) -> Option<Commands> {
        match key {
            Keycode::Key1 => Some(Commands::NoteStart(0, 0, 1.0)),
            Keycode::Key2 => Some(Commands::NoteStart(1, 0, 1.0)),
            Keycode::Key3 => Some(Commands::NoteStart(2, 0, 1.0)),
            Keycode::Key4 => Some(Commands::NoteStart(3, 0, 1.0)),
            Keycode::Key5 => Some(Commands::NoteStart(4, 0, 1.0)),
            Keycode::Key6 => Some(Commands::NoteStart(5, 0, 1.0)),
            Keycode::Key7 => Some(Commands::NoteStart(6, 0, 1.0)),
            Keycode::Key8 => Some(Commands::NoteStart(7, 0, 1.0)),
            Keycode::Key9 => Some(Commands::NoteStart(8, 0, 1.0)),
            Keycode::Key0 => Some(Commands::NoteStart(9, 0, 1.0)),
            Keycode::Minus => Some(Commands::NoteStart(9, 0, 1.0)),
            Keycode::Equal => Some(Commands::NoteStart(9, 0, 1.0)),

            Keycode::Q => Some(Commands::NoteStart(0, 1, 1.0)),
            Keycode::W => Some(Commands::NoteStart(1, 1, 1.0)),
            Keycode::E => Some(Commands::NoteStart(2, 1, 1.0)),
            Keycode::R => Some(Commands::NoteStart(3, 1, 1.0)),
            Keycode::T => Some(Commands::NoteStart(4, 1, 1.0)),
            Keycode::Y => Some(Commands::NoteStart(5, 1, 1.0)),
            Keycode::U => Some(Commands::NoteStart(6, 1, 1.0)),
            Keycode::I => Some(Commands::NoteStart(7, 1, 1.0)),
            Keycode::O => Some(Commands::NoteStart(8, 1, 1.0)),
            Keycode::P => Some(Commands::NoteStart(9, 1, 1.0)),
            Keycode::LeftBracket => Some(Commands::NoteStart(10, 1, 1.0)),
            Keycode::RightBracket => Some(Commands::NoteStart(11, 1, 1.0)),

            Keycode::Z => Some(Commands::OctaveUp(0)),
            Keycode::X => Some(Commands::OctaveUp(1)),

            // Volume controls
            Keycode::Up => Some(Commands::VolumeUp(0)),
            Keycode::Down => Some(Commands::VolumeDown(0)),

            // Metronome controls
            Keycode::M => Some(Commands::ToggleMetronome),

            // Loop recording controls
            // Note: R is already handled above for NoteStart, this pattern is unreachable
            Keycode::Space => Some(Commands::PlayLoop),
            Keycode::Escape => Some(Commands::StopLoop),

            _ => None,
        }
    }

    /// Map a key release event to a Command
    fn map_key_release(&self, key: &Keycode) -> Option<Commands> {
        match key {
            Keycode::Key1 => Some(Commands::NoteStop(0, 0)),
            Keycode::Key2 => Some(Commands::NoteStop(1, 0)),
            Keycode::Key3 => Some(Commands::NoteStop(2, 0)),
            Keycode::Key4 => Some(Commands::NoteStop(3, 0)),
            Keycode::Key5 => Some(Commands::NoteStop(4, 0)),
            Keycode::Key6 => Some(Commands::NoteStop(5, 0)),
            Keycode::Key7 => Some(Commands::NoteStop(6, 0)),
            Keycode::Key8 => Some(Commands::NoteStop(7, 0)),
            Keycode::Key9 => Some(Commands::NoteStop(8, 0)),
            Keycode::Key0 => Some(Commands::NoteStop(9, 0)),
            Keycode::Minus => Some(Commands::NoteStop(9, 0)),
            Keycode::Equal => Some(Commands::NoteStop(9, 0)),

            Keycode::Q => Some(Commands::NoteStop(0, 1)),
            Keycode::W => Some(Commands::NoteStop(1, 1)),
            Keycode::E => Some(Commands::NoteStop(2, 1)),
            Keycode::R => Some(Commands::NoteStop(3, 1)),
            Keycode::T => Some(Commands::NoteStop(4, 1)),
            Keycode::Y => Some(Commands::NoteStop(5, 1)),
            Keycode::U => Some(Commands::NoteStop(6, 1)),
            Keycode::I => Some(Commands::NoteStop(7, 1)),
            Keycode::O => Some(Commands::NoteStop(8, 1)),
            Keycode::P => Some(Commands::NoteStop(9, 1)),
            Keycode::LeftBracket => Some(Commands::NoteStop(10, 1)),
            Keycode::RightBracket => Some(Commands::NoteStop(11, 1)),

            _ => None,
        }
    }

    /// Map a key press with shift key to a Command
    fn map_key_with_shift(&self, key: &Keycode) -> Option<Commands> {
        match key {
            Keycode::Z => Some(Commands::OctaveDown(0)),
            Keycode::X => Some(Commands::OctaveDown(1)),

            // Volume controls with shift
            Keycode::Up => Some(Commands::VolumeUp(1)),
            Keycode::Down => Some(Commands::VolumeDown(1)),

            _ => None,
        }
    }
}
