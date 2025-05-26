//! Sample Analyser Library
//!
//! This library provides functionality for analyzing audio samples and
//! visualizing the results in various ways.

use wasm_bindgen::prelude::*;

// Module definitions
pub mod components;
pub mod utils;
pub mod visualization;

// Re-export main app component
pub use components::app::App;

// WASM entry point - only one should exist
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging for WASM
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");

    log::info!("Starting Sample Analyser WebAssembly app");

    // Start the Yew app
    yew::Renderer::<App>::new().render();

    Ok(())
}

// For native builds, provide a public function that can be called from main.rs
#[cfg(not(target_arch = "wasm32"))]
pub fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    colog::init();
    log::info!("Sample Analyser library initialized");

    // For native builds, you might want to start a web server
    // or provide other functionality
    Ok(())
}

/// Utility functions for the WASM frontend
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;
    use web_sys::{Element, HtmlElement, Window};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }

    pub fn set_panic_hook() {
        console_error_panic_hook::set_once();
    }

    pub fn window() -> Result<Window, JsValue> {
        web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))
    }

    pub fn document() -> Result<web_sys::Document, JsValue> {
        window()?
            .document()
            .ok_or_else(|| JsValue::from_str("No document found"))
    }

    // Fixed version from earlier
    pub fn get_element_by_id(id: &str) -> Result<Element, JsValue> {
        document()?
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("No element found with id: {}", id)))
    }

    pub fn get_html_element_by_id(id: &str) -> Result<HtmlElement, JsValue> {
        let element = document()?
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("No element found with id: {}", id)))?;

        element.dyn_into::<HtmlElement>().map_err(|_| {
            JsValue::from_str(&format!("Element with id '{}' is not an HtmlElement", id))
        })
    }
}
