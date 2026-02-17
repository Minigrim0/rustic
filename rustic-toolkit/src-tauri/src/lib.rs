use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, RwLock, mpsc};

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

/// Initialize toolkit-owned logging with colored terminal output and file logging.
fn init_toolkit_logging() {
    use simplelog::*;
    use std::fs;

    let cache_dir = directories::ProjectDirs::from("xyz", "minigrim0", "rustic")
        .map(|d: directories::ProjectDirs| d.cache_dir().to_path_buf().join("toolkit"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/rustic-toolkit"));

    let _ = fs::create_dir_all(&cache_dir);

    let log_file_path = cache_dir.join("rustic-toolkit.log");

    let term_config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_target_level(LevelFilter::Info)
        .set_location_level(LevelFilter::Debug)
        .build();

    let term_logger = TermLogger::new(
        LevelFilter::Info,
        term_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![term_logger];

    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        let file_config = ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_target_level(LevelFilter::Trace)
            .set_location_level(LevelFilter::Trace)
            .build();

        loggers.push(WriteLogger::new(LevelFilter::Trace, file_config, file));
    }

    let _ = CombinedLogger::init(loggers);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_toolkit_logging();

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
            commands::rustic::change_render_mode,
        ])
        .setup(|app| {
            // File system scope
            let scope = app.fs_scope();
            scope.allow_directory("/tmp", true)?;

            // Set up communication channels with the rustic audio engine
            log::info!("Creating communication channels");
            let (frontend_sender, backend_receiver): (Sender<Command>, Receiver<Command>) =
                mpsc::channel();
            let (backend_sender, frontend_receiver): (
                Sender<BackendEvent>,
                Receiver<BackendEvent>,
            ) = mpsc::channel();

            // Start the rustic audio engine (skip logging — toolkit owns it)
            log::info!("Starting audio engine");
            let audio_handle = rustic::start_app(backend_sender, backend_receiver, true)?;

            // Bridge backend events to Tauri frontend events
            log::info!("Starting event bridge thread");
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

            app.manage(Mutex::new(RusticState::new(frontend_sender, audio_handle)));

            log::info!("Setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
