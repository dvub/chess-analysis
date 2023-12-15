use super::two_var::generate_two_var_plots;
use crate::plots::one_var::generate_one_var_plots;
use crate::reader::GameReader;
use std::{
    fs::create_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

// TODO: add helpful error messages

pub fn generate_plots(game_reader: &GameReader) -> Result<(), Box<dyn std::error::Error>> {
    let resolution = {
        if let Some(r) = game_reader.args.resolution {
            (r as u32, r as u32)
        } else {
            (1000, 1000)
        }
    };
    let path = gen_path(&game_reader.args.output)?;
    generate_two_var_plots(game_reader, &path, resolution)?;
    if game_reader.args.one_var {
        println!("Creating one-variable histograms...");
        generate_one_var_plots(game_reader, &path, resolution)?;
    }
    Ok(())
}

/// Create the necessary directories to output graphs
fn gen_path(path: &str) -> std::io::Result<PathBuf> {
    // plot shit

    let unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // if a system clock is fucked up we will have problems.
        .as_secs();

    let path = PathBuf::from(path);
    if !path.exists() {
        println!("Path doesn't exist, trying to create...");
        create_dir(&path)?;
    }
    let new_path = path.join(unix_timestamp.to_string());
    create_dir(&new_path)?;
    create_dir(new_path.join("2-var"))?;
    create_dir(new_path.join("1-var"))?;

    Ok(new_path)
}

pub enum GraphType {
    Average,
    All,
    RelativeFrequencyX,
    RelativeFrequencyY,
}

pub fn generate_caption(graph_type: GraphType, game_reader: &GameReader) -> String {
    let elo_text = {
        if game_reader.args.min_rating.is_none() && game_reader.args.max_rating.is_none() {
            "No ELO Limit".to_string()
        } else {
            let mut str = String::new();
            if let Some(rating) = game_reader.args.min_rating {
                str.push_str(&rating.to_string());
            }
            str.push('-');
            if let Some(rating) = game_reader.args.max_rating {
                str.push_str(&rating.to_string());
            };
            str.push_str(" ELO*");
            str
        }
    };
    let title = match graph_type {
        GraphType::All => "All",
        GraphType::Average => "Average TTM",
        GraphType::RelativeFrequencyX => "RF of Time Left",
        GraphType::RelativeFrequencyY => "RF of TTM",
    };
    format!(
        "{} ({}, {} seconds, {} Games)",
        title, elo_text, game_reader.args.time_control, game_reader.games_analyzed
    )
}

#[cfg(test)]
mod tests {
    use std::fs::{read_dir, remove_dir};

    #[test]
    fn test_path() {
        let test_path = "./test";
        let res = super::gen_path(test_path).unwrap();
        assert!(read_dir(test_path).is_ok());
        assert!(read_dir(&res).is_ok());
        assert_eq!(read_dir(&res).unwrap().count(), 2);

        // TODO: change this weirdness
        remove_dir(res.join("1-var")).unwrap();
        remove_dir(res.join("2-var")).unwrap();
        remove_dir(res).unwrap();
        remove_dir(test_path).unwrap();
    }
}
