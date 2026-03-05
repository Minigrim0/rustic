use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;

use rustic::prelude::App;

pub struct RusticState {
    pub app: App,
    pub next_node_id: AtomicU64,
    /// Generator node IDs currently playing (for compile-state replay).
    pub playing_nodes: Mutex<HashSet<u64>>,
    /// Trigger filter node IDs with gate=1.0 (for compile-state replay).
    pub playing_triggers: Mutex<HashSet<u64>>,
}

impl RusticState {
    pub fn new(app: App) -> Self {
        Self {
            app,
            next_node_id: AtomicU64::new(1),
            playing_nodes: Mutex::new(HashSet::new()),
            playing_triggers: Mutex::new(HashSet::new()),
        }
    }
}
