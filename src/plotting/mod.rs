use plotters::prelude::*;

pub fn plot_data(
    data: Vec<(f32, f32)>,
    title: &str,
    x_scale: (f32, f32),
    y_scale: (f32, f32),
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = "output/".to_owned() + filename;
    let root = BitMapBackend::new(&out_path, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_scale.0..x_scale.1, y_scale.0..y_scale.1)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(data, &BLACK))?
        .label("stuff")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
