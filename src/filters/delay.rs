use std::collections::VecDeque;

use super::{Filter, SafePipe};

/// Delays it input for x samples
pub struct DelayFilter {
    source: SafePipe,
    sink: SafePipe,
    delay_for: usize,
    buffer: VecDeque<f32>,
}

impl DelayFilter {
    pub fn new(source: SafePipe, sink: SafePipe, delay: usize) -> Self {
        Self {
            source,
            sink,
            delay_for: delay,
            buffer: VecDeque::from(vec![0.0; delay]),
        }
    }
}

impl Filter for DelayFilter {
    fn transform(&mut self) {
        let input = self.source.borrow_mut().pop();
        let output = self.buffer.pop_front().unwrap_or(0.0);
        self.buffer.push_back(input);
        self.sink.borrow_mut().push(output);
    }
}
