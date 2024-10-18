use log::info;
use music::generator::sine_wave::SineWave;
use music::generator::Envelope;
use plotters::prelude::*;
use std::time::Instant;

fn main() {
    // Tone Generator
    let sine_gen = SineWave::new(440.0, 1.0);
    let mut envelope = Envelope::new(1000.0, Box::from(sine_gen));

    envelope.set_attack(0.2, 1.0, Some((0.0, 1.0)));
    envelope.set_decay(0.2, 0.6, Some((0.0, 0.6)));
    envelope.set_release(0.4, 0.0, Some((0.4, 0.6)));

    let mut results: Vec<(f32, f32)> = Vec::new();

    let sample_rate = 44100.0; // Hertz
    let duration = 1.0; // Seconds

    info!("Generating one second sample");
    let now = Instant::now();
    for sample in 0..(duration * sample_rate) as i32 {
        // Generate over one second at 1000Hz
        let current_time = sample as f32 / sample_rate;

        let val = if current_time < 0.02 * duration {
            // Note is not on yet
            envelope.get_at(current_time, None, None)
        } else if current_time < 0.5 * duration {
            // Note is on for the moment
            envelope.get_at(current_time, Some(0.02), None)
        } else {
            // Note has turned off
            envelope.get_at(current_time, Some(0.02), Some(0.5))
        };

        results.push((current_time, val));
    }
    let elapsed = now.elapsed();

    info!("Completed");
    println!("Elapsed: {:.4?}", elapsed);

    if let Err(e) = plot_data(results, &envelope, 0.02, 0.5) {
        println!("Error: {}", e.to_string());
    }
}

fn plot_data(
    data: Vec<(f32, f32)>,
    envelope: &Envelope,
    note_on: f32,
    note_off: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("envelope.png", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Envelope", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-0.1f32..1.1f32, -0.1f32..1.1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(data, &RED))?
        .label("Envelope")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // Note important times
    chart.draw_series(PointSeries::of_element(
        vec![
            (note_on, envelope.attack.start_value()),
            (note_on + envelope.attack.end(), envelope.attack.end_value()),
            (note_on + envelope.decay.end(), envelope.decay.end_value()),
            (
                note_off + envelope.release.start(),
                envelope.release.start_value(),
            ),
            (
                note_off + envelope.release.end(),
                envelope.release.end_value(),
            ),
        ],
        5,
        &BLUE,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
        },
    ))?;

    // Control points
    chart.draw_series(PointSeries::of_element(
        vec![
            envelope
                .attack
                .control
                .map(|(x, y)| (x + note_on, y))
                .unwrap_or((0.0, 0.0)),
            envelope
                .decay
                .control
                .map(|(x, y)| (x + note_on, y))
                .unwrap_or((0.0, 0.0)),
            envelope
                .release
                .control
                .map(|(x, y)| (x + note_off, y))
                .unwrap_or((0.0, 0.0)),
        ],
        5,
        &GREEN,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
        },
    ))?;

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
