use super::{Filter, SafePipe};

/// A filter that take input from two sources and combines them into a single
/// output by adding them together.
pub struct CombinatorFilter {
    source: [SafePipe; 2],
    sink: SafePipe,
}

impl CombinatorFilter {
    pub fn new(source: [SafePipe; 2], sink: SafePipe) -> Self {
        Self { source, sink }
    }
}

impl Filter for CombinatorFilter {
    fn transform(&mut self) {
        let input1 = self.source[0].borrow_mut().pop();
        let input2 = self.source[1].borrow_mut().pop();
        let output = input1 + input2;
        self.sink.borrow_mut().push(output);
    }
}
