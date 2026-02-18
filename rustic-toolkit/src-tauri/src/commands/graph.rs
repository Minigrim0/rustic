use std::sync::Mutex;
use std::sync::atomic::Ordering;

use crate::{RusticState, error::AppError};
use rustic::app::commands::{GraphCommand, NodeKind};
use rustic::prelude::Command;
use tauri::State;

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
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::AddNode {
        id,
        node_type,
        kind,
        position,
    }))
    .map_err(|_| AppError::ChannelClosed)?;
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
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::RemoveNode { id }))
        .map_err(|_| AppError::ChannelClosed)?;
    Ok(())
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
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::Connect {
        from,
        from_port,
        to,
        to_port,
    }))
    .map_err(|_| AppError::ChannelClosed)?;
    Ok(())
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
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::Disconnect { from, to }))
        .map_err(|_| AppError::ChannelClosed)?;
    Ok(())
}

#[tauri::command]
pub fn graph_start_node(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::StartNode { id }))
        .map_err(|_| AppError::ChannelClosed)?;
    Ok(())
}

#[tauri::command]
pub fn graph_stop_node(
    id: u64,
    rustic_state: State<'_, Mutex<RusticState>>,
) -> Result<(), AppError> {
    let state = rustic_state
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::StopNode { id }))
        .map_err(|_| AppError::ChannelClosed)?;
    Ok(())
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
    let tx = state
        .command_tx
        .try_lock()
        .map_err(|_| AppError::LockPoisoned)?;
    tx.send(Command::Graph(GraphCommand::SetParameter {
        node_id,
        param_name,
        value,
    }))
    .map_err(|_| AppError::ChannelClosed)?;
    Ok(())
}
