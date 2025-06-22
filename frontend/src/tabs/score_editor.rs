use egui::{Color32, RichText, Stroke, Ui, Vec2};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;
use crate::widgets::{ButtonGroup, DataGrid, LabeledCombo, SectionContainer};

/// Note duration values
#[derive(Debug, Clone, Copy, PartialEq)]
enum NoteValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl NoteValue {
    fn to_string(&self) -> &'static str {
        match self {
            NoteValue::Whole => "Whole",
            NoteValue::Half => "Half",
            NoteValue::Quarter => "Quarter",
            NoteValue::Eighth => "Eighth",
            NoteValue::Sixteenth => "Sixteenth",
        }
    }

    fn to_display_char(&self) -> &'static str {
        match self {
            NoteValue::Whole => "𝅝",     // Unicode for whole note
            NoteValue::Half => "𝅗𝅥",      // Unicode for half note
            NoteValue::Quarter => "𝅘𝅥",   // Unicode for quarter note
            NoteValue::Eighth => "𝅘𝅥𝅮",    // Unicode for eighth note
            NoteValue::Sixteenth => "𝅘𝅥𝅯", // Unicode for sixteenth note
        }
    }
}

/// Clef types for musical staves
#[derive(Debug, Clone, Copy, PartialEq)]
enum Clef {
    Treble,
    Bass,
}

impl Clef {
    fn to_string(&self) -> &'static str {
        match self {
            Clef::Treble => "Treble",
            Clef::Bass => "Bass",
        }
    }

    fn to_display_char(&self) -> &'static str {
        match self {
            Clef::Treble => "𝄞", // Unicode for treble clef
            Clef::Bass => "𝄢",   // Unicode for bass clef
        }
    }
}

/// A note in a measure
#[derive(Debug, Clone)]
struct Note {
    position: (f32, f32), // x, y position within the measure
    value: NoteValue,
    pitch: i8, // MIDI note number, or relative position on staff
}

/// A measure in a staff
#[derive(Debug, Clone)]
struct Measure {
    time_signature: (u8, u8), // numerator, denominator
    notes: Vec<Note>,
}

/// A staff in the score
#[derive(Debug, Clone)]
struct Staff {
    clef: Clef,
    measures: Vec<Measure>,
    instrument_channel: usize, // which instrument/channel this staff is linked to
}

/// Score Editor tab for editing musical scores
pub struct ScoreEditorTab {
    staves: Vec<Staff>,
    selected_note_value: NoteValue,
    current_position: Option<(usize, usize, Vec2)>, // (staff_idx, measure_idx, position)
    selected_note: Option<(usize, usize, usize)>,   // (staff_idx, measure_idx, note_idx)
    zoom_level: f32,
    show_grid: bool,

    // Placeholder instrument names for dropdowns
    instrument_names: Vec<String>,
}

impl ScoreEditorTab {
    /// Create a new ScoreEditorTab with placeholder data
    pub fn new() -> Self {
        // Create some placeholder instrument names
        let instrument_names = vec![
            "Channel 1: Piano".to_string(),
            "Channel 2: Acoustic Guitar".to_string(),
            "Channel 3: Electric Bass".to_string(),
            "Channel 4: Violin".to_string(),
            "Channel 5: Cello".to_string(),
            "Channel 6: Flute".to_string(),
            "Channel 7: Clarinet".to_string(),
            "Channel 8: Trumpet".to_string(),
            "Channel 9: Saxophone".to_string(),
            "Channel 10: Drums".to_string(),
        ];

        // Create initial staves with placeholder data
        let mut staves = Vec::new();

        // Add a treble clef staff with 4 measures
        let mut treble_staff = Staff {
            clef: Clef::Treble,
            measures: Vec::new(),
            instrument_channel: 0, // Piano
        };

        // Add a bass clef staff with 4 measures
        let mut bass_staff = Staff {
            clef: Clef::Bass,
            measures: Vec::new(),
            instrument_channel: 2, // Bass
        };

        // Add 4 measures to each staff
        for _ in 0..4 {
            let measure = Measure {
                time_signature: (4, 4), // 4/4 time
                notes: Vec::new(),
            };
            treble_staff.measures.push(measure.clone());
            bass_staff.measures.push(measure);
        }

        staves.push(treble_staff);
        staves.push(bass_staff);

        ScoreEditorTab {
            staves,
            selected_note_value: NoteValue::Quarter,
            current_position: None,
            selected_note: None,
            zoom_level: 1.0,
            show_grid: true,
            instrument_names,
        }
    }

    /// Draw the toolbar with editing controls
    fn draw_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Note value section
            SectionContainer::new("Note Value")
                .with_frame(true)
                .show(ui, |ui| {
                    // Display note values as buttons with musical symbols
                    ui.horizontal(|ui| {
                        for value in &[
                            NoteValue::Whole,
                            NoteValue::Half,
                            NoteValue::Quarter,
                            NoteValue::Eighth,
                            NoteValue::Sixteenth,
                        ] {
                            let text = RichText::new(value.to_display_char()).size(24.0).color(
                                if *value == self.selected_note_value {
                                    Color32::LIGHT_BLUE
                                } else {
                                    Color32::WHITE
                                },
                            );

                            if ui.button(text).clicked() {
                                self.selected_note_value = *value;
                            }
                        }
                    });
                });

