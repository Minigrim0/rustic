use log::error;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::ttf::FontStyle;
use std::rc::Rc;

use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::{
    rect::Rect,
    video::{Window, WindowContext},
};

use crate::manager::FontDetails;
use crate::manager::{FontManager, TextureManager};

pub struct TextBuilder {
    text: String,
    position: Rect,
    font: Option<FontDetails>,
    color: Color,
}

impl Default for TextBuilder {
    fn default() -> Self {
        TextBuilder {
            text: String::new(),
            position: Rect::new(0, 0, 0, 0),
            font: None,
            color: Color::WHITE,
        }
    }
}

impl TextBuilder {
    pub fn new(text: &str) -> Self {
        TextBuilder {
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// Sets the position & size of the text from the given rect.
    pub fn rposition(mut self, position: Rect) -> Self {
        self.position = position;
        self
    }

    /// Sets the position of the text
    pub fn position(mut self, position: (i32, i32)) -> Self {
        self.position = Rect::new(
            position.0,
            position.1,
            self.position.w as u32,
            self.position.h as u32,
        );
        self
    }

    /// Sets the size of the text
    pub fn size(mut self, size: (u32, u32)) -> Self {
        self.position = Rect::new(self.position.x, self.position.y, size.0, size.1);
        self
    }

    /// Sets the font of the text
    pub fn font(mut self, font: FontDetails) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the color of the text
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn build<'a>(
        self,
        font_manager: &mut FontManager,
        texture_manager: &mut TextureManager<'a, WindowContext>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Text {
        let font = font_manager.load(&self.font.as_ref().unwrap()).unwrap();

        let mut font_bold_details = self.font.as_ref().unwrap().clone();
        font_bold_details.style = FontStyle::BOLD;
        let font_bold = font_manager.load(&font_bold_details).unwrap();

        let button_text = font.render(&self.text).blended(self.color).unwrap();
        let button_text_bold = font_bold.render(&self.text).blended(self.color).unwrap();

        let button_texture = texture_creator
            .create_texture_from_surface(button_text)
            .unwrap();
        let button_bold_texture = texture_creator
            .create_texture_from_surface(button_text_bold)
            .unwrap();

        if let Err(e) = texture_manager.save(self.text.as_str(), Rc::from(button_texture)) {
            error!("Failed to save button texture: {}", e);
        }

        let bold_font_id = format!("{}_bold", self.text);
        if let Err(e) = texture_manager.save(bold_font_id.as_str(), Rc::from(button_bold_texture)) {
            error!("Failed to save button texture: {}", e);
        }

        Text {
            text: self.text,
            position: self.position,
            color: self.color,
        }
    }
}

// TODO: Add uuid for the text to be saved in the manager with.
// The uuid will not change when the text is updated.
pub struct Text {
    text: String,
    position: Rect,
    color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            text: String::new(),
            position: Rect::new(0, 0, 0, 0),
            color: Color::WHITE,
        }
    }
}

impl Text {
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        texture_manager: &mut TextureManager<'_, WindowContext>,
    ) {
        // Render the text using the texture
        let texture = texture_manager.load(self.text.as_str()).unwrap();

        let TextureQuery { width, height, .. } = texture.query();
        let position = Rect::new(self.position.x, self.position.y, width, height);
        canvas
            .copy(&texture, None, position.centered_on(self.position.center()))
            .unwrap();
    }
}
