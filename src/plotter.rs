use std::time::{SystemTime, UNIX_EPOCH};

use plotters::prelude::*;

use crate::reader::GameReader;

pub fn var_plot(game_reader: GameReader) -> Result<(), Box<dyn std::error::Error>> {
    // plot shit

    // TODO: probably dont get keys twice.
    let max_x = {
        if let Some(time_control) = game_reader.args.time_control {
            if time_control.contains('+') {
                time_control
                    .split('+')
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .parse::<f32>()
                    .unwrap()
            }
        }
        0
    };
    // TODO: DON't flatten twice you stupid fucking idiot
    // let min_y = *game_reader.time_map.values().flatten().min().unwrap();
    // let max_y = *game_reader.time_map.values().flatten().max().unwrap();

    let max_y = game_reader
        .time_map
        .iter()
        .map(|(_, y_values)| y_values.iter().sum::<i32>() / y_values.len() as i32)
        .max()
        .unwrap();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file = format!("graphs/{}.svg", time);
    let root = SVGBackend::new(file.as_str(), (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Time Analysis", ("sans-serif", 35).into_font())
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(max_x..0f32, 0f32..5f32)?;

    chart
        .configure_mesh()
        .y_desc("Average Time to Move for a Given X (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 10))
        .draw()?;

    let points_iter = game_reader.time_map.iter().map(|(x, y_values)| {
        /*y_values
        .iter()
        .map(|y| Circle::new((*x, *y), 2, GREEN.filled()))
        */
        // MEDIAN
        // let y = y_values.get(y_values.len() / 2).unwrap();
        // AVERAGE for each X
        let y = y_values.iter().sum::<i32>() as f32 / y_values.len() as f32;
        Circle::new((*x as f32, y), 2, GREEN.filled())
    });

    chart.draw_series(points_iter).unwrap();

    root.present()?;

    Ok(())
}
