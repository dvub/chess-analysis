use clap::Parser;
use pgn_reader::BufferedReader;
use std::{fs::File, io::BufReader};

mod args;
mod plots;
mod reader;

use args::Args;
use reader::GameReader;

use plots::plotter::gen_plots;

/* each point could be
// - (time left, delta time)
//      = one point per move
// - (time left, average time for x)
//      = one point per x axis value <- like this one the best so far
// - (time left, averave time taken per move per game)
//      = one point per game
*/

// TODO:
// - [x] break into smaller files
// - [ ] implement clap
// refactor & Optimize
// export into BOTH pgn and svg
// document with MD
// split code
// actually finish the fucking assignment LMAO
// idk what else lol.

// cargo run --release -- games/oct-2023-games.pgn -m 1000 --min-rating 1000 --max-rating 2000 --time-control 600+0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: arg validation
    let args = Args::parse();
    // open the file parsed from clap
    let games = File::open(&args.input).expect("Error reading PGN file. :( Exiting...");
    // open a bufreader
    let buf = BufReader::new(games);
    println!("Successfully found PGN file!");

    // now, we will actually read the file and the games
    let mut reader = BufferedReader::new(buf);
    let mut game_reader = GameReader::new(args);
    println!("Reading all games. This will take a moment... Or a few, if you have a lot of games.");
    reader
        .read_all(&mut game_reader)
        .unwrap_or_else(|e| println!("An error occurred reading games:\n{}", e));
    
    // print some helpful information for the user
    println!("Successfully finished reading games!");
    println!(
        "A total of {} games were analyzed out of {}.",
        game_reader.games_analyzed, game_reader.total_games
    );
    
    println!();
    println!("Now creating plot of data... This shouldn't take long. ");
    // println!("{:?}", game_reader.time_data);

    gen_plots(game_reader)
        .unwrap_or_else(|e| println!("An error occurred generating plots:\n{}", e));

    let x_values = game_reader.time_data.iter().enumerate().map(|(i, v)| {
        v.iter().map(|_| i)
    }).collect::<Vec<i32>>();
    println!("Successfully generated a plot.");
    Ok(())
}
