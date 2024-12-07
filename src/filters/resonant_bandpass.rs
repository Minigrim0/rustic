use std::collections::VecDeque;
use uuid::Uuid;

use super::Filter;
#[cfg(feature = "meta")]
use super::{FilterMetadata, Metadata};

/// Delays it input for x samples
pub struct DelayFilter {
    source: [f32; 1],
    sink: SafePipe,
    delay_for: usize,
    buffer: VecDeque<f32>,
    uuid: Uuid,
}
