pub trait Editor {
    /// Opens the editor with the three files
    fn open(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> std::io::Result<()>;
}

pub struct NeovimEditor;

impl Editor for NeovimEditor {
    fn open(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> std::io::Result<()> {
        use std::process::Command;

        let status = Command::new("nvim")
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

pub struct GenericEditor {
    pub command: String,
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

pub fn get_editor(name: &str) -> Box<dyn Editor> {
    match name.to_lowercase().as_str() {
        "nvim" | "neovim" => Box::new(NeovimEditor),
        _ => Box::new(GenericEditor {
            command: name.to_string(),
        }),
    }
}
