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
    // Clean up play-state tracking
    state.playing_nodes.lock().unwrap().remove(&id);
    state.playing_triggers.lock().unwrap().remove(&id);
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
    state.playing_nodes.lock().unwrap().insert(id);
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
    state.playing_nodes.lock().unwrap().remove(&id);
    state
        .app
        .send(Command::Graph(GraphCommand::StopNode { id }))
        .map_err(rustic_err)
}

/// Hard stop — immediately silences a generator node regardless of envelope state.
#[tauri::command]
pub fn graph_kill_node(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state.playing_nodes.lock().unwrap().remove(&id);
    state
        .app
        .send(Command::Graph(GraphCommand::KillNode { id }))
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

/// Trigger a Trigger filter node: sets gate=1.0 and tracks it for compile replay.
#[tauri::command]
pub fn graph_trigger_play(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state.playing_triggers.lock().unwrap().insert(id);
    state
        .app
        .send(Command::Graph(GraphCommand::SetParameter {
            node_id: id,
            param_name: "gate".to_string(),
            value: 1.0,
        }))
        .map_err(rustic_err)
}

/// Release a Trigger filter node: sets gate=0.0 and removes from tracking.
#[tauri::command]
pub fn graph_trigger_stop(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state.playing_triggers.lock().unwrap().remove(&id);
    state
        .app
        .send(Command::Graph(GraphCommand::SetParameter {
            node_id: id,
            param_name: "gate".to_string(),
            value: 0.0,
        }))
        .map_err(rustic_err)
}

/// Kill a Trigger filter node immediately: sets gate=-1.0 and removes from tracking.
#[tauri::command]
pub fn graph_trigger_kill(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state.playing_triggers.lock().unwrap().remove(&id);
    state
        .app
        .send(Command::Graph(GraphCommand::SetParameter {
            node_id: id,
            param_name: "gate".to_string(),
            value: -1.0,
        }))
        .map_err(rustic_err)
}

/// Recompile the current graph topology and hot-swap it into the render thread.
/// Re-starts all generator nodes and re-gates all Trigger nodes that were active
/// before the compile, preserving audio continuity.
#[tauri::command]
pub fn graph_compile(rustic_state: State<'_, Mutex<RusticState>>) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    state
        .app
        .send(Command::Graph(GraphCommand::Compile))
        .map_err(rustic_err)?;

    // Re-start all generator nodes that were playing before the compile
    let playing = state.playing_nodes.lock().unwrap().clone();
    for id in playing {
        state
            .app
            .send(Command::Graph(GraphCommand::StartNode { id }))
            .map_err(rustic_err)?;
    }

    // Re-gate all Trigger nodes that were active before the compile
    let triggers = state.playing_triggers.lock().unwrap().clone();
    for id in triggers {
        state
            .app
            .send(Command::Graph(GraphCommand::SetParameter {
                node_id: id,
                param_name: "gate".to_string(),
                value: 1.0,
            }))
            .map_err(rustic_err)?;
    }

    Ok(())
}
