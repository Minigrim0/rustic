mod main_scene;

pub mod prelude {
    pub use super::main_scene::get_main_scene;
}

use crate::Renderable;
use crate::render::quadbuffer::QuadBufferBuilder;

/// A scene is a collection of renderable elements.
#[derive(Default)]
pub struct Scene {
    pub elements: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: Box<dyn Renderable>) {
        self.elements.push(element);
    }

    pub fn get_vertices(&self) -> QuadBufferBuilder {
        let mut quad_buffer = QuadBufferBuilder::new();
        for element in &self.elements {
            quad_buffer = element.render(quad_buffer);
        }
        quad_buffer
    }
}
