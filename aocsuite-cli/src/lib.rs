use aocsuite_client::AocClientError;
use aocsuite_config::AocConfigError;
use aocsuite_editor::AocEditorError;
use aocsuite_fs::AocFileError;
use aocsuite_lang::AocLanguageError;
use aocsuite_utils::PuzzleNotReleasedError;
use git::AocGitError;
use thiserror::Error;
mod app;
mod commands;
mod git;

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

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Editor(#[from] AocEditorError),

    #[error(transparent)]
    File(#[from] AocFileError),

    #[error(transparent)]
    Git(#[from] AocGitError),
}

type AocCliResult<T> = Result<T, AocCliError>;
