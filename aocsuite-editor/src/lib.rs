mod arg_builder;
mod editor;
mod editor_types;

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use aocsuite_config::{get_config_val, AocConfigError, ConfigOpt};
use aocsuite_utils::{ProcessExecutor, ProcessRequest, SystemProcessExecutor};
use editor::Editor;
use editor_types::EditorCommand;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AocEditorError {
    #[error("error: {0}")]
    Io(#[from] std::io::Error),

    #[error("editor {0} not implemented")]
    Invalid(String),

    #[error("cannot find editor {0}")]
    NotFound(String),

    #[error(transparent)]
    Var(#[from] std::env::VarError),

    #[error(transparent)]
    Config(#[from] AocConfigError),

    #[error("editor {0} exited unexpectedly")]
    RunProgram(String),

    #[error("editor cannot use non-Unicode path {0}")]
    InvalidPath(PathBuf),
}

pub type AocEditorResult<T> = Result<T, AocEditorError>;

fn resolve_editor() -> AocEditorResult<Editor> {
    let editor_command = get_config_val(&ConfigOpt::Editor, None, None);
    let command = match editor_command {
        Ok(command) => command,
        Err(AocConfigError::NotFound { .. }) => std::env::var("EDITOR")?,
        Err(error) => return Err(error.into()),
    };
    Ok(Editor::new(EditorCommand::parse(&command)?))
}
pub fn open_solution_files(
    puzzlefile: &Path,
    examplefile: &Path,
    libfile: &Path,
    inputfile: &Path,
    env_vars: Option<HashMap<OsString, OsString>>,
) -> AocEditorResult<()> {
    let editor = resolve_editor()?;
    editor.open_solution(puzzlefile, examplefile, libfile, inputfile, env_vars)?;
    Ok(())
}
pub fn open(file: &Path, env_vars: Option<HashMap<OsString, OsString>>) -> AocEditorResult<()> {
    let editor = resolve_editor()?;
    editor.open(file, env_vars)?;
    Ok(())
}

pub fn open_browser(url: &str) -> AocEditorResult<()> {
    open_browser_with(&SystemProcessExecutor, url)
}

fn open_browser_with(executor: &dyn ProcessExecutor, url: &str) -> AocEditorResult<()> {
    #[cfg(target_os = "macos")]
    let request = ProcessRequest::new("open").arg(url).foreground();

    #[cfg(target_os = "linux")]
    let request = ProcessRequest::new("xdg-open").arg(url).foreground();

    #[cfg(target_os = "windows")]
    let request = ProcessRequest::new("cmd")
        .args(["/C", "start", url])
        .foreground();

    let output = executor.execute(&request)?;
    if !output.status.success() {
        return Err(AocEditorError::RunProgram(
            request.program.to_string_lossy().into_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod browser_tests {
    use std::{io, process::Output};

    use aocsuite_utils::{ProcessExecutor, ProcessMode, ProcessRequest};

    use super::{open_browser_with, AocEditorError};

    struct FakeExecutor;

    impl ProcessExecutor for FakeExecutor {
        fn execute(&self, request: &ProcessRequest) -> io::Result<Output> {
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
            Err(AocEditorError::RunProgram(_))
        ));
    }
}
