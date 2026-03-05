use std::sync::Mutex;
use std::sync::atomic::Ordering;

use crate::{RusticState, error::AppError};
use rustic::app::commands::{GraphCommand, NodeKind};
use rustic::prelude::Command;
use tauri::State;

fn rustic_err(e: impl std::fmt::Display) -> AppError {
    AppError::ConfigError(e.to_string())
}

#[tauri::command]
pub fn graph_add_node(
    node_type: String,
    kind: NodeKind,
    position: (f32, f32),
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<u64, AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    let id = state.next_node_id.fetch_add(1, Ordering::Relaxed);
    state
        .app
        .send(Command::Graph(GraphCommand::AddNode {
            id,
            node_type,
            kind,
            position,
        }))
        .map_err(rustic_err)?;
    Ok(id)
}

#[tauri::command]
pub fn graph_remove_node(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::RemoveNode { id }))
        .map_err(rustic_err)
}

#[tauri::command]
pub fn graph_connect(
    from: u64,
    from_port: usize,
    to: u64,
    to_port: usize,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::Connect {
            from,
            from_port,
            to,
            to_port,
        }))
        .map_err(rustic_err)
}

#[tauri::command]
pub fn graph_disconnect(
    from: u64,
    to: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::Disconnect { from, to }))
        .map_err(rustic_err)
}

#[tauri::command]
pub fn graph_start_node(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::StartNode { id }))
        .map_err(rustic_err)
}

#[tauri::command]
pub fn graph_stop_node(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::StopNode { id }))
        .map_err(rustic_err)
}

#[tauri::command]
pub fn graph_set_parameter(
    node_id: u64,
    param_name: String,
    value: f32,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::SetParameter {
            node_id,
            param_name,
            value,
        }))
        .map_err(rustic_err)
}

/// Connect a source as a modulator for a named parameter on another node.
#[tauri::command]
pub fn graph_modulate(
    from: u64,
    to: u64,
    param_name: String,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::Modulate { from, to, param_name }))
        .map_err(rustic_err)
}

/// Remove a modulation wire.
#[tauri::command]
pub fn graph_demodulate(
    from: u64,
    to: u64,
    param_name: String,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::Demodulate { from, to, param_name }))
        .map_err(rustic_err)
}

/// Recompile the current graph topology and hot-swap it into the render thread.
#[tauri::command]
pub fn graph_compile(rustic_state: State<'_, Mutex<RusticState>>) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::Compile))
        .map_err(rustic_err)
}
