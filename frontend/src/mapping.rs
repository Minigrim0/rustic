use device_query::Keycode;
use rustic::app::commands::{
    AppCommand, AudioCommand, Command, LiveCommand, LoopCommand, MixCommand, SettingsCommand,
};
use std::sync::mpsc::Sender;

/// KeyMapper for translating keyboard key codes to Commands
pub struct KeyMapper {
    app_sender: Sender<Command>,
}

impl KeyMapper {
    /// Create a new KeyMapper
    pub fn new(app_sender: Sender<Command>) -> Self {
        KeyMapper { app_sender }
    }

    /// Send a command to the rustic audio engine
    pub fn send_command(&self, command: Command) {
        let _ = self.app_sender.send(command);
    }

    /// Map a keyboard key to a Command
    pub fn map_key(&self, key: &Keycode, is_release: bool, shift_pressed: bool) -> Option<Command> {
        if is_release {
            self.map_key_release(key)
        } else if shift_pressed {
            self.map_key_with_shift(key)
        } else {
            self.map_key_press(key)
        }
    }

    /// Map a key press event to a Command
    fn map_key_press(&self, key: &Keycode) -> Option<Command> {
        match key {
            Keycode::Key1 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 0,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key2 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 1,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key3 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 2,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key4 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 3,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key5 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 4,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key6 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 5,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key7 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 6,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key8 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 7,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key9 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 8,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Key0 => Some(Command::Audio(AudioCommand::NoteStart {
                note: 9,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Minus => Some(Command::Audio(AudioCommand::NoteStart {
                note: 9,
                row: 0,
                velocity: 1.0,
            })),
            Keycode::Equal => Some(Command::Audio(AudioCommand::NoteStart {
                note: 9,
                row: 0,
                velocity: 1.0,
            })),

            Keycode::Q => Some(Command::Audio(AudioCommand::NoteStart {
                note: 0,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::W => Some(Command::Audio(AudioCommand::NoteStart {
                note: 1,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::E => Some(Command::Audio(AudioCommand::NoteStart {
                note: 2,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::R => Some(Command::Audio(AudioCommand::NoteStart {
                note: 3,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::T => Some(Command::Audio(AudioCommand::NoteStart {
                note: 4,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::Y => Some(Command::Audio(AudioCommand::NoteStart {
                note: 5,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::U => Some(Command::Audio(AudioCommand::NoteStart {
                note: 6,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::I => Some(Command::Audio(AudioCommand::NoteStart {
                note: 7,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::O => Some(Command::Audio(AudioCommand::NoteStart {
                note: 8,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::P => Some(Command::Audio(AudioCommand::NoteStart {
                note: 9,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::LeftBracket => Some(Command::Audio(AudioCommand::NoteStart {
                note: 10,
                row: 1,
                velocity: 1.0,
            })),
            Keycode::RightBracket => Some(Command::Audio(AudioCommand::NoteStart {
                note: 11,
                row: 1,
                velocity: 1.0,
            })),

            Keycode::Z => Some(Command::App(AppCommand::Live(LiveCommand::OctaveUp(0)))),
            Keycode::X => Some(Command::App(AppCommand::Live(LiveCommand::OctaveUp(1)))),

            // Volume controls
            Keycode::Up => Some(Command::App(AppCommand::Mix(MixCommand::VolumeUp(0)))),
            Keycode::Down => Some(Command::App(AppCommand::Mix(MixCommand::VolumeDown(0)))),

            // Metronome controls
            Keycode::M => Some(Command::App(AppCommand::Settings(
                SettingsCommand::ToggleMetronome,
            ))),

            // Loop recording controls
            Keycode::Space => Some(Command::App(AppCommand::Loop(LoopCommand::PlayLoop))),
            Keycode::Escape => Some(Command::App(AppCommand::Loop(LoopCommand::StopLoop))),

            _ => None,
        }
    }

    /// Map a key release event to a Command
    fn map_key_release(&self, key: &Keycode) -> Option<Command> {
        match key {
            Keycode::Key1 => Some(Command::Audio(AudioCommand::NoteStop { note: 0, row: 0 })),
            Keycode::Key2 => Some(Command::Audio(AudioCommand::NoteStop { note: 1, row: 0 })),
            Keycode::Key3 => Some(Command::Audio(AudioCommand::NoteStop { note: 2, row: 0 })),
            Keycode::Key4 => Some(Command::Audio(AudioCommand::NoteStop { note: 3, row: 0 })),
            Keycode::Key5 => Some(Command::Audio(AudioCommand::NoteStop { note: 4, row: 0 })),
            Keycode::Key6 => Some(Command::Audio(AudioCommand::NoteStop { note: 5, row: 0 })),
            Keycode::Key7 => Some(Command::Audio(AudioCommand::NoteStop { note: 6, row: 0 })),
            Keycode::Key8 => Some(Command::Audio(AudioCommand::NoteStop { note: 7, row: 0 })),
            Keycode::Key9 => Some(Command::Audio(AudioCommand::NoteStop { note: 8, row: 0 })),
            Keycode::Key0 => Some(Command::Audio(AudioCommand::NoteStop { note: 9, row: 0 })),
            Keycode::Minus => Some(Command::Audio(AudioCommand::NoteStop { note: 9, row: 0 })),
            Keycode::Equal => Some(Command::Audio(AudioCommand::NoteStop { note: 9, row: 0 })),

            Keycode::Q => Some(Command::Audio(AudioCommand::NoteStop { note: 0, row: 1 })),
            Keycode::W => Some(Command::Audio(AudioCommand::NoteStop { note: 1, row: 1 })),
            Keycode::E => Some(Command::Audio(AudioCommand::NoteStop { note: 2, row: 1 })),
            Keycode::R => Some(Command::Audio(AudioCommand::NoteStop { note: 3, row: 1 })),
            Keycode::T => Some(Command::Audio(AudioCommand::NoteStop { note: 4, row: 1 })),
            Keycode::Y => Some(Command::Audio(AudioCommand::NoteStop { note: 5, row: 1 })),
            Keycode::U => Some(Command::Audio(AudioCommand::NoteStop { note: 6, row: 1 })),
            Keycode::I => Some(Command::Audio(AudioCommand::NoteStop { note: 7, row: 1 })),
            Keycode::O => Some(Command::Audio(AudioCommand::NoteStop { note: 8, row: 1 })),
            Keycode::P => Some(Command::Audio(AudioCommand::NoteStop { note: 9, row: 1 })),
            Keycode::LeftBracket => {
                Some(Command::Audio(AudioCommand::NoteStop { note: 10, row: 1 }))
            }
            Keycode::RightBracket => {
                Some(Command::Audio(AudioCommand::NoteStop { note: 11, row: 1 }))
            }

            _ => None,
        }
    }

    /// Map a key press with shift key to a Command
    fn map_key_with_shift(&self, key: &Keycode) -> Option<Command> {
        match key {
            Keycode::Z => Some(Command::App(AppCommand::Live(LiveCommand::OctaveDown(0)))),
            Keycode::X => Some(Command::App(AppCommand::Live(LiveCommand::OctaveDown(1)))),

            // Volume controls with shift
            Keycode::Up => Some(Command::App(AppCommand::Mix(MixCommand::VolumeUp(1)))),
            Keycode::Down => Some(Command::App(AppCommand::Mix(MixCommand::VolumeDown(1)))),

            _ => None,
        }
    }
}
