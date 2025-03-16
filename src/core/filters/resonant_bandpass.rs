use biquad::{Biquad, Coefficients, DirectForm2Transposed};
use std::{f32::consts::PI, fmt};

use crate::core::graph::{AudioGraphElement, Entry, Filter};

#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Applies a bandpass filter to the input signal
/// source: https://en.wikipedia.org/wiki/Digital_biquad_filter
/// This structure implements the Direct form 2 from the above link.
#[derive(Clone, Debug)]
pub struct ResonantBandpassFilter {
    source: f32,
    index: usize,
    filter: DirectForm2Transposed<f32>,
}

impl fmt::Display for ResonantBandpassFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resonant bandpass filter")
    }
}

impl ResonantBandpassFilter {
    /// Resonant bandpass filter using a biquad design.
    /// Implemented from http://musicweb.ucsd.edu/~trsmyth/filters/Bi_quadratic_Resonant_Filte.html
    pub fn new(center_frequency: f32, quality: f32, sample_frequency: f32) -> Self {
        // magnitude of the roots

        let period = 1.0 / sample_frequency;
        let bandwidth = center_frequency / quality;

        let r = (-PI * bandwidth * period).exp();

        let coeffs: Coefficients<f32> = Coefficients {
            a1: -2.0 * r * (2.0 * PI * center_frequency * period).cos(),
            a2: r.powi(2),
            b0: 1.0,
            b1: 0.0,
            b2: -r,
        };
        // let filter = DirectForm1::<f32>::new(coeffs);
        let filter = DirectForm2Transposed::<f32>::new(coeffs);

        Self {
            source: 0.0,
            index: 0,
            filter,
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
        let output = self.filter.run(self.source) / 8.0;
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
