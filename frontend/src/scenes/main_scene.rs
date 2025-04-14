use super::Scene;

use crate::{attributes::prelude::Color, widgets::prelude::*};

pub fn get_main_scene() -> Scene {
    let mut scene = Scene::new();

    let mut div = Divider::new(&[-1.0, -1.0], &[0.5, 2.0]);
    div.set_color(Color::new(255, 255, 0, 255));
    scene.add_element(Box::from(div));

    scene
}
