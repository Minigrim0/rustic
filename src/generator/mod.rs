mod envelope;
mod segment;

/// The different types of generator shapes.
pub enum GENERATORS {
    SINE,
    SAW,
    SQUARE,
    NOISE
}

pub trait ToneGenerator: std::fmt::Debug {
    /// Generates the current time's wave value.
    fn generate(&self, time: f32) -> f32;
}

pub use envelope::{Envelope, Generator};
pub use segment::Segment;
pub mod sine_wave;
pub mod saw_tooth;
pub mod square_wave;
pub mod white_noise;
