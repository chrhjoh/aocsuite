use aocsuite_client::DownloadMode;
use aocsuite_config::ConfigOpt;
use aocsuite_lang::Language;
use aocsuite_utils::Exercise;
use clap::{Args, Subcommand};

#[derive(Subcommand, Debug)]
pub enum AocCommand {
    /// Show the Advent of Code calendar
    Calendar,

    /// Initialize a new day. Executes both Download and Template to download and create exericses
    New,

    /// Download files for given exercise (input, puzzle)
    Download {
        #[arg(long, value_enum, default_value_t=DownloadMode::All)]
        mode: DownloadMode,
    },

    /// Generate new exercise files from template
    Template {
        #[arg(long)]
        directory: Option<String>,
        #[arg(long)]
        language: Language,
    },

    /// Open the puzzle in browser
    Open,

    /// Run the exercise
    Run {
        // Run All Puzzles
        // all: bool,
        /// Input file to use instead of year{i}/day{j}/input.txt
        input_file: Option<String>, //TODO:: Add way to use a default example file. mutually exclustive with input file
        #[arg(long)]
        language: Language,
    },

    /// Submit answer to Advent of Code
    Submit {
        /// Puzzle part
        part: Exercise,

        /// Puzzle answer. Will prompt if not specified
        answer: String,
    },

    // /// Display Leaderboard
    // Leaderboard {
    //     /// Leaderboard ID for private leaderboard
    //     leaderboard_id: Option<u32>,
    // },
    /// Change a configuration (year, language, and session )
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Get a configuration value
    Get(ConfigGetArgs),

    /// Set a configuration value
    Set(ConfigSetArgs),
}

#[derive(Debug, Args)]
pub struct ConfigGetArgs {
    #[arg(value_enum)]
    pub key: ConfigOpt,
}

#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    #[arg(value_enum)]
    pub key: ConfigOpt,

    /// The value to assign to the config key
    pub value: String,
}
