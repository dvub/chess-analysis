use super::plotter::{generate_caption, GraphType};
use crate::analysis::{generate_residuals, quadratic_regression};
use crate::reader::GameReader;
use plotters::drawing::DrawingArea;
use plotters::{coord::Shift, prelude::*};
use std::error::Error;
// TODO:
// - 3 seperate functions for all points, average, and quartile displays
// - make graphs look good
pub fn generate_two_var_plots(
    game_reader: &GameReader,
    path: &std::path::Path,
    resolution: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating average TTM graph...");
    if game_reader.args.averages {
        averages(
            BitMapBackend::new(&path.join("2-var").join("ttm_averages.png"), resolution)
                .into_drawing_area(),
            game_reader,
            resolution,
        )?;
    }
    if game_reader.args.all {
        println!("Creating all TTMs graph...");
        all_points(
            BitMapBackend::new(&path.join("2-var").join("all_ttm.png"), resolution)
                .into_drawing_area(),
            game_reader,
            resolution,
        )?;
    }
    if game_reader.args.residuals {
        residuals(
            BitMapBackend::new(&path.join("2-var").join("residuals.png"), resolution)
                .into_drawing_area(),
            game_reader,
            resolution,
        )?;
    }
    Ok(())
}

fn averages<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    // ----- DATA ----- //
    let averages = game_reader
        .time_data
        .iter()
        .map(|y_values| y_values.iter().sum::<i32>() as f32 / y_values.len() as f32)
        .collect::<Vec<f32>>();

    // create an iterator of points to create our scatterplot
    let average_line = LineSeries::new(
        averages.iter().enumerate().map(|(x, y)| (x as f32, *y)),
        RED.stroke_width(2),
    );

    let max_x = game_reader.max_allowed_time as f32;
    let max_y = averages.into_iter().reduce(f32::max).unwrap() + 1f32;

    // ----- chart stuff ----- //
    root.fill(&WHITE)?;
    // u32
    let _area = resolution.0 * resolution.1;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            generate_caption(GraphType::Average, game_reader),
            ("sans-serif", 25).into_font(),
        )
        .margin(35)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 100)
        .build_cartesian_2d(max_x..0f32, 0f32..max_y)?;

    chart
        .configure_mesh()
        .y_desc("TTM (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart
        .draw_series(average_line)?
        .label("Average time taken")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 50, y)], RED.stroke_width(2)));
    if game_reader.args.overlay_regression {
        let (x_values, y_values): (Vec<f64>, Vec<f64>) = game_reader
            .time_data
            .iter()
            .enumerate()
            .flat_map(|(x, row)| row.iter().map(move |&y| (x as f64, y as f64)))
            .unzip();

        let r = quadratic_regression(&x_values, &y_values)?;

        chart.draw_series(LineSeries::new(
            (0..max_x as usize).map(|x| {
                (
                    x as f32,
                    r.0 as f32 * x as f32 * x as f32 + r.1 as f32 * x as f32 + r.2 as f32,
                )
            }),
            GREEN.stroke_width(2),
        ))?;
    }
    chart.configure_series_labels().draw()?;
    root.present()?;
    Ok(())
}
fn residuals<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    // ----- DATA ----- //
    let (x_values, y_values): (Vec<f64>, Vec<f64>) = game_reader
        .time_data
        .iter()
        .enumerate()
        .flat_map(|(x, row)| row.iter().map(move |&y| (x as f64, y as f64)))
        .unzip();
    let residual_y = generate_residuals(&x_values, &y_values)?;

    let points = x_values
        .iter()
        .zip(residual_y)
        .map(|(x, y)| Circle::new((*x as f32, y as f32), 2, BLUE.mix(0.05).filled()));
    let max_x = game_reader.max_allowed_time as f32;
    let max_y = 100f32;

    // ----- chart stuff ----- //
    root.fill(&WHITE)?;
    let _area = resolution.0 * resolution.1;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            generate_caption(GraphType::Average, game_reader),
            ("sans-serif", 25).into_font(),
        )
        .margin(35)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 100)
        .build_cartesian_2d(max_x..0f32, 0f32..max_y)?;

    chart
        .configure_mesh()
        .y_desc("TTM (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;
    chart.draw_series(points)?;

    chart.configure_series_labels().draw()?;
    root.present()?;
    Ok(())
}

fn all_points<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    _resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    let all_points = game_reader.time_data.iter().enumerate().flat_map(|(i, v)| {
        v.iter()
            .map(move |&y| Circle::new((i as f32, y as f32), 2, BLUE.mix(0.05).filled()))
    });
    let max_x = game_reader.max_allowed_time as f32;
    // ----- CHART ----- //
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            generate_caption(GraphType::All, game_reader),
            ("sans-serif", 35).into_font(),
        )
        .margin(35)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 100)
        .build_cartesian_2d(max_x..0f32, 0f32..max_x)?;

    chart
        .configure_mesh()
        .y_desc("TTM (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 25))
        .draw()?;

    chart.draw_series(all_points)?;
    chart.configure_series_labels().draw()?;
    root.present()?;

    Ok(())
}
