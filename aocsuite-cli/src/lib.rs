use aocsuite_client::AocClientError;
use aocsuite_config::AocConfigError;
use aocsuite_editor::AocEditorError;
use aocsuite_lang::AocLanguageError;
use aocsuite_parser::ParserError;
use aocsuite_storage::{ContentError, LayoutError};
use aocsuite_utils::ReleaseError;
use thiserror::Error;
mod app;
mod commands;

pub use app::run_aocsuite;

pub use commands::{AocCommand, ConfigCommand, ConfigCommandKey};

#[derive(Error, Debug)]
pub enum AocCliError {
    #[error("operation not allowed: {0}")]
    NotAllowed(&'static str),

    #[error(transparent)]
    Client(#[from] AocClientError),

    #[error(transparent)]
    Language(#[from] AocLanguageError),

    #[error(transparent)]
    Unreleased(#[from] ReleaseError),

    #[error(transparent)]
    Config(#[from] AocConfigError),

    #[error(transparent)]
    Storage(#[from] LayoutError),

    #[error(transparent)]
    Content(#[from] ContentError),

    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error("environment error: {0}")]
    Environment(#[from] std::env::VarError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Editor(#[from] AocEditorError),

    #[error(transparent)]
    Workspace(#[from] aocsuite_storage::WorkspaceError),
}

type AocCliResult<T> = Result<T, AocCliError>;
