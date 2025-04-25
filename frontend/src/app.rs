use std::collections::HashMap;

use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::video::{Window, WindowContext};
use sdl2::{event::Event, render::Canvas};

use super::scenes::Scene;
use crate::manager::{FontManager, TextureManager};

const TOP_MENU: &'static str = "TopMenu";

pub struct App {
    pub scenes: HashMap<&'static str, Box<dyn Scene>>,
    pub current_scene: usize,
    pub rustic_app: rustic::prelude::App,
}

impl App {
    pub fn new<'a>(
        texture_manager: &mut TextureManager<'a, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        let rustic_app = rustic::prelude::App::new();
        let mut scenes = HashMap::new();

        scenes.insert(
            TOP_MENU,
            Box::new({
                let mut scene = super::scenes::TopMenu::new();
                scene.load(texture_manager, font_manager, texture_creator);
                scene
            }) as Box<dyn Scene>,
        );

        Ok(App {
            scenes,
            current_scene: 0,
            rustic_app,
        })
    }

    pub fn run(
        &mut self,
        mut canvas: Canvas<Window>,
        context: &Sdl,
        texture_manager: &mut TextureManager<WindowContext>,
        font_manager: &mut FontManager,
        events: &mut sdl2::EventPump,
    ) {
        canvas.set_draw_color(Color::RGB(30, 35, 41));
        canvas.clear();
        canvas.present();
        let time_system = context.timer().unwrap();

        'main: loop {
            canvas.clear();
            let elapsed = time_system.ticks();
            self.scenes
                .get_mut(TOP_MENU)
                .and_then(|m| Some(m.update(elapsed)));

            self.scenes[TOP_MENU].draw(
                &mut canvas,
                sdl2::rect::Rect::new(0, 0, 800, 60),
                font_manager,
                texture_manager,
            );

            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,
                    event => {
                        self.scenes
                            .get_mut(TOP_MENU)
                            .and_then(|m| Some(m.handle_events(event)));
                    }
                }
            }

            self.rustic_app.run(vec![]);
            canvas.present();
        }
    }
}
