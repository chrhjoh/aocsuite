use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use aocsuite_fs::write_with_confirmation;
use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::{
    read_template_contents, result_file, template_filename, AocLanguageResult, Language,
    LanguageFile,
};

use super::LanguageRunner;

pub struct RustLanguage {
    root_dir: PathBuf,
}

const CARGO_FILE: &str = "Cargo.toml";
fn bin_name(day: PuzzleDay, year: PuzzleYear) -> String {
    format!("year{year}_day{day}")
}

impl RustLanguage {
    pub fn new(root_dir: PathBuf) -> RustLanguage {
        RustLanguage { root_dir }
    }

    fn main_path(&self, day: PuzzleDay, year: PuzzleYear) -> PathBuf {
        let bin_dir = self.src_bin_dir();
        bin_dir.join(bin_name(day, year)).with_extension("rs")
    }
    fn src_bin_dir(&self) -> PathBuf {
        self.root_dir.join("src").join("bin")
    }
    fn cargo_path(&self) -> PathBuf {
        self.root_dir.join(CARGO_FILE)
    }
}

impl LanguageRunner for RustLanguage {
    fn scaffold(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        template_dir: Option<&str>,
        overwrite: bool,
    ) -> AocLanguageResult<()> {
        fs::create_dir_all(&self.src_bin_dir())?;

        if !self.cargo_path().exists() {
            let cargo_contents = initial_cargo_contents();
            write_with_confirmation(self.cargo_path(), cargo_contents, false)?;
        }
        let bin_path = self.main_path(day, year);

        let bin_contents = match template_dir {
            Some(dir) => {
                let path = template_filename(dir, &Language::Rust);
                read_template_contents(&path)?
            }
            None => default_main_contents(),
        };
        write_with_confirmation(bin_path, bin_contents, overwrite)?;

        Ok(())
    }
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<Option<Output>> {
        let package_name = bin_name(day, year);
        let output = Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg(package_name)
            .current_dir(&self.root_dir)
            .output()?;
        Ok(Some(output))
    }

    fn run(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        part: &str,
        input: &Path,
    ) -> AocLanguageResult<Output> {
        let binary = self.get_path(day, year, LanguageFile::Executable);

        let output = Command::new(binary)
            .args([
                "-p",
                part,
                "-o",
                result_file().to_str().unwrap(),
                input.to_str().unwrap(),
            ])
            .output()?;

        Ok(output)
    }
    fn get_path(&self, day: PuzzleDay, year: PuzzleYear, file: crate::LanguageFile) -> PathBuf {
        match file {
            LanguageFile::Lib => self.main_path(day, year),
            LanguageFile::Main => self.main_path(day, year),
            LanguageFile::Executable => self
                .root_dir
                .join("target")
                .join("debug")
                .join(bin_name(day, year)),
        }
    }
}

fn initial_cargo_contents() -> String {
    let contents = r#"[package]
name = "my-aocsuite-solutions"
version = "0.1.0"
edition = "2024"

[dependencies]
aocsuite-lang-rust = "0.2.1"
"#;
    contents.to_string()
}

pub fn default_main_contents() -> String {
    let content = r#"
use aocsuite_lang_rust::run_exercises;

/// Implement your solution here

/// Solve part 1 of the puzzle
fn part1(input: &str) -> String {
    // Replace this stub with actual implementation
    format!("Part 1 not implemented yet. Input length: {}", input.len())
}

/// Solve part 2 of the puzzle
fn part2(input: &str) -> String {
    // Replace this stub with actual implementation
    format!("Part 2 not implemented yet. Input length: {}", input.len())
}

fn main() {
    run_exercises(part1, part2);
}
"#;

    content.to_string()
}
