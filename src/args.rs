use clap::Parser;

/// Analyze and graph time-related information from one or more chess game(s). Written in pure Rust!
/// NOTE: If you have a lot of games, it's not recommended to run without any arguments, but then again,
/// You're the one running the program so do whatever you want.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to PGN file. The PGN file may contain one or more games.
    pub input: String,
    /// Output path. Creates a directory titled with the UNIX timestamp in this directory.
    #[arg(short = 'o', long)]
    pub output: String,

    /// The time mode to choose from. Formatted as seconds+seconds (NOT minutes+seconds as on lichess). Refer to lichess for options
    #[arg(short = 'c', long)]
    pub time_control: String,

    /// The maximum number of games to collect data from.
    /// (NOTE: The reader will read games past the limit but will not record any data from them, due to the nature of pgn-reader)
    #[arg(long)]
    pub max_games: Option<usize>,

    /// The minimum average rating between both players in games to collect from
    #[arg(short = 'm', long)]
    pub min_rating: Option<i32>,
    #[arg(short = 'M', long)]
    /// The maxmimum average rating between both players in games to collect from
    pub max_rating: Option<i32>,

    #[arg(short, long)]
    /// Set the resolution of the output images (1:1 ratio). Default is 1000 pixels.
    pub resolution: Option<i32>,

    /// Enable this option to create a 2-variable graph showing the median, first, and third quartile of TTMs.
    #[arg(short, long)]
    pub quartiles: bool,
    /// Enable this option to create a scatterplot showing the time data for all moves in the dataset.
    #[arg(short = 'A', long)]
    pub all: bool,
    /// Enable this option to create a line graph showing average TTMs
    #[arg(short = 'a', long)]
    pub averages: bool,

    /// Enable this option to create a residual graph
    #[arg(long)]
    pub residuals: bool,

    #[arg(long)]
    pub one_var: bool,
    #[arg(long)]
    pub overlay_regression: bool,

    #[arg(short, long)]
    /// Enable this option to output SVG files in addition to the default PNG output. (the better kind)
    pub svg: bool,
}
