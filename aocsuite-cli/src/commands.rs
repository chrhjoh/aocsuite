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
    New {
        /// Copies file found at {template_dir}/{language}.template instead of default file
        #[arg(long, short)]
        template_dir: Option<String>,
        #[arg(long)]
        language: Option<Language>,
        #[arg(long)]
        overwrite: bool,
    },

    /// Download files for given day (input, puzzle)
    Download {
        #[arg(long, value_enum, default_value_t=DownloadMode::All)]
        mode: DownloadMode,
        #[arg(long)]
        overwrite: bool,
    },

    /// Generate new files from template
    Template {
        /// Copies file found at {template_dir}/{language}.template instead of default file
        #[arg(long, short)]
        template_dir: Option<String>,
        #[arg(long)]
        language: Option<Language>,
        #[arg(long)]
        overwrite: bool,
    },

    /// Open the puzzle in browser
    Open,

    /// Open the day in editor
    Edit {
        #[arg(long)]
        language: Option<Language>,
    },

    /// Run the day
    Run {
        // Run All Puzzles
        // all: bool,
        #[arg(long)]
        language: Option<Language>,

        /// Puzzle part
        part: Option<Exercise>,
    },
    /// Run the day with other input
    Test {
        /// Input file to use instead of data/year{i}/day{j}/example.txt
        #[arg(long)]
        input_file: Option<String>,

        #[arg(long)]
        language: Option<Language>,

        /// Puzzle part
        part: Option<Exercise>,
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
    pub value: Option<String>,
}
