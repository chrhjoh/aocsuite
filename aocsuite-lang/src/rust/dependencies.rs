use std::{collections::HashMap, ffi::OsString, process::Output};

use crate::{
    traits::{DepManager, Solver},
    AocLanguageError, AocLanguageResult,
};

use super::RustRunner;
use aocsuite_utils::{execute_command, CommandRequest};

impl DepManager for RustRunner<'_> {
    fn setup_env(&self) -> AocLanguageResult<Option<Output>> {
        self.migrate_runtime()?;
        Ok(None)
    }
    fn add_package(&self, package: &str) -> AocLanguageResult<()> {
        execute_command(
            self.executor,
            CommandRequest::new("cargo")
                .arg("add")
                .arg(package)
                .current_dir(&self.root_dir),
        )?;
        Ok(())
    }
    fn clean_env(&self) -> AocLanguageResult<()> {
        let cargo_path = self.root_dir.join("Cargo.toml");
        std::fs::remove_file(cargo_path)?;
        Ok(())
    }

    fn list_packages(&self) -> AocLanguageResult<Vec<String>> {
        let cargo_path = self.root_dir.join("Cargo.toml");
        if !cargo_path.exists() {
            return Ok(Vec::new());
        }

        let output = execute_command(
            self.executor,
            CommandRequest::new("cargo")
                .arg("tree")
                .arg("--depth=1")
                .arg("--prefix=none")
                .current_dir(&self.root_dir),
        )?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = stdout
            .lines()
            .skip(1) // Skip the first line which is the project itself
            .filter_map(|line| {
                line.split_whitespace()
                    .next()
                    .and_then(|dep| dep.split('@').next())
                    .map(|name| name.to_string())
            })
            .collect();

        Ok(packages)
    }
    fn remove_packages(&self, package: &str) -> AocLanguageResult<()> {
        if ["serde", "serde_json"].contains(&package) {
            return Err(AocLanguageError::DepRemove(
                package.to_string(),
                "Is required by AocSuite".to_string(),
            ));
        }
        execute_command(
            self.executor,
            CommandRequest::new("cargo")
                .arg("remove")
                .arg(package)
                .current_dir(&self.root_dir),
        )?;
        Ok(())
    }
    fn editor_environment_vars(&self) -> AocLanguageResult<HashMap<OsString, OsString>> {
        Ok(HashMap::new())
    }
}
