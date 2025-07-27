mod dir;
mod file;

use aocsuite_client::AocClientError;
pub use dir::AocContentDir;
pub use file::{update_cache_status, AocContentFile, AocFileType};
use thiserror::Error;

type AocFileResult<T> = Result<T, AocFileError>;

#[derive(Error, Debug)]
pub enum AocFileError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Client(#[from] AocClientError),

    #[error("invalid file error: {0}")]
    InvalidFile(String),
}
