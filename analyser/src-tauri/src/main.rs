// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use std::sync::{Arc, Mutex};
use tauri_plugin_fs::FsExt;

mod analysis;
mod audio;
mod commands;

fn main() {
    // Initialize logging
    colog::init();
    log::set_max_level(log::LevelFilter::Info);

    info!("Starting Rustic Sample Analyser");

    // Create the audio state to be shared across commands
    let audio_state = Arc::new(Mutex::new(commands::AudioState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .manage(audio_state)
        .invoke_handler(tauri::generate_handler![
            commands::analyze_audio_file,
            commands::get_sample_rate,
            commands::get_samples,
            commands::compute_fft_command,
            commands::list_frequencies,
            commands::compute_spectrum_command,
            commands::save_analysis,
            commands::estimate_pitch_command,
            commands::frequency_to_note_command,
        ])
        .setup(|app| {
            // allowed the given directory
            let scope = app.fs_scope();
            scope.allow_directory("/tmp", true)
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
