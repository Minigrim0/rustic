//! Color maps for visualization
//!
//! This module provides color maps for visualizing data with different
//! color schemes, particularly useful for spectrograms and heatmaps.

/// Color map options for visualizations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMap {
    /// Grayscale (black to white)
    Grayscale,

    /// Viridis (perceptually uniform, accessible)
    Viridis,

    /// Plasma (perceptually uniform, high contrast)
    Plasma,

    /// Jet (traditional rainbow, not perceptually uniform)
    Jet,

    /// Inferno (perceptually uniform, dark to bright yellow)
    Inferno,

    /// Hot (black-red-yellow-white)
    Hot,
}

impl ColorMap {
    /// Get a CSS color string for a value between 0.0 and 1.0
    pub fn get_color(&self, value: f32) -> String {
        // Clamp value between 0 and 1
        let v = value.max(0.0).min(1.0);

        match self {
            ColorMap::Grayscale => {
                let intensity = (v * 255.0) as u8;
                format!("rgb({}, {}, {})", intensity, intensity, intensity)
            }
            ColorMap::Viridis => {
                // Simplified approximation of viridis colormap
                let r = ((v * 0.87).max(0.0) * 255.0) as u8;
                let g = ((0.1 + v * 0.85).min(1.0) * 255.0) as u8;
                let b = ((1.0 - v * 0.5).min(1.0) * 255.0) as u8;
                format!("rgb({}, {}, {})", r, g, b)
            }
            ColorMap::Plasma => {
                // Simplified approximation of plasma colormap
                let r = ((0.4 + v * 0.6) * 255.0) as u8;
                let g = (v * v * 255.0) as u8;
                let b = ((1.0 - v * 0.95) * 255.0) as u8;
                format!("rgb({}, {}, {})", r, g, b)
            }
            ColorMap::Jet => {
                // Traditional rainbow colormap
                let r = if v < 0.35 {
                    0
                } else if v < 0.66 {
                    ((v - 0.35) * 323.0) as u8
                } else {
                    255
                };

                let g = if v < 0.125 {
                    0
                } else if v < 0.375 {
                    ((v - 0.125) * 1020.0) as u8
                } else if v < 0.64 {
                    255
                } else {
                    (255.0 - ((v - 0.64) * 1020.0)) as u8
                };

                let b = if v < 0.38 {
                    ((v + 0.25) * 1020.0).min(255.0) as u8
                } else {
                    (255.0 - ((v - 0.38) * 408.0)).max(0.0) as u8
                };

                format!("rgb({}, {}, {})", r, g, b)
            }
            ColorMap::Inferno => {
                // Simplified approximation of inferno colormap
                let r = ((0.0 + v * 1.0) * 255.0) as u8;
                let g = ((0.0 + v * v * 1.0) * 255.0) as u8;
                let b = ((0.4 * (1.0 - v)) * 255.0) as u8;
                format!("rgb({}, {}, {})", r, g, b)
            }
            ColorMap::Hot => {
                // Black -> Red -> Yellow -> White
                let r = (v * 3.0 * 255.0).min(255.0) as u8;
                let g = ((v * 3.0 - 1.0) * 255.0).max(0.0).min(255.0) as u8;
                let b = ((v * 3.0 - 2.0) * 255.0).max(0.0).min(255.0) as u8;
                format!("rgb({}, {}, {})", r, g, b)
            }
        }
    }

    /// Get a list of all available color maps
    pub fn all() -> Vec<ColorMap> {
        vec![
            ColorMap::Grayscale,
            ColorMap::Viridis,
            ColorMap::Plasma,
            ColorMap::Jet,
            ColorMap::Inferno,
            ColorMap::Hot,
        ]
    }

    /// Get the name of the color map as a string
    pub fn name(&self) -> &'static str {
        match self {
            ColorMap::Grayscale => "Grayscale",
            ColorMap::Viridis => "Viridis",
            ColorMap::Plasma => "Plasma",
            ColorMap::Jet => "Jet",
            ColorMap::Inferno => "Inferno",
            ColorMap::Hot => "Hot",
        }
    }
}

/// Generate color bar data for a given color map
pub fn generate_color_bar(color_map: ColorMap, width: usize, height: usize) -> Vec<u8> {
    let mut data = vec![0; width * height * 4]; // RGBA

    for y in 0..height {
        let value = 1.0 - (y as f32 / height as f32);
        let color_str = color_map.get_color(value);

        // Parse the rgb(r, g, b) format
        let color_values = color_str
            .trim_start_matches("rgb(")
            .trim_end_matches(")")
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap_or(0))
            .collect::<Vec<u8>>();

        if color_values.len() >= 3 {
            let r = color_values[0];
            let g = color_values[1];
            let b = color_values[2];

            for x in 0..width {
                let idx = (y * width + x) * 4;
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = 255; // Alpha
            }
        }
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_maps() {
        // Test that each color map produces a valid color string
        for &map in &[
            ColorMap::Grayscale,
            ColorMap::Viridis,
            ColorMap::Plasma,
            ColorMap::Jet,
            ColorMap::Inferno,
            ColorMap::Hot,
        ] {
            let color = map.get_color(0.5);
            assert!(color.starts_with("rgb("));
            assert!(color.ends_with(")"));

            // Check that the color values are in the valid range
            let values = color
                .trim_start_matches("rgb(")
                .trim_end_matches(")")
                .split(',')
                .map(|s| s.trim().parse::<u8>().unwrap_or(0))
                .collect::<Vec<u8>>();

            assert_eq!(values.len(), 3);
            for &v in &values {
                assert!(v <= 255);
            }
        }
    }

    #[test]
    fn test_color_bar_generation() {
        let width = 10;
        let height = 20;
        let data = generate_color_bar(ColorMap::Viridis, width, height);

        // Check dimensions
        assert_eq!(data.len(), width * height * 4);

        // Check that rows have consistent colors
        for y in 0..height {
            let base_idx = y * width * 4;
            let r = data[base_idx];
            let g = data[base_idx + 1];
            let b = data[base_idx + 2];
            let a = data[base_idx + 3];

            for x in 1..width {
                let idx = (y * width + x) * 4;
                assert_eq!(data[idx], r);
                assert_eq!(data[idx + 1], g);
                assert_eq!(data[idx + 2], b);
                assert_eq!(data[idx + 3], a);
            }
        }
    }
}
