use egui::{Color32, RichText, Ui};
use rustic::prelude::Commands;
use std::sync::mpsc::Sender;

use super::Tab;
use crate::widgets::{
    ButtonGroup, LabeledCombo, LabeledSlider, MessageType, SectionContainer, StatusMessage,
    prelude::{ThemeChoice, apply_scaling, configure_theme},
};

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
    theme_choices: Vec<ThemeChoice>,
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
        let theme_choices = ThemeChoice::all();

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

            theme_choices,
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
        SectionContainer::new("Audio Settings").show(ui, |ui| {
            // Sample rate selection
            LabeledCombo::new("Sample Rate:", "sample_rate")
                .with_selected_text(format!(
                    "{} Hz",
                    self.sample_rates[self.selected_sample_rate]
                ))
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, &rate) in self.sample_rates.iter().enumerate() {
                        if ui
                            .selectable_label(
                                self.selected_sample_rate == i,
                                format!("{} Hz", rate),
                            )
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;
                        }
                    }
                    result
                });

            // Buffer size selection with latency calculation
            let selected = LabeledCombo::new("Buffer Size:", "buffer_size")
                .with_selected_text(format!(
                    "{} samples",
                    self.buffer_sizes[self.selected_buffer_size]
                ))
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, &size) in self.buffer_sizes.iter().enumerate() {
                        if ui
                            .selectable_label(
                                self.selected_buffer_size == i,
                                format!("{} samples", size),
                            )
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;
                        }
                    }
                    result
                });

            if selected.is_some() {
                self.selected_buffer_size = selected.unwrap();
            }

            // Display latency calculation
            ui.horizontal(|ui| {
                ui.add_space(120.0); // Match label width
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
            LabeledCombo::new("Audio Device:", "audio_device")
                .with_selected_text(&self.audio_devices[self.selected_audio_device])
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, device) in self.audio_devices.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_audio_device == i, device)
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;
                        }
                    }
                    result
                });

            ui.add_space(5.0);

            // Test audio button
            if ui.button("Test Audio").clicked() {
                // TODO: Implement actual audio test tone functionality
                self.save_message = Some("Playing test tone...".to_string());
            }
        });
    }

    /// Draw the musical settings section
    fn draw_musical_settings(&mut self, ui: &mut Ui, app_sender: &Sender<Commands>) {
        SectionContainer::new("Musical Settings").show(ui, |ui| {
            // Time signature selection
            LabeledCombo::new("Time Signature:", "time_signature")
                .with_selected_text(&self.time_signatures[self.selected_time_signature])
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, sig) in self.time_signatures.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_time_signature == i, sig)
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;
                        }
                    }
                    result
                });

            // Key signature selection
            LabeledCombo::new("Key Signature:", "key_signature")
                .with_selected_text(&self.key_signatures[self.selected_key_signature])
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, key) in self.key_signatures.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_key_signature == i, key)
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;
                        }
                    }
                    result
                });

            // Tempo slider
            if LabeledSlider::new("Tempo:", &mut self.tempo, 40.0..=240.0)
                .with_suffix(" BPM")
                .with_label_width(120.0)
                .clamp(true)
                .smart_aim(true)
                .show(ui)
                .changed()
            {
                self.config_dirty = true;
                // TODO: Implement proper error handling for command sending
                let _ = app_sender.send(Commands::SetTempo(self.tempo as u32));
            }

            // Metronome toggle
            if ui.checkbox(&mut true, "Enable Metronome").clicked() {
                // TODO: Implement proper state tracking for metronome toggle
                let _ = app_sender.send(Commands::ToggleMetronome);
                self.config_dirty = true;
            }
        });
    }

    /// Draw the UI settings section
    fn draw_ui_settings(&mut self, ui: &mut Ui) {
        SectionContainer::new("UI Settings").show(ui, |ui| {
            // Theme selection
            LabeledCombo::new("Theme:", "theme")
                .with_selected_text(self.theme_choices[self.selected_theme].as_string())
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, theme) in self.theme_choices.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_theme == i, theme.as_string())
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;

                            // Apply the theme change immediately
                            configure_theme(*theme, ui.ctx());
                        }
                    }
                    result
                });

            // UI scaling
            LabeledCombo::new("UI Scaling:", "scaling")
                .with_selected_text(format!("{}x", self.scaling_factors[self.selected_scaling]))
                .with_label_width(120.0)
                .show_ui(ui, |ui| {
                    let mut result = None;
                    for (i, &scale) in self.scaling_factors.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_scaling == i, format!("{}x", scale))
                            .clicked()
                        {
                            result = Some(i);
                            self.config_dirty = true;

                            // Apply the scaling change immediately
                            apply_scaling(scale, ui.ctx());
                        }
                    }
                    result
                });

            // Keyboard shortcuts section
            ui.add_space(10.0);
            SectionContainer::new("Keyboard Shortcuts")
                .with_frame(false)
                .show(ui, |ui| {
                    // Convert shortcuts to format expected by DataGrid
                    let shortcut_data: Vec<Vec<&str>> = self
                        .shortcuts
                        .iter()
                        .map(|(key, action)| vec![key.as_str(), action.as_str()])
                        .collect();

                    // Display shortcuts in a grid
                    crate::widgets::DataGrid::new("keyboard_shortcuts".to_string())
                        .with_headers(vec!["Key", "Action"])
                        .with_data(shortcut_data)
                        .with_striped(true)
                        .with_col_spacing(40.0)
                        .with_emphasize_headers(true)
                        .show(ui);
                });
        });
    }

    /// Draw the config management section
    fn draw_config_management(&mut self, ui: &mut Ui) {
        SectionContainer::new("Configuration").show(ui, |ui| {
            // Button group for configuration actions
            if let Some((button_index, _)) = ButtonGroup::new()
                .add_button("Save Configuration")
                .add_button("Load Configuration")
                .add_destructive_button("Reset to Defaults")
                .horizontal()
                .with_spacing(10.0)
                .show(ui)
            {
                match button_index {
                    0 => {
                        // Save configuration
                        // TODO: Implement actual configuration saving functionality
                        self.config_saved = true;
                        self.config_dirty = false;
                        self.save_message = Some("Configuration saved successfully!".to_string());
                    }
                    1 => {
                        // Load configuration
                        // TODO: Implement actual configuration loading functionality
                        self.save_message = Some("Configuration loaded successfully!".to_string());
                    }
                    2 => {
                        // Reset to defaults
                        self.selected_sample_rate = 1; // 48000 Hz
                        self.selected_buffer_size = 2; // 256 samples
                        self.selected_audio_device = 0; // System Default
                        self.selected_time_signature = 0; // 4/4
                        self.selected_key_signature = 0; // C Major
                        self.tempo = 120.0;
                        self.selected_theme = 0; // Dark
                        self.selected_scaling = 1; // 1.0x

                        // Apply default theme and scaling
                        configure_theme(ThemeChoice::Dark, ui.ctx());
                        apply_scaling(1.0, ui.ctx());

                        self.config_dirty = true;
                        self.save_message = Some("Settings reset to defaults".to_string());
                    }
                    _ => {}
                }
            }

            ui.add_space(5.0);

            // Show status messages
            if let Some(message) = &self.save_message {
                StatusMessage::new(message)
                    .with_type(MessageType::Success)
                    .show(ui);
            }

            // Show unsaved changes warning
            if self.config_dirty {
                StatusMessage::new("You have unsaved changes")
                    .with_type(MessageType::Warning)
                    .show(ui);
            }
        });
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
            // Two column layout
            ui.columns(2, |columns| {
                // Left column: Audio and Musical settings
                columns[0].vertical(|ui| {
                    self.draw_audio_settings(ui);
                    self.draw_musical_settings(ui, app_sender);
                });

                // Right column: UI settings and Config management
                columns[1].vertical(|ui| {
                    self.draw_ui_settings(ui);
                    self.draw_config_management(ui);
                });
            });

            // Add some space at the bottom for better scrolling
            ui.add_space(20.0);
        });
    }
}