            ui.separator();

            // Staff controls section
            SectionContainer::new("Staff")
                .with_frame(true)
                .show(ui, |ui| {
                    if let Some((button_index, _)) = ButtonGroup::new()
                        .add_button("Add Treble Staff")
                        .add_button("Add Bass Staff")
                        .add_button("Remove Staff")
                        .horizontal()
                        .with_spacing(8.0)
                        .show(ui)
                    {
                        match button_index {
                            0 => self.add_staff(Clef::Treble),
                            1 => self.add_staff(Clef::Bass),
                            2 => {
                                if !self.staves.is_empty() {
                                    self.staves.pop();
                                }
                            }
                            _ => {}
                        }
                    }
                });

            ui.separator();

            // View controls section
            SectionContainer::new("View")
                .with_frame(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.show_grid, "Show Grid");

                    ui.add(
                        egui::Slider::new(&mut self.zoom_level, 0.5..=2.0)
                            .text("Zoom")
                            .fixed_decimals(1),
                    );

                    // TODO: Implement proper zoom functionality that affects all elements consistently
                });
        });
    }

    /// Add a new staff with the specified clef
    fn add_staff(&mut self, clef: Clef) {
        let mut staff = Staff {
            clef,
            measures: Vec::new(),
            instrument_channel: 0,
        };

        // Add the same number of measures as existing staves or at least 4
        let measure_count = if self.staves.is_empty() {
            4
        } else {
            self.staves[0].measures.len()
        };

        for _ in 0..measure_count {
            staff.measures.push(Measure {
                time_signature: (4, 4),
                notes: Vec::new(),
            });
        }

        self.staves.push(staff);
    }

    /// Draw the list of staves with their instrument assignments
    fn draw_staff_list(&mut self, ui: &mut Ui) {
        SectionContainer::new("Staves")
            .with_frame(true)
            .show(ui, |ui| {
                ui.set_min_width(200.0);

                for (idx, staff) in self.staves.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Staff {}: {}", idx + 1, staff.clef.to_string()));

                        // Instrument dropdown (placeholder) using LabeledCombo
                        LabeledCombo::new("", format!("staff_instrument_{}", idx).as_str())
                            .with_selected_text(&self.instrument_names[staff.instrument_channel])
                            .show_ui(ui, |ui| {
                                let mut result = None;
                                for (i, name) in self.instrument_names.iter().enumerate() {
                                    if ui
                                        .selectable_label(
                                            staff.instrument_channel == i,
                                            name.clone(),
                                        )
                                        .clicked()
                                    {
                                        result = Some(i);
                                    }
                                }
                                result
                            });

                        // TODO: Implement proper instrument assignment functionality
                    });
                }
            });
    }

    // These functions are now integrated into the ui method to avoid mutable borrow issues

    /// Convert a vertical position on the staff to a pitch value
    fn position_to_pitch(&self, y_position: f32, clef: Clef) -> i8 {
        // This is a simplified mapping of staff position to MIDI note
        // In a real implementation, this would be more sophisticated

        // Scale y from 0.0-1.0 to a pitch value
        // Quantize to the nearest staff line or space
        let raw_pitch = ((1.0 - y_position) * 16.0).round() as i8;

        // Apply clef-specific offset
        match clef {
            Clef::Treble => raw_pitch + 60, // Middle C and up
            Clef::Bass => raw_pitch + 48,   // Lower range
        }
    }
}

