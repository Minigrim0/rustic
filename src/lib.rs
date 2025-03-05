mod app;
pub mod core;
pub mod inputs;
pub mod instruments;

const APP_ID: (&str, &str, &str) = ("rustic", "minigrim0", "xyz");

pub mod prelude {
    pub use super::app::*;
    pub use super::core;
}

use crate::core::generator::{Bendable, ToneGenerator, VariableFrequency};
use core::tones::NOTES;

mod fs;

#[cfg(feature = "plotting")]
pub mod plotting;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "meta")]
pub mod metadata {
    use super::filters::{self, FilterMetadata, Metadata};

    // Todo: this function must be moved to a more correct place
    pub fn filter_metadata() -> Vec<FilterMetadata> {
        vec![
            filters::GainFilter::get_metadata(),
            filters::DelayFilter::get_metadata(),
            filters::LowPassFilter::get_metadata(),
            filters::HighPassFilter::get_metadata(),
            filters::CombinatorFilter::get_metadata(),
            filters::DuplicateFilter::get_metadata(),
        ]
    }
}

pub trait KeyboardGenerator: ToneGenerator + VariableFrequency + Bendable + Send + Sync {}

/// A note with its octave
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Note(pub NOTES, pub u8);
