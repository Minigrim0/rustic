use rustic::inputs::commands::Commands;
use sdl2::{event::Event, render::TextureCreator, ttf::FontStyle, video::WindowContext};

use crate::manager::{FontDetails, FontManager, TextureManager};

pub struct LiveScene {
    // Some fields
    pub last_input: (Option<Event>, Option<Commands>),
    pub font: FontDetails,
}

impl LiveScene {
    pub fn new() -> Self {
        LiveScene {
            last_input: (None, None),
            font: FontDetails {
                path: String::from("frontend/res/fonts/Agave/AgaveNerdFont-Regular.ttf"),
                size: 18,
                style: FontStyle::NORMAL,
            },
        }
    }
}

impl super::Scene for LiveScene {
    fn load<'b>(
        &mut self,
        resource_manager: &mut TextureManager<'b, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'b TextureCreator<WindowContext>,
    ) {
    }

    /// Updates the scene depending on the amount of time (ms) elapsed
    fn update(&mut self, delta_time: u32) {}

    /// Draws the scene on the screen
    fn draw(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        area: sdl2::rect::Rect,
        font_manager: &mut FontManager,
        texture_manager: &mut TextureManager<WindowContext>,
    ) {
    }

    fn handle_events(&mut self, event: &sdl2::event::Event) {}
}
