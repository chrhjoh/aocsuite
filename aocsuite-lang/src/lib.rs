mod rust;
use aocsuite_fs::{AocFileError, ensure_files_exist};
use aocsuite_utils::{PuzzleDay, PuzzleYear};
use clap::ValueEnum;
use rust::RustLanguage;
use std::{
    path::{Path, PathBuf},
    process::Output,
    str::FromStr,
    time::Instant,
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
            _ => Err(AocLanguageError::LangNotFound(s.to_owned())),
        }
    }
}

pub fn get_language_file(
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
    overwrite: bool,
) -> AocLanguageResult<()> {
    let runner = get_language_runner(language);
    runner.scaffold(day, year, template_dir, overwrite)
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
) -> AocLanguageResult<Option<(String, u128)>> {
    let runner = get_language_runner(language);
    let start = Instant::now();
    ensure_files_exist(vec![
        input,
        runner
            .get_path(day, year, LanguageFile::Executable)
            .as_path(),
    ])?;
    let output = runner.run(day, year, part, input)?;
    let duration_ms = start.elapsed().as_millis();
    let result = handle_command_output(output)?;
    match result {
        Some(res) => Ok(Some((res, duration_ms))),
        None => Ok(None),
    }
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
        overwrite: bool,
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
    Executable,
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
    #[error("TOML parsing error")]
    Toml(#[from] toml_edit::TomlError),
    #[error("Language not found: {0}")]
    LangNotFound(String),
    #[error("failed to read template '{path}': {source}")]
    TemplateRead {
        #[source]
        source: std::io::Error,
        path: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    File(#[from] AocFileError),
}

pub type AocLanguageResult<T> = Result<T, AocLanguageError>;

//TODO: Add option for custom template args
pub fn read_template_contents(path: &Path) -> AocLanguageResult<String> {
    let contents = std::fs::read_to_string(path).map_err(|e| AocLanguageError::TemplateRead {
        source: e,
        path: path.to_string_lossy().into_owned(),
    })?;

    Ok(contents)
}
