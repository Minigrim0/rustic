# Rustic Sample Analyser

A powerful audio sample analysis tool built with Rust, Tauri, and WebAssembly.

## Features

- Audio file loading and processing (MP3, WAV, FLAC, OGG)
- Comprehensive frequency analysis using Fast Fourier Transform (FFT)
- Pitch detection and musical note identification
- Harmonic analysis and frequency peak detection
- Spectrogram generation for time-frequency visualization
- Interactive visualizations for all analysis results
- Cross-platform desktop application (Windows, macOS, Linux)

## Architecture

The application follows a lasagna architecture with clear separation of concerns:

- **Frontend (src/)**: Visualization and user interface
  - Components for UI elements and interactive displays
  - Visualization tools for rendering analysis results
  - Utility functions for frontend processing

- **Backend (src-tauri/)**: Analysis and audio processing
  - Audio loading and decoding from various formats
  - Signal processing and analysis algorithms
  - Tauri commands to bridge frontend and backend

## Project Structure

- `src/` - Frontend code
  - `components/` - UI components
  - `visualization/` - Visualization utilities
  - `utils/` - Frontend utility functions
  
- `src-tauri/` - Backend code
  - `src/analysis/` - Audio analysis algorithms
  - `src/audio/` - Audio file loading and processing
  - `src/commands.rs` - Tauri command handlers

Each directory contains its own README with more detailed information about its purpose and implementation.

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- Node.js and npm/yarn
- Tauri CLI (`cargo install tauri-cli`)

### Installation

1. Clone the repository
2. Install dependencies:

```bash
cd rustic/analyser
npm install # or yarn install
```

### Running the Application

Start the development version:

```bash
cargo tauri dev
```

### Building for Production

```bash
cargo tauri build
```

## Development

To add new features, follow these guidelines:

1. Analysis algorithms should go in the appropriate module under `src-tauri/src/analysis/`
2. Visualization components should be added to `src/visualization/`
3. UI components should be added to `src/components/`
4. Expose new functionality to the frontend by adding commands in `src-tauri/src/commands.rs`

## License

MIT

## Acknowledgments

- Tauri Framework
- Yew Framework
- RustFFT
- Symphonia Audio Decoder
- Plotters Visualization Library