use clap::Parser;

/// Analyze and graph time-related information from one or more chess game(s). Written in pure Rust!
/// NOTE: If you have a lot of games, it's not recommended to run without any arguments, but then again,
/// You're the one running the program so do whatever you want.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to PGN file. The PGN file may contain one or more games.
    pub path: String,

    /// The maximum number of games to collect data from.
    /// (NOTE: The reader will read games past the limit but will not record any data from them, due to the nature of pgn-reader)
    #[arg(short, long)]
    pub max_games: Option<usize>,

    /// The minimum average rating between both players in games to collect from
    #[arg(long)]
    pub min_rating: Option<i32>,
    #[arg(long)]
    /// The maxmimum average rating between both players in games to collect from
    pub max_rating: Option<i32>,
    /// The time mode to choose from. Formatted as seconds+seconds
    #[arg(long)]
    pub time_control: Option<String>,
}
