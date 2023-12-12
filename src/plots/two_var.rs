use crate::reader::GameReader;
use plotters::drawing::DrawingArea;
use plotters::{coord::Shift, prelude::*};
use std::error::Error;
use super::plotter::{generate_caption, GraphType};
// TODO:
// - 3 seperate functions for all points, average, and quartile displays
// - make graphs look good

pub fn averages<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    let color = RED;

    // ----- DATA ----- //
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

    // create an iterator of points to create our scatterplot
    let average_line = LineSeries::new(
        averages.iter().enumerate().map(|(x, y)| (x as f32, *y)),
        color,
    );

    let max_x = game_reader.max_allowed_time as f32;
    //let max_y = game_reader.max_allowed_time as f32 / 2.0;
    let max_y = averages.into_iter().reduce(f32::max).unwrap() + 1f32;

    // ----- chart stuff ----- //
    root.fill(&WHITE)?;
    // u32
    let area = resolution.0 * resolution.1;
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
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 50, y)], color));
    /*
    chart.draw_series(game_reader.time_data.iter().enumerate().flat_map(|(i, v)| {
        v.iter()
            .map(move |&y| Circle::new((i as f32, y as f32), 1, BLUE.mix(0.01)))
    }))?;
    */
    chart.configure_series_labels().draw()?;
    root.present()?;
    Ok(())
}

pub fn all_points<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
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

pub fn quartiles<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    // data shit
    let medians = game_reader
        .time_data
        .iter()
        .map(|y_values| {
            let mut sorted_values = y_values.to_vec(); // Create a mutable copy
            sorted_values.sort();
            sorted_values[sorted_values.len() / 2] as f32
        })
        .collect::<Vec<f32>>();

    let first_quartile = game_reader
        .time_data
        .iter()
        .map(|y_values| {
            let mut sorted_values = y_values.to_vec(); // Create a mutable copy
            sorted_values.sort();
            let idx = sorted_values.len() as f32 * 0.25;

            sorted_values[idx as usize] as f32
        })
        .collect::<Vec<f32>>();
    let third_quartile = game_reader
        .time_data
        .iter()
        .map(|y_values| {
            let mut sorted_values = y_values.to_vec(); // Create a mutable copy
            sorted_values.sort();
            let idx = sorted_values.len() as f32 * 0.75;

            sorted_values[idx as usize] as f32
        })
        .collect::<Vec<f32>>();
    let first_quartile_line = LineSeries::new(
        first_quartile
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f32, *y)),
        BLUE.mix(0.5).stroke_width(2),
    );
    let third_quartile_line = LineSeries::new(
        third_quartile
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f32, *y)),
        BLUE.mix(0.5).stroke_width(2),
    );

    let median_line = LineSeries::new(
        medians.iter().enumerate().map(|(x, y)| (x as f32, *y)),
        BLUE.mix(0.75).stroke_width(3),
    );

    let max_x = game_reader.max_allowed_time as f32;
    let max_y = medians.into_iter().reduce(f32::max).unwrap() + 2.0;

    // ----- CHART ----- //
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            generate_caption(GraphType::Quartiles, game_reader),
            ("sans-serif", 35).into_font(),
        )
        .margin(35)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 100)
        .build_cartesian_2d(max_x..0f32, 0f32..max_y)?;
    chart
        .configure_mesh()
        .y_desc("TTM (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 25))
        .draw()?;

    chart
        .draw_series(first_quartile_line)?
        .label("First Quartile TTM ")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(third_quartile_line)?
        .label("Third Quartile TTM ")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(median_line)?
        .label("Median TTM")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart.configure_series_labels().draw()?;
    root.present()?;

    Ok(())
}

