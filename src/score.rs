// Purpose: Contains the Score struct, which is used to store a sequence of notes to be played.
use crate::player::Player;

pub struct Note {
    pub frequency: f32,
    pub start_time: f32,
    pub duration: f32,
}

pub struct Score {
    pub notes: Vec<Note>,
}

impl Score {
    pub fn new(notes: Vec<Note>) -> Self {
        Self { notes }
    }

    pub fn play(&self, player: &mut Player) {
        let mut time = 0.0;

        for note in &self.notes {
            time = note.start_time;
            player.sound_system.generator.generate(time);
            player.play(note.duration);
        }
    }
}
