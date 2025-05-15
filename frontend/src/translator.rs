use rustic::inputs::commands::Commands;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn event_to_command(event: &Event) -> Option<Commands> {
    match event {
        Event::KeyDown {
            keycode,
            repeat: false,
            ..
        } => match keycode {
            Some(Keycode::Q) => Some(Commands::NoteStart(0, 0, 1.0)),
            Some(Keycode::W) => Some(Commands::NoteStart(1, 0, 1.0)),
            Some(Keycode::E) => Some(Commands::NoteStart(2, 0, 1.0)),
            Some(Keycode::R) => Some(Commands::NoteStart(3, 0, 1.0)),
            Some(Keycode::T) => Some(Commands::NoteStart(4, 0, 1.0)),
            Some(Keycode::Y) => Some(Commands::NoteStart(5, 0, 1.0)),
            Some(Keycode::U) => Some(Commands::NoteStart(6, 0, 1.0)),
            Some(Keycode::I) => Some(Commands::NoteStart(7, 0, 1.0)),
            Some(Keycode::O) => Some(Commands::NoteStart(8, 0, 1.0)),
            Some(Keycode::P) => Some(Commands::NoteStart(9, 0, 1.0)),
            Some(Keycode::LEFTBRACKET) => Some(Commands::NoteStart(10, 0, 1.0)),
            Some(Keycode::RIGHTBRACKET) => Some(Commands::NoteStart(11, 0, 1.0)),
            _ => None,
        },
        Event::KeyUp {
            keycode,
            repeat: false,
            ..
        } => match keycode {
            Some(Keycode::Q) => Some(Commands::NoteStop(0, 0)),
            Some(Keycode::W) => Some(Commands::NoteStop(1, 0)),
            Some(Keycode::E) => Some(Commands::NoteStop(2, 0)),
            Some(Keycode::R) => Some(Commands::NoteStop(3, 0)),
            Some(Keycode::T) => Some(Commands::NoteStop(4, 0)),
            Some(Keycode::Y) => Some(Commands::NoteStop(5, 0)),
            Some(Keycode::U) => Some(Commands::NoteStop(6, 0)),
            Some(Keycode::I) => Some(Commands::NoteStop(7, 0)),
            Some(Keycode::O) => Some(Commands::NoteStop(8, 0)),
            Some(Keycode::P) => Some(Commands::NoteStop(9, 0)),
            Some(Keycode::LEFTBRACKET) => Some(Commands::NoteStop(10, 0)),
            Some(Keycode::RIGHTBRACKET) => Some(Commands::NoteStop(11, 0)),
            _ => None,
        },
        _ => None,
    }
}

pub fn multi_input_command(event: &Event) -> Option<Commands> {
    match event {
        Event::KeyDown {
            keycode,
            repeat: false,
            ..
        } => match keycode {
            Some(Keycode::Z) => Some(Commands::SelectInstrument(0, 0)),
            _ => None,
        },
        _ => None,
    }
}

pub fn multi_input_second_stroke(event: &Event, command: &Commands) -> Option<Commands> {
    match command {
        Commands::SelectInstrument(_, row) => match event {
            Event::KeyDown {
                keycode,
                repeat: false,
                ..
            } => match keycode {
                Some(Keycode::NUM_1) => Some(Commands::SelectInstrument(1, *row)),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}
