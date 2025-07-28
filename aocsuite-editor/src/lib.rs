mod arg_builder;
mod editor;
mod editor_types;

use std::path::Path;

use aocsuite_config::{get_config_val, ConfigOpt};
use editor::Editor;
use editor_types::EditorType;
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

    #[error("editor {0} exited unexpectedly")]
    RunProgram(String),
}

pub type AocEditorResult<T> = Result<T, AocEditorError>;

fn resolve_editor_type() -> AocEditorResult<EditorType> {
    let editor_type = get_config_val(&ConfigOpt::Editor, None, None);
    match editor_type {
        Ok(t) => Ok(t),
        Err(_) => {
            let program = std::env::var("EDITOR")?;
            Ok(program.parse()?)
        }
    }
}
pub fn open_solution_files(
    puzzlefile: &Path,
    examplefile: &Path,
    libfile: &Path,
    inputfile: &Path,
) -> AocEditorResult<()> {
    let editor_type = resolve_editor_type()?;
    let editor = Editor::new(&editor_type)?;
    editor.open_solution(
        puzzlefile.to_str().unwrap(),
        examplefile.to_str().unwrap(),
        libfile.to_str().unwrap(),
        inputfile.to_str().unwrap(),
    )?;
    Ok(())
}
pub fn open(file: &Path) -> AocEditorResult<()> {
    let editor_type = resolve_editor_type()?;
    let editor = Editor::new(&editor_type)?;
    editor.open(file.to_str().unwrap())?;
    Ok(())
}
