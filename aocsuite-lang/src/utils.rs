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
use aocsuite_utils::{PuzzleDay, PuzzleYear, RuntimeDirError};
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
static LINK_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
        symlink_file_with(from, to, |source, destination| {
            std::os::unix::fs::symlink(source, destination)
        })?;
    }

    #[cfg(windows)]
    {
        symlink_file_with(from, to, |source, destination| {
            std::os::windows::fs::symlink_file(source, destination)
        })?;
    }

    Ok(())
}

fn symlink_file_with(
    from: &Path,
    to: &Path,
    create_link: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> AocLanguageResult<()> {
    let destination_exists = match fs::symlink_metadata(to) {
        Ok(metadata) if metadata.file_type().is_symlink() => true,
        Ok(_) => return Err(AocLanguageError::ActiveSolutionNotLink(to.to_path_buf())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };

    let temporary_link = temporary_link_path(to)?;
    create_link(from, &temporary_link)?;

    if let Err(error) = replace_link(&temporary_link, to, destination_exists) {
        let _ = fs::remove_file(&temporary_link);
        return Err(error.into());
    }

    Ok(())
}

fn temporary_link_path(destination: &Path) -> std::io::Result<PathBuf> {
    let parent = destination.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "active solution path has no parent directory",
        )
    })?;
    let name = destination.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "active solution path has no file name",
        )
    })?;

    for _ in 0..16 {
        let sequence = LINK_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{}-{}-{sequence}.tmp",
            name.to_string_lossy(),
            std::process::id()
        ));
        match fs::symlink_metadata(&temporary) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(temporary),
            Ok(_) => continue,
            Err(error) => return Err(error),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate a temporary active solution link",
    ))
}

#[cfg(unix)]
fn replace_link(
    temporary_link: &Path,
    destination: &Path,
    _destination_exists: bool,
) -> std::io::Result<()> {
    fs::rename(temporary_link, destination)
}

#[cfg(windows)]
fn replace_link(
    temporary_link: &Path,
    destination: &Path,
    destination_exists: bool,
) -> std::io::Result<()> {
    if !destination_exists {
        return fs::rename(temporary_link, destination);
    }

    let backup_link = temporary_link_path(destination)?;
    fs::rename(destination, &backup_link)?;

    match fs::rename(temporary_link, destination) {
        Ok(()) => fs::remove_file(backup_link),
        Err(error) => {
            let _ = fs::rename(&backup_link, destination);
            Err(error)
        }
    }
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

    #[error(transparent)]
    RuntimeDir(#[from] RuntimeDirError),

    #[error("cannot create symlink for language file variant: {0:?}")]
    InvalidSymlinkTarget(SolverFile),

    #[error("refusing to replace a non-symlink active solution: {0}")]
    ActiveSolutionNotLink(PathBuf),

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

    #[error("runtime migration failed: {0}")]
    RuntimeMigration(String),

    #[error("clean error: {0:?}")]
    Clean(String),
}

pub type AocLanguageResult<T> = Result<T, AocLanguageError>;
pub type LanguageRunner = Box<dyn LanguageHandler>;

#[cfg(test)]
mod tests {
    use std::{
        fs, io,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{symlink_file, symlink_file_with, AocLanguageError};

    fn test_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("aocsuite-links-{}-{unique}", process::id()))
    }

    #[cfg(unix)]
    #[test]
    fn failed_link_creation_preserves_the_active_solution() {
        let root = test_root();
        fs::create_dir_all(&root).expect("create test runtime");
        let first_solution = root.join("first.rs");
        let active_solution = root.join("solution.rs");
        fs::write(&first_solution, "first solution").expect("write first solution");
        symlink_file(&first_solution, &active_solution).expect("create active solution");

        let result = symlink_file_with(&root.join("second.rs"), &active_solution, |_, _| {
            Err(io::Error::new(io::ErrorKind::Other, "create link failed"))
        });

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(&active_solution).expect("read preserved active solution"),
            "first solution"
        );

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[cfg(unix)]
    #[test]
    fn active_solution_does_not_replace_a_regular_file() {
        let root = test_root();
        fs::create_dir_all(&root).expect("create test runtime");
        let source = root.join("source.rs");
        let destination = root.join("solution.rs");
        fs::write(&source, "source solution").expect("write source solution");
        fs::write(&destination, "user file").expect("write user file");

        let result = symlink_file(&source, &destination);

        assert!(matches!(
            result,
            Err(AocLanguageError::ActiveSolutionNotLink(_))
        ));
        assert_eq!(
            fs::read_to_string(&destination).expect("read preserved user file"),
            "user file"
        );

        fs::remove_dir_all(root).expect("remove test runtime");
    }
}
