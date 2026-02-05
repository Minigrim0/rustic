//! Labeled combo box component for standardized dropdown selection.
//!
//! This component combines a label and combo box (dropdown) for consistent UI
//! presentation throughout the application.

use egui::{ComboBox, Ui};

/// A reusable combo box with an associated label
///
/// # Example
///
/// ```
/// use widgets::components::LabeledCombo;
///
/// fn draw_ui(ui: &mut egui::Ui) {
///     let mut selected = 0;
///     let options = vec!["Option 1", "Option 2", "Option 3"];
///
///     let combo = LabeledCombo::new("Settings:", "setting_id")
///         .with_selected_text(&options[selected]);
///
///     if let Some(new_selection) = combo.show_ui(ui, |ui| {
///         let mut result = None;
///         for (i, option) in options.iter().enumerate() {
///             if ui.selectable_label(selected == i, *option).clicked() {
///                 result = Some(i);
///             }
///         }
///         result
///     }) {
///         selected = new_selection;
///         // Do something with the selection
///     }
/// }
/// ```
pub struct LabeledCombo<'a> {
    /// The label text displayed before the combo box
    label: &'a str,
    /// Unique identifier for the combo box
    id_source: &'a str,
    /// Text to display in the combo box when closed
    selected_text: String,
    /// Optional width for the label
    label_width: Option<f32>,
    /// Optional width for the combo box
    combo_width: Option<f32>,
    /// Whether to enable the combo box
    enabled: bool,
}

impl<'a> LabeledCombo<'a> {
    /// Creates a new labeled combo box
    ///
    /// # Arguments
    ///
    /// * `label` - The label text to display
    /// * `id_source` - Unique identifier for this combo box
    ///
    /// # Returns
    ///
    /// A new `LabeledCombo` instance
    pub fn new(label: &'a str, id_source: &'a str) -> Self {
        Self {
            label,
            id_source,
            selected_text: String::new(),
            label_width: None,
            combo_width: None,
            enabled: true,
        }
    }

    /// Sets the text to display when the combo box is closed
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_selected_text(mut self, text: impl Into<String>) -> Self {
        self.selected_text = text.into();
        self
    }

    /// Sets a fixed width for the label
    ///
    /// # Arguments
    ///
    /// * `width` - The width in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_label_width(mut self, width: f32) -> Self {
        self.label_width = Some(width);
        self
    }

    /// Sets a fixed width for the combo box
    ///
    /// # Arguments
    ///
    /// * `width` - The width in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_combo_width(mut self, width: f32) -> Self {
        self.combo_width = Some(width);
        self
    }

    /// Sets whether the combo box is enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the combo box is enabled
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Displays the labeled combo box and handles the dropdown UI
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI to draw on
    /// * `content_ui` - A closure that builds the dropdown content and returns an optional result
    ///
    /// # Returns
    ///
    /// The value returned by the content_ui closure when an option is selected, or None
    pub fn show_ui<R>(
        self,
        ui: &mut Ui,
        content_ui: impl FnOnce(&mut Ui) -> Option<R>,
    ) -> Option<R> {
        let mut result = None;

        ui.horizontal(|ui| {
            if let Some(width) = self.label_width {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(width - ui.available_width());
                    ui.label(self.label);
                });
            } else {
                ui.label(self.label);
            }

            let mut combo =
                ComboBox::from_id_source(self.id_source).selected_text(self.selected_text);

            // Apply enabled state using UI instead of directly on ComboBox
            if !self.enabled {
                ui.set_enabled(false);
            }

            if let Some(width) = self.combo_width {
                combo = combo.width(width);
            }

            combo.show_ui(ui, |ui| {
                if let Some(r) = content_ui(ui) {
                    result = Some(r);
                }
            });
        });

        result
    }
}
