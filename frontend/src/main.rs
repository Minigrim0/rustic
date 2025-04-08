mod attributes;
mod render;
mod state;
mod utils;
mod widgets;
mod window;

pub trait Renderable {
    fn vertices(&self) -> (Vec<render::vertex::Vertex>, Vec<u16>);
}

use window::run;

fn main() {
    pollster::block_on(run());
}
