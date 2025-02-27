use std::{f32::consts::PI, fmt};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type};

use crate::core::graph::{AudioGraphElement, Entry, Filter};

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Applies a bandpass filter to the input signal
/// source: https://en.wikipedia.org/wiki/Digital_biquad_filter
/// This structure implements the Direct form 2 from the above link.
#[cfg_attr(feature = "meta2", derive(derive::MetaData))]
#[derive(Clone, Debug)]
pub struct ResonantBandpassFilter {
    source: f32,
    index: usize,
    buffer: [f32; 2],  // Coefficients for the filter.
    coefficients: [f32; 6],
}

impl fmt::Display for ResonantBandpassFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resonant bandpass filter")
    }
}


impl ResonantBandpassFilter {
    /// Resonant bandpass filter using a biquad design.
    /// Implemented from http://musicweb.ucsd.edu/~trsmyth/filters/Bi_quadratic_Resonant_Filte.html
    /// Parameters:
    ///     center_frequency: f32
    ///         Center frequency in Hz.
    ///     quality: f32
    ///         Quality factor for resonance.
    ///     sample_frequency: f32
    ///         Sampling frequency in Hz.
    pub fn new(center_frequency: f32, quality: f32, sample_frequency: f32) -> Self {
        let r = (-PI * (center_frequency / quality) * (1.0 / sample_frequency)).exp();
        let coefficients = [
            1.0,
            -2.0 * r * ((2.0 * PI * center_frequency) / sample_frequency).cos(),
            r.powi(2),
            1.0,
            0.0,
            -r
        ];

        Self {
            source: 0.0,
            index: 0,
            buffer: [0.0; 2],
            coefficients,
        }
    }
}

impl Entry for ResonantBandpassFilter {
    fn push(&mut self, value: f32, _port: usize) {
        self.source = value;
    }
}

impl Filter for ResonantBandpassFilter {
    fn transform(&mut self) -> Vec<f32> {
        let w = self.source - self.coefficients[1] * self.buffer[0] - self.coefficients[2] * self.buffer[1];
        let output = self.coefficients[3] * w + self.coefficients[4] * self.buffer[0] + self.coefficients[5] * self.buffer[1];

        self.buffer[1] = self.buffer[0];
        self.buffer[0] = w;

        vec![output]
    }

    fn postponable(&self) -> bool {
        false
    }
}


impl AudioGraphElement for ResonantBandpassFilter {
    fn get_name(&self) -> &str {
        "Resonant Bandpass Filter"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}
