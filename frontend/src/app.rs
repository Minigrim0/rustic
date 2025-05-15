use rustic::inputs::commands::Commands;
use rustic::prelude::{InputSystemConfig, RunMode};
use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::video::{Window, WindowContext};
use sdl2::{event::Event, render::Canvas};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use log::warn;

use super::scenes::Scene;
use crate::manager::{FontManager, TextureManager};

const MENU_SCENE: usize = 0;

pub struct App {
    pub scenes: Vec<Box<dyn Scene>>,
    pub current_scene: usize,
    pub app_sender: Sender<Commands>,
    pub _app_receiver: Receiver<Commands>,

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
        let (_backend_sender, frontend_receiver): (Sender<Commands>, Receiver<Commands>) =
            mpsc::channel();

        let _app_handle = thread::spawn(move || {
            let mut rustic_app = rustic::prelude::App::new();
            rustic_app.set_mode(RunMode::Live(InputSystemConfig::External));

            let og_interval: u128 = (1e6 / rustic_app.config.system.sample_rate as f32) as u128;
            let mut micros_debt: u128 = 0;
            loop {
                let now = Instant::now();

                if let Ok(command) = backend_receiver.recv_timeout(Duration::from_micros(1)) {
                    rustic_app.on_event(command);
                }
                rustic_app.live_tick();

                let interval = if micros_debt > 0 {
                    let max_recup = std::cmp::min(og_interval, micros_debt);
                    let adjusted = og_interval - max_recup;
                    micros_debt -= max_recup;
                    Duration::from_micros(adjusted as u64)
                } else {
                    Duration::from_micros(og_interval as u64)
                };

                if now.elapsed() > interval {
                    micros_debt += (now.elapsed() - interval).as_micros();
                    warn!("desynced timing total {}micros", micros_debt);
                }

                while now.elapsed() < interval {}
            }
        });

        Ok(App {
            scenes,
            current_scene: 0,
            app_sender: frontend_sender,
            _app_receiver: frontend_receiver,
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
