// Game reader
// this is essentially the data collection tool

use crate::args::Args;
use pgn_reader::{Skip, Visitor};
// skipping is hugely important for optimization because it could mean skipping millions of games and saving time
pub struct GameReader {
    pub games_analyzed: usize,
    pub total_games: usize,
    pub moves_analyzed: usize,
    pub time_data: Vec<Vec<i32>>,
    pub args: Args,
    time_control_offset: i32,
    pub max_allowed_time: i32,
    is_skipping: bool,
    prev_times: [i32; 2],
}

impl GameReader {
    pub fn new(args: &Args) -> GameReader {
        // max allowed time will be used to filter garbage data
        // because there is some, either by my collection methods or in the database
        // this is a huge optimization of my original implementation
        // so let me be proud.

        // another note: we're doing a bunch of rather unsafe shit with [] indexing because i will
        // (or hopefully will) add argument validation to the program :)
        let times = args.time_control.split('+').collect::<Vec<&str>>();
        let max_allowed_time = times[0].parse::<i32>().unwrap();
        let offset = times[1].parse::<i32>().unwrap();

        let max = max_allowed_time + offset;
        // allows for pretty good optimization huh?
        // neat trick!
        let time_map: Vec<Vec<i32>> = vec![Vec::new(); max as usize + 1];

        GameReader {
            // important stuff
            games_analyzed: 0,
            time_data: time_map,
            max_allowed_time: max,
            args: args.clone(),
            time_control_offset: offset,
            is_skipping: false,
            prev_times: [-1, -1],
            total_games: 0,
            moves_analyzed: 0,
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
        if key == "TimeControl" && value != self.args.time_control {
            // println!("{}, {}", self.args.time_control, value);
            self.is_skipping = true;
            return Ok(());
        // extra check to see if max/min rating is even specified
        // if not then we don't even have to waste time doing this
        } else if (self.args.max_rating.is_some() || self.args.min_rating.is_some())
            && (key == "WhiteElo" || key == "BlackElo")
        {
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
                //println!("{}", remaining_time);
                if remaining_time <= self.max_allowed_time {
                    // initialize first moves
                    // very important!!
                    if self.prev_times[1] == -1 {
                        self.prev_times[1] = remaining_time;
                    } else if self.prev_times[0] == -1 {
                        self.prev_times[0] = remaining_time;
                    } else {
                        // if we have initialized our first 2 moves, then we can actually start measuring things
                        let delta_time =
                            self.prev_times[1] - (remaining_time - self.time_control_offset);
                        self.time_data[remaining_time as usize].push(delta_time);

                        // update our previous values
                        self.prev_times[1] = self.prev_times[0];
                        self.prev_times[0] = remaining_time;
                        self.moves_analyzed += 1;
                    }
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
        self.prev_times = [-1; 2];
        // decide to skip if we have reached or exceeded the max number of games
        if self
            .args
            .max_games
            .is_some_and(|max_games| self.games_analyzed >= max_games)
        {
            self.is_skipping = true;
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
        if !self.is_skipping {
            self.games_analyzed += 1;
        }
        self.total_games += 1;
        Skip(self.is_skipping)
    }

    // in a game, we want to collect all of the times for each move,
    // so we use this function
    fn comment(&mut self, comment: pgn_reader::RawComment<'_>) {
        self.read_comment(comment)
            .unwrap_or_else(|e| println!("There was an error parsing the game comments:\n{}", e))
    }
    fn end_game(&mut self) -> Self::Result {}
}

fn convert_time(time: &str) -> Result<i32, Box<dyn std::error::Error>> {
    // convert time into a number of seconds
    // no idea how to fix this error handling :()
    let mut units = time.split(':');
    let hours = str::parse::<i32>(units.next().unwrap())?;
    let minutes = str::parse::<i32>(units.next().unwrap())?;
    let seconds = str::parse::<i32>(units.next().unwrap())?;
    let res = hours * 3600 + minutes * 60 + seconds;

    Ok(res)
}
#[cfg(test)]
mod tests {
    #[test]
    fn conversion() {
        use super::convert_time;
        assert_eq!(convert_time("0:0:0").unwrap(), 0);
        assert_eq!(convert_time("00:00:00").unwrap(), 0);
        assert_eq!(convert_time("0:0:1").unwrap(), 1);
        assert_eq!(convert_time("0:1:01").unwrap(), 61);
        assert_eq!(convert_time("00:01:01").unwrap(), 61);
        assert_eq!(convert_time("01:01:01").unwrap(), 3661);
    }
}

/*
        let comment = &comment.0[2..comment.0.len() - 2];
        if &comment[0..4] == b"%clk" {
            let remaining_time = convert_time(&comment[5..]);
            if remaining_time <= self.max_allowed_time {
                // initialize first moves
                // very important!!
                if self.prev_times[1] == -1 {
                    self.prev_times[1] = remaining_time;
                } else if self.prev_times[0] == -1 {
                    self.prev_times[0] = remaining_time;
                } else {
                    // if we have initialized our first 2 moves, then we can actually start measuring things
                    let delta_time =
                        self.prev_times[1] - (remaining_time - self.time_control_offset);
                    self.time_data[remaining_time as usize].push(delta_time);

                    // update our previous values
                    self.prev_times[1] = self.prev_times[0];
                    self.prev_times[0] = remaining_time;
                    self.moves_analyzed += 1;
                }
            }
        }
        fn convert_time(mut time: &[u8]) -> i32 {
    // convert time into a number of seconds
    fn colon(time: &mut &[u8]) -> u8 {
        let mut s = 0;
        loop {
            let byte = time[0];
            *time = &time[1..];
            if byte == b':' {
                return s;
            }
            s = s * 10 + (byte - b'0');
        }
    }
    let hours = colon(&mut time);
    let minutes = colon(&mut time);
    let mut seconds = 0;
    for &byte in time {
        seconds = seconds * 10 + (byte - b'0');
    }
    hours as i32 * 3600 + minutes as i32 * 60 + seconds as i32
}
        */
