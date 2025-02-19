use std::cell::RefCell;
use std::rc::Rc;

use crate::core::generator::prelude::*;
use crate::core::generator::sources::simple_source;
use crate::core::graph::{Source, System};

/// A HiHat instrument.
pub struct HiHat {
    graph: System,
    output: f32,
}

impl HiHat {
    pub fn new() -> Self {
        let sources = [
            simple_source(SquareWave::new(123.0, 1.0)),
            simple_source(SquareWave::new(150.0, 1.0)),
            simple_source(SquareWave::new(180.0, 1.0)),
            simple_source(SquareWave::new(219.0, 1.0)),
            simple_source(SquareWave::new(240.0, 1.0)),
            simple_source(SquareWave::new(261.0, 1.0)),
        ];

        let mut system = System::new();
        let sources = sources
            .into_iter()
            .map(|source| system.add_source(Box::new(source)));

        Self {
            graph: System::new(),
            output: 0.0,
        }
    }
}
