use super::ToneGenerator;
use crate::core::graph::{AudioGraphElement, Source};

pub struct SimpleSource<T> {
    generator: T,
    sample_rate: f32,
    index: usize,
}

impl<T> AudioGraphElement for SimpleSource<T> {
    fn get_name(&self) -> &str {
        "Simple Sine Source"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl<T> Source for SimpleSource<T>
where
    T: ToneGenerator,
{
    fn pull(&mut self) -> f32 {
        self.generator.tick(1.0 / self.sample_rate)
    }
}

/// Creates a simple source with a given generator and frequency
pub fn simple_source<T: ToneGenerator>(generator: T) -> impl Source {
    SimpleSource {
        generator,
        sample_rate: 44100.0,
        index: 0,
    }
}
