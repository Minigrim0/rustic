# Analysis Module

This module contains the audio analysis functionality for the Sample Analyser application. It provides algorithms for analyzing audio in both time and frequency domains.

## Structure

- **mod.rs**: Main module interface and common analysis utilities
- **fft.rs**: Fast Fourier Transform implementation for frequency analysis
- **spectrum.rs**: Spectrogram generation for time-frequency analysis
- **harmonics.rs**: Harmonic identification and analysis
- **pitch.rs**: Pitch detection and musical note conversion

## Core Concepts

### Frequency Analysis

The FFT (Fast Fourier Transform) is used to convert time-domain audio samples into frequency-domain data. This allows us to:

- Identify dominant frequencies in a signal
- Visualize the frequency spectrum
- Detect harmonic relationships

### Spectrogram Analysis

Spectrograms represent how the frequency content of a signal changes over time. This is implemented using Short-Time Fourier Transform (STFT) and provides:

- Time-frequency visualization
- Transient detection
- Pattern recognition in audio

### Pitch Detection

Multiple algorithms are used for pitch detection:

- Autocorrelation method
- Peak identification in frequency domain
- Harmonic product spectrum

### Harmonic Analysis

Harmonic analysis identifies related frequencies that form harmonic series, which is useful for:

- Instrument identification
- Timbre analysis
- Audio quality assessment

## Implementation Details

The analysis module is designed to be efficient and accurate:

- Uses optimized FFT libraries
- Implements windowing functions to reduce spectral leakage
- Provides normalization options for consistent results
- Includes robust peak detection algorithms

## Adding New Analysis Capabilities

To add a new analysis algorithm:

1. Create a new file in this directory
2. Implement the algorithm focusing on performance and accuracy
3. Add tests to validate the implementation
4. Export the functionality through the module interface in `mod.rs`
5. Update the command handlers to expose the new functionality to the frontend

The analysis module aims to provide a comprehensive toolkit for audio analysis while maintaining high performance and accuracy.