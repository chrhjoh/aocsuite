use crate::AocLanguageError;
use crate::{
    traits::{DepManager, Solver},
    AocLanguageResult,
};
use std::process::Command;
use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use super::PythonRunner;

impl DepManager for PythonRunner {
    fn setup_env(&self) -> AocLanguageResult<()> {
        self.migrate_runtime()?;
        let venv_path = self.root_dir.join("venv");

        if !venv_path.exists() {
            // Create virtual environment
            let output = Command::new("python3")
                .arg("-m")
                .arg("venv")
                .arg("venv")
                .current_dir(&self.root_dir)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(AocLanguageError::Env(error.into()));
            }
        }

        Ok(())
    }
    fn clean_env(&self) -> AocLanguageResult<()> {
        std::fs::remove_dir_all(self.root_dir.join("venv"))?;
        Ok(())
    }

    fn add_package(&self, package: &str) -> AocLanguageResult<()> {
        let pip_path = self.get_pip_path();

        let output = Command::new(pip_path)
            .arg("install")
            .arg(package)
            .current_dir(&self.root_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AocLanguageError::DepAdd(package.into(), error.into()));
        }

        Ok(())
    }

    fn list_packages(&self) -> AocLanguageResult<Vec<String>> {
        let venv_path = self.root_dir.join("venv");
        if !venv_path.exists() {
            return Ok(Vec::new());
        }

        let pip_path = self.get_pip_path();

        let output = Command::new(pip_path)
            .arg("list")
            .arg("--format=freeze")
            .current_dir(&self.root_dir)
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                if line.trim().is_empty() || line.starts_with('#') {
                    None
                } else {
                    line.split("==").next().map(|name| name.to_string())
                }
            })
            .collect();

        Ok(packages)
    }

    fn remove_packages(&self, package: &str) -> AocLanguageResult<()> {
        let pip_path = self.get_pip_path();

        let output = Command::new(pip_path)
            .arg("uninstall")
            .arg("-y") // Auto-confirm removal
            .arg(package)
            .current_dir(&self.root_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AocLanguageError::DepRemove(package.into(), error.into()));
        }

        Ok(())
    }
    fn editor_environment_vars(&self) -> AocLanguageResult<HashMap<OsString, OsString>> {
        let mut vars = HashMap::new();
        let python_dir = self
            .get_python_path()
            .parent()
            .expect("Python executable path always has a parent")
            .to_path_buf();
        let path = prepend_path(&python_dir, std::env::var_os("PATH"))?;
        vars.insert(OsString::from("PATH"), path);
        Ok(vars)
    }
}

fn prepend_path(directory: &Path, current_path: Option<OsString>) -> AocLanguageResult<OsString> {
    let mut paths = vec![directory.to_path_buf()];
    if let Some(current_path) = current_path {
        paths.extend(std::env::split_paths(&current_path));
    }
    std::env::join_paths(paths).map_err(|error| AocLanguageError::Env(error.to_string()))
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use super::prepend_path;

    #[test]
    fn prepended_path_round_trips_with_platform_separator() {
        let existing = std::env::join_paths([PathBuf::from("one"), PathBuf::from("two")])
            .expect("join existing PATH");
        let path = prepend_path(PathBuf::from("venv-bin").as_path(), Some(existing))
            .expect("prepend PATH");

        assert_eq!(
            std::env::split_paths(&path).collect::<Vec<_>>(),
            vec![
                PathBuf::from("venv-bin"),
                PathBuf::from("one"),
                PathBuf::from("two")
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn prepended_path_preserves_non_unicode_entries() {
        use std::os::unix::ffi::OsStringExt;

        let existing =
            std::env::join_paths([PathBuf::from(OsString::from_vec(b"old-\xff".to_vec()))])
                .expect("join non-Unicode PATH");
        let path = prepend_path(PathBuf::from("venv-bin").as_path(), Some(existing))
            .expect("prepend PATH");

        let paths = std::env::split_paths(&path).collect::<Vec<_>>();
        assert_eq!(
            paths[1].as_os_str(),
            OsString::from_vec(b"old-\xff".to_vec())
        );
    }
}

impl PythonRunner {
    fn get_pip_path(&self) -> PathBuf {
        if cfg!(windows) {
            self.root_dir.join("venv").join("Scripts").join("pip.exe")
        } else {
            self.root_dir.join("venv").join("bin").join("pip")
        }
    }

    pub fn get_python_path(&self) -> PathBuf {
        if cfg!(windows) {
            self.root_dir
                .join("venv")
                .join("Scripts")
                .join("python.exe")
        } else {
            self.root_dir.join("venv").join("bin").join("python")
        }
    }
}
