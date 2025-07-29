use std::process::Command;

use crate::{traits::DepManager, AocLanguageResult};

use super::RustRunner;

impl DepManager for RustRunner {
    fn setup_env(&self) -> AocLanguageResult<()> {
        let cargo_contents = r#"[package]
name = "aocsuite-solution-rust"
version = "0.1.0"
edition = "2024"

[dependencies]
serde_json="1.0.140"
serde = { version = "1.0.219", features = ["derive"]}
"#;
        let cargo_path = self.root_dir.join("Cargo.toml");
        if !cargo_path.exists() {
            std::fs::write(&cargo_path, cargo_contents)?
        }
        Ok(())
    }
    fn add_package(&self, package: &str) -> AocLanguageResult<()> {
        let output = Command::new("cargo")
            .arg("add")
            .arg(package)
            .current_dir(&self.root_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::AocLanguageError::DepAdd(
                package.into(),
                error.into(),
            ));
        }

        Ok(())
    }
    fn list_packages(&self) -> AocLanguageResult<Vec<String>> {
        let cargo_path = self.root_dir.join("Cargo.toml");
        if !cargo_path.exists() {
            return Ok(Vec::new());
        }

        let output = Command::new("cargo")
            .arg("tree")
            .arg("--depth=1")
            .arg("--prefix=none")
            .current_dir(&self.root_dir)
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new()); // If cargo tree fails, return empty list
        }

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
        let output = Command::new("cargo")
            .arg("remove")
            .arg(package)
            .current_dir(&self.root_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::AocLanguageError::DepRemove(
                package.into(),
                error.into(),
            ));
        }

        Ok(())
    }
}
