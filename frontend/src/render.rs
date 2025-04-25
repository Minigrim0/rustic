use log::error;

use rustic::score::{Measure, Staff};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub trait Render {
    fn render(
        &self,
        position: (i32, i32),
        size: (u32, u32),
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    );
}

impl Render for Measure {
    fn render(
        &self,
        position: (i32, i32),
        size: (u32, u32),
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ) {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        if let Err(e) = canvas.fill_rect(Rect::new(position.0, position.1, size.0, size.1)) {
            error!("Failed to fill rectangle: {}", e);
        }
    }
}

impl Render for Staff {
    fn render(
        &self,
        position: (i32, i32),
        size: (u32, u32),
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        if let Err(e) = canvas.fill_rect(Rect::new(position.0, position.1, size.0, size.1)) {
            error!("Failed to fill rectangle: {}", e);
        }
    }
}
