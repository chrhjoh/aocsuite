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

pub fn get_lib_filepath(
    lib_name: &str,
    language: &Option<LanguageType>,
) -> AocLanguageResult<PathBuf> {
    let runner = resolve_runner(language)?;
    let unallowed_names = runner.invalid_lib_names();
    validate_user_lib(lib_name, &unallowed_names)?;
    let lib_path = runner.get_lib_path(lib_name);

    if !lib_path.exists() {
        std::fs::create_dir_all(lib_path.parent().expect("is not root"))?;
    }

    Ok(lib_path)
}

pub fn remove_lib_file(lib_name: &str, language: &Option<LanguageType>) -> AocLanguageResult<()> {
    let runner = resolve_runner(language)?;
    let unallowed_names = runner.invalid_lib_names();
    validate_user_lib(lib_name, &unallowed_names)?;
    let lib_path = runner.get_lib_path(lib_name);
    if lib_path.exists() {
        std::fs::remove_file(lib_path)?;
    }
    Ok(())
}

pub fn list_lib_files(language: &Option<LanguageType>) -> AocLanguageResult<Vec<String>> {
    let runner = resolve_runner(language)?;
    let file_extention = runner.file_extention();
    let dir = runner.lib_dir();
    let files = scan_lib_directory(&dir, &file_extention)?;
    let unallowed_names = runner.invalid_lib_names();
    let filtered_files = files
        .iter()
        .filter(|f| match validate_user_lib(*f, &unallowed_names) {
            Ok(_) => true,
            Err(_) => false,
        })
        .map(|f| f.clone())
        .collect();
    Ok(filtered_files)
}

fn validate_user_lib(lib_name: &str, unallowed_names: &Vec<&str>) -> AocLanguageResult<()> {
    if lib_name.trim().is_empty() {
        return Err(AocLanguageError::LibInvalid(
            "Library name cannot be empty".to_string(),
        ));
    }

    if !lib_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AocLanguageError::LibInvalid(
            "Library name can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }

    if let Some(first_char) = lib_name.chars().next() {
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(AocLanguageError::LibInvalid(
                "Library name must start with a letter or underscore".to_string(),
            ));
        }
    }

    if unallowed_names.contains(&lib_name) {
        return Err(AocLanguageError::LibInvalid(format!(
            "'{}' is a reserved name for this language",
            lib_name
        )));
    }

    Ok(())
}
fn scan_lib_directory(dir: &Path, file_extention: &str) -> crate::AocLanguageResult<Vec<String>> {
    let mut lib_files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_stem() {
                if let Some(extension) = path.extension() {
                    if extension == file_extention {
                        let name = file_name.to_string_lossy();
                        lib_files.push(name.to_string());
                    }
                }
            }
        }
    }
    Ok(lib_files)
}
