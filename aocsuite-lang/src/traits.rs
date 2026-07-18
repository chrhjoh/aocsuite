use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Output,
};

use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::utils::{symlink_file, AocLanguageResult, SolveFile};

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
    fn clean_cache(&self) -> AocLanguageResult<()>;

    fn ensure_solvefile(&self, file: &SolveFile) -> AocLanguageResult<PathBuf> {
        let path = self.get_solvefile_path(file);
        match file {
            SolveFile::Solution(_, _) => {
                if !path.exists() {
                    std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                    let template_path = self.ensure_solvefile(&SolveFile::TemplateSolution)?;
                    std::fs::copy(template_path, &path)?;
                }
            }
            SolveFile::Main => {
                if !path.exists() {
                    std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                    std::fs::write(&path, self.main_contents())?;
                }
            }
            SolveFile::TemplateSolution => {
                if !path.exists() {
                    std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                    std::fs::write(&path, self.template_contents())?;
                }
            }
            SolveFile::LinkedSolution(linked_file) => {
                let linked_path = self.ensure_solvefile(linked_file)?;
                std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                symlink_file(&linked_path, &path)?;
            }
        }
        Ok(path)
    }
}

pub trait LibManager {
    fn get_lib_path(&self, lib_name: &str) -> PathBuf;
    fn lib_dir(&self) -> PathBuf;
    fn file_extention(&self) -> String;
    fn invalid_lib_names(&self) -> Vec<&str>;
}

pub trait DepManager {
    fn setup_env(&self) -> AocLanguageResult<()>;
    fn editor_environment_vars(&self) -> AocLanguageResult<HashMap<String, String>>;
    fn add_package(&self, package: &str) -> AocLanguageResult<()>;
    fn list_packages(&self) -> AocLanguageResult<Vec<String>>;
    fn remove_packages(&self, package: &str) -> AocLanguageResult<()>;
    fn clean_env(&self) -> AocLanguageResult<()>;
}
