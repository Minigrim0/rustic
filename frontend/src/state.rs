use winit::event::WindowEvent;
use winit::window::Window;

use super::scenes::Scene;

/// The state structure for the application.
/// It contains all the necessary resources and data to run the application.
/// as well as the window and the render pipeline.
pub struct State {
    /// The currently loaded scene.
    scene: Scene,
}

impl State {
    pub fn new() -> State {
        Self {
            scene: Default::default(),
        }
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.scene = scene;
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn update(&mut self) {
        // to implement
    }
}
