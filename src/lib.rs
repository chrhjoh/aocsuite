mod app;
mod cli;
mod commands;
mod config;
pub mod language;
pub mod utils;

pub use app::run_aocsuite;
pub use cli::parse_args;
pub use commands::AocCommand;
pub use config::AocConfig;
