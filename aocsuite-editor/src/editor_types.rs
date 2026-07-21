use std::{ffi::OsString, path::PathBuf};

use crate::{
    arg_builder::{ArgsBuilder, GenericArgs, VimArgs},
    AocEditorError, AocEditorResult,
};

#[derive(Debug, Clone, Copy)]
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
    pub fn to_args_builder(self) -> ArgsBuilder {
        match self {
            EditorType::Neovim | EditorType::Vim => Box::new(VimArgs {}),
            _ => Box::new(GenericArgs {}),
        }
    }
    fn program(&self) -> &'static str {
        match self {
            EditorType::Neovim => "nvim",
            EditorType::Vim => "vim",
            EditorType::Code => "code",
            EditorType::Helix => "hx",
            EditorType::Emacs => "emacs",
            EditorType::Gedit => "gedit",
            EditorType::Nano => "nano",
            EditorType::Sublime => "subl",
        }
    }
}

pub struct EditorCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub editor_type: Option<EditorType>,
}

impl EditorCommand {
    pub fn parse(command: &str) -> AocEditorResult<Self> {
        let parts =
            shell_words::split(command).map_err(|_| AocEditorError::Invalid(command.into()))?;
        let (program, args) = parts
            .split_first()
            .ok_or_else(|| AocEditorError::Invalid(command.into()))?;
        let editor_type = match program
            .rsplit('/')
            .next()
            .unwrap_or(program)
            .to_lowercase()
            .as_str()
        {
            "nvim" | "neovim" => Ok(EditorType::Neovim),
            "vim" => Ok(EditorType::Vim),
            "code" => Ok(EditorType::Code),
            "hx" | "helix" => Ok(EditorType::Helix),
            "emacs" => Ok(EditorType::Emacs),
            "gedit" => Ok(EditorType::Gedit),
            "nano" => Ok(EditorType::Nano),
            "subl" | "sublime" => Ok(EditorType::Sublime),
            _ => Err(()),
        }
        .ok();
        let program = match program.to_lowercase().as_str() {
            "neovim" => PathBuf::from(EditorType::Neovim.program()),
            "helix" => PathBuf::from(EditorType::Helix.program()),
            "sublime" => PathBuf::from(EditorType::Sublime.program()),
            _ => PathBuf::from(program),
        };
        let program = which::which(&program)
            .map_err(|_| AocEditorError::NotFound(program.display().to_string()))?;
        Ok(Self {
            program,
            args: args.iter().map(OsString::from).collect(),
            editor_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{EditorCommand, EditorType};

    static PATH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[cfg(unix)]
    #[test]
    fn aliases_are_resolved_after_translation() {
        use std::os::unix::fs::PermissionsExt;

        let _lock = PATH_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock PATH");
        let dir = env::temp_dir().join(format!(
            "aocsuite-editor-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir(&dir).expect("create test directory");
        for program in ["nvim", "hx", "editor"] {
            let path = dir.join(program);
            fs::write(&path, "#!/bin/sh\n").expect("write editor");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make executable");
        }
        let previous_path = env::var_os("PATH");
        env::set_var("PATH", &dir);

        let neovim = EditorCommand::parse("neovim").expect("resolve neovim alias");
        let helix = EditorCommand::parse("helix").expect("resolve helix alias");
        let custom = EditorCommand::parse(&format!("{} --wait", dir.join("editor").display()))
            .expect("parse command path and arguments");

        assert!(matches!(neovim.editor_type, Some(EditorType::Neovim)));
        assert!(matches!(helix.editor_type, Some(EditorType::Helix)));
        assert_eq!(custom.program, dir.join("editor"));
        assert_eq!(custom.args, ["--wait"]);

        match previous_path {
            Some(path) => env::set_var("PATH", path),
            None => env::remove_var("PATH"),
        }
        fs::remove_dir_all(dir).expect("remove test directory");
    }
}
