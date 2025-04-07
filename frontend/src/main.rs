mod attributes;
mod state;
mod vertex;
mod widgets;
mod window;

pub trait ToVertices {
    fn vertices(&self) -> (Vec<vertex::Vertex>, Vec<u16>);
}

use window::run;

fn main() {
    pollster::block_on(run());
}
