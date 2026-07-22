use std::{path::PathBuf, process::Output};

use aocsuite_utils::{atomic_write, ProcessExecutor, ProcessRequest};
use thiserror::Error;

const GITIGNORE: &str = r#"rust/target/
rust/src/solution.rs
python/venv/
python/solution.py
/.aocsuite-runs/
**/__pycache__/
*.pyc
"#;

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

    pub fn gitignore_path(&self) -> WorkspaceResult<PathBuf> {
        std::fs::create_dir_all(&self.directory)?;
        let path = self.directory.join(".gitignore");
        atomic_write(&path, GITIGNORE.as_bytes())?;
        Ok(path)
    }

    pub fn run_git(
        &self,
        args: &[String],
        mode: GitMode,
        executor: &dyn ProcessExecutor,
    ) -> WorkspaceResult<String> {
        if is_simple_clone(args)? {
            std::fs::create_dir_all(&self.directory)?;
            let mut clone_args = args.to_vec();
            clone_args.push(".".to_owned());
            return execute_git(executor, &self.directory, &clone_args, mode);
        }

        self.gitignore_path()?;
        execute_git(executor, &self.directory, args, mode)
    }
}

fn execute_git(
    executor: &dyn ProcessExecutor,
    directory: &PathBuf,
    args: &[String],
    mode: GitMode,
) -> WorkspaceResult<String> {
    let mut request = ProcessRequest::new("git").args(args).current_dir(directory);
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
    use super::{is_simple_clone, WorkspaceError, GITIGNORE};

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
}
