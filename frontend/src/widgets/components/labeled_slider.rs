//! Labeled slider component for standardized numerical input.
//!
//! This component combines a label and slider for consistent UI presentation
//! throughout the application.

use egui::{Slider, Ui};

/// A reusable slider with an associated label
///
/// # Example
///
/// ```
/// use widgets::components::LabeledSlider;
///
/// fn draw_ui(ui: &mut egui::Ui) {
///     let mut value = 50.0;
///
///     if LabeledSlider::new("Volume:", &mut value, 0.0..=100.0)
///         .with_suffix("%")
///         .show(ui)
///         .changed()
///     {
///         // Handle the value change
///         println!("New value: {}", value);
///     }
/// }
/// ```
pub struct LabeledSlider<'a, Num: egui::emath::Numeric> {
    /// The label text displayed before the slider
    label: &'a str,
    /// The value controlled by the slider
    value: &'a mut Num,
    /// The range of allowed values
    range: std::ops::RangeInclusive<Num>,
    /// Optional text to display after the value
    suffix: Option<String>,
    /// Optional width for the label
    label_width: Option<f32>,
    /// Optional width for the slider
    slider_width: Option<f32>,
    /// Whether to clamp the value to the range
    clamp: bool,
    /// Whether to use smart aim
    smart_aim: bool,
    /// Whether the slider is enabled
    enabled: bool,
    /// Custom text to display for the value
    custom_text: Option<String>,
    /// Custom formatting string (instead of a formatter function)
    custom_format: Option<String>,
}

impl<'a, Num: egui::emath::Numeric> LabeledSlider<'a, Num> {
    /// Creates a new labeled slider
    ///
    /// # Arguments
    ///
    /// * `label` - The label text to display
    /// * `value` - Mutable reference to the value controlled by the slider
    /// * `range` - The range of allowed values
    ///
    /// # Returns
    ///
    /// A new `LabeledSlider` instance
    pub fn new(label: &'a str, value: &'a mut Num, range: std::ops::RangeInclusive<Num>) -> Self {
        Self {
            label,
            value,
            range,
            suffix: None,
            label_width: None,
            slider_width: None,
            clamp: true,
            smart_aim: true,
            enabled: true,
            custom_text: None,
            custom_format: None,
        }
    }

    /// Sets a suffix to display after the value
    ///
    /// # Arguments
    ///
    /// * `suffix` - The suffix text
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
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

    /// Sets a fixed width for the slider
    ///
    /// # Arguments
    ///
    /// * `width` - The width in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_slider_width(mut self, width: f32) -> Self {
        self.slider_width = Some(width);
        self
    }

    /// Sets whether to clamp the value to the range
    ///
    /// # Arguments
    ///
    /// * `clamp` - Whether to clamp
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Sets whether to use smart aim
    ///
    /// # Arguments
    ///
    /// * `smart_aim` - Whether to use smart aim
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn smart_aim(mut self, smart_aim: bool) -> Self {
        self.smart_aim = smart_aim;
        self
    }

    /// Sets whether the slider is enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the slider is enabled
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets custom text to display for the slider
    ///
    /// # Arguments
    ///
    /// * `text` - The custom text
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_text(mut self, text: impl Into<String>) -> Self {
        self.custom_text = Some(text.into());
        self
    }

    /// Sets a custom format string for the value
    ///
    /// # Arguments
    ///
    /// * `format` - A format string (e.g., "{:.2}")
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_format(mut self, format: impl Into<String>) -> Self {
        self.custom_format = Some(format.into());
        self
    }

    /// Displays the labeled slider
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI to draw on
    ///
    /// # Returns
    ///
    /// The egui response for the slider
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        ui.horizontal(|ui| {
            if let Some(width) = self.label_width {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(width - ui.available_width());
                    ui.label(self.label);
                });
            } else {
                ui.label(self.label);
            }

            // Apply enabled state using UI instead of directly on the Slider
            if !self.enabled {
                ui.set_enabled(false);
            }

            let mut slider = Slider::new(self.value, self.range)
                .clamp_to_range(self.clamp)
                .smart_aim(self.smart_aim);

            // Set min width for the slider widget through UI instead of on the Slider directly
            let mut ui_width = ui.available_width();
            if let Some(width) = self.slider_width {
                ui_width = width;
            }
            ui.set_min_width(ui_width);

            if let Some(text) = self.custom_text {
                slider = slider.text(text);
            } else if let Some(suffix) = self.suffix {
                slider = slider.suffix(suffix);
            }

            // If a custom formatter is provided, we need to create a new closure
            // that captures the formatter by value but doesn't try to access self.value
            if let Some(format) = &self.custom_format {
                let _format_string = format.clone();
                slider = slider.custom_formatter(move |n, _| format!("{}", n));
            }

            ui.add(slider)
        })
        .inner
    }
}
