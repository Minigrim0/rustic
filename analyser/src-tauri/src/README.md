# Sample Analyser Backend

This directory contains the backend code for the Sample Analyser application. The backend is responsible for audio processing, analysis, and providing the results to the frontend.

## Structure

- **analysis/**: Audio analysis modules
  - **fft.rs**: Fast Fourier Transform implementation
  - **spectrum.rs**: Spectrogram analysis
  - **harmonics.rs**: Harmonic identification and analysis
  - **pitch.rs**: Pitch detection algorithms

- **audio/**: Audio processing and loading
  - **loader.rs**: Audio file loading and decoding
  - **mod.rs**: Audio utilities like normalization and windowing

- **commands.rs**: Tauri command handlers that expose functionality to the frontend

## Architecture

The backend follows a layered architecture:

1. **Command Layer**: Handles requests from the frontend and coordinates the processing
2. **Analysis Layer**: Contains the core analysis algorithms
3. **Audio Layer**: Handles audio file loading and processing

The analysis is performed on demand when requested by the frontend, and the results are sent back for visualization.

## Key Features

- Loading and decoding various audio formats
- Time-domain and frequency-domain analysis
- Pitch detection and note identification
- Harmonic analysis
- Spectrogram generation

## Adding New Analysis Capabilities

To add a new analysis feature:

1. Implement the analysis algorithm in the appropriate module
2. Export the functionality in the module's public interface
3. Add a new command handler in `commands.rs` to expose it to the frontend
4. Update the main `analyze_audio` function to include the new analysis if needed