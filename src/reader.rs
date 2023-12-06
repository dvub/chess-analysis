use crate::args::Args;
use pgn_reader::{Skip, Visitor};
use std::collections::HashMap;
// TODO: FIX UNWRAP HELL!
pub struct GameReader {
    /// Total number of games analyzed
    pub total_games: usize,
    pub time_map: HashMap<i32, Vec<i32>>,
    pub args: Args,
    // private, for data measurement to be passed between pgn-reader functions
    all_times: Vec<i32>,
    time_control_offset: i32,
    max_allowed_time: i32,
    time_control: String,
    average_rating: i32,
}

impl GameReader {
    pub fn new(args: Args) -> GameReader {
        // max allowed time will be used to filter garbage data
        // because there is some, either by my collection methods or in the database
        // this is a huge optimization of my original implementation
        // so let me be proud.
        let max_allowed_time = {
            if let Some(tc) = args.time_control.as_ref() {
                tc.split('+').nth(0).unwrap().parse::<i32>().unwrap()
            } else {
                0
            }
        };

        GameReader {
            // important stuff
            total_games: 0,
            time_map: HashMap::new(),
            max_allowed_time,
            args,
            // used for determining skips
            all_times: Vec::new(),
            time_control_offset: 0,
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
        // save time
        if !(key == "TimeControl" || key == "WhiteElo" || key == "BlackElo") {
            return;
        }

        let value = std::str::from_utf8(value.0).unwrap();
        // if it doesn't have +, it's empty, so it's just "-"
        if key == "TimeControl" && value.contains('+') {
            // find the total game time,
            //and the added time per move, if it exists (after the +)
            let time_control = value.split('+').collect::<Vec<&str>>();
            // offset is the time you get for making a move, if there is one
            self.time_control_offset = time_control.get(1).unwrap().parse().unwrap();
            self.time_control = value.to_string();
        } else if key == "WhiteElo" || key == "BlackElo" {
            self.average_rating += str::parse::<i32>(value).unwrap();
        }
    }
    // now that we've read the game headers
    // we have the necessary info to determine whether to skip reading the game
    // so we tell it to skip if that's true
    fn end_headers(&mut self) -> Skip {
        // actually take the average lol
        let avg = self.average_rating / 2;
        // reset now that we stored it in the avg variable
        self.average_rating = 0;

        if self
            .args
            .time_control
            .as_ref()
            .is_some_and(|time_control| self.time_control != *time_control)
            || self.args.min_rating.is_some_and(|rating| avg < rating)
            || self.args.max_rating.is_some_and(|rating| avg > rating)
            || self
                .args
                .max_games
                .is_some_and(|max_games| self.total_games >= max_games)
        {
            return Skip(true);
        }

        Skip(false)
    }
    // in a game, we want to collect all of the times for each move,
    // so we use this function
    fn comment(&mut self, comment: pgn_reader::RawComment<'_>) {
        // convert the raw comment into a string slice
        let comment_str = std::str::from_utf8(comment.as_bytes()).unwrap();
        // possible time-saving? not sure if this well ever do anything though.
        if comment_str.is_empty() {
            return;
        }

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
    // once a game is over, we want to use all the times we collected
    fn end_game(&mut self) -> Self::Result {
        // if the game had no moves or something like that, take care of it here
        if self.all_times.is_empty() {
            return self.total_games;
        }

        // TIL: in order for skip to work and use i - 2, skip has to be at the end.
        for (i, remaining_time) in self.all_times.iter().enumerate().skip(2) {
            if self.max_allowed_time == 0 || *remaining_time <= self.max_allowed_time {
                let previous_time = self.all_times.get(i - 2).unwrap();
                let remaining = remaining_time - self.time_control_offset;
                let delta_time = previous_time - remaining;
                // add it to our big storage place
                self.time_map
                    .entry(*remaining_time)
                    .or_default()
                    .push(delta_time);
            }
        }
        // cleanup, whatever
        self.all_times.clear();
        self.total_games += 1;
        // uncomment to see how many games were printed lol
        // println!("{}", self.total_games);
        self.total_games
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
