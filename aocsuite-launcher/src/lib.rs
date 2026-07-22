mod editor;

use std::path::PathBuf;

use aocsuite_utils::{execute_command, CommandError, CommandExecutor, CommandRequest};
use editor::Editor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AocLauncherError {
    #[error(transparent)]
    Command(#[from] CommandError),

    #[error("editor {0} not implemented")]
    Invalid(String),

    #[error("cannot find editor {0}")]
    NotFound(String),

    #[error("editor cannot use non-Unicode path {0}")]
    InvalidPath(PathBuf),
}

pub type AocLauncherResult<T> = Result<T, AocLauncherError>;

fn resolve_editor_program(program: impl Into<String>) -> AocLauncherResult<PathBuf> {
    let program: String = program.into();
    if program.trim().is_empty() {
        return Err(AocLauncherError::Invalid(program.to_owned()));
    }
    let path = PathBuf::from(program);
    which::which(&path).map_err(|_| AocLauncherError::NotFound(path.display().to_string()))
}

pub struct OpenPuzzleRequest {
    pub puzzle: PathBuf,
    pub example: PathBuf,
    pub solution: PathBuf,
    pub input: PathBuf,
    pub working_directory: PathBuf,
}

pub struct Launcher<'executor> {
    executor: &'executor dyn CommandExecutor,
}

impl<'executor> Launcher<'executor> {
    pub fn new(executor: &'executor dyn CommandExecutor) -> Self {
        Self { executor }
    }

    fn launch(&self, request: CommandRequest) -> AocLauncherResult<()> {
        execute_command(self.executor, request)?;
        Ok(())
    }

    pub fn open_browser(&self, url: &str) -> AocLauncherResult<()> {
        #[cfg(target_os = "macos")]
        let request = CommandRequest::new("open").arg(url).foreground();

        #[cfg(target_os = "linux")]
        let request = CommandRequest::new("xdg-open").arg(url).foreground();

        #[cfg(target_os = "windows")]
        let request = CommandRequest::new("cmd")
            .args(["/C", "start", url])
            .foreground();

        self.launch(request)
    }

    pub fn open_file(
        &self,
        editor_program: impl Into<String>,
        file: &std::path::Path,
        working_directory: &std::path::Path,
    ) -> AocLauncherResult<()> {
        let editor = self.editor(editor_program)?;
        self.launch(
            editor
                .command()
                .arg(file.as_os_str())
                .current_dir(working_directory),
        )
    }

    pub fn open_puzzle(
        &self,
        editor_program: impl Into<String>,
        request: OpenPuzzleRequest,
    ) -> AocLauncherResult<()> {
        let editor = self.editor(editor_program)?;
        self.launch(
            editor
                .command()
                .args(editor.open_puzzle_args(&request)?)
                .current_dir(request.working_directory),
        )
    }

    fn editor(&self, program: impl Into<String>) -> AocLauncherResult<Editor> {
        Ok(Editor::from_program(resolve_editor_program(program)?))
    }
}

#[cfg(test)]
mod browser_tests {
    use std::{io, process::Output};

    use aocsuite_utils::{CommandExecutor, CommandRequest, ProcessMode};

    use super::{AocLauncherError, Launcher};

    struct FakeExecutor;

    impl CommandExecutor for FakeExecutor {
        fn execute(&self, request: &CommandRequest) -> io::Result<Output> {
            assert_eq!(request.mode, ProcessMode::Foreground);
            Ok(failed_output())
        }
    }

    #[cfg(unix)]
    fn failed_output() -> Output {
        use std::os::unix::process::ExitStatusExt;

        Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[cfg(windows)]
    fn failed_output() -> Output {
        use std::os::windows::process::ExitStatusExt;

        Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn failed_browser_launch_returns_a_typed_error() {
        assert!(matches!(
            Launcher::new(&FakeExecutor).open_browser("https://example.com"),
            Err(AocLauncherError::Command(_))
        ));
    }
}
