mod python;
mod rust;
use aocsuite_fs::{ensure_files_exist, AocFileError};
use aocsuite_utils::{PuzzleDay, PuzzleYear};
use clap::ValueEnum;
use python::PythonLanguage;
use rust::RustLanguage;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{
    fs::File,
    io::BufReader,
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

pub fn compile(day: PuzzleDay, year: PuzzleYear, language: &Language) -> AocLanguageResult<()> {
    let runner = get_language_runner(language);
    ensure_files_exist(vec![
        runner.get_path(day, year, LanguageFile::Main).as_path(),
        runner.get_path(day, year, LanguageFile::Lib).as_path(),
    ])?;
    let output = runner.compile(day, year)?;
    match output {
        Some(output) => handle_command_output(output),
        None => Ok(()),
    }
}

#[derive(Serialize, Deserialize)]
struct PartResult {
    answer: String,
    runtime_ms: u128,
}

#[derive(Serialize, Deserialize)]
pub struct ExerciseOutput {
    part1: Option<PartResult>,
    part2: Option<PartResult>,
}

impl fmt::Display for PartResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Answer: {}", self.answer)?;
        writeln!(f, "Runtime: {} ms", self.runtime_ms)
    }
}

impl fmt::Display for ExerciseOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref p1) = self.part1 {
            writeln!(f, "=== Part 1 ===")?;
            writeln!(f, "{}", p1)?;
        }
        if let Some(ref p2) = self.part2 {
            writeln!(f, "=== Part 2 ===")?;
            writeln!(f, "{}", p2)?;
        }
        Ok(())
    }
}

pub fn run(
    day: PuzzleDay,
    year: PuzzleYear,
    part: &str,
    language: &Language,
    input: &Path,
) -> AocLanguageResult<ExerciseOutput> {
    let runner = get_language_runner(language);
    ensure_files_exist(vec![
        input,
        runner
            .get_path(day, year, LanguageFile::Executable)
            .as_path(),
    ])?;
    let output = runner.run(day, year, part, input)?;
    handle_command_output(output)?;
    let result_file = result_filename(day, year);
    let reader = BufReader::new(File::open(&result_file)?);
    let result = serde_json::from_reader(reader)?;
    std::fs::remove_file(result_file)?;
    Ok(result)
}

fn handle_command_output(output: Output) -> AocLanguageResult<()> {
    if !output.status.success() {
        // The compile command ran but failed
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AocLanguageError::Command(stderr.to_string()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout != "" {
        println!("Standard out from exercise {}", stdout)
    }
    Ok(())
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

fn get_language_runner(language: &Language) -> Box<dyn LanguageRunner> {
    let language_root_dir = language.to_string();
    let language_root_dir = Path::new(&language_root_dir).to_path_buf();
    match language {
        Language::Rust => Box::new(RustLanguage::new(language_root_dir)),
        Language::Python => Box::new(PythonLanguage::new(language_root_dir)),
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

    #[error("Error parsing result json file: {0}")]
    ResultJson(#[from] serde_json::Error),

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

pub fn result_filename(day: PuzzleDay, year: PuzzleYear) -> String {
    PathBuf::from(".aocsuite")
        .join(format!("year{year}_day{day}_result.json"))
        .to_str()
        .unwrap()
        .to_owned()
}
