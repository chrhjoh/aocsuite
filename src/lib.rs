mod app;
mod cli;
mod commands;
mod config;
mod error;
pub mod language;
mod requests;
mod scaffold;
pub mod utils;

pub use app::run_aocsuite;
pub use cli::AocArgs;
pub use commands::AocCommand;
pub use config::AocConfig;
pub use error::AocError;
pub use requests::{AocHttp, AocPage};
pub use scaffold::scaffold;

pub type AocResult<T> = Result<T, AocError>;
