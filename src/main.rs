use std::{collections::HashMap, fs::File, io::BufReader};

use pgn_reader::{BufferedReader, SanPlus, Skip, Visitor};
use plotters::prelude::*;
struct GameReader {
    moves: usize,
    total_games: usize,
    all_times: Vec<i32>,
    time_map: HashMap<i32, Vec<i32>>,
    time_control_offset: i32,
}
impl GameReader {
    fn new() -> GameReader {
        GameReader {
            moves: 0,
            total_games: 0,
            all_times: Vec::new(),
            time_map: HashMap::new(),
            time_control_offset: 0,
        }
    }
}

impl Visitor for GameReader {
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

    fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
        let str = std::str::from_utf8(key).unwrap();
        if str == "TimeControl" {
            let value_str = std::str::from_utf8(value.0).unwrap();

            let offset = value_str.split('+').nth(1).unwrap();
            self.time_control_offset = offset.parse().unwrap();
        }
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("{:?}", game_reader.time_map);

    let root = BitMapBackend::new("graphs/0.png", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        // .caption("y=x^2", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..670f32, 0f32..670f32)?;

    chart.configure_mesh().draw()?;
    chart
        .draw_series(game_reader.time_map.iter().map(|(x, y_values)| {
            Circle::new(
                (*x as f32, *y_values.first().unwrap() as f32),
                2,
                GREEN.filled(),
            )
        }))
        .unwrap();

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
