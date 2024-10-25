/// A filter that can process data from source pipes and send to sink pipes.
pub trait Filter {
    fn transform(&mut self);
}

mod amplifier;
mod combinator;
mod delay;
mod low_pass;
mod structural;

pub use amplifier::*;
pub use combinator::*;
pub use delay::*;
pub use low_pass::*;
pub use structural::*;
