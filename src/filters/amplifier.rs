use super::{Filter, SafePipe};

/// A filter that returns the input value multiplied by a constant factor.
/// Note: a factor < 1.0 will attenuate the input signal, while a factor > 1.0
/// will amplify it.
pub struct AmplifierFilter {
    source: SafePipe,
    sink: SafePipe,
    factor: f32,
}

impl AmplifierFilter {
    pub fn new(source: SafePipe, sink: SafePipe, factor: f32) -> Self {
        Self {
            source,
            sink,
            factor,
        }
    }
}

impl Filter for AmplifierFilter {
    fn transform(&mut self) {
        let input = self.source.borrow_mut().pop();
        let output = input * self.factor;
        self.sink.borrow_mut().push(output);
    }
}
