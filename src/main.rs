use std::{
    fs::File,
    io::BufReader,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use pgn_reader::BufferedReader;
use plotters::prelude::*;
mod args;
mod reader;
use args::Args;
use reader::GameReader;

/* each point could be
// - (time left, delta time)
//      = one point per move
// - (time left, average time for x)
//      = one point per x axis value <- like this one the best so far
// - (time left, averave time taken per move per game)
//      = one point per game
*/

// TODO:
// break into smaller files
// implement clap
// refactor & Optimize
// export into BOTH pgn and svg
// idk what else lol.

// cargo run --release -- games/oct-2023-games.pgn -m 1000 --min-rating 1000 --max-rating 2000 --time-control 600+0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // open the file parsed from clap
    let games = File::open(&args.path).expect("Error reading PGN file. :( Exiting...");
    // open a bufreader
    let buf = BufReader::new(games);
    println!("Successfully found PGN file!");

    // now, we will actually read the file and the games
    let mut reader = BufferedReader::new(buf);
    let mut game_reader = GameReader::new(args);
    println!("Reading all games. This will take a moment... Or a few, if you have a lot of games.");
    reader.read_all(&mut game_reader)?;

    // print some helpful information for the user
    println!("Successfully finished reading games!");
    println!(
        "A total of {} games were analyzed.",
        game_reader.total_games
    );

    println!();
    println!("Now creating plot of data...");
    // plot shit

    // TODO: probably dont get keys twice.
    let min_x = *game_reader.time_map.keys().min().unwrap();
    let max_x = *game_reader.time_map.keys().max().unwrap();

    // TODO: DON't flatten twice you stupid fucking idiot
    // let min_y = *game_reader.time_map.values().flatten().min().unwrap();
    // let max_y = *game_reader.time_map.values().flatten().max().unwrap();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file = format!("graphs/{}.svg", time);
    let root = SVGBackend::new(file.as_str(), (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Time Analysis", ("sans-serif", 35).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(max_x..min_x, 0..100)?;

    chart.configure_mesh().draw()?;

    let points_iter = game_reader.time_map.iter().map(|(x, y_values)| {
        /*y_values
        .iter()
        .map(|y| Circle::new((*x, *y), 2, GREEN.filled()))
        */
        let y: i32 = y_values.iter().sum::<i32>() / y_values.len() as i32;
        Circle::new((*x, y), 2, GREEN.filled())
    });

    chart.draw_series(points_iter).unwrap();

    root.present()?;
    println!("Successfully generated a plot.");
    Ok(())
}
