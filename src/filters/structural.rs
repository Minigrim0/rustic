/// This file contains structural filters i.e. filters that do not modify
/// values that pass through it but rather duplicate/merges its inputs
use super::Filter;

/// Duplicates the content of the input onto two outputs
struct DuplicateFilter {
    source: SafePipe,
    sinks: [SafePipe; 2],
}

impl DuplicateFilter {
    pub fn new(source: SafePipe, sinks: [SafePipe; 2]) -> Self {
        Self { source, sinks }
    }
}

impl Filter for DuplicateFilter {
    fn transform(&mut self) {
        let input = self.source.borrow_mut().pop();
        self.sinks
            .iter()
            .for_each(|sink| sink.borrow_mut().push(input));
    }
}
