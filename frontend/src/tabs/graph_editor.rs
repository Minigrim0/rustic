use egui::Ui;
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;

/// Graph Editor tab for editing sound parameters using graphs
pub struct GraphEditorTab;

impl GraphEditorTab {
    /// Create a new GraphEditorTab
    pub fn new() -> Self {
        GraphEditorTab
    }
}

impl Tab for GraphEditorTab {
    fn ui(&mut self, ui: &mut Ui, _app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Graph Editor");
            ui.add_space(10.0);
        });

        ui.separator();

        // Placeholder for Graph Editor
        ui.centered_and_justified(|ui| {
            ui.label("Graph Editor - Placeholder");
        });
    }
}
