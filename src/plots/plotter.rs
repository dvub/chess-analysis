use super::one_var::{x_histogram, y_histogram};
use super::two_var::{all_points, averages, quartiles};
use crate::reader::GameReader;
use plotters::prelude::*;
use std::{
    fs::create_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

// TODO: add helpful error messages

pub fn gen_plots(game_reader: GameReader) -> Result<(), Box<dyn std::error::Error>> {
    let resolution = {
        if let Some(r) = game_reader.args.resolution {
            (r as u32, r as u32)
        } else {
            (1000, 1000)
        }
    };
    let path = gen_path(&game_reader.args.output)?;

    // print all of our 2-variable stuff
    two_var(&game_reader, &path, resolution)?;

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
        averages(
            SVGBackend::new(&path.join("2-var.svg"), resolution).into_drawing_area(),
            &game_reader,
            resolution,
        )?;
    }

    Ok(())
}

fn two_var(
    game_reader: &GameReader,
    path: &std::path::Path,
    resolution: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Printing average TTM graph...");
    if game_reader.args.averages {
        averages(
            BitMapBackend::new(&path.join("2-var").join("ttm_averages.png"), resolution)
                .into_drawing_area(),
            game_reader,
            resolution,
        )?;
    }
    if game_reader.args.quartiles {
        println!("Printing TTM quartiles graph...");
        quartiles(
            BitMapBackend::new(&path.join("2-var").join("ttm_quartiles.png"), resolution)
                .into_drawing_area(),
            game_reader,
            resolution,
        )?;
    }

    if game_reader.args.all {
        println!("Printing all TTMs graph...");
        all_points(
            BitMapBackend::new(&path.join("2-var").join("all_ttm.png"), resolution)
                .into_drawing_area(),
            game_reader,
            resolution,
        )?;
    }
    Ok(())
}

/// Create the necessary directories to output graphs
fn gen_path(path: &str) -> std::io::Result<PathBuf> {
    // plot shit

    let unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // if a system clock is fucked up we will have problems.
        .as_secs();

    let path = PathBuf::from(path).join(unix_timestamp.to_string());
    create_dir(&path)?;
    create_dir(path.join("2-var"))?;

    Ok(path)
}
