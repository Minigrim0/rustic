use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use super::{ColorMap, VisualizationOptions};

#[derive(Properties, PartialEq)]
pub struct SpectrogramVisualizerProps {
    pub spectrogram: Vec<Vec<f32>>,
    pub sample_rate: u32,
    #[prop_or_default]
    pub options: Option<VisualizationOptions>,
    #[prop_or(ColorMap::Viridis)]
    pub color_map: ColorMap,
    #[prop_or(0.0)]
    pub min_frequency: f32,
    #[prop_or(0.0)]
    pub max_frequency: f32,
}

pub struct SpectrogramVisualizer {
    canvas_ref: NodeRef,
}

impl Component for SpectrogramVisualizer {
    type Message = ();
    type Properties = SpectrogramVisualizerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let options = ctx.props().options.clone().unwrap_or_default();

        html! {
            <div class="spectrogram-visualizer">
                <canvas
                    ref={self.canvas_ref.clone()}
                    width={options.width.to_string()}
                    height={options.height.to_string()}
                    style="width: 100%; height: 100%;"
                />
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render || !ctx.props().spectrogram.is_empty() {
            self.draw_spectrogram(ctx);
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.draw_spectrogram(ctx);
        true
    }
}

impl SpectrogramVisualizer {
    fn draw_spectrogram(&self, ctx: &Context<Self>) {
        let canvas = match self.canvas_ref.cast::<HtmlCanvasElement>() {
            Some(canvas) => canvas,
            None => return,
        };

        let context = match canvas
            .get_context("2d")
            .ok()
            .flatten()
            .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
        {
            Some(context) => context,
            None => return,
        };

        let props = ctx.props();
        let options = props.options.clone().unwrap_or_default();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        // Clear canvas
        context.set_fill_style_str(&options.background_color);
        context.fill_rect(0.0, 0.0, width, height);

        if props.spectrogram.is_empty() || props.spectrogram[0].is_empty() {
            return;
        }

        // Get spectrogram dimensions
        let time_frames = props.spectrogram.len();
        let freq_bins = props.spectrogram[0].len();

        // Determine frequency range to display
        let nyquist = props.sample_rate as f32 / 2.0;
        let min_freq = if props.min_frequency > 0.0 {
            props.min_frequency
        } else {
            0.0
        };
        let max_freq = if props.max_frequency > 0.0 {
            props.max_frequency.min(nyquist)
        } else {
            nyquist
        };

        // Calculate bin range to display
        let bin_width = nyquist / freq_bins as f32;
        let min_bin = (min_freq / bin_width).floor() as usize;
        let max_bin = ((max_freq / bin_width).ceil() as usize).min(freq_bins);
        let bins_to_display = max_bin - min_bin;

        // Find the maximum magnitude for normalization
        let mut max_magnitude = 0.0_f32;
        for frame in &props.spectrogram {
            for bin in min_bin..max_bin {
                if bin < frame.len() {
                    max_magnitude = max_magnitude.max(frame[bin]);
                }
            }
        }

        // Pixel dimensions for each spectrogram cell
        let x_step = width / time_frames as f64;
        let y_step = height / bins_to_display as f64;

        // Draw the spectrogram
        for (frame_idx, frame) in props.spectrogram.iter().enumerate() {
            let x = frame_idx as f64 * x_step;

            for bin_offset in 0..bins_to_display {
                let bin = min_bin + bin_offset;
                if bin < frame.len() {
                    // Get normalized magnitude
                    let magnitude = if max_magnitude > 0.0 {
                        frame[bin] / max_magnitude
                    } else {
                        0.0
                    };

                    // Get color for this magnitude using the selected color map
                    let color = props.color_map.get_color(magnitude);
                    context.set_fill_style_str(&color);

                    // Draw the bin cell (inverted Y-axis so low frequencies are at bottom)
                    let y = height - (bin_offset as f64 + 1.0) * y_step;
                    context.fill_rect(x, y, x_step.ceil(), y_step.ceil());
                }
            }
        }

        // Draw time and frequency axes if enabled
        if options.show_grid {
            self.draw_grid(&context, width, height, time_frames, &options);
        }

        if options.show_labels {
            self.draw_labels(
                &context,
                width,
                height,
                time_frames,
                min_freq,
                max_freq,
                props.sample_rate,
                &options,
            );
        }
    }

    fn draw_grid(
        &self,
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        _time_frames: usize,
        _options: &VisualizationOptions,
    ) {
        context.set_stroke_style_str("rgba(255, 255, 255, 0.2)");
        context.set_line_width(1.0);

        // Draw horizontal grid lines (frequency axis)
        for i in 0..5 {
            let y = height * i as f64 / 4.0;
            context.begin_path();
            context.move_to(0.0, y);
            context.line_to(width, y);
            context.stroke();
        }

        // Draw vertical grid lines (time axis)
        for i in 0..5 {
            let x = width * i as f64 / 4.0;
            context.begin_path();
            context.move_to(x, 0.0);
            context.line_to(x, height);
            context.stroke();
        }
    }

    fn draw_labels(
        &self,
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        time_frames: usize,
        min_freq: f32,
        max_freq: f32,
        sample_rate: u32,
        _options: &VisualizationOptions,
    ) {
        context.set_fill_style_str("white");
        context.set_font("12px sans-serif");

        // Draw frequency axis labels (y-axis)
        for i in 0..5 {
            let y = height * i as f64 / 4.0;
            let freq = max_freq - (max_freq - min_freq) * i as f32 / 4.0;

            let label = if freq >= 1000.0 {
                format!("{:.1}k", freq / 1000.0)
            } else {
                format!("{}", freq as i32)
            };

            context.fill_text(&label, 5.0, y + 15.0).unwrap();
        }

        // Draw time axis labels (x-axis)
        // Estimate duration from frame count (assuming standard hop size)
        let duration_sec = time_frames as f32 * (1024.0 / 4.0) / sample_rate as f32;

        for i in 0..5 {
            let x = width * i as f64 / 4.0;
            let time = duration_sec * i as f32 / 4.0;
            let label = format!("{:.1}s", time);

            context.fill_text(&label, x - 10.0, height - 5.0).unwrap();
        }

        // Draw title
        context.set_font("14px sans-serif");
        context
            .fill_text("Spectrogram", width / 2.0 - 50.0, 20.0)
            .unwrap();
    }
}
