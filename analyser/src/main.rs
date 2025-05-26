// src/main.rs - For native binary
use rustic_analyser::components::app::App;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Initialize logging for native
    colog::init();

    log::info!("Starting Sample Analyser native app");

    // For native builds, you might want to start a local server
    // or provide CLI functionality here
    println!("Sample Analyser - Native mode");
    println!("For web interface, build with --target wasm32-unknown-unknown");

    // You could add CLI functionality here, or start a local web server
    // that serves your WASM build
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // For WASM builds, the actual entry point is in lib.rs
    // This main function won't be called
}
