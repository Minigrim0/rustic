use std::collections::HashMap;

use log::error;
use sdl2::{
    pixels::Color, rect::Rect, render::TextureCreator, ttf::FontStyle, video::WindowContext,
};

use crate::{
    manager::{FontDetails, FontManager, TextureManager},
    widgets::prelude::{Button, ButtonBuilder, Text, TextBuilder},
};

const TOP_TITLE_FONT: &'static str = "TopTitleFont";

pub struct MenuScene {
    fonts: HashMap<&'static str, FontDetails>,
    title: Text,
    tabs_buttons: Vec<Button>,
    _selected_tab: Option<u8>,
}

impl MenuScene {
    pub fn new() -> Self {
        MenuScene {
            fonts: HashMap::from([(
                TOP_TITLE_FONT,
                FontDetails {
                    path: String::from("frontend/res/fonts/Agave/AgaveNerdFont-Regular.ttf"),
                    size: 24,
                    style: FontStyle::NORMAL,
                },
            )]),
            title: Text::default(),
            tabs_buttons: Vec::new(),
            _selected_tab: None, // None means we are on the home scene
        }
    }
}

impl super::Scene for MenuScene {
    fn load<'b>(
        &mut self,
        texture_manager: &mut TextureManager<'b, WindowContext>,
        font_manager: &mut FontManager,
        texture_creator: &'b TextureCreator<WindowContext>,
    ) {
        let title_width = 75;
        self.title = TextBuilder::new("Rustic")
            .color(Color::RGB(230, 225, 243))
            .font(self.fonts[TOP_TITLE_FONT].clone())
            .rposition(Rect::new(2, 2, title_width, 25))
            .build(font_manager, texture_manager, texture_creator);

        self.tabs_buttons.push(
            ButtonBuilder::new("Live")
                .rposition(Rect::new(title_width as i32 + 5, 4, 60, 25))
                .color(Color::RGBA(230, 225, 243, 255))
                .font(FontDetails {
                    path: String::from("frontend/res/fonts/Agave/AgaveNerdFont-Regular.ttf"),
                    size: 16,
                    style: FontStyle::NORMAL,
                })
                .build(font_manager, texture_manager, texture_creator),
        );

        self.tabs_buttons.push(
            ButtonBuilder::new("Score")
                .rposition(Rect::new(title_width as i32 + 65, 4, 60, 25))
                .color(Color::RGBA(230, 225, 243, 255))
                .font(FontDetails {
                    path: String::from("frontend/res/fonts/Agave/AgaveNerdFont-Regular.ttf"),
                    size: 16,
                    style: FontStyle::NORMAL,
                })
                .build(font_manager, texture_manager, texture_creator),
        );
    }

    fn update(&mut self, _delta_time: u32) {}

    fn draw(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        _area: sdl2::rect::Rect,
        _font_manager: &mut FontManager,
        texture_manager: &mut TextureManager<WindowContext>,
    ) {
        self.title.render(canvas, texture_manager);
        self.tabs_buttons
            .iter()
            .for_each(|b| b.render(canvas, texture_manager));

        canvas.set_draw_color(Color::RGB(245, 230, 225));
        if let Err(e) = canvas.draw_line((2, 30), (798, 30)) {
            error!("Unable to draw line for top menu: {}", e);
        }
    }

    fn handle_events(&mut self, event: &sdl2::event::Event) {
        for button in self.tabs_buttons.iter_mut() {
            if button.update(event) {}
        }
    }
}
