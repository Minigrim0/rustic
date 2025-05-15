use rustic::inputs::commands::Commands;
use sdl2::{event::Event, render::TextureCreator, ttf::FontStyle, video::WindowContext};

use crate::manager::{FontDetails, FontManager, TextureManager};

pub struct LiveScene {
    // Some fields
    pub _last_input: (Option<Event>, Option<Commands>),
    pub _font: FontDetails,
}

impl LiveScene {
    pub fn new() -> Self {
        LiveScene {
            _last_input: (None, None),
            _font: FontDetails {
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
        _resource_manager: &mut TextureManager<'b, WindowContext>,
        _font_manager: &mut FontManager,
        _texture_creator: &'b TextureCreator<WindowContext>,
    ) {
    }

    /// Updates the scene depending on the amount of time (ms) elapsed
    fn update(&mut self, _delta_time: u32) {}

    /// Draws the scene on the screen
    fn draw(
        &self,
        _canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        _area: sdl2::rect::Rect,
        _font_manager: &mut FontManager,
        _texture_manager: &mut TextureManager<WindowContext>,
    ) {
    }

    fn handle_events(&mut self, _event: &sdl2::event::Event) {}
}
