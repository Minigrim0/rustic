mod home;
mod live;
mod menu;
mod score;
mod settings;

use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub use home::HomeScene;
pub use live::LiveScene;
pub use menu::MenuScene;
pub use score::ScoreScene;
pub use settings::SettingsScene;

use crate::manager::{FontManager, TextureManager};

pub trait Scene {
    fn load<'b>(
        &mut self,
        resource_manager: &mut TextureManager<'b, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'b TextureCreator<WindowContext>,
    );

    /// Updates the scene depending on the amount of time (ms) elapsed
    fn update(&mut self, delta_time: u32);

    /// Draws the scene on the screen
    fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        area: sdl2::rect::Rect,
        font_manager: &mut FontManager,
        texture_manager: &mut TextureManager<WindowContext>,
    );

    fn handle_events(&mut self, event: &sdl2::event::Event);
}
