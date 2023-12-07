use crate::reader::GameReader;
use plotters::{coord::Shift, prelude::*};
use std::{
    error::Error,
    fs::create_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn gen_plots(game_reader: GameReader) -> Result<(), Box<dyn std::error::Error>> {
    let x_values: Vec<usize> = (0 as usize..game_reader.max_allowed_time as usize).collect();
    let averages = game_reader
        .time_data
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
        .time_data
        .iter()
        .map(|y_values| {
            let mut sorted_values = y_values.to_vec(); // Create a mutable copy
            sorted_values.sort();
            sorted_values[sorted_values.len() / 2] as f32
        })
        .collect::<Vec<f32>>();

    let resolution = (640, 480);
    let path = gen_path(&game_reader.args.output)?;
    // create both an svg AND png, because svg is not widely supported.
    scatterplot(
        SVGBackend::new(&path.join("2-var.svg"), resolution).into_drawing_area(),
        &x_values,
        &averages,
        &medians,
    )?;
    scatterplot(
        BitMapBackend::new(&path.join("2-var.png"), resolution).into_drawing_area(),
        &x_values,
        &averages,
        &medians,
    )?;

    histogram(
        SVGBackend::new(&path.join("x-histogram.svg"), resolution).into_drawing_area(),
        &game_reader.time_data,
    )?;

    Ok(())
}
fn gen_path(path: &str) -> std::io::Result<PathBuf> {
    // plot shit

    let unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // if a system clock is fucked up we will have problems.
        .as_secs();

    let path = PathBuf::from(path).join(unix_timestamp.to_string());
    create_dir(&path)?;
    Ok(path)
}
fn histogram<T>(
    root: DrawingArea<T, Shift>,
    data: &Vec<Vec<i32>>,
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Histogram Test", ("sans-serif", 50.0))
        .build_cartesian_2d((0u32..600).into_segmented(), 0..100)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Bucket")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;
    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.iter().enumerate().map(|(x, v)| {
                let rounded_bin = (x as u32 + 50) / 100 * 100;
                (rounded_bin, v.len() as i32)
            })),
    )?;
    //
    root.present()?;
    //
    Ok(())
}

fn scatterplot<T>(
    root: DrawingArea<T, Shift>,
    x_values: &Vec<usize>,
    averages: &Vec<f32>,
    medians: &Vec<f32>,
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
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
        .map(|(x, y)| Circle::new((*x as f32, *y), 2, GREEN.filled()));

    let median_points = x_values
        .iter()
        .zip(medians) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, *y), 2, BLUE.filled()));

    chart.draw_series(average_points)?;
    chart.draw_series(median_points)?;

    root.present()?;
    Ok(())
}
