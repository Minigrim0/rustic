use egui::{Color32, RichText, Stroke, Ui, Vec2};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;
use crate::widgets::{ButtonGroup, SectionContainer};

/// The Graph Editor tab for visual node-graph instrument building
pub struct GraphEditorTab {
    // Node types
    generator_templates: Vec<String>,
    filter_templates: Vec<String>,
    sink_templates: Vec<String>,

    // Demo nodes
    nodes: Vec<Node>,
    connections: Vec<(usize, usize)>,

    // UI state
    show_help: bool,
    canvas_offset: Vec2,
    _zoom: f32,
    selected_node: Option<usize>,
}

/// A simple node in the graph
struct Node {
    name: String,
    node_type: NodeType,
    position: Vec2,
    params: Vec<(String, f32)>,
}

/// Types of nodes in the graph
#[derive(Clone, Copy, PartialEq)]
enum NodeType {
    Generator,
    Filter,
    Sink,
}

impl NodeType {
    fn color(&self) -> Color32 {
        match self {
            NodeType::Generator => Color32::from_rgb(100, 150, 200), // Blue
            NodeType::Filter => Color32::from_rgb(100, 200, 100),    // Green
            NodeType::Sink => Color32::from_rgb(200, 100, 100),      // Red
        }
    }

    fn name(&self) -> &'static str {
        match self {
            NodeType::Generator => "Generator",
            NodeType::Filter => "Filter",
            NodeType::Sink => "Sink",
        }
    }
}

impl GraphEditorTab {
    /// Create a new graph editor tab
    pub fn new() -> Self {
        // Define placeholder node templates
        let generator_templates = vec![
            "Sine Wave".to_string(),
            "Square Wave".to_string(),
            "Sawtooth Wave".to_string(),
            "Triangle Wave".to_string(),
            "Noise Generator".to_string(),
        ];

        let filter_templates = vec![
            "Low Pass Filter".to_string(),
            "High Pass Filter".to_string(),
            "Band Pass Filter".to_string(),
            "Notch Filter".to_string(),
            "Comb Filter".to_string(),
        ];

        let sink_templates = vec![
            "Audio Output".to_string(),
            "File Output".to_string(),
            "Spectrum Analyzer".to_string(),
        ];

        // Create some example nodes
        let mut nodes = Vec::new();

        // Add a sine wave generator
        nodes.push(Node {
            name: "Sine Wave".to_string(),
            node_type: NodeType::Generator,
            position: Vec2::new(100.0, 100.0),
            params: vec![
                ("Frequency".to_string(), 440.0),
                ("Amplitude".to_string(), 0.5),
            ],
        });

        // Add a low pass filter
        nodes.push(Node {
            name: "Low Pass Filter".to_string(),
            node_type: NodeType::Filter,
            position: Vec2::new(350.0, 150.0),
            params: vec![
                ("Cutoff".to_string(), 1000.0),
                ("Resonance".to_string(), 0.7),
            ],
        });

        // Add an audio output
        nodes.push(Node {
            name: "Audio Output".to_string(),
            node_type: NodeType::Sink,
            position: Vec2::new(600.0, 100.0),
            params: vec![("Volume".to_string(), 0.8)],
        });

        // Add connections between nodes
        let connections = vec![(0, 1), (1, 2)];

        GraphEditorTab {
            generator_templates,
            filter_templates,
            sink_templates,
            nodes,
            connections,
            show_help: false,
            canvas_offset: Vec2::new(0.0, 0.0),
            _zoom: 1.0,
            selected_node: None,
        }
    }

