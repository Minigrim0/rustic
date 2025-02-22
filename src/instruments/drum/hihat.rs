use crate::core::generator::prelude::*;
use crate::core::filters::CombinatorFilter;
use crate::core::graph::simple_source;
use crate::core::graph::{Source, System};
use crate::core::graph::SimpleSink;

/// A HiHat instrument.
/// It consists of six square wave sources connected to a combinator filter. The result is then passed through a resonant bandpass filter,
/// before being shaped by an envelope generator.
pub struct HiHat {
    graph: System<6, 1>,
    output: f32,
}

impl HiHat {
    pub fn new() -> Result<Self, String> {
        let sources = [
            simple_source(SquareWave::new(123.0, 1.0)),
            simple_source(SquareWave::new(150.0, 1.0)),
            simple_source(SquareWave::new(180.0, 1.0)),
            simple_source(SquareWave::new(219.0, 1.0)),
            simple_source(SquareWave::new(240.0, 1.0)),
            simple_source(SquareWave::new(261.0, 1.0)),
        ];

        let mut system = System::<6, 1>::new();
        let combinator: CombinatorFilter<6, 1> = CombinatorFilter::new();
        let combinator_index = system.add_filter(Box::from(combinator));
        sources
            .into_iter()
            .enumerate()
            .for_each(|(index, source)| {
                system.set_source(index, source);
                system.connect_source(index, combinator_index, index);
            });

        let sink: SimpleSink = SimpleSink::new();
        system.set_sink(0, Box::from(sink));
        system.connect_sink(combinator_index, 0, 0);

        system.compute().map_err(|_| "Failed to compute".to_string())?;
        Ok(Self {
            graph: system,
            output: 0.0,
        })
    }

    pub fn play(&mut self) {
        self.graph.run();
    }
}
