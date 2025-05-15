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
            color: Color::WHITE,
        }
    }

    /// Sets the position & size of the button from the given rect.
    pub fn rposition(mut self, position: Rect) -> Self {
        self.position = position;
        self
    }

    /// Sets the position of the button
    pub fn _position(mut self, position: (i32, i32)) -> Self {
        self.position = Rect::new(
            position.0,
            position.1,
            self.position.w as u32,
            self.position.h as u32,
        );
        self
    }

    /// Sets the size of the button
    pub fn _size(mut self, size: (u32, u32)) -> Self {
        self.position = Rect::new(self.position.x, self.position.y, size.0, size.1);
        self
    }

    /// Sets the font of the button
    pub fn font(mut self, font: FontDetails) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the color of the text of the button
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

        Button {
            text: self.text,
            _bold: bold_font_id,
            position: self.position,
            ..Default::default()
        }
    }
}

pub struct Button {
    text: String,
    _bold: String,
    position: Rect,
    hovered: bool,
    _selected: bool,
}

impl Default for Button {
    fn default() -> Button {
        Button {
            text: String::default(),
            _bold: String::default(),
            position: Rect::new(0, 0, 0, 0),
            hovered: false,
            _selected: false,
        }
    }
}

impl Button {
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        texture_manager: &mut TextureManager<'_, WindowContext>,
    ) {
        canvas.set_draw_color(Color::RGB(240, 240, 240));
        if let Err(e) = canvas.draw_line(
            (self.position.x, self.position.y),
            (self.position.x, self.position.y + self.position.h),
        ) {
            error!("Unable to draw rect: {}", e);
        }
        if let Err(e) = canvas.draw_line(
            (self.position.x + self.position.w, self.position.y),
            (
                self.position.x + self.position.w,
                self.position.y + self.position.h,
            ),
        ) {
            error!("Unable to draw rect: {}", e);
        }

        let texture = if self.hovered {
            texture_manager.load(&self._bold.as_str()).unwrap()
        } else {
            texture_manager.load(&self.text.as_str()).unwrap()
        };

        // Render the button using the texture
        let TextureQuery { width, height, .. } = texture.query();
        let position = Rect::new(self.position.x, self.position.y, width, height);
        canvas
            .copy(&texture, None, position.centered_on(self.position.center()))
            .unwrap();
    }

    /// Updates the button, returns true if the event has been handled
    /// and should not be propagated.
    pub fn update(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseMotion { x, y, .. } => {
                self.hovered = self.position.contains_point((*x, *y));
                false
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                if self.position.contains_point((*x, *y)) && mouse_btn == &MouseButton::Left {
                    // TODO: Handle click
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
