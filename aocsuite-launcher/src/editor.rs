use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use aocsuite_utils::CommandRequest;

use crate::{AocLauncherError, AocLauncherResult, OpenPuzzleRequest};

#[derive(Debug)]
pub(crate) enum Editor {
    Neovim(PathBuf),
    Vim(PathBuf),
    Code(PathBuf),
    Helix(PathBuf),
    Emacs(PathBuf),
    Gedit(PathBuf),
    Nano(PathBuf),
    Sublime(PathBuf),
    Generic(PathBuf),
}

impl Editor {
    pub(crate) fn from_program(program: PathBuf) -> Self {
        match program
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase())
            .as_deref()
        {
            Some("nvim" | "neovim") => Self::Neovim(program),
            Some("vim") => Self::Vim(program),
            Some("code") => Self::Code(program),
            Some("hx" | "helix") => Self::Helix(program),
            Some("emacs") => Self::Emacs(program),
            Some("gedit") => Self::Gedit(program),
            Some("nano") => Self::Nano(program),
            Some("subl" | "sublime") => Self::Sublime(program),
            _ => Self::Generic(program),
        }
    }

    pub(crate) fn command(&self) -> CommandRequest {
        CommandRequest::new(self.program()).foreground()
    }

    pub(crate) fn open_puzzle_args(
        &self,
        request: &OpenPuzzleRequest,
    ) -> AocLauncherResult<Vec<OsString>> {
        match self {
            Self::Neovim(_) | Self::Vim(_) => Ok(vec![
                request.solution.as_os_str().to_owned(),
                request.input.as_os_str().to_owned(),
                OsString::from(vim_command("vsplit", &request.example)?),
                OsString::from(vim_command("split", &request.puzzle)?),
            ]),
            _ => Ok([
                &request.puzzle,
                &request.example,
                &request.solution,
                &request.input,
            ]
            .into_iter()
            .map(|path| path.as_os_str().to_owned())
            .collect()),
        }
    }

    fn program(&self) -> &Path {
        match self {
            Self::Neovim(program)
            | Self::Vim(program)
            | Self::Code(program)
            | Self::Helix(program)
            | Self::Emacs(program)
            | Self::Gedit(program)
            | Self::Nano(program)
            | Self::Sublime(program)
            | Self::Generic(program) => program,
        }
    }
}

fn vim_command(command: &str, path: &Path) -> AocLauncherResult<String> {
    let path = path
        .to_str()
        .ok_or_else(|| AocLauncherError::InvalidPath(path.to_path_buf()))?;
    let path = path.replace('\\', "\\\\").replace('\'', "''");
    Ok(format!("+execute '{command}' fnameescape('{path}')"))
}
