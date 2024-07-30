pub trait ToneGenerator {
    fn generate(&self, time: f32) -> f32;
}