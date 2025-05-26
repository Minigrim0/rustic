# Components Module

This module contains the UI components for the Sample Analyser application. These components are responsible for user interaction and visualization of audio analysis results.

## Structure

- **app.rs**: Main application component that orchestrates the UI and manages application state
- **audio_visualizer.rs**: Component for visualizing audio waveforms in the time domain
- **file_upload.rs**: Component for handling file uploads and drag-and-drop functionality
- **frequency_chart.rs**: Component for visualizing frequency spectrum analysis results
- **spectrum_display.rs**: Component for visualizing spectrograms and time-frequency analysis

## Design Principles

The components in this module follow these design principles:

1. **Separation of Concerns**: Each component has a specific responsibility and doesn't mix visualization logic with analysis logic
2. **Reusability**: Components are designed to be reusable with different data sources
3. **Composition**: Complex UIs are built by composing simpler components
4. **Unidirectional Data Flow**: Data flows from parent to child components through properties

## Adding New Components

When adding a new component:

1. Create a new file in this directory with the component name
2. Add the component to the `mod.rs` file exports
3. Follow the Yew component lifecycle (create, update, view, etc.)
4. Keep rendering logic separate from business logic
5. Use props for configuration and callbacks for event handling

## Component Communication

Components communicate through:

- **Properties**: For parent-to-child data passing
- **Callbacks**: For child-to-parent event notification
- **Context**: For sharing state across the component tree

Each component should document its props interface and the events it emits via callbacks.