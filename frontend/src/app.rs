use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::video::{Window, WindowContext};
use sdl2::{event::Event, render::Canvas};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use super::mapping::KeyMapper;
use rustic::prelude::Commands;

use super::scenes::Scene;
use crate::manager::{FontManager, TextureManager};

const MENU_SCENE: usize = 0;

pub struct App {
    pub scenes: Vec<Box<dyn Scene>>,
    pub current_scene: usize,
    pub _app_receiver: Receiver<Commands>,
    pub mapping: KeyMapper,
    pub rustic_apphandle: JoinHandle<()>,

    /// If not none, the current command waits for a second input
    /// to complete the command and send it to the app.
    pub multi_command: Option<Commands>,
}

impl App {
    pub fn new<'a>(
        texture_manager: &mut TextureManager<'a, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
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

        let (frontend_sender, backend_receiver): (Sender<Commands>, Receiver<Commands>) =
            mpsc::channel();
        let (backend_sender, frontend_receiver): (Sender<Commands>, Receiver<Commands>) =
            mpsc::channel();

        let rustic_apphandle = rustic::start_app(backend_sender, backend_receiver);
        Ok(App {
            scenes,
            current_scene: 0,
            _app_receiver: frontend_receiver,
            mapping: KeyMapper::new(frontend_sender),
            rustic_apphandle,
            multi_command: None,
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
                        // Direct-hit commands (one key press)
                        let command = if self.multi_command.is_none() {
                            if let Some(multi_input_command) =
                                super::translator::multi_input_command(&event)
                            {
                                self.multi_command = Some(multi_input_command);
                                None
                            } else {
                                super::translator::event_to_command(&event)
                            }
                        } else {
                            if let Some(command) = super::translator::multi_input_second_stroke(
                                &event,
                                &self.multi_command.as_ref().unwrap(),
                            ) {
                                self.multi_command = None;
                                Some(command)
                            } else {
                                None
                            }
                        };

                        if let Some(command) = command {
                            let _ = self.app_sender.send(command);
                        }
                    }
                }
            }

            canvas.present();
        }
    }
}
