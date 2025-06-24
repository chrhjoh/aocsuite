use std::io;
use thiserror::Error;

use crate::{
    AocPage,
    utils::{PuzzleDay, PuzzleYear},
};

#[derive(Debug, Error)]
pub enum AocError {
    #[error(transparent)]
    Http(#[from] AocHttpError),

    #[error(transparent)]
    Parse(#[from] AocParseError),

    #[error("Advent of code not released yet: {0} December {1}")]
    Unreleased(PuzzleDay, PuzzleYear),

    #[error("unknown error: {0}")]
    Other(String),

    #[error(transparent)]
    Io(#[from] AocIoError),

    #[error(transparent)]
    Execution(#[from] AocExecutionError),
}

#[derive(Debug, Error)]
pub enum AocParseError {
    #[error("failed to parse puzzle input: {0}")]
    InputParseError(String),

    #[error("unexpected HTML structure for page {0:?}")]
    HtmlParseError(AocPage),

    #[error("unexpected toml structure")]
    TomlParseError(#[source] toml_edit::TomlError),
}

#[derive(Debug, Error)]
pub enum AocHttpError {
    #[error("failed to GET page {0:?}: {1}")]
    GetError(AocPage, #[source] reqwest::Error),

    #[error("failed to POST to page {0:?}: {1}")]
    PostError(AocPage, #[source] reqwest::Error),

    #[error("Unexpected response from {0:?}: {1}")]
    ResponseError(AocPage, #[source] reqwest::Error),

    #[error("http client could not add session: {0}")]
    ClientError(#[source] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum AocIoError {
    #[error("Failed to read file: {0}")]
    ReadError(#[source] io::Error),

    #[error("Failed to write file: {0}")]
    WriteError(#[source] io::Error),

    #[error("Failed to create directory: {0}")]
    CreateDirError(#[source] io::Error),

    #[error("I/O error: {0}")]
    Other(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum AocExecutionError {
    #[error("failed to compile program: {0}")]
    Compile(#[source] io::Error),

    #[error("failed to run program: {0}")]
    Run(#[from] io::Error),
}
