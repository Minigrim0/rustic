use std::sync::{Mutex, mpsc::Sender};

use rustic::{audio::AudioHandle, prelude::Command};

pub struct RusticState {
    pub command_tx: Mutex<Sender<Command>>,
    pub audio_handle: Mutex<Option<AudioHandle>>,
}

impl RusticState {
    pub fn new(command_tx: Sender<Command>, audio_handle: AudioHandle) -> Self {
        Self {
            command_tx: Mutex::new(command_tx),
            audio_handle: Mutex::new(Some(audio_handle)),
        }
    }
}
