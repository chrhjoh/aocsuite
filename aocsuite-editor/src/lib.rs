mod arg_builder;
mod editor;
mod editor_types;

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use aocsuite_utils::{
    execute_command, CommandError, CommandExecutor, CommandRequest, SystemCommandExecutor,
};
use editor::Editor;
use editor_types::EditorCommand;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AocEditorError {
    #[error("error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Command(#[from] CommandError),

    #[error("editor {0} not implemented")]
    Invalid(String),

    #[error("cannot find editor {0}")]
    NotFound(String),

    #[error("editor {0} exited unexpectedly")]
    RunProgram(String),

    #[error("editor cannot use non-Unicode path {0}")]
    InvalidPath(PathBuf),
}

pub type AocEditorResult<T> = Result<T, AocEditorError>;

fn resolve_editor(command: &str) -> AocEditorResult<Editor> {
    Ok(Editor::new(EditorCommand::parse(command)?))
}
pub fn open_solution_files(
    editor: &str,
    puzzlefile: &Path,
    examplefile: &Path,
    libfile: &Path,
    inputfile: &Path,
    env_vars: Option<HashMap<OsString, OsString>>,
) -> AocEditorResult<()> {
    let editor = resolve_editor(editor)?;
    editor.open_solution(puzzlefile, examplefile, libfile, inputfile, env_vars)?;
    Ok(())
}
pub fn open(
    editor: &str,
    file: &Path,
    env_vars: Option<HashMap<OsString, OsString>>,
) -> AocEditorResult<()> {
    let editor = resolve_editor(editor)?;
    editor.open(file, env_vars)?;
    Ok(())
}

pub fn open_browser(url: &str) -> AocEditorResult<()> {
    open_browser_with(&SystemCommandExecutor, url)
}

fn open_browser_with(executor: &dyn CommandExecutor, url: &str) -> AocEditorResult<()> {
    #[cfg(target_os = "macos")]
    let request = CommandRequest::new("open").arg(url).foreground();

    #[cfg(target_os = "linux")]
    let request = CommandRequest::new("xdg-open").arg(url).foreground();

    #[cfg(target_os = "windows")]
    let request = CommandRequest::new("cmd")
        .args(["/C", "start", url])
        .foreground();

    execute_command(executor, request)?;
    Ok(())
}

#[cfg(test)]
mod browser_tests {
    use std::{io, process::Output};

    use aocsuite_utils::{CommandExecutor, CommandRequest, ProcessMode};

    use super::{open_browser_with, AocEditorError};

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
            open_browser_with(&FakeExecutor, "https://example.com"),
            Err(AocEditorError::Command(_))
        ));
    }
}
