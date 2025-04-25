use log::error;
use std::collections::HashMap;

use sdl2::render::TextureCreator;
use std::rc::Rc;

use crate::{
    manager::{FontDetails, FontManager, TextureManager},
    widgets::prelude::{Button, ButtonBuilder},
};

const TOP_TITLE_FONT: &'static str = "TopTitleFont";

use sdl2::{pixels::Color, rect::Rect, render::TextureQuery, video::WindowContext};

pub struct TopMenu {
    fonts: HashMap<&'static str, FontDetails>,
    tabs_buttons: Vec<Button>,
    selected_tab: u8,
}

impl TopMenu {
    pub fn new() -> Self {
        TopMenu {
            fonts: HashMap::from([(
                TOP_TITLE_FONT,
                FontDetails {
                    path: String::from("res/fonts/Agave/AgaveNerdFont-Regular.ttf"),
                    size: 24,
                },
            )]),
            tabs_buttons: Vec::new(),
            selected_tab: 0,
        }
    }
}

impl super::Scene for TopMenu {
    fn load<'b>(
        &mut self,
        texture_manager: &mut TextureManager<'b, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'b TextureCreator<WindowContext>,
    ) {
        let title_font = font_manager.load(&self.fonts[TOP_TITLE_FONT]).unwrap();

        let title = title_font
            .render("Rustic")
            .blended(Color::RGBA(230, 225, 243, 255))
            .unwrap();
        let title_texture = texture_creator.create_texture_from_surface(title).unwrap();
        let TextureQuery { width, height, .. } = title_texture.query();
        if let Err(e) = texture_manager.save("TopMenuTitle", Rc::from(title_texture)) {
            error!("Error saving texture to the texture manager: {}", e);
        }

        self.tabs_buttons.push(
            ButtonBuilder::new("Live")
                .position(Rect::new(width as i32 + 5, 0, 100, height))
                .color(Color::RGBA(230, 225, 243, 255))
                .font(FontDetails {
                    path: String::from("res/fonts/Agave/AgaveNerdFont-Regular.ttf"),
                    size: 16,
                })
                .build(font_manager, texture_manager, texture_creator),
        );
    }

    fn update(&mut self, _delta_time: u32) {}

    fn draw(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        _area: sdl2::rect::Rect,
        font_manager: &mut FontManager,
        texture_manager: &mut TextureManager<WindowContext>,
    ) {
        let title_texture = texture_manager.load("TopMenuTitle").unwrap();

        let TextureQuery {
            width: title_w,
            height: title_h,
            ..
        } = title_texture.query();
        let title_rect = Rect::new(0, 0, title_w, title_h);

        canvas.copy(&title_texture, None, Some(title_rect)).unwrap();
        self.tabs_buttons
            .iter()
            .for_each(|b| b.render(canvas, texture_manager));
    }

    fn handle_events(&mut self, _event: sdl2::event::Event) {}
}
