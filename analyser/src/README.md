# Sample Analyser Frontend

This directory contains the frontend code for the Sample Analyser application. The frontend is responsible for visualizing audio analysis results and providing a user interface for loading and interacting with audio files.

## Structure

- **components/**: UI components for the application
  - **app.rs**: Main application component
  - **audio_visualizer.rs**: Component for visualizing audio waveforms
  - **file_upload.rs**: Component for handling file uploads
  - **frequency_chart.rs**: Component for visualizing frequency spectrum
  - **spectrum_display.rs**: Component for visualizing spectrograms

- **visualization/**: Visualization utilities and rendering logic
  - **color_maps.rs**: Color mapping utilities for visualizations
  - **frequency.rs**: Frequency spectrum visualization
  - **spectrogram.rs**: Spectrogram visualization
  - **waveform.rs**: Waveform visualization

- **utils/**: Utility functions for the frontend
  - **audio.rs**: Audio-specific utility functions
  - **error.rs**: Error handling utilities

## Architecture

The frontend follows a component-based architecture using the Yew framework. Components communicate with the Tauri backend through command invocations, and the backend provides analysis results that are then visualized by the frontend.

The flow of data is:

1. User uploads an audio file via the `FileUpload` component
2. File is sent to the Tauri backend for analysis
3. Analysis results are returned to the frontend
4. Results are visualized using various visualization components

## Adding New Visualizations

To add a new visualization:

1. Create a new component in the `components/` or `visualization/` directory
2. Implement the `Component` trait for your visualization
3. Update the `App` component to include your new visualization