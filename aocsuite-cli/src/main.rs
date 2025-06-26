use aocsuite_cli::{AocCliError, AocCommand, run_aocsuite};
use aocsuite_utils::{PuzzleDay, PuzzleYear};

use clap::Parser;

/// Advent of Code tool for downloading, executing, submitting, etc...
#[derive(Parser, Debug)]
struct AocCli {
    #[command(subcommand)]
    /// Command to execute
    command: AocCommand,

    /// Specify day for exercises etc. (default: current)
    #[arg(long, default_value_t=aocsuite_utils::today_day())]
    day: PuzzleDay,

    /// Specify year for calendar, exercises, etc (default: current)
    #[arg(long, default_value_t=aocsuite_utils::today_year())]
    year: PuzzleYear,
}

fn terminate_with_error(err: AocCliError) {
    eprintln!("encountered error: {err}");
    std::process::exit(1);
}

fn main() {
    let args = AocCli::parse();
    if let Err(err) = run_aocsuite(args.command, args.day, args.year) {
        terminate_with_error(err);
    }
}
