use chrono::Datelike;
use clap::Parser;
use std::process;

use crate::{
    Command,
    utils::{PuzzleDay, PuzzleYear, today},
};

/// Advent of Code tool for downloading, executing, submitting, etc...
#[derive(Parser, Debug)]
pub struct AocArgs {
    #[command(subcommand)]
    /// Command to execute
    command: Command,

    /// Specify day for exercises etc. (default: current)
    #[arg(long)]
    pub day: Option<PuzzleDay>,

    /// Specify year for calendar, exercises, etc (default: current)
    #[arg(long)]
    pub year: Option<PuzzleYear>,
}

pub fn parse_args() -> AocArgs {
    let args = AocArgs::parse();
    if let Some(day) = args.day {
        if day > 25 || day == 0 {
            eprintln!("Error: Day must be between 1 and 25, but got {}", day);
            process::exit(1);
        }
    }
    if let Some(year) = args.year {
        let current_year = today().year();
        if year < 2015 || current_year < year {
            eprintln!(
                "Error: Year must be between 2015 and {}, but got {}",
                current_year, year
            );
            process::exit(1);
        }
    }
    args
}
