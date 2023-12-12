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

    gen_plots(&game_reader)
        .unwrap_or_else(|e| println!("An error occurred generating plots:\n{}", e));

    let (x_values, y_values): (Vec<f64>, Vec<f64>) = game_reader
        .time_data
        .iter()
        .enumerate()
        .flat_map(|(x, row)| row.iter().map(move |&y| (x as f64, y as f64)))
        .unzip();
    println!("{:?}", quadratic_regression(&x_values, &y_values).unwrap());

    println!("Successfully generated a plot.");
    Ok(())
}

fn quadratic_regression(x_values: &Vec<f64>, y_values: &Vec<f64>) -> Option<(f64, f64, f64)> {
    let n = x_values.len() as f64;

    if n != y_values.len() as f64 || n < 3.0 {
        return None;
    }

    let sum_x = x_values.iter().sum::<f64>();
    let sum_x_squared = x_values.iter().map(|&x| x * x).sum::<f64>();
    let sum_x_cubed = x_values.iter().map(|&x| x * x * x).sum::<f64>();
    let sum_x_fourth = x_values.iter().map(|&x| x * x * x * x).sum::<f64>();
    let sum_y = y_values.iter().sum::<f64>();
    let sum_xy = x_values
        .iter()
        .zip(y_values.iter())
        .map(|(&x, &y)| x * y)
        .sum::<f64>();
    let sum_x_squared_y = x_values
        .iter()
        .zip(y_values.iter())
        .map(|(&x, &y)| x * x * y)
        .sum::<f64>();
    let sum_x_cubed_y = x_values
        .iter()
        .zip(y_values.iter())
        .map(|(&x, &y)| x * x * x * y)
        .sum::<f64>();

    let denom = n * sum_x_squared * sum_x_fourth
        - sum_x_cubed * sum_x_cubed
        - sum_x_squared * sum_x_squared;

    if denom.abs() < 1e-10 {
        return None; // Avoid division by nearly zero
    }

    let a = (n * sum_x_cubed_y * sum_x_squared
        - sum_y * sum_x_squared * sum_x_cubed
        - sum_x * sum_x_cubed_y
        + sum_x_cubed * sum_y)
        / denom;

    let b = (n * sum_x_cubed * sum_x_squared_y
        - sum_x_cubed * sum_x_squared * sum_xy
        - sum_x_squared * sum_x_squared_y
        + sum_x_squared * sum_x_squared * sum_xy)
        / denom;

    let c = (n * sum_x_cubed * sum_x_squared * sum_xy
        - sum_x_cubed * sum_x_fourth * sum_y
        - sum_x_squared * sum_x_squared_y
        + sum_x_squared * sum_x_fourth * sum_xy)
        / denom;

    Some((a, b, c))
}
#[cfg(test)]
mod tests {
    #[test]
    fn quadratic_regression() {
        let x_values = vec![
            -5f64, -4f64, -3f64, -2f64, -1f64, 0f64, 1f64, 2f64, 3f64, 4f64,
        ];
        let y_values = vec![
            12.55, 15.61, 10.20, 11.77, 10.24, 9.84, 8.07, 11.63, 12.82, 15.85,
        ];
        assert_eq!(
            super::quadratic_regression(&x_values, &y_values),
            Some((0.2484, 0.2837, 9.8881))
        );
    }
}
