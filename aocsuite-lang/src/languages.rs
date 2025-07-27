use std::str::FromStr;

use aocsuite_utils::resolve_aocsuite_dir;
use clap::ValueEnum;

use crate::{
    python::PythonRunner,
    rust::RustRunner,
    utils::{AocLanguageError, LanguageRunner},
    AocLanguageResult,
};

#[derive(Clone, Debug, ValueEnum)]
pub enum LanguageType {
    Rust,
    Python,
}

impl ToString for LanguageType {
    fn to_string(&self) -> String {
        match self {
            LanguageType::Rust => "rust".to_owned(),
            LanguageType::Python => "python".to_owned(),
        }
    }
}

impl LanguageType {
    pub fn to_runner(&self) -> AocLanguageResult<LanguageRunner> {
        let root_dir = resolve_aocsuite_dir().join(self.to_string());
        std::fs::create_dir_all(&root_dir)?;
        let runner: LanguageRunner = match self {
            LanguageType::Rust => Box::new(RustRunner::new(root_dir)),
            LanguageType::Python => Box::new(PythonRunner::new(root_dir)),
        };
        Ok(runner)
    }
}

impl FromStr for LanguageType {
    type Err = AocLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(LanguageType::Rust),
            "python" => Ok(LanguageType::Python),
            _ => Err(AocLanguageError::LangNotFound(s.to_owned())),
        }
    }
}
