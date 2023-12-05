use crate::args::Args;
use pgn_reader::{SanPlus, Skip, Visitor};
use std::collections::HashMap;

pub struct GameReader {
    pub moves: usize,
    pub total_games: usize,
    pub all_times: Vec<i32>,
    pub time_map: HashMap<i32, Vec<i32>>,
    pub time_control_offset: i32,
    pub skip: bool,
    pub args: Args,
}

impl GameReader {
    pub fn new(args: Args) -> GameReader {
        GameReader {
            moves: 0,
            total_games: 0,
            all_times: Vec::new(),
            time_map: HashMap::new(),
            time_control_offset: 0,
            skip: false,
            args,
        }
    }
}

impl Visitor for GameReader {
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

    fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
        let max_games = 1000;
        let desired_tc = "600+0";
        let min_rating = 1000;
        let max_rating = 2000;

        let key = std::str::from_utf8(key).unwrap();
        let value = std::str::from_utf8(value.0).unwrap();

        // we're doing 2 things here:
        // 1. get the time control offset, which is how much time a player GETS for moving
        //      we need to know this to find the actual time the player took to make a move in this kind of tc mode
        // 2. validate that the tc mode is the one we want
        let mut time_control = "";
        if key == "TimeControl" {
            time_control = value;
            if value == desired_tc {
                let offset = value.split('+').nth(1).unwrap_or("0");
                self.time_control_offset = offset.parse().unwrap();
            }
        }
        // here we'll check rating of each player
        // we then make sure it's in the range by averaging
        let mut white_rating = 0;
        let mut black_rating = 0;
        if key == "WhiteElo" {
            white_rating = value.parse::<i32>().unwrap();
        }
        if key == "BlackElo" {
            black_rating = value.parse::<i32>().unwrap();
        }
        let average_rating = white_rating + black_rating / 2;
        // evaluate filters/constraints to know if we should read the time data from this game.
        if time_control != desired_tc
            || !(average_rating > min_rating && average_rating < max_rating)
            || self.total_games >= max_games
        {
            self.skip = true;
        }
    }
    // now that we've read the game headers
    // we have the necessary info to determine whether to skip reading the game
    // so we tell it to skip if that's true
    fn end_headers(&mut self) -> Skip {
        if self.skip {
            return Skip(true);
        }

        Skip(false)
    }

    // once a game is over, we want to use all the times we collected
    fn end_game(&mut self) -> Self::Result {
        // if the game had no moves or something like that, take care of it here
        if self.all_times.is_empty() {
            return self.total_games;
        }
        // initialize a prev value for calculating deltas
        let mut last_time = *self.all_times.first().unwrap();

        //
        for time in self.all_times.iter().step_by(2) {
            let remaining_time = *time;
            let delta_time = last_time - (time - self.time_control_offset);

            self.time_map
                .entry(remaining_time)
                .or_default()
                .push(delta_time);

            last_time = *time;
        }

        // cleanup, whatever
        self.all_times.clear();
        self.total_games
    }
    // in a game, we want to collect all of the times for each move,
    // so we use this function
    fn comment(&mut self, comment: pgn_reader::RawComment<'_>) {
        // convert the raw comment into a string slice
        let comment_str = std::str::from_utf8(comment.as_bytes()).unwrap();
        // get rid of brackets in comments, clean and split
        let cleaned = comment_str.replace(&['[', ']'][..], "");
        let comment_vec: Vec<&str> = cleaned.split(' ').collect();

        // basically if we find %clk, we know the next element is a time, so lets convert that and add it to our vec of times
        for (i, term) in comment_vec.iter().enumerate() {
            if *term == "%clk" {
                let time = convert_time(comment_vec.get(i + 1).unwrap());
                self.all_times.push(time);
            }
        }
    }
}

fn convert_time(time: &str) -> i32 {
    // convert time into a number of seconds
    // TODO: fix error handling
    let units: Vec<&str> = time.split(':').collect();
    let hours = str::parse::<i32>(units.first().unwrap()).unwrap();
    let minutes = str::parse::<i32>(units.get(1).unwrap()).unwrap();
    let seconds = str::parse::<i32>(units.get(2).unwrap()).unwrap();
    hours * 3600 + minutes * 60 + seconds
}
