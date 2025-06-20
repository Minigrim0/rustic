use egui::{Color32, RichText, Stroke, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;

/// Graph Editor tab for editing sound parameters using graphs
pub struct GraphEditorTab {
    selected_parameter: Parameter,
    selected_instrument: usize,
    points: Vec<[f64; 2]>,
    is_editing: bool,
    zoom_level: f32,
}

/// Sound parameters that can be edited
#[derive(Debug, Clone, Copy, PartialEq)]
enum Parameter {
    Volume,
    Pitch,
    Filter,
    Reverb,
    Delay,
}

impl GraphEditorTab {
    /// Create a new GraphEditorTab
    pub fn new() -> Self {
        // Create some initial points for visualization
        let initial_points = vec![
            [0.0, 0.5],
            [0.2, 0.8],
            [0.4, 0.4],
            [0.6, 0.6],
            [0.8, 0.7],
            [1.0, 0.5],
        ];

        GraphEditorTab {
            selected_parameter: Parameter::Volume,
            selected_instrument: 0,
            points: initial_points,
            is_editing: false,
            zoom_level: 1.0,
        }
    }

    /// Draw the parameter graph
    fn draw_graph(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        ui.horizontal(|ui| {
            // Left toolbar
            ui.vertical(|ui| {
                ui.heading("Parameters");
                ui.separator();

                // Parameter selection
                ui.label("Parameter:");
                if ui
                    .radio_value(&mut self.selected_parameter, Parameter::Volume, "Volume")
                    .clicked()
                {
                    self.update_parameter_data(Parameter::Volume);
                }

                if ui
                    .radio_value(&mut self.selected_parameter, Parameter::Pitch, "Pitch")
                    .clicked()
                {
                    self.update_parameter_data(Parameter::Pitch);
                }

                if ui
                    .radio_value(&mut self.selected_parameter, Parameter::Filter, "Filter")
                    .clicked()
                {
                    self.update_parameter_data(Parameter::Filter);
                }

                if ui
                    .radio_value(&mut self.selected_parameter, Parameter::Reverb, "Reverb")
                    .clicked()
                {
                    self.update_parameter_data(Parameter::Reverb);
                }

                if ui
                    .radio_value(&mut self.selected_parameter, Parameter::Delay, "Delay")
                    .clicked()
                {
                    self.update_parameter_data(Parameter::Delay);
                }

                ui.separator();

                // Instrument selection
                ui.label("Instrument:");
                ui.add(egui::Slider::new(&mut self.selected_instrument, 0..=10).text("Instrument"));

                if ui.button("Apply to Instrument").clicked() {
                    self.apply_parameter_to_instrument(app_sender);
                }

                ui.separator();

                ui.label("Editing:");
                if ui
                    .button(if self.is_editing {
                        "Finish Editing"
                    } else {
                        "Start Editing"
                    })
                    .clicked()
                {
                    self.is_editing = !self.is_editing;
                }

                if ui.button("Reset Graph").clicked() {
                    self.reset_graph();
                }

                ui.separator();

                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom_level, 0.5..=2.0).text("Zoom"));
            });

            // Main graph view
            ui.vertical(|ui| {
                ui.heading(format!("{:?} Graph", self.selected_parameter));

                let plot = Plot::new("parameter_plot")
                    .height(250.0 * self.zoom_level)
                    .width(400.0 * self.zoom_level)
                    .show_x(true)
                    .show_y(true)
                    .x_axis_label("Time")
                    .y_axis_label("Value")
                    .allow_drag(self.is_editing)
                    .allow_zoom(self.is_editing)
                    .allow_boxed_zoom(self.is_editing)
                    .show_axes([false, false])
                    .legend(egui_plot::Legend::default());

                plot.show(ui, |plot_ui| {
                    // Create the line from our points
                    let line = Line::new(PlotPoints::from(self.points.clone()))
                        .color(self.get_parameter_color())
                        .stroke(Stroke::new(2.0, self.get_parameter_color()))
                        .name(format!("{:?}", self.selected_parameter));

                    plot_ui.line(line);

                    // If editing, allow point manipulation
                    if self.is_editing {
                        if let Some(pointer) = plot_ui.pointer_coordinate() {
                            if plot_ui.plot_clicked() {
                                self.add_point_at([pointer.x, pointer.y]);
                            }
                        }
                    }
                });

                // Display instructions based on edit mode
                if self.is_editing {
                    ui.label(
                        RichText::new("Click on the graph to add points. Drag to move the view.")
                            .color(Color32::LIGHT_BLUE),
                    );
                } else {
                    ui.label(
                        RichText::new("Click 'Start Editing' to modify the graph.")
                            .color(Color32::LIGHT_GRAY),
                    );
                }
            });
        });
    }

    /// Get the color for the current parameter
    fn get_parameter_color(&self) -> Color32 {
        match self.selected_parameter {
            Parameter::Volume => Color32::from_rgb(25, 150, 25), // Green
            Parameter::Pitch => Color32::from_rgb(150, 25, 25),  // Red
            Parameter::Filter => Color32::from_rgb(25, 25, 150), // Blue
            Parameter::Reverb => Color32::from_rgb(150, 150, 25), // Yellow
            Parameter::Delay => Color32::from_rgb(150, 25, 150), // Purple
        }
    }

    /// Update graph data when changing parameters
    fn update_parameter_data(&mut self, parameter: Parameter) {
        // In a real implementation, this would load the correct parameter data
        // For now, we'll just set some default patterns
        match parameter {
            Parameter::Volume => {
                self.points = vec![
                    [0.0, 0.5],
                    [0.2, 0.8],
                    [0.4, 0.4],
                    [0.6, 0.6],
                    [0.8, 0.7],
                    [1.0, 0.5],
                ];
            }
            Parameter::Pitch => {
                self.points = vec![
                    [0.0, 0.5],
                    [0.2, 0.6],
                    [0.4, 0.7],
                    [0.6, 0.5],
                    [0.8, 0.3],
                    [1.0, 0.5],
                ];
            }
            Parameter::Filter => {
                self.points = vec![
                    [0.0, 0.1],
                    [0.2, 0.3],
                    [0.4, 0.5],
                    [0.6, 0.7],
                    [0.8, 0.9],
                    [1.0, 0.7],
                ];
            }
            Parameter::Reverb => {
                self.points = vec![
                    [0.0, 0.3],
                    [0.2, 0.3],
                    [0.4, 0.3],
                    [0.6, 0.5],
                    [0.8, 0.7],
                    [1.0, 0.9],
                ];
            }
            Parameter::Delay => {
                self.points = vec![
                    [0.0, 0.9],
                    [0.2, 0.7],
                    [0.4, 0.5],
                    [0.6, 0.3],
                    [0.8, 0.1],
                    [1.0, 0.0],
                ];
            }
        }
    }

    /// Reset the graph to default values
    fn reset_graph(&mut self) {
        self.update_parameter_data(self.selected_parameter);
    }

    /// Add a point to the graph at the specified position
    fn add_point_at(&mut self, position: [f64; 2]) {
        // Clamp values to reasonable ranges
        let x = position[0].max(0.0).min(1.0);
        let y = position[1].max(0.0).min(1.0);

        // Find insertion point to maintain x-order
        let mut insert_idx = self.points.len();
        for (i, point) in self.points.iter().enumerate() {
            if point[0] > x {
                insert_idx = i;
                break;
            } else if (point[0] - x).abs() < 0.01 {
                // If very close to existing x value, replace that point
                self.points[i] = [x, y];
                return;
            }
        }

        self.points.insert(insert_idx, [x, y]);
    }

    /// Apply the current parameter graph to the selected instrument
    fn apply_parameter_to_instrument(&self, app_sender: &Sender<Commands>) {
        // Convert the graph to appropriate commands for the audio engine
        match self.selected_parameter {
            Parameter::Volume => {
                // Find the average value
                let avg_volume =
                    self.points.iter().map(|p| p[1]).sum::<f64>() / self.points.len() as f64;
                let _ = app_sender.send(Commands::SetVolume(
                    avg_volume as f32,
                    self.selected_instrument as u8,
                ));
            }
            Parameter::Reverb => {
                // Use the maximum value for reverb
                let max_reverb = self.points.iter().map(|p| p[1]).fold(0.0, f64::max);
                let _ = app_sender.send(Commands::Reverb(
                    max_reverb as f32,
                    self.selected_instrument as u8,
                ));
            }
            Parameter::Delay => {
                // Use average value for delay time, max value for feedback
                let avg_value =
                    self.points.iter().map(|p| p[1]).sum::<f64>() / self.points.len() as f64;
                let max_value = self.points.iter().map(|p| p[1]).fold(0.0, f64::max);
                let _ = app_sender.send(Commands::Delay(
                    avg_value as f32,
                    max_value as f32,
                    self.selected_instrument as u8,
                ));
            }
            Parameter::Filter => {
                // Use median value for cutoff, max-min for resonance
                let mut values: Vec<f64> = self.points.iter().map(|p| p[1]).collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let median = if values.len() % 2 == 0 {
                    (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
                } else {
                    values[values.len() / 2]
                };

                let min_val = values.first().unwrap_or(&0.0);
                let max_val = values.last().unwrap_or(&1.0);
                let resonance = max_val - min_val;

                let _ = app_sender.send(Commands::Filter(
                    median as f32,
                    resonance as f32,
                    self.selected_instrument as u8,
                ));
            }
            Parameter::Pitch => {
                // This is just a placeholder - actual pitch modulation would be more complex
                if let Some(avg_pitch) = self.calculate_average_pitch_offset() {
                    if avg_pitch > 0.0 {
                        let _ = app_sender.send(Commands::PitchBendUp(
                            avg_pitch as f32,
                            self.selected_instrument as u8,
                        ));
                    } else if avg_pitch < 0.0 {
                        let _ = app_sender.send(Commands::PitchBendDown(
                            -avg_pitch as f32,
                            self.selected_instrument as u8,
                        ));
                    } else {
                        let _ = app_sender
                            .send(Commands::PitchBendReset(self.selected_instrument as u8));
                    }
                }
            }
        }
    }

    /// Calculate the average pitch offset from the center point (0.5)
    fn calculate_average_pitch_offset(&self) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }

        let sum = self.points.iter().map(|p| p[1] - 0.5).sum::<f64>();
        Some(sum / self.points.len() as f64)
    }
}

impl Tab for GraphEditorTab {
    fn ui(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Graph Editor");
            ui.add_space(10.0);
        });

        ui.separator();

        // Draw the parameter graph editor
        self.draw_graph(ui, app_sender);

        ui.add_space(20.0);

        // Help text
        ui.collapsing("How to use the Graph Editor", |ui| {
            ui.label("The Graph Editor allows you to create custom envelopes and modulation curves for various sound parameters.");
            ui.label("1. Select a parameter type (Volume, Pitch, Filter, etc.)");
            ui.label("2. Click 'Start Editing' to modify the graph");
            ui.label("3. Click on the graph to add control points");
            ui.label("4. Click 'Apply to Instrument' to send the changes to the audio engine");
            ui.label("5. Use 'Reset Graph' to start over");

            ui.add_space(10.0);
            ui.label(RichText::new("Note: Changes will only affect the selected instrument.").strong());
        });
    }
}
