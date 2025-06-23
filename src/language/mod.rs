mod rust;
use std::{io, path::PathBuf, process::Output};

use clap::ValueEnum;
use rust::RustLanguage;

use crate::utils::{PuzzleDay, PuzzleYear};

#[derive(ValueEnum, Clone, Debug)]
pub enum Language {
    Rust,
    Python,
}

pub fn base_language_dir(language: &Language) -> PathBuf {
    let lang_dir = match language {
        Language::Rust => "rust",
        Language::Python => "python",
    };
    return PathBuf::from(lang_dir);
}

pub trait LanguageRunner {
    fn scaffold(&self, day: PuzzleDay, year: PuzzleYear, template_dir: Option<String>);
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> Option<io::Result<Output>>;
    fn run(&self, day: PuzzleDay, year: PuzzleYear) -> io::Result<Output>;
}

pub fn get_language_runner(language: &Language) -> impl LanguageRunner {
    let language_root_dir = base_language_dir(language);
    match language {
        Language::Rust => RustLanguage::new(language_root_dir),
        Language::Python => RustLanguage::new(language_root_dir),
    }
}
