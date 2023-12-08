use super::histogram::{x_histogram, y_histogram};
use super::scatterplot::scatterplot;
use crate::reader::GameReader;
use plotters::prelude::*;
use std::{
    fs::create_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn gen_plots(game_reader: GameReader) -> Result<(), Box<dyn std::error::Error>> {
    let resolution = {
        if let Some(r) = game_reader.args.resolution {
            (r as u32, r as u32)
        } else {
            (1000, 1000)
        }
    };
    let path = gen_path(&game_reader.args.output)?;

    scatterplot(
        BitMapBackend::new(&path.join("2-var.png"), resolution).into_drawing_area(),
        &game_reader,
        resolution,
    )?;
    x_histogram(
        BitMapBackend::new(&path.join("x-histogram.png"), resolution).into_drawing_area(),
        &game_reader,
        resolution,
    )?;
    y_histogram(
        BitMapBackend::new(&path.join("y-histogram.png"), resolution).into_drawing_area(),
        &game_reader,
    )?;

    if game_reader.args.svg {
        x_histogram(
            SVGBackend::new(&path.join("x-histogram.svg"), resolution).into_drawing_area(),
            &game_reader,
            resolution,
        )?;
        scatterplot(
            SVGBackend::new(&path.join("2-var.svg"), resolution).into_drawing_area(),
            &game_reader,
            resolution,
        )?;
    }

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
