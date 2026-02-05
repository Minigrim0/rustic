//! Status message component for displaying feedback to users.
//!
//! This component provides a standardized way to display status messages,
//! warnings, errors, and other feedback to users.

use egui::{Color32, RichText, Ui};

/// Message type for status messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Information message (neutral)
    Info,
    /// Success message (positive)
    Success,
    /// Warning message (caution)
    Warning,
    /// Error message (negative)
    Error,
}

/// A reusable component for displaying status messages
///
/// # Example
///
/// ```
/// use frontend::widgets::{StatusMessage, MessageType};
///
/// fn draw_ui(ui: &mut egui::Ui) {
///     // Display a success message
///     StatusMessage::new("Configuration saved successfully!")
///         .with_type(MessageType::Success)
///         .show(ui);
///
///     // Display an error message
///     if let Some(error) = get_error() {
///         StatusMessage::new(&error)
///             .with_type(MessageType::Error)
///             ._with_dismiss_button(true)
///             .show(ui);
///     }
/// }
///
/// fn get_error() -> Option<String> {
///     // Return an error message if there is one
///     None
/// }
/// ```
pub struct StatusMessage<'a> {
    /// The message text
    message: &'a str,
    /// The message type
    message_type: MessageType,
    /// Whether to show a dismiss button
    dismiss_button: bool,
    /// Whether to show an icon
    show_icon: bool,
    /// Whether to show a background
    show_background: bool,
    /// Optional minimum height
    min_height: Option<f32>,
}

impl<'a> StatusMessage<'a> {
    /// Creates a new status message
    ///
    /// # Arguments
    ///
    /// * `message` - The message text
    ///
    /// # Returns
    ///
    /// A new `StatusMessage` instance
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            message_type: MessageType::Info,
            dismiss_button: false,
            show_icon: true,
            show_background: true,
            min_height: None,
        }
    }

    /// Sets the message type
    ///
    /// # Arguments
    ///
    /// * `message_type` - The message type
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Sets whether to show a dismiss button
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show the button
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_dismiss_button(mut self, show: bool) -> Self {
        self.dismiss_button = show;
        self
    }

    /// Sets whether to show an icon
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show the icon
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    /// Sets whether to show a background
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show the background
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_background(mut self, show: bool) -> Self {
        self.show_background = show;
        self
    }

    /// Sets a minimum height for the message
    ///
    /// # Arguments
    ///
    /// * `height` - The minimum height in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Gets the color for the current message type
    ///
    /// # Returns
    ///
    /// The appropriate color for the message type
    fn get_color(&self) -> Color32 {
        match self.message_type {
            MessageType::Info => Color32::from_rgb(100, 150, 200), // Blue
            MessageType::Success => Color32::from_rgb(100, 200, 100), // Green
            MessageType::Warning => Color32::from_rgb(230, 180, 80), // Amber
            MessageType::Error => Color32::from_rgb(220, 80, 80),  // Red
        }
    }

    /// Gets the background color for the current message type
    ///
    /// # Returns
    ///
    /// The appropriate background color for the message type
    fn get_background_color(&self) -> Color32 {
        match self.message_type {
            MessageType::Info => Color32::from_rgba_premultiplied(100, 150, 200, 40), // Blue with transparency
            MessageType::Success => Color32::from_rgba_premultiplied(100, 200, 100, 40), // Green with transparency
            MessageType::Warning => Color32::from_rgba_premultiplied(230, 180, 80, 40), // Amber with transparency
            MessageType::Error => Color32::from_rgba_premultiplied(220, 80, 80, 40), // Red with transparency
        }
    }

    /// Gets the icon for the current message type
    ///
    /// # Returns
    ///
    /// The appropriate icon for the message type
    fn get_icon(&self) -> &'static str {
        match self.message_type {
            MessageType::Info => "ℹ️",
            MessageType::Success => "✓",
            MessageType::Warning => "⚠️",
            MessageType::Error => "❌",
        }
    }

    /// Displays the status message
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI to draw on
    ///
    /// # Returns
    ///
    /// `true` if the message was dismissed, `false` otherwise
    pub fn show(self, ui: &mut Ui) -> bool {
        // Create a struct to hold the state that needs to be modified in the closure
        struct UiState {
            dismissed: bool,
        }

        let mut state = UiState { dismissed: false };

        // Get the appropriate colors and icon
        let text_color = self.get_color();
        let bg_color = self.get_background_color();
        let icon = self.get_icon();
        let min_height = self.min_height;
        let message = self.message;
        let show_icon = self.show_icon;
        let dismiss_button = self.dismiss_button;

        // Define the content drawing function
        let content = |ui: &mut Ui, state: &mut UiState| {
            let min_height_val = min_height.unwrap_or(0.0);
            ui.horizontal(|ui| {
                ui.set_min_height(min_height_val);

                // Show icon if enabled
                if show_icon {
                    ui.label(RichText::new(icon).color(text_color).size(16.0));
                    ui.add_space(4.0);
                }

                // Show message
                ui.label(RichText::new(message).color(text_color));

                // Add dismiss button if enabled
                if dismiss_button {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            state.dismissed = true;
                        }
                    });
                }
            });
        };

        if self.show_background {
            egui::Frame::none()
                .fill(bg_color)
                .inner_margin(egui::style::Margin::symmetric(8.0, 4.0))
                .rounding(4.0)
                .show(ui, |ui| content(ui, &mut state));
        } else {
            content(ui, &mut state);
        }

        state.dismissed
    }
}
