use egui::{Color32, RichText, Ui};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;

/// Score Editor tab for editing musical scores
pub struct ScoreEditorTab {
    selected_note: Option<(usize, usize)>, // (measure, position)
    selected_instrument: usize,
    zoom_level: f32,
    measures: Vec<Measure>,
    current_note_value: NoteValue,
}

/// A musical measure containing notes
struct Measure {
    notes: Vec<Note>,
    time_signature: (u8, u8), // (numerator, denominator)
}

/// A musical note
struct Note {
    pitch: u8,        // MIDI note number (0-127)
    value: NoteValue, // Duration of the note
    velocity: f32,    // Volume/intensity (0.0-1.0)
    position: f32,    // Position in the measure (0.0-1.0)
}

/// Note duration values
#[derive(Debug, Clone, Copy, PartialEq)]
enum NoteValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl ScoreEditorTab {
    /// Create a new ScoreEditorTab
    pub fn new() -> Self {
        // Create some example measures for visualization
        let mut measures = Vec::new();
        for _ in 0..4 {
            measures.push(Measure {
                notes: Vec::new(),
                time_signature: (4, 4),
            });
        }

        ScoreEditorTab {
            selected_note: None,
            selected_instrument: 0,
            zoom_level: 1.0,
            measures,
            current_note_value: NoteValue::Quarter,
        }
    }

    /// Draw the musical score
    fn draw_score(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left toolbar
            ui.vertical(|ui| {
                ui.heading("Tools");
                ui.separator();

                if ui.button("Add Measure").clicked() {
                    self.measures.push(Measure {
                        notes: Vec::new(),
                        time_signature: (4, 4),
                    });
                }

                ui.label("Note Value:");
                if ui
                    .radio_value(&mut self.current_note_value, NoteValue::Whole, "Whole")
                    .clicked()
                {}
                if ui
                    .radio_value(&mut self.current_note_value, NoteValue::Half, "Half")
                    .clicked()
                {}
                if ui
                    .radio_value(&mut self.current_note_value, NoteValue::Quarter, "Quarter")
                    .clicked()
                {}
                if ui
                    .radio_value(&mut self.current_note_value, NoteValue::Eighth, "Eighth")
                    .clicked()
                {}
                if ui
                    .radio_value(
                        &mut self.current_note_value,
                        NoteValue::Sixteenth,
                        "Sixteenth",
                    )
                    .clicked()
                {}

                ui.separator();

                ui.label("Instrument:");
                ui.add(egui::Slider::new(&mut self.selected_instrument, 0..=10).text("Instrument"));

                ui.separator();

                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom_level, 0.5..=2.0).text("Zoom"));
            });

            // Main score view
            ui.vertical(|ui| {
                ui.heading("Score");

                // Draw the staff
                for (measure_idx, measure) in self.measures.iter().enumerate() {
                    self.draw_measure(ui, measure_idx, measure);
                }

                // Display the current cursor position or selected note
                if let Some((measure, position)) = self.selected_note {
                    ui.label(format!(
                        "Selected: Measure {}, Position {}",
                        measure + 1,
                        position
                    ));
                }
            });
        });
    }

    /// Draw a single measure
    fn draw_measure(&mut self, ui: &mut Ui, measure_idx: usize, measure: &Measure) {
        let (numerator, denominator) = measure.time_signature;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Draw measure number and time signature
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("M{}", measure_idx + 1)).size(16.0));
                    ui.label(RichText::new(format!("{}/{}", numerator, denominator)).size(14.0));
                });

                // Draw the staff lines
                let staff_width = 300.0 * self.zoom_level;
                let staff_height = 100.0 * self.zoom_level;
                let line_height = staff_height / 6.0;

                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();

                // Draw 5 staff lines
                for i in 0..5 {
                    let y = rect.min.y + (i as f32 + 1.0) * line_height;
                    painter.line_segment(
                        [
                            egui::pos2(rect.min.x + 30.0, y),
                            egui::pos2(rect.min.x + staff_width, y),
                        ],
                        egui::Stroke::new(1.0, Color32::GRAY),
                    );
                }

                // Draw measure divider
                painter.line_segment(
                    [
                        egui::pos2(rect.min.x + staff_width, rect.min.y + line_height),
                        egui::pos2(rect.min.x + staff_width, rect.min.y + line_height * 5.0),
                    ],
                    egui::Stroke::new(2.0, Color32::GRAY),
                );

                // Reserve space for the staff
                ui.allocate_space(egui::vec2(staff_width, staff_height));
            });

            // Add note functionality
            if ui.button("Add Note to this Measure").clicked() {
                self.selected_note = Some((measure_idx, 0));
            }
        });
    }

    /// Show controls for editing the score
    fn draw_controls(&self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        ui.horizontal(|ui| {
            if ui.button("Play Score").clicked() {
                // Send command to play the score
                let _ = app_sender.send(Commands::PlaySession);
            }

            if ui.button("Stop").clicked() {
                // Send command to stop playing
                let _ = app_sender.send(Commands::StopSession);
            }

            if ui.button("Save").clicked() {
                // Placeholder for save functionality
            }

            if ui.button("Load").clicked() {
                // Placeholder for load functionality
            }
        });
    }
}

impl Tab for ScoreEditorTab {
    fn ui(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Score Editor");
            ui.add_space(10.0);
        });

        ui.separator();

        // Draw the controls
        self.draw_controls(ui, app_sender);

        ui.separator();

        // Draw the score
        self.draw_score(ui);
    }
}
