use std::{fs, fs::OpenOptions, path::PathBuf};

use aocsuite_utils::PuzzleId;
use thiserror::Error;

pub struct ExampleStore {
    examples_dir: PathBuf,
}

impl ExampleStore {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self {
            examples_dir: workspace_dir.join("examples"),
        }
    }

    fn path(&self, puzzle: PuzzleId) -> PathBuf {
        self.examples_dir.join(format!("{puzzle}.txt"))
    }

    pub fn ensure(&self, puzzle: PuzzleId) -> ExampleResult<PathBuf> {
        let path = self.path(puzzle);
        fs::create_dir_all(&self.examples_dir)?;
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && path.is_file() => {
                Ok(path)
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ExampleError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type ExampleResult<T> = Result<T, ExampleError>;
