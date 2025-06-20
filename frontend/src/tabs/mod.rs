mod graph_editor;
mod live_playing;
mod score_editor;

pub use graph_editor::GraphEditorTab;
pub use live_playing::LivePlayingTab;
pub use score_editor::ScoreEditorTab;

use egui::Ui;
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

/// Common interface for all tabs in the application
pub trait Tab {
    /// Display the tab's UI using egui
    fn ui(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>);
}

// Re-export public fields
pub mod exports {
    pub use super::live_playing::LivePlayingTab;
}
