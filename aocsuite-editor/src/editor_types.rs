use clap::ValueEnum;
use std::str::FromStr;
use which::which;

use crate::{
    arg_builder::{ArgsBuilder, GenericArgs, VimArgs},
    AocEditorError, AocEditorResult,
};

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
    pub fn to_args_builder(&self) -> ArgsBuilder {
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
