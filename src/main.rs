use aocsuite::{
    AocCommand,
    language::Language,
    utils::{DownloadMode, Exercise, PuzzleDay, PuzzleYear},
};

use clap::Parser;
use clap::Subcommand;
use std::process;

/// Advent of Code tool for downloading, executing, submitting, etc...
#[derive(Parser, Debug)]
pub struct AocArgs {
    #[command(subcommand)]
    /// Command to execute
    pub command: AocCommand,

    /// Specify day for exercises etc. (default: current)
    #[arg(long)]
    pub day: Option<PuzzleDay>,

    /// Specify year for calendar, exercises, etc (default: current)
    #[arg(long)]
    pub year: Option<PuzzleYear>,
}

fn main() {
    let args = AocArgs::parse();
    run(args);
}
