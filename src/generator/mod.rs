mod envelope;
mod segment;

pub trait ToneGenerator: std::fmt::Debug {
    /// Generates the current time's wave value.
    fn generate(&self, time: f64) -> f64;
}

pub use envelope::{Envelope, Generator};
pub use segment::Segment;
pub mod sine_wave;
