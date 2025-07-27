use thiserror::Error;

trait Editor {
    /// Opens the editor with the three files
    fn open(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> std::io::Result<()>;
}

struct NeovimEditor {
    command: String,
}

impl Editor for NeovimEditor {
    fn open(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> std::io::Result<()> {
        use std::process::Command;

        let status = Command::new(&self.command)
            .arg(libfile)
            .arg(inputfile)
            .arg(format!("+vsplit {}", examplefile))
            .arg(format!("+split {}", puzzlefile))
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Neovim exited with error",
            ))
        }
    }
}

struct GenericEditor {
    command: String,
}

impl Editor for GenericEditor {
    fn open(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> std::io::Result<()> {
        let mut parts = self.command.split_whitespace();
        let cmd = parts.next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Empty editor command")
        })?;

        let mut command = std::process::Command::new(cmd);
        for arg in parts {
            command.arg(arg);
        }

        command
            .arg(puzzlefile)
            .arg(examplefile)
            .arg(libfile)
            .arg(inputfile)
            .status()
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Editor exited with error",
                    ))
                }
            })
    }
}

pub fn edit_files(
    editor_name: &str,
    puzzlefile: &str,
    examplefile: &str,
    libfile: &str,
    inputfile: &str,
) -> AocEditorResult<()> {
    // ensure_files_exist(vec![puzzlefile, libfile, inputfile])?; //example file not required to exist
    let editor: Box<dyn Editor> = match editor_name.to_lowercase().as_str() {
        "nvim" | "vi" => Box::new(NeovimEditor {
            command: editor_name.to_string(),
        }),
        _ => Box::new(GenericEditor {
            command: editor_name.to_string(),
        }),
    };

    editor.open(puzzlefile, examplefile, libfile, inputfile)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum AocEditorError {
    #[error("editor io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AocEditorResult<T> = Result<T, AocEditorError>;
