# Audio Module

This module handles audio file loading, processing, and manipulation for the Sample Analyser application.

## Structure

- **mod.rs**: Core audio processing utilities and windowing functions
- **loader.rs**: Audio file loading and decoding from various formats

## Core Features

### Audio Loading

The `AudioLoader` class provides a unified interface for loading audio files in various formats:

- WAV files via the `hound` library
- MP3, FLAC, OGG, and other formats via the `symphonia` library
- Automatic format detection based on file extension
- Conversion to normalized mono samples for analysis

### Audio Buffer

The `AudioBuffer` struct holds audio data and metadata:

- Sample data (normalized to -1.0 to 1.0 range)
- Sample rate
- Duration
- Number of channels
- Bit depth

### Audio Processing

The module provides various audio processing functions:

- Windowing (Hann, Hamming, Blackman, Rectangular)
- Normalization
- Resampling
- Gain adjustment

## Usage

The audio module is typically used at the beginning of the analysis pipeline:

1. Load an audio file with `AudioLoader`
2. Access the sample data and metadata from the `AudioBuffer`
3. Apply any necessary pre-processing (windowing, normalization)
4. Pass the processed audio data to analysis functions

## Implementation Details

The implementation prioritizes:

- Broad format support
- Accurate decoding
- Efficient memory usage
- Consistent interface regardless of source format

## Adding New Functionality

When extending this module:

1. Add new processing functions to the appropriate file
2. Update tests to validate the new functionality
3. Document any new public interfaces
4. Consider performance implications for large audio files

This module serves as the foundation for all audio analysis in the application, providing clean, normalized audio data for the analysis pipeline.