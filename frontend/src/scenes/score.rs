pub struct ScoreScene {}

impl ScoreScene {
    pub fn new() -> Self {
        ScoreScene {}
    }
}

impl super::Scene for ScoreScene {
    fn load<'b>(
        &mut self,
        resource_manager: &mut crate::manager::TextureManager<'b, sdl2::video::WindowContext>,
        font_manager: &mut crate::manager::FontManager,
        texture_creator: &'b sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    ) {
    }

    fn update(&mut self, delta_time: u32) {}

    fn handle_events(&mut self, event: &sdl2::event::Event) {}

    fn draw(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        area: sdl2::rect::Rect,
        font_manager: &mut crate::manager::FontManager,
        texture_manager: &mut crate::manager::TextureManager<sdl2::video::WindowContext>,
    ) {
    }
}
