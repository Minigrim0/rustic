//! Custom theme implementation for the Rustic application.
//!
//! This module provides functionality to customize the egui theme with
//! Rustic-specific styling and colors to maintain a consistent look and feel.

use egui::{
    Color32, Rounding, Stroke, Style, Visuals,
    style::{Selection, Widgets},
};

use super::constants::{colors, margins, rounding, spacing, strokes};

/// Theme options available in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    /// Dark theme (default)
    Dark,
    /// Light theme
    Light,
    /// High contrast theme for accessibility
    HighContrast,
    /// Custom theme (user-defined)
    Custom,
}

impl ThemeChoice {
    /// Convert theme choice to string representation
    pub fn as_string(&self) -> &'static str {
        match self {
            ThemeChoice::Dark => "Dark",
            ThemeChoice::Light => "Light",
            ThemeChoice::HighContrast => "High Contrast",
            ThemeChoice::Custom => "Custom",
        }
    }

    /// Get all available theme choices
    pub fn all() -> Vec<ThemeChoice> {
        vec![
            ThemeChoice::Dark,
            ThemeChoice::Light,
            ThemeChoice::HighContrast,
            ThemeChoice::Custom,
        ]
    }
}

/// Configure the application theme based on the selected theme choice
///
/// # Arguments
///
/// * `theme` - The selected theme choice
/// * `ctx` - The egui context to apply the theme to
///
/// # Examples
///
/// ```
/// use frontend::widgets::{configure_theme, ThemeChoice};
///
/// fn update_theme(ctx: &egui::Context) {
///     configure_theme(ThemeChoice::Dark, ctx);
/// }
/// ```
pub fn configure_theme(theme: ThemeChoice, ctx: &egui::Context) {
    let style = match theme {
        ThemeChoice::Dark => create_dark_theme(),
        ThemeChoice::Light => create_light_theme(),
        ThemeChoice::HighContrast => create_high_contrast_theme(),
        ThemeChoice::Custom => create_custom_theme(),
    };

    ctx.set_style(style);
}

/// Create the dark theme style
fn create_dark_theme() -> Style {
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(spacing::ITEM_SPACING, spacing::ITEM_SPACING);
    style.spacing.window_margin = margins::DEFAULT;
    style.spacing.button_padding = egui::vec2(spacing::CONTAINER_PADDING, spacing::SMALL_PADDING);

    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(colors::TEXT);
    visuals.widgets.noninteractive.bg_fill = colors::BACKGROUND_DARK;
    visuals.widgets.inactive.bg_fill = colors::BACKGROUND_MED;
    visuals.widgets.hovered.bg_fill = colors::BACKGROUND_LIGHT;
    visuals.widgets.active.bg_fill = colors::PRIMARY;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.open.bg_fill = colors::BACKGROUND_LIGHT;

    visuals.selection = Selection {
        bg_fill: colors::PRIMARY.linear_multiply(0.8),
        stroke: strokes::HIGHLIGHT,
    };

    visuals.window_rounding = Rounding::same(rounding::DEFAULT);
    visuals.window_shadow.extrusion = 8.0;

    visuals.menu_rounding = Rounding::same(rounding::DEFAULT);
    // Note: panel_rounding is not available in newer egui versions
    // Individual widgets handle their own rounding

    visuals.popup_shadow.extrusion = 6.0;

    style.visuals = visuals;
    style
}

/// Create the light theme style
fn create_light_theme() -> Style {
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(spacing::ITEM_SPACING, spacing::ITEM_SPACING);
    style.spacing.window_margin = margins::DEFAULT;
    style.spacing.button_padding = egui::vec2(spacing::CONTAINER_PADDING, spacing::SMALL_PADDING);

    let mut visuals = Visuals::light();
    visuals.override_text_color = Some(Color32::from_rgb(20, 20, 20));

    // Define light theme specific colors
    let bg_fill = Color32::from_rgb(240, 240, 245);
    let bg_med = Color32::from_rgb(220, 220, 230);
    let bg_light = Color32::from_rgb(230, 230, 240);
    let primary = Color32::from_rgb(40, 120, 200);

    visuals.widgets.noninteractive.bg_fill = bg_fill;
    visuals.widgets.inactive.bg_fill = bg_med;
    visuals.widgets.hovered.bg_fill = bg_light;
    visuals.widgets.active.bg_fill = primary;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.open.bg_fill = bg_light;

    visuals.selection = Selection {
        bg_fill: primary.linear_multiply(0.8),
        stroke: Stroke::new(2.0, primary),
    };

    visuals.window_rounding = Rounding::same(rounding::DEFAULT);
    visuals.window_shadow.extrusion = 8.0;

    visuals.menu_rounding = Rounding::same(rounding::DEFAULT);
    // Panel rounding handled through individual frame styling

    visuals.popup_shadow.extrusion = 6.0;

    style.visuals = visuals;
    style
}

