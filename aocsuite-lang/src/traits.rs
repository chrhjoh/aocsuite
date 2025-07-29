use std::{
    path::{Path, PathBuf},
    process::Output,
};

use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::utils::{AocLanguageResult, SolveFile};

pub trait LanguageHandler: Solver + DepManager + LibManager {}
impl<T> LanguageHandler for T where T: Solver + DepManager + LibManager {}

pub trait Solver {
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<Option<Output>>;
    fn run(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        part: &str,
        input: &Path,
        output: &std::path::Path,
    ) -> AocLanguageResult<Output>;

    fn get_solvefile_path(&self, file: &SolveFile) -> PathBuf;
    fn setup_solver(&self) -> AocLanguageResult<()>;
    fn main_contents(&self) -> String;
    fn template_contents(&self) -> String;
}

pub trait LibManager {
    fn get_lib_path(&self, lib_name: &str) -> PathBuf;
    fn list_lib_files(&self) -> AocLanguageResult<Vec<String>>;
    fn remove_lib_file(&self, lib_name: &str) -> AocLanguageResult<()>;
}

pub trait DepManager {
    fn setup_env(&self) -> AocLanguageResult<()>;
    fn add_package(&self, package: &str) -> AocLanguageResult<()>;
    fn list_packages(&self) -> AocLanguageResult<Vec<String>>;
    fn remove_packages(&self, package: &str) -> AocLanguageResult<()>;
}
