use std::{
    path::{Path, PathBuf},
    process::Output,
};

use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::utils::{AocLanguageResult, SolveFile};

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

    fn _get_path(&self, file: &SolveFile) -> PathBuf;
    fn ensure_files(&self) -> AocLanguageResult<()>;
    fn main_contents(&self) -> String;
    fn template_contents(&self) -> String;
}
