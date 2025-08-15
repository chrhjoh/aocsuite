mod dir;
mod file;

use std::path::PathBuf;

use aocsuite_client::AocClientError;
use aocsuite_utils::{PuzzleDay, PuzzleYear};
pub use dir::AocCacheDir;
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

    #[error("error cleaning cache: {0}")]
    CleanError(String),
}

pub fn clean_cache(year: Option<PuzzleYear>, day: Option<PuzzleDay>) -> AocFileResult<()> {
    let cache_base_dir = AocCacheDir::new();
    let dir: PathBuf;
    match (year, day) {
        (None, Some(_)) => {
            return Err(AocFileError::CleanError(
                "Year was not specified but day was".to_string(),
            ))
        }
        (Some(year), Some(day)) => dir = cache_base_dir.daily_data_dir(day, year),
        (Some(year), None) => dir = cache_base_dir.yearly_data_dir(year),
        (None, None) => dir = aocsuite_utils::get_aocsuite_dir().join("data"),
    }
    if !std::fs::exists(&dir)? {
        return Err(AocFileError::FileNotFound(
            dir.to_str().unwrap().to_string(),
        ));
    }
    std::fs::remove_dir_all(dir)?;

    Ok(())
}
