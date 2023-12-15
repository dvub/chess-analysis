use clap::Parser;
use pgn_reader::BufferedReader;
use std::{fs::File, io::BufReader};

mod analysis;
mod args;
mod plots;
mod reader;

use args::Args;
use reader::GameReader;

use plots::plotter::generate_plots;

use crate::analysis::{determination, quadratic_regression, to_precision};

/* each point could be
// - (time left, delta time)
//      = one point per move
// - (time left, average time for x)
//      = one point per x axis value <- like this one the best so far
// - (time left, averave time taken per move per game)
//      = one point per game
*/

// TODO:
// rework parameters to take 2 vectors instead of a gamereader
// document with MD
// actually finish the fucking assignment LMAO
// idk what else lol.

// cargo run --release -- games/oct-2023-games.pgn -m 1000 --min-rating 1000 --max-rating 2000 --time-control 600+0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: arg validation
    let args = Args::parse();
    // open the file parsed from clap
    let mut game_reader = GameReader::new(&args);
    data_collection(&mut game_reader);
    plots(&game_reader);
    analysis(&game_reader)?;
    Ok(())
}
fn plots(game_reader: &GameReader) {
    println!();
    println!(" --- Plots --- ");
    println!();
    println!("Now creating data plots... This shouldn't take long. ");
    generate_plots(game_reader)
        .unwrap_or_else(|e| println!("An error occurred generating plots:\n{}", e));

    println!("Successfully generated plots.");
}

fn data_collection(game_reader: &mut GameReader) {
    println!();
    println!(" --- Data Collection --- ");
    println!();
    let games = File::open(&game_reader.args.input).expect("Error reading PGN file. :( Exiting...");
    // open a bufreader
    let buf = BufReader::new(games);
    println!("Successfully found PGN file!");
    println!("NOTE: games are read sequentially read and not randomly sampled.");

    // now, we will actually read the file and the games
    let mut reader = BufferedReader::new(buf);
    println!("Reading all games. This will take a moment... Or a few, if you have a lot of games.");
    println!();
    reader
        .read_all(game_reader)
        .unwrap_or_else(|e| println!("An error occurred reading games:\n{}", e));

    // print some helpful information for the user
    println!("Successfully finished reading games!");
    println!(
        "A total of {} games were analyzed out of {}.",
        game_reader.games_analyzed, game_reader.total_games
    );
    println!(
        "A total of {} moves were analyzed.",
        game_reader.moves_analyzed
    );
}

fn analysis(game_reader: &GameReader) -> Result<(), Box<dyn std::error::Error>> {
    let (x_values, y_values): (Vec<f64>, Vec<f64>) = game_reader
        .time_data
        .iter()
        .enumerate()
        .flat_map(|(x, row)| row.iter().map(move |&y| (x as f64, y as f64)))
        .unzip();
    // print cool little title
    println!();
    println!(" --- Regression Analysis --- ");
    println!();

    let line = quadratic_regression(&x_values, &y_values)?;
    let det = determination(&x_values, &y_values)?;

    println!(
        "Quadratic Regression: {}x^2 {}x {}",
        to_precision(line.0, 4),
        to_precision(line.1, 4),
        to_precision(line.2, 4)
    );
    println!("Coefficient of Determination (R^2) = {}", det);
    println!(
        "In other words, ~{}% of variance in TTM is explained by time remaining.",
        (det * 100.0).round()
    );
    println!("Thus, the Correlation r = {}.", to_precision(det.sqrt(), 4));
    Ok(())
}
