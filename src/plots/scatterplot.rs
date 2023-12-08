use crate::reader::GameReader;
use plotters::drawing::DrawingArea;
use plotters::{coord::Shift, prelude::*};
use std::error::Error;

//
pub fn scatterplot<T>(
    root: DrawingArea<T, Shift>,
    game_reader: &GameReader,
    resolution: (u32, u32),
) -> Result<(), Box<dyn Error + 'static>>
where
    T: IntoDrawingArea,
    <T as DrawingBackend>::ErrorType: 'static,
{
    let x_values: Vec<usize> = (0..game_reader.max_allowed_time as usize).collect();
    // data
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
    // create an iterator of points to create our scatterplot
    let average_points = x_values
        .iter()
        .zip(&averages) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, *y), 2, GREEN.filled()));

    let median_points = x_values
        .iter()
        .zip(&medians) // zip is so cool dude WHAT !!
        .map(|(x, y)| Circle::new((*x as f32, *y), 2, BLUE.filled()));

    let max_x = *x_values.iter().max().unwrap() as f32;
    let max_y = averages.clone().into_iter().reduce(f32::max).unwrap() + 1f32;

    // chart stuff
    root.fill(&WHITE)?;

    let caption_text = {
        let elo_text = {
            if game_reader.args.min_rating.is_none() && game_reader.args.max_rating.is_none() {
                "No ELO Limit".to_string()
            } else {
                let mut str = String::new();
                if let Some(rating) = game_reader.args.min_rating {
                    str.push_str(&rating.to_string());
                }
                str.push('-');
                if let Some(rating) = game_reader.args.max_rating {
                    str.push_str(&rating.to_string());
                };
                str.push_str(" ELO*");
                str
            }
        };
        format!(
            "{}; {} seconds; {} Games",
            elo_text, game_reader.args.time_control, game_reader.total_games
        )
    };
    // u32
    let area = resolution.0 * resolution.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(caption_text, ("sans-serif", 50).into_font())
        .margin(50)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 100)
        .build_cartesian_2d(max_x..0f32, 0f32..max_y)?;

    chart
        .configure_mesh()
        .y_desc("Average Time Taken to Move (S)")
        .x_desc("Time Left on Player Clock (S)")
        .axis_desc_style(("sans-serif", 50))
        .draw()?;

    chart
        .draw_series(average_points)?
        .label("Average time taken")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));
    chart
        .draw_series(median_points)?
        .label("Median time taken")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

    chart.configure_series_labels().draw()?;

    root.present()?;
    Ok(())
}
