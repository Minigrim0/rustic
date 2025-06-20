use egui::{Color32, ComboBox, RichText, Ui, Vec2};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;

/// Settings tab for application configuration
pub struct SettingsTab {
    // Audio settings
    sample_rates: Vec<u32>,
    selected_sample_rate: usize,
    buffer_sizes: Vec<u32>,
    selected_buffer_size: usize,
    audio_devices: Vec<String>,
    selected_audio_device: usize,

    // Musical settings
    time_signatures: Vec<String>,
    selected_time_signature: usize,
    key_signatures: Vec<String>,
    selected_key_signature: usize,
    tempo: f32,

    // UI settings
    themes: Vec<String>,
    selected_theme: usize,
    scaling_factors: Vec<f32>,
    selected_scaling: usize,
    shortcuts: Vec<(String, String)>,

    // Config status
    config_dirty: bool,
    config_saved: bool,
    save_message: Option<String>,
}

impl SettingsTab {
    /// Create a new SettingsTab with placeholder data
    pub fn new() -> Self {
        // Audio settings placeholder data
        let sample_rates = vec![44100, 48000, 88200, 96000];
        let buffer_sizes = vec![64, 128, 256, 512, 1024, 2048];
        let audio_devices = vec![
            "System Default".to_string(),
            "Built-in Output".to_string(),
            "External Audio Interface".to_string(),
            "HDMI Output".to_string(),
            "Bluetooth Headphones".to_string(),
        ];

        // Musical settings placeholder data
        let time_signatures = vec![
            "4/4".to_string(),
            "3/4".to_string(),
            "6/8".to_string(),
            "5/4".to_string(),
            "7/8".to_string(),
        ];

        let key_signatures = vec![
            "C Major".to_string(),
            "G Major".to_string(),
            "D Major".to_string(),
            "A Major".to_string(),
            "E Major".to_string(),
            "A Minor".to_string(),
            "E Minor".to_string(),
            "B Minor".to_string(),
        ];

        // UI settings placeholder data
        let themes = vec![
            "Dark".to_string(),
            "Light".to_string(),
            "High Contrast".to_string(),
            "Custom".to_string(),
        ];

        let scaling_factors = vec![0.8, 1.0, 1.2, 1.5, 2.0];

        let shortcuts = vec![
            ("Esc".to_string(), "Cancel/Exit".to_string()),
            ("Ctrl+S".to_string(), "Save Configuration".to_string()),
            ("Ctrl+O".to_string(), "Open Configuration".to_string()),
            ("F1".to_string(), "Show Help".to_string()),
            ("Tab".to_string(), "Switch Tab".to_string()),
            ("Space".to_string(), "Play/Pause".to_string()),
            ("R".to_string(), "Record".to_string()),
            ("M".to_string(), "Toggle Mute".to_string()),
        ];

        SettingsTab {
            sample_rates,
            selected_sample_rate: 1, // 48000 Hz
            buffer_sizes,
            selected_buffer_size: 2, // 256 samples
            audio_devices,
            selected_audio_device: 0, // System Default

            time_signatures,
            selected_time_signature: 0, // 4/4
            key_signatures,
            selected_key_signature: 0, // C Major
            tempo: 120.0,

            themes,
            selected_theme: 0, // Dark
            scaling_factors,
            selected_scaling: 1, // 1.0x
            shortcuts,

            config_dirty: false,
            config_saved: false,
            save_message: None,
        }
    }

