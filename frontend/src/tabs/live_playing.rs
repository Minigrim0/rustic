use egui::{Color32, Grid, RichText, Ui, Vec2};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;

/// Channel data for display in the live playing tab
struct ChannelData {
    number: usize,
    instrument_name: String,
    octave: u8,
    volume: f32,
    is_muted: bool,
    is_linked: bool,
    is_playing: bool,
}

/// Live Playing tab for real-time instrument performance
pub struct LivePlayingTab {
    pub is_enabled: bool,
    channels: Vec<ChannelData>,
    active_notes: Vec<(usize, u8)>, // (channel_index, note_value)
}

impl LivePlayingTab {
    /// Create a new LivePlayingTab with placeholder data
    pub fn new() -> Self {
        // Create placeholder channel data
        let mut channels = Vec::new();

        // Instrument names for placeholder data
        let instrument_names = [
            "Grand Piano",
            "Electric Piano",
            "Organ",
            "Synth Lead",
            "Synth Pad",
            "Strings",
            "Guitar",
            "Bass",
            "Drums",
            "Brass",
            "Flute",
            "Clarinet",
            "Saxophone",
            "Violin",
            "Cello",
            "Harp",
            "Marimba",
            "Vibraphone",
            "Accordion",
            "Voice",
        ];

        // Generate 20 channels with placeholder data
        for i in 0..20 {
            let instrument_idx = i % instrument_names.len();
            channels.push(ChannelData {
                number: i + 1,
                instrument_name: instrument_names[instrument_idx].to_string(),
                octave: 4 + (i % 5) as u8,
                volume: 0.75 + (i as f32 * 0.01),
                is_muted: i % 7 == 0,
                is_linked: i % 2 == 0 && i > 0,
                is_playing: false,
            });
        }

        LivePlayingTab {
            is_enabled: true,
            channels,
            active_notes: Vec::new(),
        }
    }

    /// Simulate note activation for visual feedback
    fn activate_note(&mut self, channel_idx: usize, note_value: u8) {
        if channel_idx < self.channels.len() {
            self.channels[channel_idx].is_playing = true;
            self.active_notes.push((channel_idx, note_value));
        }
    }

    /// Simulate note deactivation
    fn deactivate_note(&mut self, channel_idx: usize, note_value: u8) {
        if channel_idx < self.channels.len() {
            // Remove the note from active notes
            self.active_notes
                .retain(|&(ch, note)| ch != channel_idx || note != note_value);

            // If no more notes are active for this channel, set is_playing to false
            if !self.active_notes.iter().any(|&(ch, _)| ch == channel_idx) {
                self.channels[channel_idx].is_playing = false;
            }
        }
    }

    /// Draw the channel grid
    fn draw_channel_grid(&mut self, ui: &mut Ui) {
        // Calculate number of columns based on available width
        let available_width = ui.available_width();
        let column_width = 380.0; // Width of a single channel column
        let columns_per_row = (available_width / column_width).max(1.0).floor() as usize;

        // Group channels into rows
        for chunk_index in 0..(self.channels.len() + columns_per_row - 1) / columns_per_row {
            ui.horizontal(|ui| {
                let start_idx = chunk_index * columns_per_row;
                let end_idx = (start_idx + columns_per_row).min(self.channels.len());

                for channel_idx in start_idx..end_idx {
                    let channel = &self.channels[channel_idx];

                    // Channel card
                    ui.group(|ui| {
                        ui.set_min_width(column_width);
                        ui.set_max_width(column_width);

                        // Channel header
                        ui.horizontal(|ui| {
                            // Channel number and name with visual playing indicator
                            let bg_color = if channel.is_playing {
                                Color32::from_rgb(70, 120, 70) // Green background when playing
                            } else if channel.is_muted {
                                Color32::from_rgb(120, 70, 70) // Red background when muted
                            } else {
                                Color32::from_rgb(60, 60, 80) // Default dark blue-gray
                            };

                            ui.painter()
                                .rect_filled(ui.min_rect().expand(2.0), 4.0, bg_color);

                            let channel_text =
                                format!("#{}: {}", channel.number, channel.instrument_name);
                            ui.label(RichText::new(channel_text).size(16.0).strong());

                            // Right-aligned mute indicator
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mute_text = if channel.is_muted {
                                        "🔇 Muted"
                                    } else {
                                        "🔊"
                                    };
                                    ui.label(RichText::new(mute_text).color(if channel.is_muted {
                                        Color32::LIGHT_RED
                                    } else {
                                        Color32::LIGHT_GREEN
                                    }));
                                },
                            );
                        });

                        ui.add_space(4.0);

                        // Channel details in a grid
                        Grid::new(format!("channel_grid_{}", channel_idx))
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                // Octave row
                                ui.label("Octave:");
                                ui.label(RichText::new(format!("{}", channel.octave)).strong());
                                ui.end_row();

                                // Volume row
                                ui.label("Volume:");
                                ui.label(RichText::new(format!("{:.2}", channel.volume)).strong());
                                ui.end_row();

                                // Linked status
                                ui.label("Linked:");
                                let linked_text = if channel.is_linked {
                                    RichText::new("Yes").color(Color32::LIGHT_BLUE)
                                } else {
                                    RichText::new("No").color(Color32::GRAY)
                                };
                                ui.label(linked_text);
                                ui.end_row();
                            });

                        // Visual indicator for active playing
                        if channel.is_playing {
                            ui.add_space(4.0);

                            // Activity indicator
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 8.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                rect,
                                4.0,
                                Color32::from_rgb(100, 255, 100), // Bright green for activity
                            );
                        }
                    });

