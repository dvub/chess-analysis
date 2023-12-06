use std::time::{SystemTime, UNIX_EPOCH};

use plotters::prelude::*;

use crate::reader::GameReader;

// TODO: REFACTOR INTO SMALLER PIECES
pub fn var_plot(game_reader: GameReader) -> Result<(), Box<dyn std::error::Error>> {
    // plot shit

    let averages = game_reader
        .time_map
        .values()
        .map(|y_values| {
            /*y_values
            .iter()
            .map(|y| Circle::new((*x, *y), 2, GREEN.filled()))
            */
            // MEDIAN
            // let y = y_values.get(y_values.len() / 2).unwrap();
            // AVERAGE for each X
            y_values.iter().sum::<i32>() as f32 / y_values.len() as f32
        })
        .collect::<Vec<f32>>();

    let medians = game_reader
        .time_map
        .values()
        .map(|y_values| {
            // MEDIAN
            let mut sorted_values = y_values.to_vec(); // Create a mutable copy
            sorted_values.sort();
            *sorted_values.get(sorted_values.len() / 2).unwrap() as f32
        })
        .collect::<Vec<f32>>();

    // for graphing
    let x_values = game_reader.time_map.keys();
    let max_x = *x_values.max().unwrap() as f32;
    // TODO: fix clone
    let max_y = averages.clone().into_iter().reduce(f32::max).unwrap() + 1f32;

    let unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let file = format!("graphs/{}.svg", unix_timestamp);
    let root = SVGBackend::new(file.as_str(), (640, 480)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Time Analysis", ("sans-serif", 35).into_font())
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(max_x..0f32, 0f32..max_y)?;

    chart
        .configure_mesh()
        .y_desc("Average Time to Move for a Given X (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 10))
        .draw()?;

    // create an iterator of points to create our scatterplot
    let average_points = game_reader
        .time_map
        .keys()
        .zip(averages) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, y), 2, GREEN.filled()));

    let median_points = game_reader
        .time_map
        .keys()
        .zip(medians) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, y), 2, BLUE.filled()));

    chart.draw_series(average_points).unwrap();
    chart.draw_series(median_points).unwrap();

    root.present()?;
    Ok(())
}
