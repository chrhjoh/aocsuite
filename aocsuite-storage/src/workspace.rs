use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::Output,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use aocsuite_utils::{atomic_write, CommandExecutor, CommandRequest, LanguageId, PuzzleId};
use thiserror::Error;

const GITIGNORE: &str = r#"rust/target/
rust/src/solution.rs
python/venv/
python/solution.py
/.aocsuite-runs/
**/__pycache__/
*.pyc
"#;

static RESULT_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct Workspace {
    directory: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitMode {
    Captured,
    Foreground,
}

impl Workspace {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn root_dir(&self) -> &Path {
        &self.directory
    }

    pub fn gitignore_path(&self) -> PathBuf {
        self.directory.join(".gitignore")
    }

    pub fn ensure(&self) -> WorkspaceResult<()> {
        std::fs::create_dir_all(&self.directory)?;
        atomic_write(&self.gitignore_path(), GITIGNORE.as_bytes())?;
        Ok(())
    }

    pub fn language_project_dir(&self, language: LanguageId) -> PathBuf {
        self.directory.join(language.to_string())
    }

    pub fn allocate_run_result_file(&self) -> WorkspaceResult<PathBuf> {
        let runs_dir = self.directory.join(".aocsuite-runs");
        fs::create_dir_all(&runs_dir)?;

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

    pub fn ensure_example(&self, puzzle: PuzzleId) -> WorkspaceResult<PathBuf> {
        let examples_dir = self.directory.join("examples");
        let path = examples_dir.join(format!("{puzzle}.txt"));
        fs::create_dir_all(&examples_dir)?;
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && path.is_file() => {
                Ok(path)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn run_git(
        &self,
        args: &[String],
        mode: GitMode,
        executor: &dyn CommandExecutor,
    ) -> WorkspaceResult<String> {
        if is_simple_clone(args)? {
            std::fs::create_dir_all(&self.directory)?;
            let mut clone_args = args.to_vec();
            clone_args.push(".".to_owned());
            return execute_git(executor, &self.directory, &clone_args, mode);
        }

        self.ensure()?;
        execute_git(executor, &self.directory, args, mode)
    }
}

fn execute_git(
    executor: &dyn CommandExecutor,
    directory: &PathBuf,
    args: &[String],
    mode: GitMode,
) -> WorkspaceResult<String> {
    let mut request = CommandRequest::new("git").args(args).current_dir(directory);
    if mode == GitMode::Captured {
        request = request
            .env("GIT_PAGER", "cat")
            .env("GIT_TERMINAL_PROMPT", "0");
    } else {
        request = request.foreground();
    }
    let output = executor.execute(&request)?;
    git_output(output, mode)
}

fn git_output(output: Output, mode: GitMode) -> WorkspaceResult<String> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let code = output.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if mode == GitMode::Captured && stderr.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !stdout.is_empty() {
            return Ok(stdout);
        }
    }
    Err(WorkspaceError::CommandFailed { code, stderr })
}

fn is_simple_clone(args: &[String]) -> WorkspaceResult<bool> {
    if !matches!(args.first(), Some(arg) if arg == "clone") {
        return Ok(false);
    }
    if args.len() != 2 {
        return Err(WorkspaceError::Clone);
    }
    Ok(true)
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Git command exited with code {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },
    #[error("only clone in format `git clone my_git_repo` is supported")]
    Clone,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[cfg(test)]
mod tests {
    use super::{is_simple_clone, Workspace, WorkspaceError, GITIGNORE};
    use aocsuite_utils::LanguageId;

    #[test]
    fn empty_args_are_not_a_clone() {
        assert!(!is_simple_clone(&[]).expect("empty args are valid"));
    }

    #[test]
    fn clone_requires_exactly_one_repository_argument() {
        assert!(
            is_simple_clone(&["clone".to_string(), "repository".to_string()])
                .expect("simple clone is accepted")
        );
        assert!(matches!(
            is_simple_clone(&["clone".to_string()]),
            Err(WorkspaceError::Clone)
        ));
    }

    #[test]
    fn workspace_gitignore_preserves_tracked_project_files() {
        assert!(GITIGNORE.contains("rust/target/"));
        assert!(GITIGNORE.contains("python/venv/"));
        assert!(!GITIGNORE.contains("Cargo.lock"));
        assert!(!GITIGNORE.contains("config.json"));
    }

    #[test]
    fn workspace_allocates_unique_result_files_in_its_ignored_runs_directory() {
        let temp = tempfile::tempdir().expect("create temporary workspace");
        let workspace = Workspace::new(temp.path().to_path_buf());

        assert_eq!(
            workspace.language_project_dir(LanguageId::Rust),
            temp.path().join("rust")
        );

        let first = workspace
            .allocate_run_result_file()
            .expect("allocate first result path");
        let second = workspace
            .allocate_run_result_file()
            .expect("allocate second result path");
        let runs_dir = temp.path().join(".aocsuite-runs");

        assert_ne!(first, second);
        assert_eq!(first.parent(), Some(runs_dir.as_path()));
        assert!(first.parent().expect("result parent").is_dir());
        assert!(!first.exists());
    }
}
