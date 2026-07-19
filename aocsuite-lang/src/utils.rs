use std::{
    fmt,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    process::Output,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use aocsuite_config::AocConfigError;
use aocsuite_utils::{PuzzleDay, PuzzleYear};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::traits::LanguageHandler;

#[derive(Debug, Clone)]
pub enum SolverFile {
    PuzzleSolution(PuzzleDay, PuzzleYear),
    Entrypoint,
    ActiveSolution(PuzzleDay, PuzzleYear),
    SolutionTemplate,
}

#[derive(Serialize, Deserialize)]
struct PartResult {
    answer: String,
    runtime_ms: u128,
}

impl fmt::Display for PartResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Answer: {}", self.answer)?;
        writeln!(f, "Runtime: {} ms", self.runtime_ms)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ExerciseOutput {
    part1: Option<PartResult>,
    part2: Option<PartResult>,
}

impl fmt::Display for ExerciseOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref p1) = self.part1 {
            writeln!(f, "\n┌──────────────┐")?;
            writeln!(f, "│   Part 1     │")?;
            writeln!(f, "└──────────────┘")?;
            writeln!(f, "{}", p1)?;
        }

        if self.part1.is_some() && self.part2.is_some() {
            writeln!(f)?;
        }

        if let Some(ref p2) = self.part2 {
            writeln!(f, "\n┌──────────────┐")?;
            writeln!(f, "│   Part 2     │")?;
            writeln!(f, "└──────────────┘")?;
            writeln!(f, "{}", p2)?;
        }

        Ok(())
    }
}

static RESULT_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn new_result_file_path(runs_dir: &Path) -> AocLanguageResult<PathBuf> {
    fs::create_dir_all(runs_dir)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after the Unix epoch")
        .as_nanos();

    for _ in 0..16 {
        let sequence = RESULT_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = runs_dir.join(format!(
            "result-{}-{timestamp}-{sequence}.json",
            std::process::id()
        ));
        if !path.exists() {
            return Ok(path);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate a unique result file",
    )
    .into())
}

pub fn with_result_file<T>(
    result_file: &Path,
    operation: impl FnOnce(&Path) -> AocLanguageResult<T>,
) -> AocLanguageResult<T> {
    let result = operation(result_file);
    match fs::remove_file(result_file) {
        Ok(()) => result,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => result,
        Err(error) => result.and(Err(error.into())),
    }
}

pub fn read_result(result_file: &Path) -> AocLanguageResult<ExerciseOutput> {
    let reader = BufReader::new(File::open(result_file)?);
    Ok(serde_json::from_reader(reader)?)
}

pub fn handle_command_output(output: Output) -> AocLanguageResult<()> {
    if !output.status.success() {
        // The compile command ran but failed
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AocLanguageError::Command(stderr.to_string()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout != "" {
        println!("Standard out from exercise {}", stdout)
    }
    Ok(())
}

pub fn symlink_file(from: &Path, to: &Path) -> AocLanguageResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = std::fs::remove_file(to); // Best-effort remove
        symlink(from, to)?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        let _ = std::fs::remove_file(to); // Best-effort remove
        symlink_file(from, to)?;
    }

    Ok(())
}
#[derive(Error, Debug)]
pub enum AocLanguageError {
    #[error("error executing command: {0}")]
    Command(String),

    #[error("Language not found: {0}")]
    LangNotFound(String),
    #[error("failed to read template '{path}': {source}")]
    TemplateRead {
        #[source]
        source: std::io::Error,
        path: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error parsing result json file: {0}")]
    ResultJson(#[from] serde_json::Error),

    #[error(transparent)]
    Config(#[from] AocConfigError),

    #[error("cannot create symlink for language file variant: {0:?}")]
    InvalidSymlinkTarget(SolverFile),

    #[error("Editing not allowed for language file: {0:?}")]
    FileEditNotAllowed(SolverFile),

    #[error("file not found: {0:?}")]
    FileNotFound(SolverFile),

    #[error("environment error: {0:?}")]
    Env(String),

    #[error("Dependency {0:?} could not be added: {1:?}")]
    DepAdd(String, String),

    #[error("Dependency {0:?} could not be removed: {1:?}")]
    DepRemove(String, String),

    #[error("Lib name not valid: {0:?}")]
    LibInvalid(String),

    #[error("clean error: {0:?}")]
    Clean(String),
}

pub type AocLanguageResult<T> = Result<T, AocLanguageError>;
pub type LanguageRunner = Box<dyn LanguageHandler>;
