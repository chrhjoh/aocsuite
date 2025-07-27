use aocsuite_config::ConfigOpt;
use aocsuite_lang::LanguageType;
use aocsuite_utils::Exercise;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AocCommand {
    /// Show the Advent of Code calendar
    Calendar,

    /// view the puzzle in browser
    View,

    /// Open the day in editor
    Open {
        #[arg(long)]
        language: Option<LanguageType>,
    },
    /// Manage library files
    Dep {
        #[command(subcommand)]
        action: DepAction,

        #[arg(long)]
        language: Option<LanguageType>,
    },
    /// Manage library files
    Lib {
        #[command(subcommand)]
        action: LibAction,

        #[arg(long)]
        language: Option<LanguageType>,
    },
    /// Edit template file
    Template {
        #[arg(long)]
        language: Option<LanguageType>,
    },

    /// Run the day
    Run {
        #[arg(long)]
        language: Option<LanguageType>,

        /// Puzzle part
        part: Option<Exercise>,

        /// Input file to use instead of AoC input. Provided example file can be used by supplying
        /// --test with no arg
        #[arg(long, default_missing_value = "", num_args=0..=1)]
        test: Option<String>,
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
    //TODO: Need a clean command
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Get a configuration value
    Get {
        #[arg(value_enum)]
        key: ConfigOpt,
    },
    /// Set a configuration value
    Set {
        #[arg(value_enum)]
        key: ConfigOpt,
    },
}

//TODO: Implement Lib and Dep
#[derive(Debug, Subcommand)]
pub enum LibAction {
    /// Add a library file
    Edit,
    /// Remove a library file
    Remove,
    /// List library files
    List,
}

#[derive(Debug, Subcommand)]
pub enum DepAction {
    /// Add a new library file
    Add,
    /// Remove a library file
    Remove,
    /// List library files
    List,
}
