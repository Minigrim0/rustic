use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

/// The actual application logic.
mod app;
/// Resources managers for fonts and textures.
mod manager;
/// Scenes are a collection of widgets & input handlers
mod scenes;
/// Translates user inputs to app commands
mod translator;
/// Widgets for the user interface (buttons, ...)
mod widgets;

use app::App;
use manager::{FontManager, TextureManager};

pub fn main() -> Result<(), String> {
    colog::init();
    let context: sdl2::Sdl = sdl2::init()?;
    let video = context.video()?;
    let window = video
        .window("Rustic", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    let canvas: sdl2::render::Canvas<sdl2::video::Window> = window
        .into_canvas()
        .build()
        .map_err(|e| format!("Failed to create canvas: {}", e))?;

    let mut events: sdl2::EventPump = context.event_pump()?;
    let texture_creator: TextureCreator<WindowContext> = canvas.texture_creator();
    let font_context: sdl2::ttf::Sdl2TtfContext = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut font_manager = FontManager::new(&font_context);
    let mut texture_manager = TextureManager::new(&texture_creator);

    let mut app = match App::new(&mut texture_manager, &mut font_manager, &texture_creator) {
        Ok(app) => app,
        Err(e) => {
            return Err(format!("Error creating app: {}", e));
        }
    };

    app.run(
        canvas,
        &context,
        &mut texture_manager,
        &mut font_manager,
        &mut events,
    );
    Ok(())
}
