mod rust;
use aocsuite_utils::{PuzzleDay, PuzzleYear};
use clap::ValueEnum;
use rust::RustLanguage;
use std::{
    path::{Path, PathBuf},
    process::Output,
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Debug, ValueEnum)]
pub enum Language {
    Rust,
    Python,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::Rust => "rust".to_owned(),
            Language::Python => "python".to_owned(),
        }
    }
}

impl FromStr for Language {
    type Err = AocLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(Language::Rust),
            "python" => Ok(Language::Python),
            _ => Err(AocLanguageError::NotFound(s.to_owned())),
        }
    }
}

pub fn get_exercise_file(
    day: PuzzleDay,
    year: PuzzleYear,
    language: &Language,
    file: LanguageFile,
) -> PathBuf {
    let runner = get_language_runner(language);
    runner.get_path(day, year, file)
}

pub fn scaffold(
    day: PuzzleDay,
    year: PuzzleYear,
    language: &Language,
    template_dir: Option<&str>,
) -> AocLanguageResult<()> {
    let runner = get_language_runner(language);
    runner.scaffold(day, year, template_dir)
}

pub fn compile(
    day: PuzzleDay,
    year: PuzzleYear,
    language: &Language,
) -> AocLanguageResult<Option<String>> {
    let runner = get_language_runner(language);
    let output = runner.compile(day, year)?;
    match output {
        Some(output) => handle_command_output(output),
        None => Ok(None),
    }
}

pub fn run(
    day: PuzzleDay,
    year: PuzzleYear,
    part: &str,
    language: &Language,
    input: &Path,
) -> AocLanguageResult<Option<String>> {
    let runner = get_language_runner(language);
    let output = runner.run(day, year, part, input)?;
    handle_command_output(output)
}

fn handle_command_output(output: Output) -> AocLanguageResult<Option<String>> {
    if !output.status.success() {
        // The compile command ran but failed
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AocLanguageError::Command(stderr.to_string()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout == "" {
        return Ok(None);
    }
    Ok(Some(stdout))
}

trait LanguageRunner {
    fn scaffold(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        template_dir: Option<&str>,
    ) -> AocLanguageResult<()>;
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<Option<Output>>;
    fn run(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        part: &str,
        input: &Path,
    ) -> AocLanguageResult<Output>;

    fn get_path(&self, day: PuzzleDay, year: PuzzleYear, file: LanguageFile) -> PathBuf;
}

pub enum LanguageFile {
    Lib,
    Main,
}

fn get_language_runner(language: &Language) -> impl LanguageRunner {
    let language_root_dir = language.to_string();
    let language_root_dir = Path::new(&language_root_dir).to_path_buf();
    match language {
        Language::Rust => RustLanguage::new(language_root_dir),
        Language::Python => RustLanguage::new(language_root_dir),
    }
}
#[derive(Error, Debug)]
pub enum AocLanguageError {
    #[error("error executing command: {0}")]
    Command(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error")]
    Toml(#[from] toml_edit::TomlError),
    #[error("Language not found: {0}")]
    NotFound(String),
}

pub type AocLanguageResult<T> = Result<T, AocLanguageError>;
