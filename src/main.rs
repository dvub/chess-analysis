use std::{collections::HashMap, fs::File, io::BufReader};

use pgn_reader::{BufferedReader, SanPlus, Skip};
use plotters::prelude::*;
struct GameReader {
    moves: usize,
    total_games: usize,
    all_times: Vec<i32>,
    map: HashMap<i32, Vec<i32>>,
}
impl GameReader {
    fn new() -> GameReader {
        GameReader {
            moves: 0,
            total_games: 0,
            all_times: Vec::new(),
            map: HashMap::new(),
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
            let remaining_time = time;
            let delta_time = last_time - time;

            self.map
                .entry(*remaining_time)
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
    let file_name = "games/some-games.pgn";
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

    println!("{:?}", game_reader.map);

    let root = BitMapBackend::new("plotters-doc-data/0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
            &RED,
        ))?
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
