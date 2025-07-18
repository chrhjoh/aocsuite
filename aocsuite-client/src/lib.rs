mod parsing;
mod requests;

use clap::ValueEnum;
pub use parsing::{HttpParser, ParserType, parse_html};
pub use requests::{AocHttp, AocPage, open_puzzle_page};
use std::io;
use thiserror::Error;

pub type AocClientResult<T> = Result<T, AocClientError>;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum DownloadMode {
    All,
    Input,
    Puzzle,
}

#[derive(Debug, Error)]
pub enum AocClientError {
    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("UnreleasedError: {0}")]
    Unreleased(#[from] aocsuite_utils::PuzzleNotReleasedError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("HTML parsing error: {0}")]
    HtmlError(String),
}
