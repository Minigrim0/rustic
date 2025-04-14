use crate::attributes::prelude::Color;

use super::{staging::StagingBuffer, vertex::Vertex};

pub struct QuadBufferBuilder {
    vertex_data: Vec<Vertex>,
    index_data: Vec<u32>,
    current_quad: u32,
}

impl QuadBufferBuilder {
    pub fn new() -> Self {
        Self {
            vertex_data: Vec::new(),
            index_data: Vec::new(),
            current_quad: 0,
        }
    }

    pub fn push_quad(
        mut self,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        color: Color,
    ) -> Self {
        self.vertex_data.extend(&[
            Vertex::new((min_x, min_y), color),
            Vertex::new((max_x, min_y), color),
            Vertex::new((max_x, max_y), color),
            Vertex::new((min_x, max_y), color),
        ]);
        self.index_data.extend(&[
            self.current_quad * 4 + 0,
            self.current_quad * 4 + 1,
            self.current_quad * 4 + 2,
            self.current_quad * 4 + 0,
            self.current_quad * 4 + 2,
            self.current_quad * 4 + 3,
        ]);
        self.current_quad += 1;
        self
    }

    pub fn build(self, device: &wgpu::Device) -> (StagingBuffer, StagingBuffer, u32) {
        (
            StagingBuffer::new(device, &self.vertex_data, false),
            StagingBuffer::new(device, &self.index_data, true),
            self.index_data.len() as u32,
        )
    }
}
