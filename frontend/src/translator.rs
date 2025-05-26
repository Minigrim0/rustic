use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

use rustic::prelude::Commands;

pub fn event_to_command(event: &Event) -> Option<Commands> {
    match event {
        Event::KeyDown {
            keycode,
            repeat: false,
            keymod: Mod::NOMOD,
            ..
        } => match keycode {
            Some(Keycode::NUM_1) => Some(Commands::NoteStart(0, 0, 1.0)),
            Some(Keycode::NUM_2) => Some(Commands::NoteStart(1, 0, 1.0)),
            Some(Keycode::NUM_3) => Some(Commands::NoteStart(2, 0, 1.0)),
            Some(Keycode::NUM_4) => Some(Commands::NoteStart(3, 0, 1.0)),
            Some(Keycode::NUM_5) => Some(Commands::NoteStart(4, 0, 1.0)),
            Some(Keycode::NUM_6) => Some(Commands::NoteStart(5, 0, 1.0)),
            Some(Keycode::NUM_7) => Some(Commands::NoteStart(6, 0, 1.0)),
            Some(Keycode::NUM_8) => Some(Commands::NoteStart(7, 0, 1.0)),
            Some(Keycode::NUM_9) => Some(Commands::NoteStart(8, 0, 1.0)),
            Some(Keycode::NUM_0) => Some(Commands::NoteStart(9, 0, 1.0)),
            Some(Keycode::UNDERSCORE) => Some(Commands::NoteStart(9, 0, 1.0)),
            Some(Keycode::PLUS) => Some(Commands::NoteStart(9, 0, 1.0)),

            Some(Keycode::Q) => Some(Commands::NoteStart(0, 1, 1.0)),
            Some(Keycode::W) => Some(Commands::NoteStart(1, 1, 1.0)),
            Some(Keycode::E) => Some(Commands::NoteStart(2, 1, 1.0)),
            Some(Keycode::R) => Some(Commands::NoteStart(3, 1, 1.0)),
            Some(Keycode::T) => Some(Commands::NoteStart(4, 1, 1.0)),
            Some(Keycode::Y) => Some(Commands::NoteStart(5, 1, 1.0)),
            Some(Keycode::U) => Some(Commands::NoteStart(6, 1, 1.0)),
            Some(Keycode::I) => Some(Commands::NoteStart(7, 1, 1.0)),
            Some(Keycode::O) => Some(Commands::NoteStart(8, 1, 1.0)),
            Some(Keycode::P) => Some(Commands::NoteStart(9, 1, 1.0)),
            Some(Keycode::LEFTBRACKET) => Some(Commands::NoteStart(10, 1, 1.0)),
            Some(Keycode::RIGHTBRACKET) => Some(Commands::NoteStart(11, 1, 1.0)),
            Some(Keycode::Z) => Some(Commands::OctaveUp(0)),
            Some(Keycode::X) => Some(Commands::OctaveUp(1)),
            _ => None,
        },
        Event::KeyUp {
            keycode,
            repeat: false,
            ..
        } => match keycode {
            Some(Keycode::NUM_1) => Some(Commands::NoteStop(0, 0)),
            Some(Keycode::NUM_2) => Some(Commands::NoteStop(1, 0)),
            Some(Keycode::NUM_3) => Some(Commands::NoteStop(2, 0)),
            Some(Keycode::NUM_4) => Some(Commands::NoteStop(3, 0)),
            Some(Keycode::NUM_5) => Some(Commands::NoteStop(4, 0)),
            Some(Keycode::NUM_6) => Some(Commands::NoteStop(5, 0)),
            Some(Keycode::NUM_7) => Some(Commands::NoteStop(6, 0)),
            Some(Keycode::NUM_8) => Some(Commands::NoteStop(7, 0)),
            Some(Keycode::NUM_9) => Some(Commands::NoteStop(8, 0)),
            Some(Keycode::NUM_0) => Some(Commands::NoteStop(9, 0)),
            Some(Keycode::UNDERSCORE) => Some(Commands::NoteStop(9, 0)),
            Some(Keycode::PLUS) => Some(Commands::NoteStop(9, 0)),

            Some(Keycode::Q) => Some(Commands::NoteStop(0, 1)),
            Some(Keycode::W) => Some(Commands::NoteStop(1, 1)),
            Some(Keycode::E) => Some(Commands::NoteStop(2, 1)),
            Some(Keycode::R) => Some(Commands::NoteStop(3, 1)),
            Some(Keycode::T) => Some(Commands::NoteStop(4, 1)),
            Some(Keycode::Y) => Some(Commands::NoteStop(5, 1)),
            Some(Keycode::U) => Some(Commands::NoteStop(6, 1)),
            Some(Keycode::I) => Some(Commands::NoteStop(7, 1)),
            Some(Keycode::O) => Some(Commands::NoteStop(8, 1)),
            Some(Keycode::P) => Some(Commands::NoteStop(9, 1)),
            Some(Keycode::LEFTBRACKET) => Some(Commands::NoteStop(10, 1)),
            Some(Keycode::RIGHTBRACKET) => Some(Commands::NoteStop(11, 1)),
            _ => None,
        },
        Event::KeyDown {
            keycode,
            repeat: false,
            keymod: Mod::LSHIFTMOD,
            ..
        } => match keycode {
            Some(Keycode::Z) => Some(Commands::OctaveDown(0)),
            Some(Keycode::X) => Some(Commands::OctaveDown(1)),
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
