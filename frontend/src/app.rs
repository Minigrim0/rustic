use std::collections::HashMap;

use rustic::prelude::RunMode;
use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::video::{Window, WindowContext};
use sdl2::{event::Event, render::Canvas};

use super::scenes::Scene;
use crate::manager::{FontManager, TextureManager};

const MENU_SCENE: usize = 0;
const HOME_SCENE: usize = 1;
const LIVE_SCENE: usize = 2;
const SCORE_SCENE: usize = 3;

pub struct App {
    pub scenes: Vec<Box<dyn Scene>>,
    pub current_scene: usize,
    pub rustic_app: rustic::prelude::App,
}

impl App {
    pub fn new<'a>(
        texture_manager: &mut TextureManager<'a, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        let mut rustic_app = rustic::prelude::App::new();
        rustic_app.set_mode(RunMode::Live);
        let mut scenes = Vec::new();

        scenes.push(Box::new({
            let mut scene = super::scenes::MenuScene::new();
            scene.load(texture_manager, font_manager, texture_creator);
            scene
        }) as Box<dyn Scene>);

        scenes.push(Box::new({
            let mut scene = super::scenes::HomeScene::new();
            scene.load(texture_manager, font_manager, texture_creator);
            scene
        }) as Box<dyn Scene>);

        scenes.push(Box::new({
            let mut scene = super::scenes::LiveScene::new();
            scene.load(texture_manager, font_manager, texture_creator);
            scene
        }) as Box<dyn Scene>);

        scenes.push(Box::new({
            let mut scene = super::scenes::ScoreScene::new();
            scene.load(texture_manager, font_manager, texture_creator);
            scene
        }) as Box<dyn Scene>);

        scenes.push(Box::new({
            let mut scene = super::scenes::SettingsScene::new();
            scene.load(texture_manager, font_manager, texture_creator);
            scene
        }) as Box<dyn Scene>);

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
            canvas.set_draw_color(Color::RGB(30, 35, 41));
            canvas.clear();
            let elapsed = time_system.ticks();
            self.scenes
                .get_mut(MENU_SCENE)
                .and_then(|m| Some(m.update(elapsed)));

            self.scenes
                .get_mut(self.current_scene)
                .and_then(|s| Some(s.update(elapsed)));

            self.scenes[MENU_SCENE].draw(
                &mut canvas,
                sdl2::rect::Rect::new(0, 0, 800, 60),
                font_manager,
                texture_manager,
            );

            self.scenes[self.current_scene].draw(
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
                            .get_mut(MENU_SCENE)
                            .and_then(|m| Some(m.handle_events(&event)));
                        self.scenes
                            .get_mut(self.current_scene)
                            .and_then(|s| Some(s.handle_events(&event)));
                    }
                }
            }

            self.rustic_app.tick(None);
            canvas.present();
        }
    }
}
