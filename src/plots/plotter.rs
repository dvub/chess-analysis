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

pub fn gen_plots(game_reader: &GameReader) -> Result<(), Box<dyn std::error::Error>> {
    let resolution = {
        if let Some(r) = game_reader.args.resolution {
            (r as u32, r as u32)
        } else {
            (1000, 1000)
        }
    };
    let path = gen_path(&game_reader.args.output)?;

    // print all of our 2-variable stuff
    two_var(game_reader, &path, resolution)?;

    x_histogram(
        BitMapBackend::new(&path.join("x-histogram.png"), resolution).into_drawing_area(),
        game_reader,
        resolution,
    )?;
    y_histogram(
        BitMapBackend::new(&path.join("y-histogram.png"), resolution).into_drawing_area(),
        game_reader,
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

pub enum GraphType {
    Average,
    Quartiles,
    All,
    RelativeFrequencyX,
    RelativeFrequencyY,
}

pub fn generate_caption(graph_type: GraphType, game_reader: &GameReader) -> String {
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
    let title = match graph_type {
        GraphType::All => "All",
        GraphType::Average => "Average TTM",
        GraphType::Quartiles => "TTM Quartiles (1,2,3)",
        GraphType::RelativeFrequencyX => "RF of Time Left",
        GraphType::RelativeFrequencyY => "RF of TTM",
    };
    format!(
        "{} ({}, {} seconds, {} Games)",
        title, elo_text, game_reader.args.time_control, game_reader.games_analyzed
    )
}