    /// Draw the audio settings section
    fn draw_audio_settings(&mut self, ui: &mut Ui) {
        ui.heading("Audio Settings");
        ui.add_space(5.0);

        // Sample rate selection
        ui.horizontal(|ui| {
            ui.label("Sample Rate:");
            ComboBox::from_id_source("sample_rate")
                .selected_text(format!(
                    "{} Hz",
                    self.sample_rates[self.selected_sample_rate]
                ))
                .show_ui(ui, |ui| {
                    for (i, &rate) in self.sample_rates.iter().enumerate() {
                        if ui
                            .selectable_label(
                                self.selected_sample_rate == i,
                                format!("{} Hz", rate),
                            )
                            .clicked()
                        {
                            self.selected_sample_rate = i;
                            self.config_dirty = true;
                        }
                    }
                });
        });

        // Buffer size selection
        ui.horizontal(|ui| {
            ui.label("Buffer Size:");
            ComboBox::from_id_source("buffer_size")
                .selected_text(format!(
                    "{} samples",
                    self.buffer_sizes[self.selected_buffer_size]
                ))
                .show_ui(ui, |ui| {
                    for (i, &size) in self.buffer_sizes.iter().enumerate() {
                        if ui
                            .selectable_label(
                                self.selected_buffer_size == i,
                                format!("{} samples", size),
                            )
                            .clicked()
                        {
                            self.selected_buffer_size = i;
                            self.config_dirty = true;
                        }
                    }
                });

            ui.label(
                RichText::new(format!(
                    "({:.1} ms latency)",
                    self.buffer_sizes[self.selected_buffer_size] as f32 * 1000.0
                        / self.sample_rates[self.selected_sample_rate] as f32
                ))
                .color(Color32::LIGHT_GRAY),
            );
        });

        // Audio device selection
        ui.horizontal(|ui| {
            ui.label("Audio Device:");
            ComboBox::from_id_source("audio_device")
                .selected_text(&self.audio_devices[self.selected_audio_device])
                .show_ui(ui, |ui| {
                    for (i, device) in self.audio_devices.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_audio_device == i, device)
                            .clicked()
                        {
                            self.selected_audio_device = i;
                            self.config_dirty = true;
                        }
                    }
                });
        });

        // Test audio button
        if ui.button("Test Audio").clicked() {
            // This would play a test tone in a real implementation
            self.save_message = Some("Playing test tone...".to_string());
        }
    }

    /// Draw the musical settings section
    fn draw_musical_settings(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        ui.heading("Musical Settings");
        ui.add_space(5.0);

        // Time signature selection
        ui.horizontal(|ui| {
            ui.label("Time Signature:");
            ComboBox::from_id_source("time_signature")
                .selected_text(&self.time_signatures[self.selected_time_signature])
                .show_ui(ui, |ui| {
                    for (i, sig) in self.time_signatures.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_time_signature == i, sig)
                            .clicked()
                        {
                            self.selected_time_signature = i;
                            self.config_dirty = true;
                        }
                    }
                });
        });

        // Key signature selection
        ui.horizontal(|ui| {
            ui.label("Key Signature:");
            ComboBox::from_id_source("key_signature")
                .selected_text(&self.key_signatures[self.selected_key_signature])
                .show_ui(ui, |ui| {
                    for (i, key) in self.key_signatures.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_key_signature == i, key)
                            .clicked()
                        {
                            self.selected_key_signature = i;
                            self.config_dirty = true;
                        }
                    }
                });
        });

        // Tempo slider
        ui.horizontal(|ui| {
            ui.label("Tempo:");
            if ui
                .add(
                    egui::Slider::new(&mut self.tempo, 40.0..=240.0)
                        .text("BPM")
                        .clamp_to_range(true)
                        .smart_aim(true),
                )
                .changed()
            {
                self.config_dirty = true;
                // Send tempo change to audio engine
                let _ = app_sender.send(Commands::SetTempo(self.tempo as u32));
            }
        });

        // Metronome toggle
        if ui.checkbox(&mut true, "Enable Metronome").clicked() {
            // Toggle metronome
            let _ = app_sender.send(Commands::ToggleMetronome);
            self.config_dirty = true;
        }
    }

    /// Draw the UI settings section
    fn draw_ui_settings(&mut self, ui: &mut Ui) {
        ui.heading("UI Settings");
        ui.add_space(5.0);

        // Theme selection
        ui.horizontal(|ui| {
            ui.label("Theme:");
            ComboBox::from_id_source("theme")
                .selected_text(&self.themes[self.selected_theme])
                .show_ui(ui, |ui| {
                    for (i, theme) in self.themes.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_theme == i, theme)
                            .clicked()
                        {
                            self.selected_theme = i;
                            self.config_dirty = true;
                        }
                    }
                });
        });

        // UI scaling
        ui.horizontal(|ui| {
            ui.label("UI Scaling:");
            ComboBox::from_id_source("scaling")
                .selected_text(format!("{}x", self.scaling_factors[self.selected_scaling]))
                .show_ui(ui, |ui| {
                    for (i, &scale) in self.scaling_factors.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_scaling == i, format!("{}x", scale))
                            .clicked()
                        {
                            self.selected_scaling = i;
                            self.config_dirty = true;
                        }
                    }
                });
        });

        // Keyboard shortcuts table
        ui.add_space(10.0);
        ui.label("Keyboard Shortcuts:");
        egui::Grid::new("shortcuts_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (key, action) in &self.shortcuts {
                    ui.label(RichText::new(key).monospace().strong());
                    ui.label(action);
                    ui.end_row();
                }
            });
    }

    /// Draw the config management section
    fn draw_config_management(&mut self, ui: &mut Ui) {
        ui.heading("Configuration");
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            if ui.button("Save Configuration").clicked() {
                // Simulate saving configuration
                self.config_saved = true;
                self.config_dirty = false;
                self.save_message = Some("Configuration saved successfully!".to_string());

                // In a real implementation, this would write to a config file
            }

            if ui.button("Load Configuration").clicked() {
                // Simulate loading configuration
                self.save_message = Some("Configuration loaded successfully!".to_string());

                // In a real implementation, this would read from a config file
            }

            if ui.button("Reset to Defaults").clicked() {
                // Reset to defaults
                self.selected_sample_rate = 1; // 48000 Hz
                self.selected_buffer_size = 2; // 256 samples
                self.selected_audio_device = 0; // System Default
                self.selected_time_signature = 0; // 4/4
                self.selected_key_signature = 0; // C Major
                self.tempo = 120.0;
                self.selected_theme = 0; // Dark
                self.selected_scaling = 1; // 1.0x

                self.config_dirty = true;
                self.save_message = Some("Settings reset to defaults".to_string());
            }
        });

        // Show save status message
        if let Some(message) = &self.save_message {
            ui.horizontal(|ui| {
                ui.label(RichText::new(message).color(Color32::LIGHT_GREEN));
            });
        }

        // Show unsaved changes warning
        if self.config_dirty {
            ui.add_space(5.0);
            ui.label(RichText::new("* You have unsaved changes").color(Color32::LIGHT_YELLOW));
        }
    }
}

impl Tab for SettingsTab {
    fn ui(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        ui.vertical_centered(|ui| {
            ui.heading("Settings");
            ui.add_space(10.0);
        });

        ui.separator();

        // Use a scrollable area for all settings
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Three column layout
            ui.columns(2, |columns| {
                // Left column: Audio and Musical settings
                columns[0].vertical(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(ui.style().visuals.faint_bg_color)
                        .show(ui, |ui| {
                            self.draw_audio_settings(ui);
                        });

                    ui.add_space(10.0);

                    egui::Frame::group(ui.style())
                        .fill(ui.style().visuals.faint_bg_color)
                        .show(ui, |ui| {
                            self.draw_musical_settings(ui, app_sender);
                        });
                });

                // Right column: UI settings and Config management
                columns[1].vertical(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(ui.style().visuals.faint_bg_color)
                        .show(ui, |ui| {
                            self.draw_ui_settings(ui);
                        });

                    ui.add_space(10.0);

                    egui::Frame::group(ui.style())
                        .fill(ui.style().visuals.faint_bg_color)
                        .show(ui, |ui| {
                            self.draw_config_management(ui);
                        });
                });
            });

            // Add some space at the bottom for better scrolling
            ui.add_space(20.0);
        });
    }
}