                    ui.add_space(8.0); // Space between channels
                }
            });

            ui.add_space(8.0); // Space between rows
        }
    }
}

impl Tab for LivePlayingTab {
    fn ui(&mut self, ui: &mut Ui, _app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Live Playing");

            // Toggle switch for enabling/disabling live mode
            ui.horizontal(|ui| {
                ui.label("Live Mode:");
                if ui.checkbox(&mut self.is_enabled, "Enabled").changed() {
                    // Notify user of mode change
                    if self.is_enabled {
                        log::info!("Live mode enabled");
                    } else {
                        log::info!("Live mode disabled");
                    }
                }

                ui.add_space(20.0);

                // Display status
                let status_text = if self.is_enabled {
                    RichText::new("LIVE").color(Color32::LIGHT_GREEN).strong()
                } else {
                    RichText::new("DISABLED").color(Color32::LIGHT_RED).strong()
                };
                ui.label(status_text);

                // Keyboard shortcut hint
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Press keys to play notes").color(Color32::LIGHT_GRAY));
                });
            });

            // Information about keyboard controls
            ui.collapsing("Keyboard Controls", |ui| {
                Grid::new("keyboard_controls")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Number Keys (1-0)").strong());
                        ui.label("Play notes on row 1");
                        ui.end_row();

                        ui.label(RichText::new("QWERTY Row").strong());
                        ui.label("Play notes on row 2");
                        ui.end_row();

                        ui.label(RichText::new("Z / X").strong());
                        ui.label("Octave up for row 1 / 2");
                        ui.end_row();

                        ui.label(RichText::new("Shift+Z / Shift+X").strong());
                        ui.label("Octave down for row 1 / 2");
                        ui.end_row();

                        ui.label(RichText::new("Space").strong());
                        ui.label("Play loop");
                        ui.end_row();

                        ui.label(RichText::new("R").strong());
                        ui.label("Start recording");
                        ui.end_row();

                        ui.label(RichText::new("Esc").strong());
                        ui.label("Stop loop/recording");
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
        });

        // Only process keyboard events if the live mode is enabled
        if self.is_enabled {
            // Simulate some activity for demonstration purposes
            // In a real implementation, this would be connected to actual note events

            // Randomly activate some notes (this is just for visual demonstration)
            let time = ui.input(|i| i.time);
            let active_time = (time * 2.0) as usize % 20;

            // Reset all channels first
            for channel in &mut self.channels {
                channel.is_playing = false;
            }

            // Activate a channel based on time
            if active_time < self.channels.len() {
                self.channels[active_time].is_playing = true;
            }
        }

        // Scrollable area for the channel grid
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.draw_channel_grid(ui);
        });
    }
}
