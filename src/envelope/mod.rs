mod envelope;
mod segment;

/// A shape that can be used to modulate a signal over time.
pub trait Shape: std::fmt::Display {
    /// Returns the shaper values at the given point in time
    fn get_at(&self, time: f32) -> f32;
}

pub use envelope::*;
pub use segment::*;
