use std::sync::{Mutex, atomic::AtomicU64, mpsc::Sender};

use rustic::{audio::AudioHandle, prelude::Command};

pub struct RusticState {
    pub command_tx: Mutex<Sender<Command>>,
    pub audio_handle: Mutex<Option<AudioHandle>>,
    pub next_node_id: AtomicU64,
}

impl RusticState {
    pub fn new(command_tx: Sender<Command>, audio_handle: AudioHandle) -> Self {
        Self {
            command_tx: Mutex::new(command_tx),
            audio_handle: Mutex::new(Some(audio_handle)),
            next_node_id: AtomicU64::new(1),
        }
    }
}
