use device_query::Keycode;
use rustic::prelude::Commands;

/// Convert a keyboard key to a command
///
/// This function handles both key press and key release events
///
/// # Arguments
/// * `key` - The key that was pressed or released
/// * `is_release` - True if this is a key release event, false if it's a key press
pub fn key_to_command(key: &Keycode, is_release: bool) -> Option<Commands> {
    if is_release {
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
            Keycode::LBracket => Some(Commands::NoteStop(10, 1)),
            Keycode::RBracket => Some(Commands::NoteStop(11, 1)),
            _ => None,
        }
    } else {
        // Handle key press with different modifiers
        if is_shift_pressed() {
            match key {
                Keycode::Z => Some(Commands::OctaveDown(0)),
                Keycode::X => Some(Commands::OctaveDown(1)),
                _ => None,
            }
        } else {
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
                Keycode::LBracket => Some(Commands::NoteStart(10, 1, 1.0)),
                Keycode::RBracket => Some(Commands::NoteStart(11, 1, 1.0)),
                Keycode::Z => Some(Commands::OctaveUp(0)),
                Keycode::X => Some(Commands::OctaveUp(1)),
                _ => None,
            }
        }
    }
}

/// Check if the shift key is currently pressed
///
/// This function uses the DeviceState to get the current state of all keys
/// and checks if either shift key is pressed
fn is_shift_pressed() -> bool {
    use device_query::{DeviceQuery, DeviceState};
    let device_state = DeviceState::new();
    let keys = device_state.get_keys();
    keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift)
}

/// Get the first key of a multi-key command sequence
///
/// This function handles the first keypress of a multi-key command
pub fn multi_input_command(key: &Keycode) -> Option<Commands> {
    match key {
        Keycode::Z => Some(Commands::SelectInstrument(0, 0)),
        _ => None,
    }
}

/// Process the second key of a multi-key command sequence
///
/// This function handles the second keypress of a multi-key command
pub fn multi_input_second_stroke(key: &Keycode, command: &Commands) -> Option<Commands> {
    match command {
        Commands::SelectInstrument(_, row) => match key {
            Keycode::Key1 => Some(Commands::SelectInstrument(1, *row)),
            _ => None,
        },
        _ => None,
    }
}
