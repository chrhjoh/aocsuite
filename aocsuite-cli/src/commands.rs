use aocsuite_config::ConfigKey;
use aocsuite_utils::{LanguageId, PuzzlePart};
use clap::{Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ConfigCommandKey {
    Language,
    Year,
    Editor,
    RunHistoryLimit,
    Session,
}

impl std::fmt::Display for ConfigCommandKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Language => "language",
            Self::Year => "year",
            Self::Editor => "editor",
            Self::RunHistoryLimit => "run_history_limit",
            Self::Session => "session",
        })
    }
}

impl From<ConfigCommandKey> for ConfigKey {
    fn from(value: ConfigCommandKey) -> Self {
        match value {
            ConfigCommandKey::Language => Self::Language,
            ConfigCommandKey::Year => Self::Year,
            ConfigCommandKey::Editor => Self::Editor,
            ConfigCommandKey::RunHistoryLimit => Self::RunHistoryLimit,
            ConfigCommandKey::Session => Self::Session,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum AocCommand {
    /// Show the Advent of Code calendar
    Calendar,

    /// view the puzzle in browser
    View,

    /// Open the day in editor
    Open {
        #[arg(long)]
        language: Option<LanguageId>,
    },
    /// Manage library files
    Env {
        #[command(subcommand)]
        action: EnvAction,

        #[arg(long)]
        language: Option<LanguageId>,
    },
    /// Manage library files
    Lib {
        #[command(subcommand)]
        action: LibAction,

        #[arg(long)]
        language: Option<LanguageId>,
    },
    /// Edit template file
    Template {
        #[arg(long)]
        language: Option<LanguageId>,

        #[arg(long)]
        reset: bool,
    },

    /// Run the day
    Run {
        #[arg(long)]
        language: Option<LanguageId>,

        /// Puzzle part
        #[arg(long)]
        part: Option<PuzzlePart>,

        /// Input file to use instead of AoC input. AocSuite Example file can be used by supplying
        /// --test with no arg
        #[arg(long, default_missing_value = "", num_args=0..=1)]
        test: Option<String>,
    },

    /// Submit answer to Advent of Code
    Submit {
        /// Puzzle part
        #[arg(long)]
        part: PuzzlePart,

        /// Puzzle answer. Will prompt if not specified
        answer: Option<String>,
    },
    /// Open Leaderboard in browser
    Leaderboard {
        /// Leaderboard ID for private leaderboard
        id: Option<u32>,
    },
    /// Change a configuration (year, language, and session )
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Git command on the AoCSuite directory
    Git {
        /// Git arguments to pass through
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Edit gitignore file
    GitIgnore,

    /// Clean cached AoC files and language files
    Clean {
        #[command(subcommand)]
        action: CleanAction,
    },

    /// Completely remove all files
    Uninstall,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Get a configuration value
    Get {
        #[arg(value_enum)]
        key: ConfigCommandKey,
    },
    /// Set a configuration value
    Set {
        #[arg(value_enum)]
        key: ConfigCommandKey,
    },
}

#[derive(Debug, Subcommand)]
pub enum LibAction {
    /// Edit or create a library file
    Edit {
        /// Library name (without file extention)
        lib: String,
    },
    /// Remove a library file
    Remove {
        /// Library name (without file extention)
        #[arg(required_unless_present = "all")]
        lib: Option<String>,

        /// Remove all lib files
        #[arg(long, short, conflicts_with = "lib", required_unless_present = "lib")]
        all: bool,

        /// Force removal
        #[arg(long, short)]
        force: bool,
    },
    /// List library files
    List,
}

#[derive(Debug, Subcommand)]
pub enum EnvAction {
    /// Add a new package
    Add {
        /// Package name to add (e.g., "requests" or "serde=1.0")
        package: String,
    },
    /// Remove a package
    Remove {
        /// Package name to remove
        package: String,
    },
    /// List installed packages
    List,

    /// Clean and reset dependencies
    Clean {
        /// Force removal of environment
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum CleanAction {
    /// Clean cached AoC files (pages, inputs)
    Cache {
        /// Remove all cached files
        #[arg(long, group = "target")]
        all: bool,
        /// Remove all cached files for specific year
        #[arg(long, group = "target")]
        year_all: bool,
        /// Force removal without confirmation
        #[arg(long, short)]
        force: bool,
    },
    /// Clean language-related files (caches such as target dir in rust)
    Lang {
        /// Language to clean files for
        #[arg(long)]
        language: Option<LanguageId>,
        /// Force removal without confirmation
        #[arg(long, short)]
        force: bool,
    },
}
