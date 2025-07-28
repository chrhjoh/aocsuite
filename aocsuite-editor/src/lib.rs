use std::{path::Path, process::Command, str::FromStr};

use aocsuite_config::{get_config_val, ConfigOpt};
use clap::ValueEnum;
use thiserror::Error;
use which::which;

trait EditArgsBuilder {
    /// Opens the editor with the three files
    fn solution_command(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> Vec<String>;
    fn files_command(&self, files: Vec<&str>) -> Vec<String>;
}

struct VimArgs {}

impl EditArgsBuilder for VimArgs {
    fn solution_command(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> Vec<String> {
        let mut args = Vec::new();
        args.push(libfile.to_string());
        args.push(inputfile.to_string());
        args.push(format!("+vsplit {}", examplefile));
        args.push(format!("+split {}", puzzlefile));
        args
    }
    fn files_command(&self, files: Vec<&str>) -> Vec<String> {
        files.iter().map(|file| file.to_string()).collect()
    }
}

struct GenericArgs {}

impl EditArgsBuilder for GenericArgs {
    fn solution_command(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> Vec<String> {
        self.files_command(vec![puzzlefile, examplefile, libfile, inputfile])
    }
    fn files_command(&self, files: Vec<&str>) -> Vec<String> {
        files.iter().map(|file| file.to_string()).collect()
    }
}

pub fn open_solution_files(
    puzzlefile: &Path,
    examplefile: &Path,
    libfile: &Path,
    inputfile: &Path,
) -> AocEditorResult<()> {
    let editor_type = resolve_editor_type()?;
    let editor = EditorCommand::new(&editor_type)?;
    editor.open_solution(
        puzzlefile.to_str().unwrap(),
        examplefile.to_str().unwrap(),
        libfile.to_str().unwrap(),
        inputfile.to_str().unwrap(),
    )?;
    Ok(())
}

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

type ArgsBuilder = Box<dyn EditArgsBuilder>;

#[derive(Debug, Clone, ValueEnum)]
pub enum EditorType {
    Neovim,
    Vim,
    Code,
    Helix,
    Emacs,
    Gedit,
    Nano,
    Sublime,
}

impl EditorType {
    fn to_args_builder(&self) -> ArgsBuilder {
        match self {
            EditorType::Neovim | EditorType::Vim => Box::new(VimArgs {}),
            _ => Box::new(GenericArgs {}),
        }
    }
    pub fn to_program(&self) -> AocEditorResult<String> {
        let program = match self {
            EditorType::Neovim => "nvim",
            EditorType::Vim => "vim",
            EditorType::Code => "code",
            EditorType::Helix => "hx",
            EditorType::Emacs => "emacs",
            EditorType::Gedit => "gedit",
            EditorType::Nano => "nano",
            EditorType::Sublime => "subl",
        };

        if which(program).is_ok() {
            Ok(program.to_string())
        } else {
            Err(AocEditorError::NotFound(program.to_string()))
        }
    }
}

impl FromStr for EditorType {
    type Err = AocEditorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if which(s).is_err() {
            return Err(AocEditorError::NotFound(s.to_string()));
        }
        match s.to_lowercase().as_str() {
            "nvim" | "neovim" => Ok(EditorType::Neovim),
            "vim" => Ok(EditorType::Vim),
            "code" => Ok(EditorType::Code),
            "hx" | "helix" => Ok(EditorType::Helix),
            "emacs" => Ok(EditorType::Emacs),
            "gedit" => Ok(EditorType::Gedit),
            "nano" => Ok(EditorType::Nano),
            "subl" | "sublime" => Ok(EditorType::Sublime),
            other => Err(AocEditorError::Invalid(other.to_string())),
        }
    }
}

struct EditorCommand {
    program: String,
    args_builder: ArgsBuilder,
}

impl EditorCommand {
    fn new(editor_type: &EditorType) -> AocEditorResult<Self> {
        let program = editor_type.to_program()?;
        let args_builder = editor_type.to_args_builder();
        Ok(EditorCommand {
            program,
            args_builder,
        })
    }
    fn run(&self, args: Vec<String>) -> AocEditorResult<()> {
        let mut command = Command::new(&self.program);
        command.args(args);

        let status = command.status()?;
        if !status.success() {
            return Err(AocEditorError::RunProgram(self.program.clone()));
        }
        Ok(())
    }

    pub fn open_solution(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> AocEditorResult<()> {
        let args = self
            .args_builder
            .solution_command(puzzlefile, examplefile, libfile, inputfile);
        self.run(args)?;
        Ok(())
    }
}

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
