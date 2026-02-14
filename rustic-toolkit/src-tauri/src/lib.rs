use std::sync::mpsc::{Receiver, Sender};
use std::sync::{RwLock, mpsc};

use rustic::audio::BackendEvent;
use rustic::prelude::Command;
use tauri::{Emitter, Manager};
use tauri_plugin_fs::FsExt;

mod analysis;
mod audio;
mod commands;
mod error;
mod rustic_state;
mod state;
mod types;

use rustic_state::RusticState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    colog::init();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Starting Rustic Toolkit");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(RwLock::new(state::AudioState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::analyze::analyze_audio_file,
            commands::query::get_waveform,
            commands::query::get_spectrum,
            commands::query::get_top_frequencies,
            commands::query::get_spectrogram,
            commands::utils::frequency_to_note_command,
            commands::utils::save_analysis,
            commands::meta::get_graph_metadata,
        ])
        .setup(|app| {
            // File system scope
            let scope = app.fs_scope();
            scope.allow_directory("/tmp", true)?;

            // Set up communication channels with the rustic audio engine
            let (frontend_sender, backend_receiver): (Sender<Command>, Receiver<Command>) =
                mpsc::channel();
            let (backend_sender, frontend_receiver): (
                Sender<BackendEvent>,
                Receiver<BackendEvent>,
            ) = mpsc::channel();

            // Start the rustic audio engine
            let audio_handle = rustic::start_app(backend_sender, backend_receiver)?;

            // Bridge backend events to Tauri frontend events
            let tauri_handle = app.handle().clone();
            std::thread::spawn(move || {
                log::info!("Rustic event bridge started");
                while let Ok(event) = frontend_receiver.recv() {
                    if let Err(e) = tauri_handle.emit("rustic-event", &event) {
                        log::error!("Failed to emit rustic event: {e}");
                    }
                }
                log::info!("Rustic event bridge shut down");
            });

            app.manage(RusticState::new(frontend_sender, audio_handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
