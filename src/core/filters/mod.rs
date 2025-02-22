#[cfg(feature = "meta")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "meta")]
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterMetadata {
    pub name: String,        // Name of the filter
    pub description: String, // Description of the filter
    pub inputs: usize,       // Number of input pipes
    pub outputs: usize,      // Number of output pipes
}

#[cfg(feature = "meta")]
pub trait Metadata {
    fn get_metadata() -> FilterMetadata;
}

mod amplifier;
mod combinator;
mod delay;
mod high_pass;
mod low_pass;
mod structural;

pub use amplifier::*;
pub use combinator::*;
pub use delay::*;
pub use high_pass::*;
pub use low_pass::*;
pub use structural::*;
