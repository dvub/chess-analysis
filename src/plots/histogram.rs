use crate::reader::GameReader;
use plotters::drawing::DrawingArea;
use plotters::{coord::Shift, prelude::*};
use std::collections::HashMap;
use std::error::Error;

pub fn x_histogram<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    root.fill(&WHITE)?;
    let bucket_size = 50;

    let mut sum_map: HashMap<i32, i32> = HashMap::new();
    for (index, vector) in game_reader.time_data.iter().enumerate() {
        let rounded_bin = round_to_bucket(index as i32, bucket_size);
        let entry = sum_map.entry(rounded_bin).or_insert(0);
        *entry += vector.len() as i32;
    }
    let max_tuple = sum_map.iter().max_by(|x, y| (x.0).cmp(y.0)).unwrap();

    let max_x = *max_tuple.0 + (bucket_size * 2);
    // add a little extra
    let max_y = *max_tuple.1 + (*max_tuple.1 as f32 * 0.1) as i32;
    let data = sum_map.into_iter();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(30)
        .caption("Freq. of time left", ("sans-serif", 50.0))
        .build_cartesian_2d((0..max_x).step(bucket_size).into_segmented(), 0..max_y)?;

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
    Ok(())
}

fn round_to_bucket(value: i32, bucket: i32) -> i32 {
    (value + bucket / 2) / bucket * bucket
}