impl Tab for ScoreEditorTab {
    fn ui(&mut self, ui: &mut Ui, _app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Score Editor");
            ui.add_space(8.0);
        });

        // Draw the toolbar
        self.draw_toolbar(ui);

        ui.separator();

        // Main content area with staff list and score editor
        ui.horizontal(|ui| {
            // Staff list with instrument assignment
            self.draw_staff_list(ui);

            // Score editor wrapped in a section container
            SectionContainer::new("Score")
                .show_title(false)
                .with_frame(true)
                .show(ui, |ui| {
                    // Clone the staves data to avoid mutable borrow issues
                    let staves_clone = self.staves.clone();
                    let zoom_level = self.zoom_level;
                    let selected_note_value = self.selected_note_value;
                    let show_grid = self.show_grid;

                    egui::ScrollArea::both().show(ui, |ui| {
                        for staff_idx in 0..staves_clone.len() {
                            ui.group(|ui| {
                                let staff = &staves_clone[staff_idx];

                                // Draw staff with individual components
                                let staff_height = 100.0 * zoom_level;
                                let clef_width = 40.0 * zoom_level;

                                ui.horizontal(|ui| {
                                    // Draw the clef
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(clef_width, staff_height),
                                        egui::Sense::hover(),
                                    );

                                    // Draw staff lines
                                    for i in 0..5 {
                                        let y =
                                            rect.min.y + (i as f32 + 1.0) * (staff_height / 6.0);
                                        ui.painter().line_segment(
                                            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                                            Stroke::new(1.0, Color32::GRAY),
                                        );
                                    }

                                    // Draw clef symbol
                                    ui.painter().text(
                                        egui::pos2(
                                            rect.min.x + 10.0,
                                            rect.min.y + staff_height / 2.0,
                                        ),
                                        egui::Align2::CENTER_CENTER,
                                        staff.clef.to_display_char(),
                                        egui::FontId::proportional(32.0 * zoom_level),
                                        Color32::WHITE,
                                    );

                                    // Draw measures
                                    for (measure_idx, measure) in staff.measures.iter().enumerate()
                                    {
                                        let measure_width = 160.0 * zoom_level;

                                        ui.group(|ui| {
                                            let (rect, response) = ui.allocate_exact_size(
                                                Vec2::new(measure_width, staff_height),
                                                egui::Sense::click_and_drag(),
                                            );

                                            // Handle clicks for note placement
                                            if response.clicked() {
                                                let pos = response.interact_pointer_pos().unwrap();
                                                let local_pos = Vec2::new(
                                                    (pos.x - rect.min.x) / measure_width,
                                                    (pos.y - rect.min.y) / staff_height,
                                                );

                                                // Store the click for later processing in the outer scope
                                                self.current_position =
                                                    Some((staff_idx, measure_idx, local_pos));
                                            }

                                            // Draw staff lines
                                            for i in 0..5 {
                                                let y = rect.min.y
                                                    + (i as f32 + 1.0) * (staff_height / 6.0);
                                                ui.painter().line_segment(
                                                    [
                                                        egui::pos2(rect.min.x, y),
                                                        egui::pos2(rect.max.x, y),
                                                    ],
                                                    Stroke::new(1.0, Color32::GRAY),
                                                );
                                            }

                                            // Draw measure number and time signature
                                            let (num, denom) = measure.time_signature;
                                            ui.painter().text(
                                                egui::pos2(
                                                    rect.min.x + 10.0,
                                                    rect.min.y + 15.0 * zoom_level,
                                                ),
                                                egui::Align2::LEFT_TOP,
                                                format!("{}. {}/{}", measure_idx + 1, num, denom),
                                                egui::FontId::proportional(14.0 * zoom_level),
                                                Color32::LIGHT_GRAY,
                                            );

                                            // Draw key signature placeholder (C major)
                                            ui.painter().text(
                                                egui::pos2(
                                                    rect.min.x + 50.0 * zoom_level,
                                                    rect.min.y + 15.0 * zoom_level,
                                                ),
                                                egui::Align2::LEFT_TOP,
                                                "C major",
                                                egui::FontId::proportional(14.0 * zoom_level),
                                                Color32::LIGHT_GRAY,
                                            );

                                            // Draw grid if enabled
                                            if show_grid {
                                                // Vertical grid lines
                                                for i in 1..4 {
                                                    let x = rect.min.x
                                                        + (i as f32 * measure_width / 4.0);
                                                    ui.painter().line_segment(
                                                        [
                                                            egui::pos2(x, rect.min.y),
                                                            egui::pos2(x, rect.max.y),
                                                        ],
                                                        Stroke::new(
                                                            0.5,
                                                            Color32::from_rgb(70, 70, 70),
                                                        ),
                                                    );
                                                }
                                            }

                                            // Draw measure end bar line
                                            ui.painter().line_segment(
                                                [
                                                    egui::pos2(rect.max.x - 1.0, rect.min.y),
                                                    egui::pos2(rect.max.x - 1.0, rect.max.y),
                                                ],
                                                Stroke::new(2.0, Color32::GRAY),
                                            );

                                            // Draw notes
                                            for note in &measure.notes {
                                                let x =
                                                    rect.min.x + note.position.0 * measure_width;
                                                let y = rect.min.y + note.position.1 * staff_height;

                                                // Draw the note
                                                ui.painter().text(
                                                    egui::pos2(x, y),
                                                    egui::Align2::CENTER_CENTER,
                                                    note.value.to_display_char(),
                                                    egui::FontId::proportional(24.0 * zoom_level),
                                                    Color32::WHITE,
                                                );
                                            }
                                        });
                                    }
                                });
                            });
                            ui.add_space(20.0 * zoom_level);
                        }

                        ui.allocate_space(ui.available_size()); // Use all available space
                    });

                    // Process any clicks that happened
                    if let Some((staff_idx, measure_idx, pos)) = self.current_position.take() {
                        if staff_idx < self.staves.len()
                            && measure_idx < self.staves[staff_idx].measures.len()
                        {
                            // Add a note at the clicked position
                            let staff_clef = self.staves[staff_idx].clef;
                            let pitch = self.position_to_pitch(pos.y, staff_clef);

                            // TODO: Implement note collision detection and proper placement
                            self.staves[staff_idx].measures[measure_idx]
                                .notes
                                .push(Note {
                                    position: (pos.x, pos.y),
                                    value: selected_note_value,
                                    pitch,
                                });
                        }
                    }
                });
        });
    }
}
