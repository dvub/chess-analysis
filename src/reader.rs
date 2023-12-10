// Game reader
// this is essentially the data collection tool

use crate::args::Args;
use pgn_reader::{Skip, Visitor};
// skipping is hugely important for optimization because it could mean skipping millions of games and saving time
pub struct GameReader {
    pub total_games: usize,
    pub time_data: Vec<Vec<i32>>,
    pub args: Args,
    time_control_offset: i32,
    pub max_allowed_time: i32,
    is_skipping: bool,
    two_ago: i32,
    prev: i32,
}

impl GameReader {
    pub fn new(args: Args) -> GameReader {
        // max allowed time will be used to filter garbage data
        // because there is some, either by my collection methods or in the database
        // this is a huge optimization of my original implementation
        // so let me be proud.

        // another note: we're doing a bunch of rather unsafe shit with [] indexing because i will
        // (or hopefully will) add argument validation to the program :)
        let max_allowed_time = args.time_control.split('+').collect::<Vec<&str>>()[0]
            .parse::<i32>()
            .unwrap();
        // allows for pretty good optimization huh?
        // neat trick!
        let time_map: Vec<Vec<i32>> = vec![Vec::new(); max_allowed_time as usize + 1];

        GameReader {
            // important stuff
            total_games: 0,
            time_data: time_map,
            max_allowed_time,
            args,
            time_control_offset: 0,
            is_skipping: false,
            two_ago: 0,
            prev: 0,
        }
    }

    fn read_header(
        &mut self,
        key: &[u8],
        value: pgn_reader::RawHeader<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // if we know we're skipping because of some other condition,
        // just return here
        if self.is_skipping {
            return Ok(());
        }

        let key = std::str::from_utf8(key)?;
        // skip unnecessary headers
        if !(key == "TimeControl" || key == "WhiteElo" || key == "BlackElo") {
            return Ok(());
        }

        let value = std::str::from_utf8(value.0)?;
        // if it doesn't have +, it's empty, so it's just "-"
        if key == "TimeControl" && value.contains('+') {
            // decide whether or not to skip based on arguments
            // skip wrong time control
            if value != self.args.time_control {
                self.is_skipping = true;
                // skipping early will save a little time
                return Ok(());
            }

            // find the total game time,
            //and the added time per move, if it exists (after the +)
            let time_control = value.split('+').collect::<Vec<&str>>();
            // offset is the time you get for making a move, if there is one
            self.time_control_offset = time_control[1].parse()?;
            //
        } else if key == "WhiteElo" || key == "BlackElo" {
            // now, NOTE: we ARE assuming that whichever elo comes first is close to the same as the second one.
            let val = value.parse::<i32>()?;
            // decide whether or not to skip based on arguments
            // skip wrong rating
            if self.args.min_rating.is_some_and(|rating| val < rating)
                || self.args.max_rating.is_some_and(|rating| val > rating)
            {
                self.is_skipping = true;
            }
        }
        Ok(())
    }
    fn read_comment(
        &mut self,
        comment: pgn_reader::RawComment<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // convert the raw comment into a string slice
        let comment_str = std::str::from_utf8(comment.as_bytes())?;
        // possible time-saving? not sure if this well ever do anything though.
        if comment_str.is_empty() {
            return Ok(());
        }

        // get rid of brackets in comments, clean and split
        let cleaned = comment_str.replace(&['[', ']'][..], "");
        let comment_vec: Vec<&str> = cleaned.split(' ').collect();

        // basically if we find %clk, we know the next element is a time, so lets convert that and add it to our vec of times
        for (i, term) in comment_vec.iter().enumerate() {
            if *term == "%clk" {
                let remaining_time = convert_time(comment_vec[i + 1])?;
                if remaining_time <= self.max_allowed_time {
                    let delta_time = self.two_ago - (remaining_time - self.time_control_offset);
                    self.time_data[remaining_time as usize].push(delta_time);

                    // update our previous values
                    self.two_ago = self.prev;
                    self.prev = remaining_time;
                }
            }
        }
        Ok(())
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
        self.is_skipping = false;
        // decide to skip if we have reached or exceeded the max number of games
        if self
            .args
            .max_games
            .is_some_and(|max_games| self.total_games >= max_games)
        {
            self.is_skipping = true;
        } else {
            self.total_games += 1;
        }
    }

    // first of all, we will read the headers to determine if we should even read this game.
    // had to rewrite inside a wrapper type shit to have good error handling :)
    fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
        self.read_header(key, value)
            .unwrap_or_else(|e| println!("An error occurred while reading the header:\n{}", e));
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
        self.read_comment(comment)
            .unwrap_or_else(|e| println!("There was an error parsing the game comments:\n{}", e))
    }
    fn end_game(&mut self) -> Self::Result {
        // uncomment to see how many games were printed lol
        // println!("{}", self.total_games);
    }
}

fn convert_time(time: &str) -> Result<i32, Box<dyn std::error::Error>> {
    // convert time into a number of seconds
    // no idea how to fix this error handling :()
    let mut units = time.split(':');
    let hours = str::parse::<i32>(units.next().unwrap())?;
    let minutes = str::parse::<i32>(units.next().unwrap())?;
    let seconds = str::parse::<i32>(units.next().unwrap())?;
    Ok(hours * 3600 + minutes * 60 + seconds)
}
