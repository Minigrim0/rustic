use super::{Filter, FilterMetadata, Metadata, SafePipe};
use uuid::Uuid;

/// A filter that take input from two sources and combines them into a single
/// output by adding them together.
pub struct CombinatorFilter {
    source: [SafePipe; 2],
    sink: SafePipe,
    uuid: Uuid,
}

impl CombinatorFilter {
    pub fn new(source: [SafePipe; 2], sink: SafePipe) -> Self {
        Self {
            source,
            sink,
            uuid: Uuid::new_v4(),
        }
    }
}

impl Filter for CombinatorFilter {
    fn transform(&mut self) {
        let input1 = self.source[0].borrow_mut().pop();
        let input2 = self.source[1].borrow_mut().pop();
        let output = input1 + input2;
        self.sink.borrow_mut().push(output);
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

impl Metadata for CombinatorFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "CombinatorFilter".to_string(),
            description: "Combines two inputs by adding them together".to_string(),
            inputs: 2,
            outputs: 1,
        }
    }
}
