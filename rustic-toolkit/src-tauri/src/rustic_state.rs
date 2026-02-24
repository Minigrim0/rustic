use std::sync::atomic::AtomicU64;

use rustic::prelude::App;

pub struct RusticState {
    pub app: App,
    pub next_node_id: AtomicU64,
}

impl RusticState {
    pub fn new(app: App) -> Self {
        Self {
            app,
            next_node_id: AtomicU64::new(1),
        }
    }
}
