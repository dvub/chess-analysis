use clap::Parser;

/// Analyze and graph time-related information from one or more chess game(s). Written in pure Rust!
/// NOTE: If you have a lot of games, it's not recommended to run without any arguments, but then again,
/// You're the one running the program so do whatever you want.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to PGN file. The PGN file may contain one or more games.
    pub path: String,

    #[arg(short, long)]
    pub max_games: Option<usize>,

    #[arg(long)]
    pub min_rating: Option<i32>,
    #[arg(long)]
    pub max_rating: Option<i32>,
    #[arg(long)]
    pub time_control: Option<String>,
}
