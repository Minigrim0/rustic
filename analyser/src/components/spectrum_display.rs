use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SpectrumDisplayProps {
    pub spectrogram: Vec<Vec<f32>>,
    pub min_frequency: Option<f32>,
    pub max_frequency: Option<f32>,
    pub sample_rate: u32,
}

pub struct SpectrumDisplay {
    canvas_ref: NodeRef,
    context: Option<CanvasRenderingContext2d>,
    _render_task: Option<Timeout>,
}

pub enum Msg {
    Render,
}

impl Component for SpectrumDisplay {
    type Message = Msg;
    type Properties = SpectrumDisplayProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Schedule a render task after the component has been mounted
        let link = ctx.link().clone();
        let render_task = Timeout::new(50, move || {
            link.send_message(Msg::Render);
        });

        Self {
            canvas_ref: NodeRef::default(),
            context: None,
            _render_task: Some(render_task),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render => {
                // Initialize canvas context if needed
                if self.context.is_none() {
                    if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                        match canvas.get_context("2d") {
                            Ok(Some(context)) => {
                                self.context =
                                    Some(context.unchecked_into::<CanvasRenderingContext2d>());
                            }
                            _ => {
                                log::error!("Failed to get canvas context");
                                return false;
                            }
                        }
                    }
                }

                // Render the spectrogram
                if let Some(context) = &self.context {
                    self.render_spectrogram(context, ctx.props());
                }

                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="spectrum-display">
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="800"
                    height="400"
                    style="width: 100%; height: 300px; background-color: #111;"
                />
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        // Re-render when props change
        if let Some(context) = &self.context {
            self.render_spectrogram(context, ctx.props());
        }

        false
    }
}

impl SpectrumDisplay {
    fn render_spectrogram(&self, context: &CanvasRenderingContext2d, props: &SpectrumDisplayProps) {
        if props.spectrogram.is_empty() {
            return;
        }

        let canvas = context.canvas().unwrap();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        // Clear the canvas
        context.clear_rect(0.0, 0.0, width, height);

        // Get spectrogram dimensions
        let num_frames = props.spectrogram.len();
        let num_bins = props.spectrogram[0].len();

        // Determine frequency range to display
        let min_freq = props.min_frequency.unwrap_or(0.0);
        let max_freq = props
            .max_frequency
            .unwrap_or(props.sample_rate as f32 / 2.0);

        // Calculate bin range to display based on frequency range
        let bin_width = props.sample_rate as f32 / (2.0 * num_bins as f32);
        let min_bin = (min_freq / bin_width).floor() as usize;
        let max_bin = ((max_freq / bin_width).ceil() as usize).min(num_bins);

        // Calculate pixel dimensions
        let x_scale = width / num_frames as f64;
        let y_scale = height / (max_bin - min_bin) as f64;

        // Find the maximum magnitude for normalization
        let mut max_magnitude: f32 = 0.0;
        for frame in &props.spectrogram {
            for &magnitude in frame.iter().take(max_bin).skip(min_bin) {
                max_magnitude = max_magnitude.max(magnitude);
            }
        }

        // Draw the spectrogram
        for (x, frame) in props.spectrogram.iter().enumerate() {
            for bin in min_bin..max_bin {
                if bin < frame.len() {
                    let magnitude = frame[bin] / max_magnitude;

                    // Map magnitude to color (using a heat map: black -> red -> yellow -> white)
                    let (r, g, b) = if magnitude < 0.33 {
                        let v = magnitude * 3.0;
                        (v * 255.0, 0.0, 0.0)
                    } else if magnitude < 0.66 {
                        let v = (magnitude - 0.33) * 3.0;
                        (255.0, v * 255.0, 0.0)
                    } else {
                        let v = (magnitude - 0.66) * 3.0;
                        (255.0, 255.0, v * 255.0)
                    };

                    // Draw the pixel
                    context
                        .set_fill_style_str(&format!("rgb({},{},{})", r as u8, g as u8, b as u8));

                    let x_pos = x as f64 * x_scale;
                    let y_pos = height - (bin - min_bin) as f64 * y_scale;

                    context.fill_rect(x_pos, y_pos - y_scale, x_scale, y_scale);
                }
            }
        }

        // Draw frequency axis labels
        context.set_fill_style_str("white");
        context.set_font("12px sans-serif");

        // Draw 5 evenly spaced frequency labels
        for i in 0..5 {
            let freq = min_freq + (max_freq - min_freq) * i as f32 / 4.0;
            let y_pos = height - (((freq / bin_width) as usize - min_bin) as f64 * y_scale);

            context
                .fill_text(&format!("{:.0} Hz", freq), 5.0, y_pos - 5.0)
                .unwrap();
        }

        // Draw time axis labels if we have time information
        if num_frames > 1 {
            let duration = num_frames as f32 * num_frames as f32 / props.sample_rate as f32;

            for i in 0..5 {
                let time = duration * i as f32 / 4.0;
                let x_pos = width * i as f64 / 4.0;

                context
                    .fill_text(&format!("{:.2} s", time), x_pos, height - 5.0)
                    .unwrap();
            }
        }
    }
}
