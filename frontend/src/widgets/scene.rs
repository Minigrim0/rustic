use crate::Renderable;

pub struct Scene {
    pub elements: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            elements: Vec::new(),
        }
    }
}

impl Renderable for Scene {
    fn render(&self) {
        for element in &self.elements {
            element.render();
        }
    }
}
