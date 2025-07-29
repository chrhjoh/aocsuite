use crate::AocLanguageError;
use crate::{traits::DepManager, AocLanguageResult};
use std::path::PathBuf;
use std::process::Command;

use super::PythonRunner;

impl DepManager for PythonRunner {
    fn setup_env(&self) -> AocLanguageResult<()> {
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
