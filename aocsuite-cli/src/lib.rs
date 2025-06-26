use aocsuite_client::AocClientError;
use aocsuite_config::AocConfigError;
use aocsuite_lang::AocLanguageError;
use aocsuite_utils::PuzzleNotReleasedError;
use thiserror::Error;
mod app;
mod commands;

pub use app::run_aocsuite;

pub use commands::{AocCommand, ConfigCommand};

#[derive(Error, Debug)]
pub enum AocCliError {
    #[error(transparent)]
    Client(#[from] AocClientError),

    #[error(transparent)]
    Language(#[from] AocLanguageError),

    #[error(transparent)]
    Unreleased(#[from] PuzzleNotReleasedError),

    #[error(transparent)]
    Config(#[from] AocConfigError),
}

type AocCliResult<T> = Result<T, AocCliError>;
