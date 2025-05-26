# Visualization Module

This module contains the visualization tools for the Sample Analyser application. These tools are responsible for rendering audio data and analysis results in various visual formats.

## Structure

- **mod.rs**: Defines common visualization interfaces and utilities
- **waveform.rs**: Visualization of audio waveforms in the time domain
- **frequency.rs**: Visualization of frequency spectrum data
- **spectrogram.rs**: Visualization of time-frequency data as spectrograms
- **color_maps.rs**: Color mapping utilities for visualizations

## Core Concepts

### Visualization Options

The `VisualizationOptions` struct provides common configuration options for all visualizations:

- Dimensions (width/height)
- Colors (background/foreground)
- Display options (grid, labels)
- Scaling modes (linear/logarithmic/decibel)

### Scale Modes

Different scaling options are available for visualizations:

- **Linear**: No transformation, direct mapping
- **Logarithmic**: Log-scale mapping, useful for frequency displays
- **Decibel**: Power/amplitude scaling for audio levels
- **Mel**: Perceptual scaling for frequency data

### Color Maps

The `ColorMap` enum provides various color schemes for spectrograms and heatmaps:

- Grayscale
- Viridis
- Plasma
- Jet
- Inferno
- Hot

## Implementation Details

Each visualization component follows a common pattern:

1. Accepts data and configuration options as props
2. Renders to a canvas element
3. Provides methods for customizing the display
4. Automatically updates when data or options change

## Usage Examples

To use these visualizations in a component:

1. Import the needed visualization from this module
2. Create a component that provides data to the visualization
3. Configure options as needed for your specific use case
4. Add the visualization to your component's view

## Adding New Visualizations

When adding a new visualization:

1. Create a new file in this directory
2. Export it from the module's public interface in `mod.rs`
3. Follow the existing pattern for canvas-based rendering
4. Ensure it accepts standard `VisualizationOptions`
5. Add appropriate tests