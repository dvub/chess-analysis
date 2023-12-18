use clap::Parser;
use pgn_reader::BufferedReader;
use std::{fs::File, io::BufReader, time::Instant};

mod analysis;
mod args;
mod plots;
mod reader;

use args::Args;
use reader::GameReader;

use plots::plotter::generate_plots;

use crate::analysis::{
    determination, generate_residuals, quadratic_regression, standard_deviation, to_precision,
};

// TODO:
// rework parameters to take 2 vectors instead of a gamereader
// document with MD
// actually finish the fucking assignment LMAO
// idk what else lol.

/*
read 4096 pgns out of the file
relatively light, cant be more than 50mb
and then feed that to 4 threads
and keep a buffer of 4 1024 pgns to pass out from the main thread
 */

// cargo run --release -- games/oct-2023-games.pgn -m 1000 --min-rating 1000 --max-rating 2000 --time-control 600+0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: arg validation
    let args = Args::parse();
    // open the file parsed from clap
    let mut game_reader = GameReader::new(&args);
    data_collection(&mut game_reader);
    plots(&game_reader);
    one_var_analysis(&game_reader);
    analysis(&game_reader)?;

    let mut c = 0;
    game_reader.time_data[150].iter().for_each(|t| {
        if *t == 10 {
            c += 1;
        }
    });
    let conditional = c as f32 / game_reader.time_data[150].len() as f32;
    println!(
        "Probability that TTM < 10s, given 150s left: {}",
        conditional
    );

    let expected = game_reader
        .time_data
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let p = 1.0 / game_reader.max_allowed_time as f32;
            i as f32 * p
        })
        .sum::<f32>();
    println!("Expected time remaining: {:.2}", expected);
    let (x_values, _y_values): (Vec<i32>, Vec<i32>) = game_reader
        .time_data
        .iter()
        .enumerate()
        .flat_map(|(x, row)| row.iter().map(move |&y| (x as i32, y)))
        .unzip();

    let average = x_values.iter().sum::<i32>() as f32 / x_values.len() as f32;
    println!("{}", x_values.len());
    println!("Average time remaining: {:.2}", average);

    let stdev = standard_deviation(&x_values.iter().map(|f| *f as f64).collect::<Vec<_>>());
    println!("Standard Deviation: {}", stdev);
    println!("Variance: {}", stdev.powi(2));
    Ok(())
}
fn one_var_analysis(game_reader: &GameReader) {
    if game_reader.args.x_percentile.is_none() && game_reader.args.y_percentile.is_none() {
        return;
    }

    println!();
    println!(" --- One variable analysis --- ");
    println!();

    let (mut x_values, mut y_values): (Vec<i32>, Vec<i32>) = game_reader
        .time_data
        .iter()
        .enumerate()
        .flat_map(|(x, row)| row.iter().map(move |&y| (x as i32, y)))
        .unzip();
    x_values.sort();
    x_values.reverse();
    y_values.sort();

    if let Some(percentile) = game_reader.args.x_percentile {
        let idx = (x_values.len() as f64 * (percentile as f64 / 100.0)) as usize;
        println!(
            "The {}th percentile of time left is {} seconds remaining",
            percentile, x_values[idx]
        );
    }
    if let Some(percentile) = game_reader.args.y_percentile {
        let idx = (y_values.len() as f64 * (percentile as f64 / 100.0)) as usize;
        println!(
            "The {}th percentile of time taken is {} seconds to move",
            percentile, y_values[idx]
        );
    }
}

// whatever (bladee)

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
    let start = Instant::now();
    reader
        .read_all(game_reader)
        .unwrap_or_else(|e| println!("An error occurred reading games:\n{}", e));
    let duration = start.elapsed();

    // print some helpful information for the user
    println!(
        "Successfully finished reading games! {}.{}s elapsed.",
        duration.as_secs(),
        duration.as_millis(),
    );
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
    let residuals = generate_residuals(&x_values, &y_values)?;
    let stdev = standard_deviation(&residuals);

    println!(
        "Quadratic Regression: {}x^2 {}{}x {}{}",
        to_precision(line.0, 4),
        if line.1.is_sign_positive() { "+" } else { "" },
        to_precision(line.1, 4),
        if line.2.is_sign_positive() { "+" } else { "" },
        to_precision(line.2, 4)
    );

    println!("Coefficient of Determination (R^2) = {}", det);
    println!("Correlation (r) = {}", to_precision(det.sqrt(), 4));
    println!("Residuals Standard Deviation: {stdev}");

    println!();
    Ok(())
}
