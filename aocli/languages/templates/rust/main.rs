use clap::{Parser, ValueEnum};
use std::fmt;
use std::fs;

use crate::exercise::{exercise1, exercise2};
mod exercise;

/// AoC for Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct AocArgs {
    /// Exercise to execute. (1 or 2)
    #[arg(short, long, value_enum)]
    exercise: Exercise,

    /// Number of times to greet
    #[arg(short, long)]
    data_path: String,
}

#[derive(Clone, Debug, ValueEnum)]
enum Exercise {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
}
impl fmt::Display for Exercise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let exercise = match self {
            Self::One => 1,
            Self::Two => 2,
        };
        write!(f, "{}", exercise)
    }
}

fn load_data(data_path: &str) -> String {
    fs::read_to_string(data_path).unwrap()
}

fn main() {
    let args = AocArgs::parse();
    let data = load_data(&args.data_path);
    let exercise_function = match args.exercise {
        Exercise::One => exercise1,
        Exercise::Two => exercise2,
    };
    let answer = exercise_function(&data);

    println!("Answer to exercise {}: {}", args.exercise, answer)
}
