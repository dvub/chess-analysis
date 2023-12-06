use crate::reader::GameReader;
use plotters::{coord::Shift, prelude::*};
use std::{
    error::Error,
    fs::create_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn var_plot(game_reader: GameReader) -> Result<(), Box<dyn std::error::Error>> {
    // plot shit

    let unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let path = PathBuf::from(&game_reader.args.output).join(unix_timestamp.to_string());
    create_dir(&path).unwrap();

    let resolution = (640, 480);

    // create both an svg AND png, because svg is not widely supported.
    scatterplot(
        SVGBackend::new(&path.join("2-var.svg"), resolution).into_drawing_area(),
        &game_reader,
    )?;
    scatterplot(
        BitMapBackend::new(&path.join("2-var.png"), resolution).into_drawing_area(),
        &game_reader,
    )?;

    Ok(())
}

fn scatterplot<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    let x_values: Vec<usize> = game_reader
        .time_map
        .iter()
        .enumerate()
        .map(|(i, _)| i)
        .collect();

    let averages = game_reader
        .time_map
        .iter()
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
        .iter()
        .map(|y_values| {
            // MEDIAN
            let mut sorted_values = y_values.to_vec(); // Create a mutable copy
            sorted_values.sort();
            *sorted_values.get(sorted_values.len() / 2).unwrap() as f32
        })
        .collect::<Vec<f32>>();

    // for graphing

    let max_x = *x_values.iter().max().unwrap() as f32;
    // TODO: fix clone
    let max_y = averages.clone().into_iter().reduce(f32::max).unwrap() + 1f32;
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
    let average_points = x_values
        .iter()
        .zip(averages) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, y), 2, GREEN.filled()));

    let median_points = x_values
        .iter()
        .zip(medians) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, y), 2, BLUE.filled()));

    chart.draw_series(average_points).unwrap();
    chart.draw_series(median_points).unwrap();

    root.present()?;
    Ok(())
}
