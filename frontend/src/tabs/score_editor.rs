use egui::Ui;
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;

/// Score Editor tab for editing musical scores
pub struct ScoreEditorTab;

impl ScoreEditorTab {
    /// Create a new ScoreEditorTab
    pub fn new() -> Self {
        ScoreEditorTab
    }
}

impl Tab for ScoreEditorTab {
    fn ui(&mut self, ui: &mut Ui, _app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Score Editor");
            ui.add_space(10.0);
        });

        ui.separator();

        // Placeholder for Score Editor
        ui.centered_and_justified(|ui| {
            ui.label("Score Editor - Placeholder");
        });
    }
}
