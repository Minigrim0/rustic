use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FrequencyChartProps {
    // Change this to accept pre-computed frequencies instead of raw samples
    pub frequencies: Vec<(f32, f32)>, // (frequency, magnitude)
    pub sample_rate: u32,
}

pub struct FrequencyChart {
    canvas_ref: NodeRef,
}

impl Component for FrequencyChart {
    type Message = ();
    type Properties = FrequencyChartProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="frequency-chart-container">
                <h3>{"Frequency Analysis"}</h3>
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="600"
                    height="300"
                    class="frequency-canvas"
                />
                <div class="chart-legend">
                    <div class="legend-item">
                        <span class="legend-color" style="background-color: #FF5722;"></span>
                        <span class="legend-label">{"Frequency Magnitude"}</span>
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        self.render_chart(ctx);
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.render_chart(ctx);
        true
    }
}

impl FrequencyChart {
    fn render_chart(&self, ctx: &Context<Self>) {
        let frequency_data = &ctx.props().frequencies;

        if frequency_data.is_empty() {
            return;
        }

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

                // Draw background grid
                self.draw_grid(&ctx_2d, width, height);

                // Draw frequency spectrum
                ctx_2d.set_fill_style_str("#FF5722");

                let bar_width = width / (frequency_data.len() as f64).min(width);

                for (i, (_freq, magnitude)) in frequency_data.iter().enumerate() {
                    let bar_height = height * (*magnitude as f64);
                    let x = i as f64 * bar_width;
                    let y = height - bar_height;

                    ctx_2d.fill_rect(x, y, bar_width - 1.0, bar_height);
                }

                // Draw frequency labels
                self.draw_frequency_labels(&ctx_2d, width, height, frequency_data);
            }
        }
    }

    fn draw_grid(&self, ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
        // Draw background
        ctx.set_fill_style_str("#f5f5f5");
        ctx.fill_rect(0.0, 0.0, width, height);

        // Draw grid lines
        ctx.set_stroke_style_str("#dddddd");
        ctx.set_line_width(1.0);

        // Horizontal grid lines
        let steps = 5;
        for i in 0..=steps {
            let y = (i as f64 * height) / steps as f64;
            ctx.begin_path();
            ctx.move_to(0.0, y);
            ctx.line_to(width, y);
            ctx.stroke();
        }

        // Vertical grid lines
        let freq_steps = 10;
        for i in 0..=freq_steps {
            let x = (i as f64 * width) / freq_steps as f64;
            ctx.begin_path();
            ctx.move_to(x, 0.0);
            ctx.line_to(x, height);
            ctx.stroke();
        }
    }

    fn draw_frequency_labels(
        &self,
        ctx: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        frequency_data: &[(f32, f32)],
    ) {
        ctx.set_fill_style_str("#333333");
        ctx.set_font("12px Arial");
        ctx.set_text_align("center");

        // Draw frequency labels along x-axis
        let max_frequency = frequency_data.last().map(|(f, _)| *f).unwrap_or(20000.0);
        let freq_steps = 10;

        for i in 0..=freq_steps {
            let frequency = (i as f32 * max_frequency) / freq_steps as f32;
            let x = (i as f64 * width) / freq_steps as f64;

            let label = if frequency >= 1000.0 {
                format!("{:.1}k", frequency / 1000.0)
            } else {
                format!("{:.0}", frequency)
            };

            ctx.fill_text(&label, x, height - 5.0).unwrap();
        }

        // Draw magnitude labels along y-axis
        ctx.set_text_align("right");
        let steps = 5;

        for i in 0..=steps {
            let y = height - (i as f64 * height) / steps as f64;
            let value = i as f64 / steps as f64;
            let label = format!("{:.1}", value);

            ctx.fill_text(&label, 25.0, y + 5.0).unwrap();
        }

        // X-axis label
        ctx.set_text_align("center");
        ctx.fill_text("Frequency (Hz)", width / 2.0, height - 20.0)
            .unwrap();

        // Y-axis label
        ctx.save();
        ctx.translate(15.0, height / 2.0).unwrap();
        ctx.rotate(-std::f64::consts::PI / 2.0).unwrap();
        ctx.fill_text("Magnitude", 0.0, 0.0).unwrap();
        ctx.restore();
    }
}
