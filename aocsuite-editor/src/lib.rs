mod arg_builder;
mod editor;
mod editor_types;

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use aocsuite_config::{get_config_val, AocConfigError, ConfigOpt};
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
