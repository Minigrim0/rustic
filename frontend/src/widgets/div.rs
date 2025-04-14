use crate::{attributes::prelude::*, render::quadbuffer::QuadBufferBuilder};

#[derive(Default)]
pub struct Divider {
    position: [f32; 2],
    size: [f32; 2],
    color: Color,
    _z_index: f32,
}

impl Divider {
    pub fn new(position: &[f32; 2], size: &[f32; 2]) -> Self {
        Self {
            position: position.clone(),
            size: size.clone(),
            ..Default::default()
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl crate::Renderable for Divider {
    fn render(&self, builder: QuadBufferBuilder) -> QuadBufferBuilder {
        builder.push_quad(
            self.position[0],
            self.position[1],
            self.position[0] + self.size[0],
            self.position[1] + self.size[1],
            self.color,
        )
    }
}
