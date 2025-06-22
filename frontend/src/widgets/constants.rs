//! UI constants and theme settings for consistent styling across the application.
//!
//! This module provides standard values for spacing, colors, and other UI properties
//! to maintain visual consistency throughout the application.

use egui::{Color32, Margin, Stroke, Vec2};

/// Spacing constants for consistent UI layout
pub mod spacing {
    /// Standard spacing between UI elements
    pub const ITEM_SPACING: f32 = 8.0;

    /// Small spacing between tightly related UI elements
    pub const SMALL_SPACING: f32 = 4.0;

    /// Large spacing for section separations
    pub const LARGE_SPACING: f32 = 16.0;

    /// Standard padding inside containers
    pub const CONTAINER_PADDING: f32 = 10.0;

    /// Small padding for tight containers
    pub const SMALL_PADDING: f32 = 5.0;

    /// Large padding for prominent containers
    pub const LARGE_PADDING: f32 = 20.0;

    /// Standard width for labels in forms
    pub const LABEL_WIDTH: f32 = 120.0;

    /// Standard height for sliders and other controls
    pub const CONTROL_HEIGHT: f32 = 24.0;
}

/// Color constants for consistent UI theming
pub mod colors {
    use super::Color32;

    /// The primary brand color
    pub const PRIMARY: Color32 = Color32::from_rgb(90, 170, 255);

    /// Secondary brand color
    pub const SECONDARY: Color32 = Color32::from_rgb(100, 200, 150);

    /// Error color for warnings and errors
    pub const ERROR: Color32 = Color32::from_rgb(230, 80, 80);

    /// Warning color for cautions and alerts
    pub const WARNING: Color32 = Color32::from_rgb(230, 180, 80);

    /// Success color for positive feedback
    pub const SUCCESS: Color32 = Color32::from_rgb(80, 210, 120);

    /// Info color for neutral information
    pub const INFO: Color32 = Color32::from_rgb(90, 170, 255);

    /// Dark background for sections and panels
    pub const BACKGROUND_DARK: Color32 = Color32::from_rgb(30, 30, 35);

    /// Medium background for containers
    pub const BACKGROUND_MED: Color32 = Color32::from_rgb(45, 45, 50);

    /// Light background for highlights
    pub const BACKGROUND_LIGHT: Color32 = Color32::from_rgb(60, 60, 70);

    /// Text color for most text elements
    pub const TEXT: Color32 = Color32::from_rgb(230, 230, 230);

    /// Text color for secondary/dimmed text
    pub const TEXT_DIM: Color32 = Color32::from_rgb(170, 170, 180);
}

/// Standard strokes for outlines and borders
pub mod strokes {
    use super::{Color32, Stroke};

    /// Default stroke for containers
    pub const DEFAULT: Stroke = Stroke {
        width: 1.0,
        color: Color32::from_gray(150),
    };

    /// Thin stroke for subtle separations
    pub const THIN: Stroke = Stroke {
        width: 0.5,
        color: Color32::from_gray(150),
    };

    /// Thick stroke for emphasis
    pub const THICK: Stroke = Stroke {
        width: 2.0,
        color: Color32::from_gray(180),
    };

    /// Highlighted stroke for selected elements
    pub const HIGHLIGHT: Stroke = Stroke {
        width: 2.0,
        color: Color32::from_rgb(100, 200, 255),
    };
}

/// Standard sizes for UI elements
pub mod sizes {
    use super::Vec2;

    /// Default button size
    pub const BUTTON: Vec2 = Vec2::new(100.0, 28.0);

    /// Small button size
    pub const BUTTON_SMALL: Vec2 = Vec2::new(80.0, 24.0);

    /// Large button size
    pub const BUTTON_LARGE: Vec2 = Vec2::new(140.0, 32.0);

    /// Icon button size
    pub const BUTTON_ICON: Vec2 = Vec2::new(28.0, 28.0);

    /// Default panel width
    pub const PANEL_WIDTH: f32 = 240.0;
}

/// Standard margins for containers
pub mod margins {
    use super::Margin;

    /// Small margin for compact containers
    pub const SMALL: Margin = Margin {
        top: 4.0,
        right: 4.0,
        bottom: 4.0,
        left: 4.0,
    };

    /// Default margin for most containers
    pub const DEFAULT: Margin = Margin {
        top: 8.0,
        right: 8.0,
        bottom: 8.0,
        left: 8.0,
    };

    /// Large margin for spacious containers
    pub const LARGE: Margin = Margin {
        top: 16.0,
        right: 16.0,
        bottom: 16.0,
        left: 16.0,
    };
}

/// Font sizes for different text elements
pub mod fonts {
    /// Small font size for captions and secondary text
    pub const SMALL: f32 = 12.0;

    /// Default font size for most text
    pub const DEFAULT: f32 = 14.0;

    /// Medium font size for important text
    pub const MEDIUM: f32 = 16.0;

    /// Large font size for headings
    pub const LARGE: f32 = 20.0;

    /// Extra large font size for main headings
    pub const XLARGE: f32 = 24.0;
}

/// Rounding radius for different UI elements
pub mod rounding {
    /// No rounding (square corners)
    pub const NONE: f32 = 0.0;

    /// Slight rounding for subtle effect
    pub const SLIGHT: f32 = 2.0;

    /// Default rounding for most elements
    pub const DEFAULT: f32 = 4.0;

    /// Medium rounding for more pronounced effect
    pub const MEDIUM: f32 = 6.0;

    /// Large rounding for buttons and prominent elements
    pub const LARGE: f32 = 8.0;

    /// Full rounding for circular elements
    pub const FULL: f32 = f32::INFINITY;
}

/// Common animation durations
pub mod animation {
    /// Fast animation duration (ms)
    pub const FAST: f32 = 100.0;

    /// Default animation duration (ms)
    pub const DEFAULT: f32 = 200.0;

    /// Slow animation duration (ms)
    pub const SLOW: f32 = 400.0;
}
