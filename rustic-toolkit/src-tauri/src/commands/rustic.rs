//! Rustic-related commands, e.g. changing settings, app behaviour, ...

use std::sync::Mutex;

use crate::{RusticState, error::AppError};
use rustic::audio::RenderMode;
use rustic::prelude::{AudioCommand, Command};
use tauri::State;

#[tauri::command]
pub fn change_render_mode(
    render_mode: String,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    let tx_channel = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    match render_mode.as_str() {
        "graph" => tx_channel
            .send(Command::Audio(AudioCommand::SetRenderMode(
                RenderMode::Graph,
            )))
            .map_err(|_| AppError::ChannelClosed)?,
        "instrument" => tx_channel
            .send(Command::Audio(AudioCommand::SetRenderMode(
                RenderMode::Instruments,
            )))
            .map_err(|_| AppError::ChannelClosed)?,
        _ => {
            log::warn!("Unknown render mode: {}", render_mode);
            return Err(AppError::UnknownRenderMode(render_mode));
        }
    }
    Ok(())
}
