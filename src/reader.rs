use crate::args::Args;
use pgn_reader::{Skip, Visitor};
use std::collections::HashMap;
// TODO: FIX UNWRAP HELL!
// skipping is hugely important for optimization because it could mean skipping millions of games and saving time
pub struct GameReader {
    /// Total number of games analyzed
    pub total_games: usize,
    pub time_map: HashMap<i32, Vec<i32>>,
    pub args: Args,
    // private, for data measurement to be passed between pgn-reader functions
    all_times: Vec<i32>,
    time_control_offset: i32,
    max_allowed_time: i32,
    is_skipping: bool,
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
            is_skipping: false,
        }
    }
}

impl Visitor for GameReader {
    type Result = ();

    // honestly probably not necessary, but left it in here from the docs
    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn begin_game(&mut self) {
        // reset variables, IMPORTANT!
        self.all_times.clear();
        self.is_skipping = false;
        // decide to skip if we have reached or exceeded the max number of games
        if self
            .args
            .max_games
            .is_some_and(|max_games| self.total_games >= max_games)
        {
            self.is_skipping = true;
        }
    }

    // first of all, we will read the headers to determine if we should even read this game.
    fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
        // if we know we're skipping because of some other condition,
        // just return here
        if self.is_skipping {
            return;
        };

        let key = std::str::from_utf8(key).unwrap();
        // skip unnecessary headers
        if !(key == "TimeControl" || key == "WhiteElo" || key == "BlackElo") {
            return;
        }

        let value = std::str::from_utf8(value.0).unwrap();
        // if it doesn't have +, it's empty, so it's just "-"
        if key == "TimeControl" && value.contains('+') {
            // decide whether or not to skip based on arguments
            // skip wrong time control
            if self
                .args
                .time_control
                .as_ref()
                .is_some_and(|time_control| value != *time_control)
            {
                self.is_skipping = true;
                // skipping early will save a little time
                return;
            }

            // find the total game time,
            //and the added time per move, if it exists (after the +)
            let time_control = value.split('+').collect::<Vec<&str>>();
            // offset is the time you get for making a move, if there is one
            self.time_control_offset = time_control.get(1).unwrap().parse().unwrap();
            //
        } else if key == "WhiteElo" || key == "BlackElo" {
            // now, NOTE: we ARE assuming that whichever elo comes first is close to the same as the second one.
            let val = value.parse::<i32>().unwrap();
            // decide whether or not to skip based on arguments
            // skip wrong rating
            if self.args.min_rating.is_some_and(|rating| val < rating)
                || self.args.max_rating.is_some_and(|rating| val > rating)
            {
                self.is_skipping = true;
            }
        }
    }
    // SUPER IMPORTANT!
    // now that we've read the game headers
    // we have the necessary info to determine whether to skip reading the game
    fn end_headers(&mut self) -> Skip {
        Skip(self.is_skipping)
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
            return;
        }

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
        self.total_games += 1;
        // uncomment to see how many games were printed lol
        // println!("{}", self.total_games);
    }
}

fn convert_time(time: &str) -> i32 {
    // convert time into a number of seconds
    // TODO: fix error handling
    let mut units = time.split(':');
    let hours = str::parse::<i32>(units.next().unwrap()).unwrap();
    let minutes = str::parse::<i32>(units.next().unwrap()).unwrap();
    let seconds = str::parse::<i32>(units.next().unwrap()).unwrap();
    hours * 3600 + minutes * 60 + seconds
}
