//! UI Components for the Sample Analyser
//!
//! This module contains reusable UI components for visualization
//! and user interaction.

pub mod app;
pub mod audio_visualizer;
pub mod file_upload;
pub mod frequency_chart;
pub mod spectrum_display;

// Component utility functions
pub mod utils {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

    /// Gets a canvas element and its 2D rendering context
    pub fn get_canvas_context(
        canvas_id: &str,
    ) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window found"))?
            .document()
            .ok_or_else(|| JsValue::from_str("No document found"))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("No canvas found with id: {}", canvas_id)))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get canvas context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok((canvas, context))
    }

    /// Normalizes audio samples to the range [-1.0, 1.0]
    pub fn normalize_samples(samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return vec![];
        }

        let max_abs = samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, |a, b| a.max(b));

        if max_abs > 0.0 {
            samples.iter().map(|s| s / max_abs).collect()
        } else {
            samples.to_vec()
        }
    }
}
