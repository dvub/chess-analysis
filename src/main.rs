use std::{fs::File, io::BufReader};

use pgn_reader::{BufferedReader, SanPlus, Skip};
struct GameReader {
    moves: usize,
    total_games: usize,
}
impl GameReader {
    fn new() -> GameReader {
        GameReader {
            moves: 0,
            total_games: 0,
        }
    }
}

impl pgn_reader::Visitor for GameReader {
    type Result = usize;

    fn begin_game(&mut self) {
        self.moves = 0;
        self.total_games += 1;
    }

    fn san(&mut self, _san_plus: SanPlus) {
        self.moves += 1;
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) -> Self::Result {
        self.moves
    }
    fn comment(&mut self, comment: pgn_reader::RawComment<'_>) {
        // convert the raw comment into a string slice
        let comment_str = std::str::from_utf8(comment.as_bytes()).unwrap();

        // r"(?!%clk (\d:\d{2}:\d{2}))"
        let cleaned = comment_str.replace(r"(?!\d:\d{2}:\d{2})", "");
        // split by spaces
        let split_vec: Vec<&str> = cleaned.split(' ').collect();

        // iterate

        for (i, term) in split_vec.iter().enumerate() {
            println!("{}", term)
        }
    }
}

fn convert_time(time: &str) -> usize {
    // convert time into a number of seconds
    // TODO: fix error handling
    let units: Vec<&str> = time.split(':').collect();
    let hours = str::parse::<usize>(units.first().unwrap()).unwrap();
    let minutes = str::parse::<usize>(units.get(1).unwrap()).unwrap();
    let seconds = str::parse::<usize>(units.get(2).unwrap()).unwrap();
    hours * 3600 + minutes * 60 + seconds
}

fn main() -> std::io::Result<()> {
    // current parent directory
    let current = std::env::current_dir()?;

    // get the correct game file within games directory
    let file_name = "games/sample.pgn";
    let file = current.join(file_name);

    // open
    let games = File::open(file)?;
    let buf = BufReader::new(games);
    let mut reader = BufferedReader::new(buf);

    let mut game_reader = GameReader::new();

    reader.read_all(&mut game_reader)?;

    //
    println!(
        "A total of {} games were analyzed.",
        game_reader.total_games
    );

    Ok(())
}
