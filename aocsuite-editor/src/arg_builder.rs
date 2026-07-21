use std::{ffi::OsString, path::Path};

use crate::{AocEditorError, AocEditorResult};

pub trait EditArgsBuilder {
    /// Opens the editor with the three files
    fn solution_command(
        &self,
        puzzlefile: &Path,
        examplefile: &Path,
        libfile: &Path,
        inputfile: &Path,
    ) -> AocEditorResult<Vec<OsString>>;
}

pub struct VimArgs {}

impl EditArgsBuilder for VimArgs {
    fn solution_command(
        &self,
        puzzlefile: &Path,
        examplefile: &Path,
        libfile: &Path,
        inputfile: &Path,
    ) -> AocEditorResult<Vec<OsString>> {
        let mut args = vec![
            libfile.as_os_str().to_owned(),
            inputfile.as_os_str().to_owned(),
        ];
        args.push(OsString::from(vim_command("vsplit", examplefile)?));
        args.push(OsString::from(vim_command("split", puzzlefile)?));
        Ok(args)
    }
}

pub struct GenericArgs {}

impl EditArgsBuilder for GenericArgs {
    fn solution_command(
        &self,
        puzzlefile: &Path,
        examplefile: &Path,
        libfile: &Path,
        inputfile: &Path,
    ) -> AocEditorResult<Vec<OsString>> {
        let files = [puzzlefile, examplefile, libfile, inputfile];
        Ok(files
            .iter()
            .map(|file| file.as_os_str().to_owned())
            .collect())
    }
}

pub type ArgsBuilder = Box<dyn EditArgsBuilder>;

fn vim_command(command: &str, path: &Path) -> AocEditorResult<String> {
    let path = path
        .to_str()
        .ok_or_else(|| AocEditorError::InvalidPath(path.to_path_buf()))?;
    // Vim's fnameescape handles spaces and Ex metacharacters; doubled quotes preserve literals.
    let path = path.replace('\\', "\\\\").replace('\'', "''");
    Ok(format!("+execute '{command}' fnameescape('{path}')"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{vim_command, EditArgsBuilder, GenericArgs};

    #[test]
    fn vim_commands_escape_special_paths() {
        assert_eq!(
            vim_command("vsplit", Path::new("example file|'quote'\\test")).expect("command"),
            "+execute 'vsplit' fnameescape('example file|''quote''\\\\test')"
        );
    }

    #[cfg(unix)]
    #[test]
    fn vim_commands_reject_non_unicode_paths_without_panicking() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let path = std::path::PathBuf::from(OsString::from_vec(b"invalid-\xff".to_vec()));
        assert!(vim_command("split", &path).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn generic_commands_preserve_non_unicode_paths() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let path = std::path::PathBuf::from(OsString::from_vec(b"input-\xff".to_vec()));
        let args = GenericArgs {}
            .solution_command(&path, &path, &path, &path)
            .expect("generic command");
        assert_eq!(args[0], path.into_os_string());
    }
}
