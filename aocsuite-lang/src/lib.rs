mod languages;
mod python;
mod rust;
mod traits;
mod utils;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use aocsuite_config::{get_config_val, ConfigOpt};
use aocsuite_utils::{get_aocsuite_dir, PuzzleDay, PuzzleYear};
pub use languages::LanguageType;
use utils::{handle_command_output, read_result, symlink_file, ExerciseOutput, LanguageRunner};
pub use utils::{AocLanguageError, AocLanguageResult, SolveFile};

pub fn run(
    day: PuzzleDay,
    year: PuzzleYear,
    part: &str,
    input: &Path,
    language: &Option<LanguageType>,
) -> AocLanguageResult<ExerciseOutput> {
    let runner = resolve_runner(&language)?;
    let output_file = get_aocsuite_dir().join("result.json");

    let output = runner.run(day, year, part, input, &output_file)?;
    handle_command_output(output)?;

    let result = read_result(&output_file)?;
    Ok(result)
}

pub fn compile(
    day: PuzzleDay,
    year: PuzzleYear,
    language: &Option<LanguageType>,
) -> AocLanguageResult<()> {
    let runner = resolve_runner(language)?;
    match runner.compile(day, year)? {
        Some(output) => handle_command_output(output),
        None => Ok(()),
    }
}

pub fn get_path(file: &SolveFile, language: &Option<LanguageType>) -> AocLanguageResult<PathBuf> {
    let runner = resolve_runner(language)?;
    let path = runner.get_solvefile_path(file);
    match file {
        SolveFile::Solution(_, _) => {
            if !&path.exists() {
                std::fs::create_dir_all(&path.parent().expect("is not root"))?;
                let template_path = get_path(&SolveFile::TemplateSolution, language)?;
                std::fs::copy(&template_path, &path)?;
            }
        }
        SolveFile::Main => {
            if !&path.exists() {
                let contents = runner.main_contents();
                std::fs::write(&path, contents)?;
            }
        }
        SolveFile::TemplateSolution => {
            if !&path.exists() {
                let contents = runner.template_contents();
                std::fs::write(&path, contents)?;
            }
        }
        SolveFile::LinkedSolution(linked_file) => {
            let linked_path = get_path(&linked_file, language)?;
            symlink_file(&linked_path, &path)?;
        }
    }
    Ok(path)
}

fn resolve_runner(language: &Option<LanguageType>) -> AocLanguageResult<LanguageRunner> {
    let language = get_config_val(&ConfigOpt::Language, None, language.clone())?;
    let runner = language.to_runner()?;
    runner.setup_solver()?;
    runner.setup_env()?;
    Ok(runner)
}

pub fn add_package(package: &str, language: &Option<LanguageType>) -> AocLanguageResult<()> {
    let runner = resolve_runner(language)?;
    runner.add_package(package)
}

pub fn remove_package(package: &str, language: &Option<LanguageType>) -> AocLanguageResult<()> {
    let runner = resolve_runner(language)?;
    runner.remove_packages(package)
}

pub fn list_packages(language: &Option<LanguageType>) -> AocLanguageResult<Vec<String>> {
    let runner = resolve_runner(language)?;
    runner.list_packages()
}

pub fn editor_enviroment_vars(
    language: &Option<LanguageType>,
) -> AocLanguageResult<HashMap<String, String>> {
    let runner = resolve_runner(language)?;
    runner.editor_environment_vars()
}