    /// Draw the node palette
    fn draw_palette(&self, ui: &mut Ui) {
        SectionContainer::new("Node Palette")
            .with_frame(true)
            .show(ui, |ui| {
                ui.set_min_width(200.0);

                // Generator nodes
                SectionContainer::new("Generators")
                    .collapsible(&mut true)
                    .with_frame(false)
                    .show(ui, |ui| {
                        for name in &self.generator_templates {
                            // TODO: Implement actual node creation functionality
                            if ui.button(name).clicked() {
                                // This would add a new node in a real implementation
                            }
                        }
                    });

                // Filter nodes
                SectionContainer::new("Filters")
                    .collapsible(&mut true)
                    .with_frame(false)
                    .show(ui, |ui| {
                        for name in &self.filter_templates {
                            // TODO: Implement actual node creation functionality
                            if ui.button(name).clicked() {
                                // This would add a new node in a real implementation
                            }
                        }
                    });

                // Sink nodes
                SectionContainer::new("Sinks")
                    .collapsible(&mut true)
                    .with_frame(false)
                    .show(ui, |ui| {
                        for name in &self.sink_templates {
                            // TODO: Implement actual node creation functionality
                            if ui.button(name).clicked() {
                                // This would add a new node in a real implementation
                            }
                        }
                    });

                ui.separator();

                // Controls section
                SectionContainer::new("Canvas Controls")
                    .with_frame(false)
                    .show(ui, |ui| {
                        ui.label("• Drag nodes to move");
                        ui.label("• Connect nodes via ports");
                        ui.label("• Right-click for context menu");
                    });

                // Help button
                if ButtonGroup::new()
                    .add_button(if self.show_help {
                        "Hide Help"
                    } else {
                        "Show Help"
                    })
                    .fill_width(true)
                    .show(ui)
                    .is_some()
                {
                    // TODO: Implement help toggle functionality
                }
            });
    }

