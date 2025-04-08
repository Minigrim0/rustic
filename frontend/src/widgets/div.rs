use crate::{attributes::prelude::*, render::vertex::Vertex};

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
}

impl crate::Renderable for Divider {
    fn vertices(&self) -> (Vec<Vertex>, Vec<u16>) {
        (
            vec![
                Vertex::new(
                    [self.position[0], self.position[1], self._z_index],
                    self.color.as_array(),
                ),
                Vertex::new(
                    [
                        self.position[0],
                        self.position[1] + self.size[1],
                        self._z_index,
                    ],
                    self.color.as_array(),
                ),
                Vertex::new(
                    [
                        self.position[0] + self.size[0],
                        self.position[1] + self.size[1],
                        self._z_index,
                    ],
                    self.color.as_array(),
                ),
                Vertex::new(
                    [
                        self.position[0] + self.size[0],
                        self.position[1],
                        self._z_index,
                    ],
                    self.color.as_array(),
                ),
            ],
            vec![3, 1, 0, 3, 2, 1],
        )
    }
}
