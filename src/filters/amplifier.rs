use super::{Filter, FilterMetadata, Metadata, SafePipe};
use uuid::Uuid;

/// A filter that returns the input value multiplied by a constant factor.
/// Note: a factor < 1.0 will attenuate the input signal, while a factor > 1.0
/// will amplify it.
// #[cfg_attr(feature = "meta", derive(Metadata))]
pub struct AmplifierFilter {
    source: SafePipe,
    sink: SafePipe,
    factor: f32,
    uuid: Uuid,
}

impl AmplifierFilter {
    pub fn new(source: SafePipe, sink: SafePipe, factor: f32) -> Self {
        Self {
            source,
            sink,
            factor,
            uuid: Uuid::new_v4(),
        }
    }
}

impl Filter for AmplifierFilter {
    fn transform(&mut self) {
        let input = self.source.borrow_mut().pop();
        let output = input * self.factor;
        self.sink.borrow_mut().push(output);
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

impl Metadata for AmplifierFilter {
    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            name: "AmplifierFilter".to_string(),
            description: "A filter that returns the input value multiplied by a constant factor."
                .to_string(),
            inputs: 1,
            outputs: 1,
        }
    }
}