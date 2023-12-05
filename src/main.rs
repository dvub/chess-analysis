use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    time::{SystemTime, UNIX_EPOCH},
};

use pgn_reader::{BufferedReader, SanPlus, Skip, Visitor};
use plotters::prelude::*;
struct GameReader {
    moves: usize,
    total_games: usize,
    all_times: Vec<i32>,
    time_map: HashMap<i32, Vec<i32>>,
    time_control_offset: i32,
    time_control: String,
    rating: i32,
}
impl GameReader {
    fn new() -> GameReader {
        GameReader {
            moves: 0,
            total_games: 0,
            all_times: Vec::new(),
            time_map: HashMap::new(),
            time_control_offset: 0,
            time_control: "".to_string(),
            rating: 0,
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
        let str = std::str::from_utf8(key).unwrap();
        let value_str = std::str::from_utf8(value.0).unwrap();
        if str == "TimeControl" {
            let offset = value_str.split('+').nth(1).unwrap_or("0");
            self.time_control_offset = offset.parse().unwrap();
            self.time_control = value_str.to_string();
        }

        if str == "WhiteElo" || str == "BlackElo" {
            self.rating = value_str.parse::<i32>().unwrap();
        }
    }
    fn end_headers(&mut self) -> Skip {
        if self.time_control != "600+0"
            || !(self.rating > 1000 && self.rating < 2000)
            || self.total_games >= 1000
        {
            return Skip(true);
        }
        self.total_games += 1;
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
/* each point could be
// - (time left, delta time)
//      = one point per move
// - (time left, average time for x)
//      = one point per x axis value <- like this one the best so far
// - (time left, averave time taken per move per game)
//      = one point per game
*/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // current parent directory
    let current = std::env::current_dir()?;

    // get the correct game file within games directory
    let file_name = "games/oct-2023-games.pgn";
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

    // TODO: probably dont get keys twice.
    let min_x = *game_reader.time_map.keys().min().unwrap();
    let max_x = *game_reader.time_map.keys().max().unwrap();

    // TODO: DON't flatten twice you stupid fucking idiot
    let min_y = *game_reader.time_map.values().flatten().min().unwrap();
    let max_y = *game_reader.time_map.values().flatten().max().unwrap();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file = format!("graphs/{}.svg", time);
    let root = SVGBackend::new(file.as_str(), (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Time Analysis", ("sans-serif", 35).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(max_x..min_x, 0..100)?;

    chart.configure_mesh().draw()?;

    let points_iter = game_reader.time_map.iter().map(|(x, y_values)| {
        /*y_values
        .iter()
        .map(|y| Circle::new((*x, *y), 2, GREEN.filled()))
        */
        let y: i32 = y_values.iter().sum::<i32>() / y_values.len() as i32;
        Circle::new((*x, y), 2, GREEN.filled())
    });

    chart.draw_series(points_iter).unwrap();

    root.present()?;

    Ok(())
}
