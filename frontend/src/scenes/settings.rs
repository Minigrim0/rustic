pub struct SettingsScene {}

impl SettingsScene {
    pub fn new() -> Self {
        SettingsScene {}
    }
}

impl super::Scene for SettingsScene {
    fn load<'b>(
        &mut self,
        _resource_manager: &mut crate::manager::TextureManager<'b, sdl2::video::WindowContext>,
        _font_manager: &mut crate::manager::FontManager,
        _texture_creator: &'b sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) {
    }

    fn update(&mut self, _delta_time: u32) {}

    fn handle_events(&mut self, _event: &sdl2::event::Event) {}

    fn draw(
        &self,
        _canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        _area: sdl2::rect::Rect,
        _font_manager: &mut crate::manager::FontManager,
        _texture_manager: &mut crate::manager::TextureManager<sdl2::video::WindowContext>,
    ) {
    }
}
