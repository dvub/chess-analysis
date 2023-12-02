use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use pgn_reader::{SanPlus, Skip, BufferedReader};

struct Reader {
    moves: usize,
}
impl Reader {
    fn new() -> Reader {
        Reader { moves: 0 }
    }
}

impl pgn_reader::Visitor for Reader {
    type Result = usize;

    fn begin_game(&mut self) {
        self.moves = 0;
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
}

fn main() -> std::io::Result<()> {
    // current parent directory
    let current = std::env::current_dir()?;

    // get the correct game file within games directory
    let file_name = "games/some-games.pgn";
    let file = current.join(file_name);

    // open
    let games = File::open(file)?;
    // create a bufreader
    let reader = BufReader::new(games);

    let mut previous = String::new();
    for line in reader.lines() {
        let current = line?;

        // if the previous line was empty and the current line starts with "[",
        // a new game has started. we know this because of the formatting of the pgn file and also because of metadata tags, etc.

        if previous.is_empty() && current[0..1] == *"[" {
            let reader = Reader::new();
            let r = BufferedReader::new_cursor(inner)
            reader.re
        }

        previous = current;
    }

    Ok(())
}
