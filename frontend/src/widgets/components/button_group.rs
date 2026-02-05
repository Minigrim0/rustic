//! Button group component for standardized action buttons.
//!
//! This component provides a consistent way to display a group of related
//! buttons with proper spacing and layout.

use egui::{Response, Ui};

/// A group of buttons with consistent styling and layout
///
/// # Example
///
/// ```
/// use frontend::widgets::ButtonGroup;
///
/// fn draw_ui(ui: &mut egui::Ui) {
///     let response = ButtonGroup::new()
///         .add_button("Save")
///         .add_button("Cancel")
///         .add_button("Reset")
///         .horizontal()
///         .show(ui);
///
///     if let Some((button_index, _)) = response {
///         match button_index {
///             0 => println!("Save clicked"),
///             1 => println!("Cancel clicked"),
///             2 => println!("Reset clicked"),
///             _ => {}
///         }
///     }
/// }
/// ```
pub struct ButtonGroup<'a> {
    /// The buttons to display
    buttons: Vec<ButtonConfig<'a>>,
    /// Whether to display the buttons horizontally
    horizontal: bool,
    /// Spacing between buttons
    spacing: f32,
    /// Minimum button width
    min_width: Option<f32>,
    /// Whether to fill the available width
    fill_width: bool,
    /// Whether all buttons are enabled
    enabled: bool,
}

/// Configuration for a single button
struct ButtonConfig<'a> {
    /// The text to display on the button
    text: &'a str,
    /// Optional tooltip text
    tooltip: Option<&'a str>,
    /// Whether the button is primary (emphasized)
    primary: bool,
    /// Whether the button is destructive (usually red)
    destructive: bool,
    /// Whether the button is enabled
    enabled: bool,
}

impl<'a> ButtonGroup<'a> {
    /// Creates a new empty button group
    ///
    /// # Returns
    ///
    /// A new `ButtonGroup` instance
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            horizontal: false,
            spacing: 8.0,
            min_width: None,
            fill_width: false,
            enabled: true,
        }
    }

    /// Adds a button to the group
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display on the button
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_button(mut self, text: &'a str) -> Self {
        self.buttons.push(ButtonConfig {
            text,
            tooltip: None,
            primary: false,
            destructive: false,
            enabled: true,
        });
        self
    }

    /// Adds a primary (emphasized) button to the group
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display on the button
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _add_primary_button(mut self, text: &'a str) -> Self {
        self.buttons.push(ButtonConfig {
            text,
            tooltip: None,
            primary: true,
            destructive: false,
            enabled: true,
        });
        self
    }

    /// Adds a destructive (usually red) button to the group
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display on the button
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_destructive_button(mut self, text: &'a str) -> Self {
        self.buttons.push(ButtonConfig {
            text,
            tooltip: None,
            primary: false,
            destructive: true,
            enabled: true,
        });
        self
    }

    /// Adds a button with a tooltip to the group
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display on the button
    /// * `tooltip` - The tooltip text
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _add_button_with_tooltip(mut self, text: &'a str, tooltip: &'a str) -> Self {
        self.buttons.push(ButtonConfig {
            text,
            tooltip: Some(tooltip),
            primary: false,
            destructive: false,
            enabled: true,
        });
        self
    }

    /// Sets whether to display the buttons horizontally
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Sets whether to display the buttons vertically
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _vertical(mut self) -> Self {
        self.horizontal = false;
        self
    }

    /// Sets the spacing between buttons
    ///
    /// # Arguments
    ///
    /// * `spacing` - The spacing in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets a minimum width for all buttons
    ///
    /// # Arguments
    ///
    /// * `width` - The minimum width in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Sets whether the buttons should fill the available width
    ///
    /// # Arguments
    ///
    /// * `fill` - Whether to fill the width
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn fill_width(mut self, fill: bool) -> Self {
        self.fill_width = fill;
        self
    }

    /// Sets whether all buttons are enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the buttons are enabled
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Displays the button group
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI to draw on
    ///
    /// # Returns
    ///
    /// Option containing the index of the clicked button and its response, or None if no button was clicked
    pub fn show(self, ui: &mut Ui) -> Option<(usize, Response)> {
        let mut clicked_button = None;

        let layout_fn = |ui: &mut Ui| {
            for (index, button_config) in self.buttons.iter().enumerate() {
                let enabled = self.enabled && button_config.enabled;

                // Create the button
                let mut button = egui::Button::new(button_config.text);

                // Apply styling based on button type
                if button_config.primary {
                    // Use accent color for primary buttons
                    button = button.fill(ui.style().visuals.selection.bg_fill);
                } else if button_config.destructive {
                    // Use red color for destructive buttons
                    button = button.fill(egui::Color32::from_rgb(200, 60, 60));
                }

                // Apply sizing
                if self.fill_width {
                    button = button.min_size(egui::vec2(ui.available_width(), 0.0));
                } else if let Some(min_width) = self.min_width {
                    button = button.min_size(egui::vec2(min_width, 0.0));
                }

                // Add the button with enabled state
                let mut response = ui.add_enabled(enabled, button);

                // Add tooltip if specified
                if let Some(tooltip_text) = button_config.tooltip {
                    response = response.on_hover_text(tooltip_text);
                }

                // Check if button was clicked
                if response.clicked() {
                    clicked_button = Some((index, response));
                }

                // Add spacing between buttons if not the last one
                if self.horizontal && index < self.buttons.len() - 1 {
                    ui.add_space(self.spacing);
                }
            }
        };

        // Choose layout direction
        if self.horizontal {
            ui.horizontal(layout_fn);
        } else {
            ui.vertical(layout_fn);
        }

        clicked_button
    }
}

impl<'a> Default for ButtonGroup<'a> {
    fn default() -> Self {
        Self::new()
    }
}
