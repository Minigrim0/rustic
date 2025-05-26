use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use crate::components::utils;

#[derive(Properties, PartialEq)]
pub struct AudioVisualizerProps {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub struct AudioVisualizer {
    canvas_ref: NodeRef,
    animation_frame_id: Option<i32>,
}

pub enum Msg {
    Render,
    AnimationFrame,
}

impl Component for AudioVisualizer {
    type Message = Msg;
    type Properties = AudioVisualizerProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Schedule initial render
        ctx.link().send_message(Msg::Render);

        Self {
            canvas_ref: NodeRef::default(),
            animation_frame_id: None,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="visualizer-container">
                <h3>{"Waveform Visualization"}</h3>
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="600"
                    height="200"
                    class="waveform-canvas"
                />
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render => {
                // Cancel any existing animation frame
                if let Some(id) = self.animation_frame_id {
                    let window = web_sys::window().unwrap();
                    window.cancel_animation_frame(id).unwrap();
                    self.animation_frame_id = None;
                }

                // Start new animation
                self.request_animation_frame(ctx);
                false
            }
            Msg::AnimationFrame => {
                self.render_waveform(ctx);
                self.animation_frame_id = None;
                self.request_animation_frame(ctx);
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _props: &Self::Properties) -> bool {
        // Re-render when props change
        ctx.link().send_message(Msg::Render);
        false
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // Clean up animation frame on component destruction
        if let Some(id) = self.animation_frame_id {
            if let Some(window) = web_sys::window() {
                let _ = window.cancel_animation_frame(id);
            }
        }
    }
}

impl AudioVisualizer {
    fn request_animation_frame(&mut self, ctx: &Context<Self>) {
        // Schedule next animation frame
        let link = ctx.link().clone();
        let callback = Closure::once(move || {
            link.send_message(Msg::AnimationFrame);
        });

        if let Some(window) = web_sys::window() {
            match window.request_animation_frame(callback.as_ref().unchecked_ref()) {
                Ok(id) => {
                    self.animation_frame_id = Some(id);
                    callback.forget(); // Prevent callback from being dropped
                }
                Err(_) => {
                    log::error!("Error requesting animation frame");
                }
            }
        }
    }

    fn render_waveform(&self, ctx: &Context<Self>) {
        if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
            if let Ok(ctx_2d) = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
            {
                let width = canvas.width() as f64;
                let height = canvas.height() as f64;

                // Clear canvas
                ctx_2d.clear_rect(0.0, 0.0, width, height);

                let samples = &ctx.props().samples;
                if samples.is_empty() {
                    return;
                }

                // Normalize samples
                let normalized = utils::normalize_samples(samples);

                // Draw waveform
                ctx_2d.set_stroke_style_str("#2196F3");
                ctx_2d.set_line_width(2.0);
                ctx_2d.begin_path();

                let step = (normalized.len() as f64 / width).max(1.0);
                let mid_height = height / 2.0;

                for i in 0..(width as usize) {
                    let sample_idx = (i as f64 * step) as usize;
                    if sample_idx < normalized.len() {
                        let sample = normalized[sample_idx];
                        let y = mid_height * (1.0 - sample as f64);

                        if i == 0 {
                            ctx_2d.move_to(0.0, y);
                        } else {
                            ctx_2d.line_to(i as f64, y);
                        }
                    }
                }

                ctx_2d.stroke();
            }
        }
    }
}
