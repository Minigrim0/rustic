//! Rustic related commands, e.g. changing settings, app behaviour, ...

use std::sync::Mutex;

use tauri::State;
use crate::{RusticState, error::AppError};

#[tauri::command]
pub fn change_render_mode(render_mode: String, rustic_state: State<'_, Mutex<RusticState>>) -> Result<(), AppError> {
    match render_mode.as_str() {
        "graph" => rustic_state.try_lock().map_err(|_| AppError::LockPoisoned)?
    }
}