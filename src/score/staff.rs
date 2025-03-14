use serde::{Deserialize, Serialize};

use super::{measure::Measure, score::TimeSignature};

/// A staff is an instrument line.

/// A staff is a single line in a music score. It has an associated instrument,
/// contains multiple measures with their notes
#[derive(Serialize, Deserialize, Default)]
pub struct Staff {
    instrument: usize, // Instrument index in the intrument map
    measures: Vec<Measure>,
}

impl Staff {
    pub fn with_measures(mut self, measures: usize, signature: &TimeSignature) -> Self {
        self.measures = Vec::from_iter((0..measures).map(|_| Measure::new(signature)));
        self
    }
}
