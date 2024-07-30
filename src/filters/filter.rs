pub trait Filter {
    fn apply(&mut self, input: f32) -> f32;
}