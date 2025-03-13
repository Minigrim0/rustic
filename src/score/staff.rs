use crate::instruments::Instrument;

use super::measure::Measure;

/// A staff is an instrument line.

/// A staff is a single line in a music score. It has an associated instrument,
/// contains multiple measures with their notes
pub struct Staff<const LENGTH: usize> {
    instrument: Box<dyn Instrument>,
    measures: [Measure; LENGTH],
}
