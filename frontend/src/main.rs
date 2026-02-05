use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::{App, CreationContext, Frame, NativeOptions};
use egui::Context;
use log::info;
use rustic::AudioHandle;
use rustic::audio::{AudioError, BackendEvent};
use rustic::prelude::Commands;

mod mapping;
mod tabs;
mod widgets;

use mapping::KeyMapper;
use tabs::{GraphEditorTab, LivePlayingTab, ScoreEditorTab, SettingsTab, Tab};
use widgets::{SectionContainer, ThemeChoice, configure_theme};

/// Main application state that integrates with egui
pub struct RusticApp {
    // Tab state
    current_tab: usize,
    tabs: Vec<&'static str>,
    live_playing_tab: LivePlayingTab,
    score_editor_tab: ScoreEditorTab,
    graph_editor_tab: GraphEditorTab,
    settings_tab: SettingsTab,

    // Rustic audio engine communication
    app_sender: Sender<Commands>,
    app_receiver: Receiver<BackendEvent>,
    _rustic_apphandle: AudioHandle,

    // Input state
    device_state: DeviceState,
    pressed_keys: Vec<Keycode>,
    focused: bool,
    key_mapper: KeyMapper,
}

impl RusticApp {
    /// Create a new instance of the app
    fn new(cc: &CreationContext) -> Result<Self, AudioError> {
        info!("Building application structure");
        // Set up the custom theme
        let ctx = &cc.egui_ctx;
        configure_theme(ThemeChoice::Dark, ctx);

        // Set up communication channels with the rustic audio engine
        let (frontend_sender, backend_receiver): (Sender<Commands>, Receiver<Commands>) =
            mpsc::channel();
        let (backend_sender, frontend_receiver): (Sender<BackendEvent>, Receiver<BackendEvent>) =
            mpsc::channel();

        // Start the rustic audio engine
        let rustic_apphandle = rustic::start_app(backend_sender, backend_receiver)?;

        // Create and return the app
        Ok(RusticApp {
            current_tab: 0,
            tabs: vec!["Live Playing", "Score Editor", "Graph Editor", "Settings"],
            live_playing_tab: LivePlayingTab::new(),
            score_editor_tab: ScoreEditorTab::new(),
            graph_editor_tab: GraphEditorTab::new(),
            settings_tab: SettingsTab::new(),

            app_sender: frontend_sender.clone(),
            app_receiver: frontend_receiver,
            _rustic_apphandle: rustic_apphandle,

            device_state: DeviceState::new(),
            pressed_keys: Vec::new(),
            focused: true,
            key_mapper: KeyMapper::new(frontend_sender),
        })
    }

    /// Process keyboard input and convert to Commands
    fn process_keyboard_input(&mut self) {
        // Only process keyboard input if the app is focused
        if !self.focused {
            return;
        }

        // Only process keyboard input if we're on the Live Playing tab
        if self.current_tab != 0 {
            return;
        }

        // Get the current pressed keys
        let keys = self.device_state.get_keys();

        // Process newly pressed keys (keys that are in the current set but weren't in the previous set)
        for key in &keys {
            if !self.pressed_keys.contains(key) {
                // Convert key press to command and send it
                let shift_pressed =
                    keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift);
                if let Some(command) = self.key_mapper.map_key(key, false, shift_pressed) {
                    log::debug!("Sending command from key press: {:?}", command);
                    self.key_mapper.send_command(command);
                }
            }
        }

        // Process released keys (keys that were in the previous set but aren't in the current set)
        for key in &self.pressed_keys {
            if !keys.contains(key) {
                // Convert key release to command and send it
                if let Some(command) = self.key_mapper.map_key(key, true, false) {
                    log::debug!("Sending command from key release: {:?}", command);
                    self.key_mapper.send_command(command);
                }
            }
        }

        // Update the stored pressed keys
        self.pressed_keys = keys;
    }
}

impl App for RusticApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Process keyboard input
        self.process_keyboard_input();

        // Check if the window is focused
        self.focused = ctx.input(|i| i.focused);

        // Check for any messages from the rustic audio engine
        if let Ok(command) = self.app_receiver.try_recv() {
            log::debug!("Received command from rustic: {:?}", command);
            // Handle commands from the rustic audio engine as needed
        }

        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            SectionContainer::new("Tabs")
                .show_title(false)
                .with_frame(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (index, tab_name) in self.tabs.iter().enumerate() {
                            if ui
                                .selectable_label(self.current_tab == index, *tab_name)
                                .clicked()
                            {
                                self.current_tab = index;
                            }
                        }
                    });
                });
        });

        // Main content area for the selected tab
        egui::CentralPanel::default().show(ctx, |ui| {
            SectionContainer::new("Content")
                .show_title(false)
                .with_frame(false)
                .show(ui, |ui| {
                    match self.current_tab {
                        0 => {
                            // Live Playing Tab
                            self.live_playing_tab.ui(ui, &self.app_sender);

                            // Check if the tab is enabled for keyboard input
                            // (This is for informational purposes - actual enabling/disabling happens in the tab UI)
                            if !self.focused {
                                ui.label("Window not focused - keyboard input disabled");
                            }
                        }
                        1 => {
                            // Score Editor Tab
                            self.score_editor_tab.ui(ui, &self.app_sender);
                        }
                        2 => {
                            // Graph Editor Tab
                            self.graph_editor_tab.ui(ui, &self.app_sender);
                        }
                        3 => {
                            // Settings Tab
                            self.settings_tab.ui(ui, &self.app_sender);
                        }
                        _ => {
                            // Fallback
                            ui.heading("Unknown Tab");
                        }
                    }
                });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logger
    colog::init();
    log::info!("Starting Rustic frontend with egui");

    // Set up the native options
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("Rustic Audio Workstation"),
        follow_system_theme: false, // We'll handle theming ourselves
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };


    // Run the app
    eframe::run_native(
        "Rustic",
        options,
        Box::new(|cc| {
            Box::new(RusticApp::new(cc).unwrap())
        }),
    )
}
