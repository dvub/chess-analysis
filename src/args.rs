use clap::Parser;

/// Analyze and graph time-related information from one or more chess game(s). Written in pure Rust!
/// NOTE: If you have a lot of games, it's not recommended to run without any arguments, but then again,
/// You're the one running the program so do whatever you want.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to PGN file. The PGN file may contain one or more games.
    pub input: String,
    /// Output path. Creates a directory titled with the UNIX timestamp in this directory.
    pub output: String,

    /// The maximum number of games to collect data from.
    /// (NOTE: The reader will read games past the limit but will not record any data from them, due to the nature of pgn-reader)
    #[arg(short = 'M', long)]
    pub max_games: Option<usize>,

    /// The minimum average rating between both players in games to collect from
    #[arg(short = 'r', long)]
    pub min_rating: Option<i32>,
    #[arg(short = 'R', long)]
    /// The maxmimum average rating between both players in games to collect from
    pub max_rating: Option<i32>,
    /// The time mode to choose from. Formatted as seconds+seconds (NOT minutes+seconds as on lichess). Refer to lichess for options
    #[arg(short = 'c', long)]
    pub time_control: String,
}
