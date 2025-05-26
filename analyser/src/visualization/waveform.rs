use log::info;
use std::path::Path;

use plotters::prelude::*;

pub fn plot_freq(data: &Vec<(u32, f32)>, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let width = 640;
    let height = 480;

    let mut buffer = vec![0u8; (width * height * 3) as usize]; // RGB format
    let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(0)
        .caption("Frequencies", ("sans-serif", 24.0))
        .build_cartesian_2d((0u32..22050u32).into_segmented(), 0u32..1000u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Magnitude (10e-6)")
        .x_desc("Frequency")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.iter().map(|(index, x)| (*index, (*x * 1e6f32) as u32))),
    )?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    info!(
        "Result has been saved to {}",
        output.to_str().unwrap_or("Error")
    );

    Ok(())
}
