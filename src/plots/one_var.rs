use crate::plots::plotter::GraphType;
use crate::reader::GameReader;
use plotters::drawing::DrawingArea;
use plotters::{coord::Shift, prelude::*};

use std::error::Error;

use super::plotter::generate_caption;

pub fn generate_one_var_plots(
    game_reader: &GameReader,
    path: &std::path::Path,
    resolution: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    x_histogram(
        BitMapBackend::new(&path.join("1-var").join("x-histogram.png"), resolution)
            .into_drawing_area(),
        game_reader,
        resolution,
    )?;
    y_histogram(
        BitMapBackend::new(&path.join("1-var").join("y-histogram.png"), resolution)
            .into_drawing_area(),
        game_reader,
    )?;
    Ok(())
}

fn x_histogram<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    _resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    // ----- DATA -----
    let num_buckets = 8;
    let bucket_size = game_reader.max_allowed_time as f32 / num_buckets as f32;
    let max_x = game_reader.max_allowed_time as f32 + bucket_size;
    let sum = game_reader
        .time_data
        .iter()
        .flatten()
        .collect::<Vec<&i32>>()
        .len();
    let data = game_reader.time_data.iter().enumerate().map(|(i, x)| {
        let num = x.len() as f32 / sum as f32;
        (i as f32, num)
    });
    // ----- chart stuff!! -----
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(30)
        .caption(
            generate_caption(GraphType::RelativeFrequencyX, game_reader),
            ("sans-serif", 20.0),
        )
        .build_cartesian_2d((0f32..max_x).step(bucket_size).use_round(), 0f32..1.0)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Relative Frequency of Moves Made (percent)")
        .x_desc("Time left (S)")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;
    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .margin(1)
            .data(data),
    )?;
    //
    root.present()?;
    //
    Ok(())
}
fn y_histogram<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    // ----- DATA -----

    let all_moves = game_reader
        .time_data
        .iter()
        .flatten()
        .collect::<Vec<&i32>>();

    let total = all_moves.len();
    let num_buckets = 100;
    let bucket_size = game_reader.max_allowed_time as f32 / num_buckets as f32;
    let max_x = 100.0;

    let data = all_moves.iter().map(|v| (**v as f32, 1f32 / total as f32));

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(30)
        .caption(
            generate_caption(GraphType::RelativeFrequencyY, game_reader),
            ("sans-serif", 20.0),
        )
        .build_cartesian_2d((0f32..max_x).step(bucket_size).use_round(), 0f32..1.0)?;
    // ----- CHART STUFF ----- //
    root.fill(&WHITE)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Relative Frequency of Moves Made (percent)")
        .x_desc("Time Taken to Move (S)")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;
    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .margin(1)
            .data(data),
    )?;
    //
    root.present()?;
    Ok(())
}
