use crate::args::Args;
use pgn_reader::{Skip, Visitor};
use std::collections::HashMap;

pub struct GameReader {
    pub total_games: usize,
    pub time_map: HashMap<i32, Vec<i32>>,
    // private, for data measurement to be passed between pgn-reader functions
    all_times: Vec<i32>,
    time_control_offset: i32,
    max_allowed_time: i32,
    args: Args,
    time_control: String,
    average_rating: i32,
}

impl GameReader {
    pub fn new(args: Args) -> GameReader {
        GameReader {
            total_games: 0,
            all_times: Vec::new(),
            time_map: HashMap::new(),
            time_control_offset: 0,
            max_allowed_time: 0,
            args,
            time_control: "".to_string(),
            average_rating: 0,
        }
    }
}

impl Visitor for GameReader {
    type Result = usize;

    // honestly probably not necessary, but left it in here from the docs
    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }
    // first of all, we will read the headers to determine if we should even read this game.
    fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
        let key = std::str::from_utf8(key).unwrap();
        let value = std::str::from_utf8(value.0).unwrap();
        // if it doesn't have +, it's empty, so it's just "-"
        if key == "TimeControl" && value.contains('+') {
            // find the
            let time_control = value.split('+').collect::<Vec<&str>>();
            self.time_control_offset = time_control.get(1).unwrap().parse().unwrap();
            self.max_allowed_time = time_control.get(0).unwrap().parse().unwrap();
            self.time_control = value.to_string();
        }
        if key == "WhiteElo" || key == "BlackElo" {
            self.average_rating += str::parse::<i32>(value).unwrap();
        }
    }
    // now that we've read the game headers
    // we have the necessary info to determine whether to skip reading the game
    // so we tell it to skip if that's true
    fn end_headers(&mut self) -> Skip {
        // actually take the average lol
        let avg = self.average_rating / 2;
        // reset

        self.average_rating = 0;

        if let Some(time_control) = &self.args.time_control {
            if self.time_control != *time_control {
                return Skip(true);
            }
        }
        if let Some(rating) = self.args.min_rating {
            if avg < rating {
                return Skip(true);
            }
        }
        if let Some(rating) = self.args.max_rating {
            if avg > rating {
                return Skip(true);
            }
        }
        if let Some(max_games) = self.args.max_games {
            if self.total_games >= max_games {
                return Skip(true);
            }
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
        let mut prev_time = *self.all_times.first().unwrap();

        //
        for time in self.all_times.iter().skip(2).step_by(2) {
            if *time <= self.max_allowed_time {
                let remaining_time = *time;
                let delta_time = prev_time - (time - self.time_control_offset);
                self.time_map
                    .entry(remaining_time)
                    .or_default()
                    .push(delta_time);
            }

            prev_time = *time;
        }
        for time in self.all_times.iter().skip(1).step_by(2) {
            if *time <= self.max_allowed_time {
                let remaining_time = *time;
                let delta_time = prev_time - (time - self.time_control_offset);
                self.time_map
                    .entry(remaining_time)
                    .or_default()
                    .push(delta_time);
            }

            prev_time = *time;
        }

        // cleanup, whatever
        self.all_times.clear();
        self.total_games += 1;
        println!("{}", self.total_games);

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
