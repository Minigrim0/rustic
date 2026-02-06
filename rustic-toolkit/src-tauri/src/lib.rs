use std::sync::RwLock;

use log::info;
use tauri::Emitter;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri_plugin_fs::FsExt;

mod analysis;
mod audio;
mod commands;
mod error;
mod state;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    colog::init();
    log::set_max_level(log::LevelFilter::Info);

    info!("Starting Rustic Sample Analyser");

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
        ])
        .setup(|app| {
            // File system scope
            let scope = app.fs_scope();
            scope.allow_directory("/tmp", true)?;

            // Native menu
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(
                    &MenuItemBuilder::with_id("open", "Open File")
                        .accelerator("CmdOrCtrl+O")
                        .build(app)?,
                )
                .separator()
                .item(&PredefinedMenuItem::quit(app, Some("Quit"))?)
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(&MenuItemBuilder::with_id("about", "About Rustic").build(app)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .items(&[&file_menu, &help_menu])
                .build()?;

            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                let _ = app.emit(event.id().as_ref(), ());
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
