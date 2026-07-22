use crate::{
    traits::{DepManager, Solver},
    AocLanguageResult,
};
use std::{path::PathBuf, process::Output};

use super::PythonRunner;
use aocsuite_utils::{execute_command, CommandRequest};

impl DepManager for PythonRunner<'_> {
    fn setup_env(&self) -> AocLanguageResult<Option<Output>> {
        self.migrate_runtime()?;
        let venv_path = self.root_dir.join("venv");

        if !venv_path.exists() {
            // Create virtual environment
            return Ok(Some(execute_command(
                self.executor,
                CommandRequest::new("python3")
                    .arg("-m")
                    .arg("venv")
                    .arg("venv")
                    .current_dir(&self.root_dir),
            )?));
        }

        Ok(None)
    }
    fn clean_env(&self) -> AocLanguageResult<()> {
        std::fs::remove_dir_all(self.root_dir.join("venv"))?;
        Ok(())
    }

    fn add_package(&self, package: &str) -> AocLanguageResult<()> {
        let pip_path = self.get_pip_path();

        execute_command(
            self.executor,
            CommandRequest::new(pip_path)
                .arg("install")
                .arg(package)
                .current_dir(&self.root_dir),
        )?;
        Ok(())
    }

    fn list_packages(&self) -> AocLanguageResult<Vec<String>> {
        let venv_path = self.root_dir.join("venv");
        if !venv_path.exists() {
            return Ok(Vec::new());
        }

        let pip_path = self.get_pip_path();

        let diagnostic = execute_command(
            self.executor,
            CommandRequest::new(pip_path)
                .arg("list")
                .arg("--format=freeze")
                .current_dir(&self.root_dir),
        )?;

        let stdout = String::from_utf8_lossy(&diagnostic.stdout);
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

        execute_command(
            self.executor,
            CommandRequest::new(pip_path)
                .arg("uninstall")
                .arg("-y")
                .arg(package)
                .current_dir(&self.root_dir),
        )?;
        Ok(())
    }
}

impl PythonRunner<'_> {
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