/// Create a high contrast theme for accessibility
fn create_high_contrast_theme() -> Style {
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(spacing::ITEM_SPACING, spacing::ITEM_SPACING);
    style.spacing.window_margin = margins::LARGE;
    style.spacing.button_padding = egui::vec2(spacing::LARGE_PADDING, spacing::CONTAINER_PADDING);

    let mut visuals = Visuals::dark();

    // Define high contrast specific colors
    let bg_fill = Color32::BLACK;
    let text_color = Color32::WHITE;
    let primary = Color32::from_rgb(255, 255, 0); // Yellow for high contrast
    let secondary = Color32::from_rgb(50, 255, 255); // Cyan for high contrast

    visuals.override_text_color = Some(text_color);

    visuals.widgets = Widgets::default();
    visuals.widgets.noninteractive.bg_fill = bg_fill;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.5, text_color);

    visuals.widgets.inactive.bg_fill = bg_fill;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.5, text_color);

    visuals.widgets.hovered.bg_fill = primary;
    visuals.widgets.hovered.fg_stroke = Stroke::new(2.0, Color32::BLACK);

    visuals.widgets.active.bg_fill = secondary;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::BLACK);

    visuals.widgets.open.bg_fill = primary.linear_multiply(0.8);
    visuals.widgets.open.fg_stroke = Stroke::new(2.0, Color32::BLACK);

    visuals.selection = Selection {
        bg_fill: primary,
        stroke: Stroke::new(2.0, text_color),
    };

    visuals.window_rounding = Rounding::same(rounding::SLIGHT);
    visuals.window_shadow.extrusion = 10.0;

    visuals.menu_rounding = Rounding::same(rounding::SLIGHT);
    // Panel rounding handled through individual frame styling

    visuals.popup_shadow.extrusion = 8.0;

    style.visuals = visuals;
    style
}

/// Create a custom theme with user-defined settings
///
/// This is a placeholder for future customization options.
/// Currently returns the dark theme with slight modifications.
fn create_custom_theme() -> Style {
    // Start with the dark theme
    let mut style = create_dark_theme();

    // TODO: Load custom theme settings from configuration file

    // Apply some custom modifications as an example
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(180, 100, 255); // Purple accent
    style.visuals.selection.bg_fill = Color32::from_rgb(180, 100, 255).linear_multiply(0.8);
    style.visuals.selection.stroke = Stroke::new(2.0, Color32::from_rgb(200, 120, 255));

    style
}

/// Apply a specific scale factor to the UI
///
/// # Arguments
///
/// * `scale` - The scaling factor (1.0 is the default scale)
/// * `ctx` - The egui context to apply the scaling to
///
/// # Examples
///
/// ```
/// use frontend::widgets::apply_scaling;
///
/// fn update_scale(ctx: &egui::Context) {
///     apply_scaling(1.2, ctx); // Scale UI by 120%
/// }
/// ```
pub fn apply_scaling(scale: f32, ctx: &egui::Context) {
    ctx.set_pixels_per_point(scale);
}

/// Update the fonts for the application
///
/// # Arguments
///
/// * `ctx` - The egui context to update fonts for
///
/// This function configures the fonts used in the application.
/// It's currently a placeholder for future font customization.
pub fn _configure_fonts(ctx: &egui::Context) {
    // Get the current fonts
    let fonts = egui::FontDefinitions::default();

    // TODO: Add custom font configuration

    // Apply the fonts
    ctx.set_fonts(fonts);
}
