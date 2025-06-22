use crate::utils::{DownloadMode, Exercise};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Show the Advent of Code calendar
    Calendar,

    /// Initialize a new day. Executes both Download and Start to download and create exericses
    Init,

    /// Download files for given exercise (input, puzzle)
    Download {
        #[command(subcommand)]
        mode: Option<DownloadMode>,
    },

    /// Generate new exercise files from template
    New { template: Option<String> },

    /// Open the puzzle in browser
    Open,

    /// Edit the exercise with your editor
    Edit,

    /// Run the exercise
    Run {
        // Run All Puzzles
        // all: bool,
        /// Input file to use instead of year{i}/day{j}/input.txt
        input_file: Option<String>,
    },

    /// Submit answer to Advent of Code
    Submit {
        /// Puzzle part
        part: Exercise,

        /// Puzzle answer. Will prompt if not specified
        answer: Option<String>,
    },

    /// Display Leaderboard
    Leaderboard {
        /// Leaderboard ID for private leaderboard
        leaderboard_id: Option<u32>,
    },

    /// Change a configuration (year, language, and session )
    Config,
}
