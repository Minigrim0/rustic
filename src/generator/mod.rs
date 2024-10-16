mod envelope;
mod segment;

pub trait ToneGenerator {
    /// Generates the current time's wave value.
    fn generate(&self, time: f32) -> f32;
}

pub use envelope::Envelope;
pub mod sine_wave;
