use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use super::{ScaleMode, VisualizationOptions};

#[derive(Properties, PartialEq)]
pub struct FrequencyVisualizerProps {
    pub frequencies: Vec<(f32, f32)>, // (frequency, magnitude) pairs
    pub sample_rate: u32,
    #[prop_or_default]
    pub options: Option<VisualizationOptions>,
}

pub struct FrequencyVisualizer {
    canvas_ref: NodeRef,
}

impl Component for FrequencyVisualizer {
    type Message = ();
    type Properties = FrequencyVisualizerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let options = _ctx.props().options.clone().unwrap_or_default();

        html! {
            <div class="frequency-visualizer">
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
        if first_render || ctx.props().frequencies.len() > 0 {
            self.draw_frequency_spectrum(ctx);
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.draw_frequency_spectrum(ctx);
        true
    }
}

impl FrequencyVisualizer {
    fn draw_frequency_spectrum(&self, ctx: &Context<Self>) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let options = ctx.props().options.clone().unwrap_or_default();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        // Clear canvas
        context.set_fill_style_str(&options.background_color);
        context.fill_rect(0.0, 0.0, width, height);

        if ctx.props().frequencies.is_empty() {
            return;
        }

        // Determine frequency range to display
        let nyquist = ctx.props().sample_rate as f32 / 2.0;
        let min_freq = 20.0; // Typical human hearing lower bound
        let max_freq = nyquist.min(20000.0); // Typical human hearing upper bound

        // Set up drawing style
        context.set_stroke_style_str(&options.foreground_color);
        context.set_line_width(2.0);
        context.begin_path();

        // Draw frequency spectrum
        let frequencies = &ctx.props().frequencies;
        let mut first = true;

        for (freq, magnitude) in frequencies {
            // Skip frequencies outside our range of interest
            if *freq < min_freq || *freq > max_freq {
                continue;
            }

            // Apply scaling based on options
            let x = match options.x_scale {
                ScaleMode::Linear => (freq - min_freq) / (max_freq - min_freq) * width as f32,
                ScaleMode::Logarithmic => {
                    ((freq / min_freq).log10() / (max_freq / min_freq).log10()) * width as f32
                }
                ScaleMode::Mel => {
                    // Mel scale formula: m = 2595 * log10(1 + f/700)
                    let mel_min = 2595.0 * (1.0 + min_freq / 700.0).log10();
                    let mel_max = 2595.0 * (1.0 + max_freq / 700.0).log10();
                    let mel = 2595.0 * (1.0 + freq / 700.0).log10();

                    (mel - mel_min) / (mel_max - mel_min) * width as f32
                }
                _ => (freq - min_freq) / (max_freq - min_freq) * width as f32,
            };

            // Apply y-scaling based on options
            let y = match options.y_scale {
                ScaleMode::Linear => height as f32 * (1.0 - magnitude),
                ScaleMode::Logarithmic => {
                    if *magnitude <= 0.0 {
                        height as f32
                    } else {
                        height as f32 * (1.0 - (1.0 + magnitude.ln() / 5.0).max(0.0).min(1.0))
                    }
                }
                ScaleMode::Decibel => {
                    // Convert to dB scale (clamped to -60dB)
                    let db = if *magnitude <= 0.0 {
                        -60.0
                    } else {
                        20.0 * magnitude.log10()
                    };
                    height as f32 * (1.0 - ((db + 60.0) / 60.0).max(0.0).min(1.0))
                }
                _ => height as f32 * (1.0 - magnitude),
            };

            if first {
                context.move_to(x as f64, y as f64);
                first = false;
            } else {
                context.line_to(x as f64, y as f64);
            }
        }

        context.stroke();

        // Draw grid and labels if enabled
        if options.show_grid {
            self.draw_grid(&context, width, height, min_freq, max_freq, &options);
        }

        if options.show_labels {
            self.draw_labels(&context, width, height, min_freq, max_freq, &options);
        }
    }

    fn draw_grid(
        &self,
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        min_freq: f32,
        max_freq: f32,
        options: &VisualizationOptions,
    ) {
        context.set_stroke_style_str("rgba(255, 255, 255, 0.2)");
        context.set_line_width(1.0);

        // Draw horizontal grid lines
        for i in 0..5 {
            let y = height * i as f64 / 4.0;
            context.begin_path();
            context.move_to(0.0, y);
            context.line_to(width, y);
            context.stroke();
        }

        // Draw vertical grid lines based on scale mode
        match options.x_scale {
            ScaleMode::Logarithmic => {
                // For logarithmic scale, draw lines at decades
                let min_decade = (min_freq.log10().floor() as i32).max(1);
                let max_decade = (max_freq.log10().ceil() as i32).min(5);

                for decade in min_decade..=max_decade {
                    let freq = 10.0_f32.powi(decade);
                    if freq >= min_freq && freq <= max_freq {
                        let x = ((freq / min_freq).log10() / (max_freq / min_freq).log10())
                            * width as f32;

                        context.begin_path();
                        context.move_to(x as f64, 0.0);
                        context.line_to(x as f64, height);
                        context.stroke();
                    }
                }
            }
            _ => {
                // For linear scale, draw evenly spaced lines
                for i in 0..5 {
                    let x = width * i as f64 / 4.0;
                    context.begin_path();
                    context.move_to(x, 0.0);
                    context.line_to(x, height);
                    context.stroke();
                }
            }
        }
    }

    fn draw_labels(
        &self,
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        min_freq: f32,
        max_freq: f32,
        options: &VisualizationOptions,
    ) {
        context.set_fill_style_str("white");
        context.set_font("12px sans-serif");

        // Draw amplitude labels
        for i in 0..5 {
            let y = height * i as f64 / 4.0;
            let label = match options.y_scale {
                ScaleMode::Decibel => {
                    format!("{} dB", -60 + i * 15)
                }
                ScaleMode::Logarithmic => {
                    format!("{:.2}", (1.0 - i as f32 / 4.0).exp())
                }
                _ => {
                    format!("{:.1}", 1.0 - i as f32 / 4.0)
                }
            };

            context.fill_text(&label, 5.0, y + 15.0).unwrap();
        }

        // Draw frequency labels based on scale mode
        match options.x_scale {
            ScaleMode::Logarithmic => {
                // For logarithmic scale, draw labels at decades
                let min_decade = (min_freq.log10().floor() as i32).max(1);
                let max_decade = (max_freq.log10().ceil() as i32).min(5);

                for decade in min_decade..=max_decade {
                    let freq = 10.0_f32.powi(decade);
                    if freq >= min_freq && freq <= max_freq {
                        let x = ((freq / min_freq).log10() / (max_freq / min_freq).log10())
                            * width as f32;
                        let label = if freq >= 1000.0 {
                            format!("{:.1}k", freq / 1000.0)
                        } else {
                            format!("{}", freq as i32)
                        };

                        context
                            .fill_text(&label, x as f64 - 10.0, height - 5.0)
                            .unwrap();
                    }
                }
            }
            _ => {
                // For linear scale, draw evenly spaced labels
                for i in 0..5 {
                    let x = width * i as f64 / 4.0;
                    let freq = min_freq + (max_freq - min_freq) * i as f32 / 4.0;
                    let label = if freq >= 1000.0 {
                        format!("{:.1}k", freq / 1000.0)
                    } else {
                        format!("{}", freq as i32)
                    };

                    context.fill_text(&label, x - 10.0, height - 5.0).unwrap();
                }
            }
        }

        // Draw title
        context.set_font("14px sans-serif");
        context
            .fill_text("Frequency Spectrum", width / 2.0 - 70.0, 20.0)
            .unwrap();
    }
}