    /// Draw a node on the canvas
    fn draw_node(&self, ui: &mut Ui, node: &Node, is_selected: bool) {
        let node_color = node.node_type.color();
        let node_size = Vec2::new(180.0, 120.0);
        let node_pos = node.position + self.canvas_offset;

        // Node background
        let node_rect = egui::Rect::from_min_size(egui::pos2(node_pos.x, node_pos.y), node_size);

        // Draw node background
        ui.painter().rect(
            node_rect,
            4.0,
            node_color,
            if is_selected {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::new(1.0, Color32::BLACK)
            },
        );

        // Draw node title bar
        let title_rect = egui::Rect::from_min_size(node_rect.min, Vec2::new(node_size.x, 24.0));

        ui.painter().rect(
            title_rect,
            4.0,
            node_color.linear_multiply(0.8),
            Stroke::NONE,
        );

        // Draw node title
        ui.painter().text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            &node.name,
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        // Draw node type
        ui.painter().text(
            egui::pos2(title_rect.min.x + 5.0, title_rect.min.y + 5.0),
            egui::Align2::LEFT_TOP,
            node.node_type.name(),
            egui::FontId::proportional(10.0),
            Color32::from_rgba_premultiplied(255, 255, 255, 180),
        );

        // Draw parameters
        for (i, (name, value)) in node.params.iter().enumerate() {
            let param_y = node_rect.min.y + 30.0 + i as f32 * 20.0;

            // Parameter name
            ui.painter().text(
                egui::pos2(node_rect.min.x + 10.0, param_y),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            // Parameter value
            ui.painter().text(
                egui::pos2(node_rect.max.x - 10.0, param_y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", value),
                egui::FontId::proportional(12.0),
                Color32::LIGHT_YELLOW,
            );
        }

        // TODO: Implement proper drag-and-drop functionality for node parameters

        // Draw connection points based on node type
        if node.node_type != NodeType::Sink {
            // Output port (on right side)
            let out_pos = egui::pos2(node_rect.right(), node_rect.min.y + 60.0);
            ui.painter().circle(
                out_pos,
                6.0,
                Color32::LIGHT_GREEN,
                Stroke::new(1.0, Color32::BLACK),
            );
        }

        if node.node_type != NodeType::Generator {
            // Input port (on left side)
            let in_pos = egui::pos2(node_rect.left(), node_rect.min.y + 60.0);
            ui.painter().circle(
                in_pos,
                6.0,
                Color32::LIGHT_BLUE,
                Stroke::new(1.0, Color32::BLACK),
            );
        }
    }

    /// Draw the canvas with nodes and connections
    fn draw_canvas(&self, ui: &mut Ui) {
        let canvas_rect = ui.available_rect_before_wrap();
        ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        // Draw a grid in the background
        self.draw_grid(ui, canvas_rect);

        // Draw connections
        for (from_idx, to_idx) in &self.connections {
            if *from_idx < self.nodes.len() && *to_idx < self.nodes.len() {
                let from_node = &self.nodes[*from_idx];
                let to_node = &self.nodes[*to_idx];

                // Calculate connection points
                let start_pos = egui::pos2(
                    from_node.position.x + 180.0 + self.canvas_offset.x,
                    from_node.position.y + 60.0 + self.canvas_offset.y,
                );

                let end_pos = egui::pos2(
                    to_node.position.x + self.canvas_offset.x,
                    to_node.position.y + 60.0 + self.canvas_offset.y,
                );

                // Draw a bezier curve for the connection
                self.draw_connection(ui, start_pos, end_pos);
            }
        }

        // Draw nodes
        for (idx, node) in self.nodes.iter().enumerate() {
            self.draw_node(ui, node, Some(idx) == self.selected_node);
        }

        // Show help window if enabled
        if self.show_help {
            self.draw_help_window(ui);
        }
    }

    /// Draw a connection between two points
    fn draw_connection(&self, ui: &mut Ui, start: egui::Pos2, end: egui::Pos2) {
        let control_distance = (end.x - start.x).abs().max(50.0) * 0.5;
        let control1 = egui::pos2(start.x + control_distance, start.y);
        let control2 = egui::pos2(end.x - control_distance, end.y);

        ui.painter().add(egui::Shape::CubicBezier(
            egui::epaint::CubicBezierShape::from_points_stroke(
                [start, control1, control2, end],
                false,
                Color32::TRANSPARENT,
                Stroke::new(2.0, Color32::LIGHT_BLUE),
            ),
        ));
    }

    /// Draw a grid background
    fn draw_grid(&self, ui: &mut Ui, rect: egui::Rect) {
        let grid_color = Color32::from_rgb(50, 50, 50);
        let grid_size = 20.0;

        // Draw vertical grid lines
        for x in (rect.min.x as i32..(rect.max.x as i32)).step_by(grid_size as usize) {
            ui.painter().line_segment(
                [
                    egui::pos2(x as f32, rect.min.y),
                    egui::pos2(x as f32, rect.max.y),
                ],
                Stroke::new(0.5, grid_color),
            );
        }

        // Draw horizontal grid lines
        for y in (rect.min.y as i32..(rect.max.y as i32)).step_by(grid_size as usize) {
            ui.painter().line_segment(
                [
                    egui::pos2(rect.min.x, y as f32),
                    egui::pos2(rect.max.x, y as f32),
                ],
                Stroke::new(0.5, grid_color),
            );
        }
    }

    /// Draw the help window
    fn draw_help_window(&self, ui: &mut Ui) {
        egui::Window::new("Graph Editor Help").show(ui.ctx(), |ui| {
            ui.heading("Node Graph Editor Help");
            ui.separator();

            ui.label("This is a visual editor for creating audio processing graphs.");
            ui.label("You can connect nodes to create complex audio instruments.");

            ui.separator();

            SectionContainer::new("Node Types")
                .with_frame(false)
                .show(ui, |ui| {
                    ui.label(RichText::new("• Generators").color(Color32::from_rgb(100, 150, 200)));
                    ui.label("  Produce audio signals (sine waves, etc.)");
                    ui.label(RichText::new("• Filters").color(Color32::from_rgb(100, 200, 100)));
                    ui.label("  Process incoming audio (low pass, reverb, etc.)");
                    ui.label(RichText::new("• Sinks").color(Color32::from_rgb(200, 100, 100)));
                    ui.label("  Output destinations (speakers, files, etc.)");
                });

            ui.separator();
            ui.label("Note: This is a placeholder implementation.");

            // TODO: Implement actual node graph editing functionality
        });
    }
}

impl Tab for GraphEditorTab {
    fn ui(&mut self, ui: &mut Ui, _app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Graph Editor");
            ui.add_space(10.0);
        });

        ui.separator();

        // Use SectionContainer for the main content area
        SectionContainer::new("")
            .show_title(false)
            .with_frame(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Node palette on the left
                    self.draw_palette(ui);

                    // Canvas with nodes on the right
                    ui.vertical(|ui| {
                        let available_size = ui.available_size();

                        // Draw the canvas with a dark background
                        let frame =
                            egui::Frame::canvas(ui.style()).fill(Color32::from_rgb(30, 30, 30));

                        frame.show(ui, |ui| {
                            ui.set_min_size(available_size);
                            self.draw_canvas(ui);
                        });
                    });
                });
            });
    }
}
