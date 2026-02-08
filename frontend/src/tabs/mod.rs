mod graph_editor;
mod live_playing;
mod score_editor;
mod settings;

pub use graph_editor::GraphEditorTab;
pub use live_playing::LivePlayingTab;
pub use score_editor::ScoreEditorTab;
pub use settings::SettingsTab;

use egui::Ui;
use rustic::app::commands::Command;
use std::sync::mpsc::Sender;

/// Common interface for all tabs in the application
pub trait Tab {
    /// Display the tab's UI using egui
    fn ui(&mut self, ui: &mut Ui, app_sender: &Sender<Command>);
}

// Re-export public fields
pub mod exports {}
