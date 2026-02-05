//! Section container component for consistent grouping of UI elements.
//!
//! This component provides a standardized way to create visually distinct sections
//! within the UI with proper styling and layout.

use egui::{Frame, Ui};

/// A container for a section of the UI with consistent styling
///
/// # Example
///
/// ```
/// use widgets::components::SectionContainer;
///
/// fn draw_ui(ui: &mut egui::Ui) {
///     SectionContainer::new("Audio Settings")
///         .show(ui, |ui| {
///             ui.label("Sample Rate:");
///             ui.add(egui::Slider::new(&mut 48000, 44100..=96000));
///         });
/// }
/// ```
pub struct SectionContainer<'a> {
    /// The title of the section
    title: &'a str,
    /// Optional subtitle
    subtitle: Option<String>,
    /// Whether to show the title
    show_title: bool,
    /// Spacing after the title
    title_spacing: f32,
    /// Whether to use a frame
    with_frame: bool,
    /// Whether to add a bottom margin
    with_bottom_margin: bool,
    /// Optional collapsible state
    collapsible: Option<&'a mut bool>,
    /// Whether the section is enabled
    enabled: bool,
}

impl<'a> SectionContainer<'a> {
    /// Creates a new section container
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the section
    ///
    /// # Returns
    ///
    /// A new `SectionContainer` instance
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            show_title: true,
            title_spacing: 5.0,
            with_frame: true,
            with_bottom_margin: true,
            collapsible: None,
            enabled: true,
        }
    }

    /// Sets a subtitle for the section
    ///
    /// # Arguments
    ///
    /// * `subtitle` - The subtitle text
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets whether to show the title
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show the title
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    /// Sets the spacing after the title
    ///
    /// # Arguments
    ///
    /// * `spacing` - The spacing in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_title_spacing(mut self, spacing: f32) -> Self {
        self.title_spacing = spacing;
        self
    }

    /// Sets whether to use a frame
    ///
    /// # Arguments
    ///
    /// * `with_frame` - Whether to use a frame
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_frame(mut self, with_frame: bool) -> Self {
        self.with_frame = with_frame;
        self
    }

    /// Sets whether to add a bottom margin
    ///
    /// # Arguments
    ///
    /// * `with_margin` - Whether to add a bottom margin
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _with_bottom_margin(mut self, with_margin: bool) -> Self {
        self.with_bottom_margin = with_margin;
        self
    }

    /// Makes the section collapsible
    ///
    /// # Arguments
    ///
    /// * `open` - Mutable reference to a boolean that tracks the open/closed state
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn collapsible(mut self, open: &'a mut bool) -> Self {
        self.collapsible = Some(open);
        self
    }

    /// Sets whether the section is enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the section is enabled
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn _enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Displays the section container with the given content
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI to draw on
    /// * `add_contents` - A closure that adds the content to the section
    ///
    /// # Returns
    ///
    /// `true` if the section is expanded (or not collapsible), `false` otherwise
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> bool {
        // Default to expanded for non-collapsible sections
        let mut is_expanded = true;

        // Handle title display with potential collapsing header
        if self.show_title {
            if let Some(open) = self.collapsible {
                // Start with current open state
                is_expanded = *open;

                // Check if header was clicked
                let clicked = ui
                    .collapsing(self.title, |_| {})
                    .header_response
                    .clicked_by(egui::PointerButton::Primary);

                // Toggle open state on click
                if clicked {
                    *open = !*open;
                    is_expanded = *open;
                }
            } else {
                ui.heading(self.title);
            }

            if let Some(subtitle) = &self.subtitle {
                ui.label(subtitle);
            }

            ui.add_space(self.title_spacing);
        }

        // Only show contents if expanded (or not collapsible)
        if is_expanded {
            let inner_ui = |ui: &mut Ui| {
                if self.enabled {
                    add_contents(ui);
                } else {
                    ui.group(|ui| {
                        ui.set_enabled(false);
                        add_contents(ui);
                    });
                }
            };

            // Choose whether to use a frame or not
            if self.with_frame {
                Frame::group(ui.style())
                    .fill(ui.style().visuals.faint_bg_color)
                    .show(ui, inner_ui);
            } else {
                inner_ui(ui);
            }

            // Add bottom margin if requested
            if self.with_bottom_margin {
                ui.add_space(10.0);
            }
        }

        is_expanded
    }
}
