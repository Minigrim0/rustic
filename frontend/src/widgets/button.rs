use log::error;
use std::rc::Rc;

use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::{
    rect::Rect,
    video::{Window, WindowContext},
};

use crate::manager::FontDetails;
use crate::manager::{FontManager, TextureManager};

pub struct ButtonBuilder {
    text: String,
    position: Rect,
    font: Option<FontDetails>,
    color: Color,
}

impl ButtonBuilder {
    pub fn new<S: AsRef<str>>(text: S) -> Self {
        ButtonBuilder {
            text: text.as_ref().to_string(),
            position: Rect::new(0, 0, 0, 0),
            font: None,
            color: Color::RGBA(0, 0, 0, 0),
        }
    }

    pub fn position(mut self, position: Rect) -> Self {
        self.position = position;
        self
    }

    pub fn font(mut self, font: FontDetails) -> Self {
        self.font = Some(font);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn build<'a>(
        self,
        font_manager: &mut FontManager,
        texture_manager: &mut TextureManager<'a, WindowContext>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Button {
        let font = font_manager.load(&self.font.as_ref().unwrap()).unwrap();
        let button_text = font.render(&self.text).blended(self.color).unwrap();
        let button_texture = texture_creator
            .create_texture_from_surface(button_text)
            .unwrap();
        if let Err(e) = texture_manager.save(self.text.as_str(), Rc::from(button_texture)) {
            error!("Failed to save button texture: {}", e);
        }

        Button {
            text: self.text,
            position: self.position,
            font: self.font.unwrap(),
        }
    }
}

pub struct Button {
    text: String,
    position: Rect,
    font: FontDetails,
}

impl Button {
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        texture_manager: &mut TextureManager<'_, WindowContext>,
    ) {
        let texture = texture_manager.load(&self.text.as_str()).unwrap();
        // Render the button using the texture
        let TextureQuery { width, height, .. } = texture.query();
        let position = Rect::new(self.position.x, self.position.y, width, height);
        canvas
            .copy(&texture, None, position.centered_on(self.position.center()))
            .unwrap();
    }
}
