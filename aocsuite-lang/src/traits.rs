use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Output,
};

use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::utils::{symlink_file, AocLanguageResult, SolverFile};

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

    fn solver_file_path(&self, file: &SolverFile) -> PathBuf;
    fn setup_solver(&self) -> AocLanguageResult<()>;
    fn main_contents(&self) -> String;
    fn template_contents(&self) -> String;
    fn clean_cache(&self) -> AocLanguageResult<()>;

    fn ensure_solver_file(&self, file: &SolverFile) -> AocLanguageResult<PathBuf> {
        let path = self.solver_file_path(file);
        match file {
            SolverFile::PuzzleSolution(_, _) => {
                if !path.exists() {
                    std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                    let template_path = self.ensure_solver_file(&SolverFile::SolutionTemplate)?;
                    std::fs::copy(template_path, &path)?;
                }
            }
            SolverFile::Entrypoint => {
                if !path.exists() {
                    std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                    std::fs::write(&path, self.main_contents())?;
                }
            }
            SolverFile::SolutionTemplate => {
                if !path.exists() {
                    std::fs::create_dir_all(path.parent().expect("solve file is not root"))?;
                    std::fs::write(&path, self.template_contents())?;
                }
            }
            SolverFile::ActiveSolution(day, year) => {
                let linked_path =
                    self.ensure_solver_file(&SolverFile::PuzzleSolution(*day, *year))?;
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
