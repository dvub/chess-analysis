use crate::reader::GameReader;
use plotters::drawing::DrawingArea;
use plotters::{coord::Shift, prelude::*};
use std::error::Error;

pub fn x_histogram<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    root.fill(&WHITE)?;
    let bucket_size = 50f32;

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
    // chart stuff!!
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(30)
        .caption("Freq. of time left", ("sans-serif", 50.0))
        .build_cartesian_2d(
            (0f32..max_x).step(bucket_size).use_round().into_segmented(),
            0f32..1.0,
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Number of Moves Made")
        .x_desc("Time Left")
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
pub fn y_histogram<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    root.fill(&WHITE)?;
    let bucket_size = 5f32;

    let all_moves = game_reader
        .time_data
        .iter()
        .flatten()
        .collect::<Vec<&i32>>();

    let total = all_moves.len();
    let max_x = **all_moves.iter().max().unwrap() as f32;

    let data = all_moves.iter().map(|v| (**v as f32, 1f32 / total as f32));

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(30)
        .caption("Freq. of time left", ("sans-serif", 50.0))
        .build_cartesian_2d(
            (0f32..max_x).step(bucket_size).use_round().into_segmented(),
            0f32..1.0,
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Number of Moves Made")
        .x_desc("Time Left")
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
