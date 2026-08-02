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
        std::fs::create_dir_all(&self.directory).map_err(|source| WorkspaceError::Ensure {
            path: self.directory.clone(),
            source,
        })?;
        Ok(())
    }

    pub fn ensure_git(&self, executor: &dyn CommandExecutor) -> WorkspaceResult<()> {
        self.ensure()?;
        let gitignore_path = self.gitignore_path();
        if !gitignore_path.exists() {
            atomic_write(&gitignore_path, GITIGNORE.as_bytes()).map_err(|source| {
                WorkspaceError::Gitignore {
                    path: gitignore_path,
                    source,
                }
            })?;
        }
        if !self.directory.join(".git").exists() {
            execute_git(
                executor,
                &self.directory,
                &["init".to_owned()],
                GitMode::Captured,
            )?;
        }
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
            self.ensure()?;
            let mut clone_args = args.to_vec();
            clone_args.push(".".to_owned());
            let output = execute_git(executor, &self.directory, &clone_args, mode)?;
            let gitignore_path = self.gitignore_path();
            if !gitignore_path.exists() {
                atomic_write(&gitignore_path, GITIGNORE.as_bytes()).map_err(|source| {
                    WorkspaceError::Gitignore {
                        path: gitignore_path,
                        source,
                    }
                })?;
            }
            return Ok(output);
        }

        self.ensure_git(executor)?;
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
    let output = executor
        .execute(&request)
        .map_err(|source| WorkspaceError::GitLaunch {
            args: args.to_vec(),
            current_dir: directory.clone(),
            source,
        })?;
    git_output(output, args, directory)
}

fn git_output(output: Output, args: &[String], directory: &Path) -> WorkspaceResult<String> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let code = output.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Err(WorkspaceError::GitFailed {
        args: args.to_vec(),
        current_dir: directory.to_path_buf(),
        code,
        stderr,
    })
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
    #[error("could not launch `git {}` in '{}': {source}", args.join(" "), current_dir.display())]
    GitLaunch {
        args: Vec<String>,
        current_dir: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("`git {}` in '{}' exited with code {code}: {stderr}", args.join(" "), current_dir.display())]
    GitFailed {
        args: Vec<String>,
        current_dir: PathBuf,
        code: i32,
        stderr: String,
    },
    #[error("only clone in format `git clone my_git_repo` is supported")]
    Clone,
    #[error("could not create workspace directory at '{path}': {source}")]
    Ensure {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not create workspace Git ignore file at '{path}': {source}")]
    Gitignore {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[cfg(test)]
mod tests {
    use std::{io, process::Output, sync::Mutex};

    use super::{is_simple_clone, GitMode, Workspace, WorkspaceError, GITIGNORE};
    use aocsuite_utils::{CommandExecutor, CommandRequest, LanguageId, ProcessMode};

    #[derive(Default)]
    struct RecordingExecutor {
        requests: Mutex<Vec<CommandRequest>>,
        fail: bool,
    }

    impl CommandExecutor for RecordingExecutor {
        fn execute(&self, request: &CommandRequest) -> io::Result<Output> {
            self.requests.lock().unwrap().push(request.clone());
            Ok(Output {
                status: status(!self.fail),
                stdout: Vec::new(),
                stderr: if self.fail {
                    b"failed init".to_vec()
                } else {
                    Vec::new()
                },
            })
        }
    }

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
    fn workspace_ensure_is_filesystem_only() {
        let temp = tempfile::tempdir().expect("create temporary workspace");
        let directory = temp.path().join("workspace");
        let workspace = Workspace::new(directory.clone());

        workspace.ensure().expect("ensure workspace");

        assert!(directory.is_dir());
        assert!(!workspace.gitignore_path().exists());
        assert!(!directory.join(".git").exists());
    }

    #[test]
    fn ensure_git_initializes_captured_noninteractive_git_once_and_preserves_gitignore() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(temp.path().join("workspace"));
        workspace.ensure().unwrap();
        std::fs::write(workspace.gitignore_path(), "user rule\n").unwrap();
        let executor = RecordingExecutor::default();

        workspace.ensure_git(&executor).unwrap();

        assert_eq!(
            std::fs::read_to_string(workspace.gitignore_path()).unwrap(),
            "user rule\n"
        );
        let requests = executor.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].program, "git");
        assert_eq!(requests[0].args, ["init"]);
        assert_eq!(
            requests[0].current_dir.as_deref(),
            Some(workspace.root_dir())
        );
        assert_eq!(requests[0].mode, ProcessMode::Captured);
        assert_eq!(
            requests[0].environment,
            [
                ("GIT_PAGER".into(), "cat".into()),
                ("GIT_TERMINAL_PROMPT".into(), "0".into())
            ]
        );
    }

    #[test]
    fn ensure_git_accepts_existing_git_file_and_reports_init_failure_typedly() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(temp.path().join("workspace"));
        workspace.ensure().unwrap();
        std::fs::write(workspace.root_dir().join(".git"), "gitdir: elsewhere").unwrap();
        let executor = RecordingExecutor::default();
        workspace.ensure_git(&executor).unwrap();
        assert!(executor.requests.lock().unwrap().is_empty());

        std::fs::remove_file(workspace.root_dir().join(".git")).unwrap();
        let failed = RecordingExecutor {
            fail: true,
            ..Default::default()
        };
        assert!(matches!(
            workspace.ensure_git(&failed),
            Err(WorkspaceError::GitFailed {
                ref args,
                ref current_dir,
                code: 1,
                ref stderr,
            }) if args == &["init"]
                && current_dir == workspace.root_dir()
                && stderr == "failed init"
        ));
    }

    #[test]
    fn git_launch_errors_preserve_command_and_workspace_context() {
        struct MissingGit;
        impl CommandExecutor for MissingGit {
            fn execute(&self, _: &CommandRequest) -> io::Result<Output> {
                Err(io::Error::new(io::ErrorKind::NotFound, "git missing"))
            }
        }

        let temp = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(temp.path().join("workspace"));
        let result = workspace.run_git(&["status".into()], GitMode::Captured, &MissingGit);

        assert!(matches!(
            result,
            Err(WorkspaceError::GitLaunch {
                ref args,
                ref current_dir,
                ref source,
            }) if args == &["init"]
                && current_dir == workspace.root_dir()
                && source.kind() == io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn clone_bypasses_init_and_adds_gitignore_only_after_success() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(temp.path().join("workspace"));
        let executor = RecordingExecutor::default();
        workspace
            .run_git(
                &["clone".into(), "repository".into()],
                GitMode::Captured,
                &executor,
            )
            .unwrap();
        assert!(workspace.gitignore_path().is_file());
        let requests = executor.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].args, ["clone", "repository", "."]);

        let other = Workspace::new(temp.path().join("failed"));
        let failed = RecordingExecutor {
            fail: true,
            ..Default::default()
        };
        assert!(other
            .run_git(
                &["clone".into(), "repository".into()],
                GitMode::Captured,
                &failed
            )
            .is_err());
        assert!(!other.gitignore_path().exists());
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

    #[cfg(unix)]
    fn status(success: bool) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(if success { 0 } else { 1 << 8 })
    }

    #[cfg(windows)]
    fn status(success: bool) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(if success { 0 } else { 1 })
    }
}
