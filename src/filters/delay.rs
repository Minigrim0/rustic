use std::collections::VecDeque;
use uuid::Uuid;

use super::{Filter, FilterMetadata, Metadata, SafePipe};

/// Delays it input for x samples
pub struct DelayFilter {
    source: SafePipe,
    sink: SafePipe,
    delay_for: usize,
    buffer: VecDeque<f32>,
    uuid: Uuid,
}

impl DelayFilter {
    pub fn new(source: SafePipe, sink: SafePipe, delay: usize) -> Self {
        Self {
            source,
            sink,
            delay_for: delay,
            buffer: VecDeque::from(vec![0.0; delay]),
            uuid: Uuid::new_v4(),
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

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

impl Metadata for DelayFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "DelayFilter".to_string(),
            description: "Delays its input for x samples".to_string(),
            inputs: 1,
            outputs: 1,
        }
    }
}
