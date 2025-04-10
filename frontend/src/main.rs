mod attributes;
mod render;
mod scenes;
mod state;
mod utils;
mod widgets;
mod window;

use render::quadbuffer::QuadBufferBuilder;

pub trait Renderable {
    fn render(&self, builder: QuadBufferBuilder) -> QuadBufferBuilder;
}

use window::run;

fn main() {
    pollster::block_on(run());
}
