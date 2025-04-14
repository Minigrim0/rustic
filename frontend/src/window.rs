use winit::dpi::PhysicalSize;
// use winit::window::Fullscreen;
use winit::window::WindowBuilder;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use super::state::State;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let size = PhysicalSize::new(800, 600);
    // let monitor = event_loop.primary_monitor().unwrap();
    // let video_mode = monitor.video_modes().next();
    // let size = video_mode
    //     .clone()
    //     .map_or(PhysicalSize::new(800, 600), |vm| vm.size());
    let window = WindowBuilder::new()
        .with_visible(true)
        .with_title("Rustic")
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new();
    let mut renderer = super::render::Renderer::new(&window, size).await;
    state.set_scene(crate::scenes::prelude::get_main_scene());
    let mut surface_configured = false;

    let window = &window;
    if let Err(e) = event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        log::info!("physical_size: {physical_size:?}");
                        surface_configured = true;
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        if !surface_configured {
                            return;
                        }

                        state.update();
                        renderer.render(state.scene());
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }) {
        log::error!("Event loop error: {}", e);
    }
}
