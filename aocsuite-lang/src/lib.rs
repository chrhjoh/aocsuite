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
use utils::{handle_command_output, read_result, ExerciseOutput, LanguageRunner};
pub use utils::{AocLanguageError, AocLanguageResult, SolveFile};

pub struct Language {
    name: String,
    runner: LanguageRunner,
}

impl Language {
    pub fn resolve(language: &Option<LanguageType>) -> AocLanguageResult<Self> {
        let language = get_config_val(&ConfigOpt::Language, None, language.clone())?;
        Ok(Self {
            name: language.to_string(),
            runner: language.to_runner()?,
        })
    }

    pub fn run(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        part: &str,
        input: &Path,
    ) -> AocLanguageResult<ExerciseOutput> {
        self.setup_solution(day, year)?;
        let output_file = get_aocsuite_dir().join("result.json");
        let output = self.runner.run(day, year, part, input, &output_file)?;
        handle_command_output(output)?;
        read_result(&output_file)
    }

    pub fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<()> {
        self.setup_solution(day, year)?;
        match self.runner.compile(day, year)? {
            Some(output) => handle_command_output(output),
            None => Ok(()),
        }
    }

    pub fn prepare_solvefile(&self, file: &SolveFile) -> AocLanguageResult<PathBuf> {
        self.runner.setup_solver()?;
        self.runner.ensure_solvefile(file)
    }

    pub fn add_package(&self, package: &str) -> AocLanguageResult<()> {
        self.runner.setup_env()?;
        self.runner.add_package(package)
    }

    pub fn remove_package(&self, package: &str) -> AocLanguageResult<()> {
        self.runner.setup_env()?;
        self.runner.remove_packages(package)
    }

    pub fn list_packages(&self) -> AocLanguageResult<Vec<String>> {
        self.runner.list_packages()
    }

    pub fn editor_environment_vars(&self) -> AocLanguageResult<HashMap<String, String>> {
        self.runner.editor_environment_vars()
    }

    pub fn get_lib_filepath(&self, lib_name: &str) -> AocLanguageResult<PathBuf> {
        let unallowed_names = self.runner.invalid_lib_names();
        validate_user_lib(lib_name, &unallowed_names)?;
        let lib_path = self.runner.get_lib_path(lib_name);

        if !lib_path.exists() {
            std::fs::create_dir_all(lib_path.parent().expect("is not root"))?;
        }

        Ok(lib_path)
    }

    pub fn remove_lib_file(&self, lib_name: &str) -> AocLanguageResult<()> {
        let unallowed_names = self.runner.invalid_lib_names();
        validate_user_lib(lib_name, &unallowed_names)?;
        let lib_path = self.runner.get_lib_path(lib_name);
        if lib_path.exists() {
            std::fs::remove_file(lib_path)?;
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn list_lib_files(&self) -> AocLanguageResult<Vec<String>> {
        let file_extention = self.runner.file_extention();
        let dir = self.runner.lib_dir();
        let files = scan_lib_directory(&dir, &file_extention)?;
        let unallowed_names = self.runner.invalid_lib_names();
        Ok(files
            .into_iter()
            .filter(|file| validate_user_lib(file, &unallowed_names).is_ok())
            .collect())
    }

    pub fn clean_cache(&self) -> AocLanguageResult<()> {
        self.runner.clean_cache()
    }

    pub fn clean_env(&self) -> AocLanguageResult<()> {
        self.runner.clean_env()
    }

    fn setup_solution(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<()> {
        self.runner.setup_solver()?;
        self.runner
            .ensure_solvefile(&SolveFile::linked_solution(day, year))?;
        self.runner.setup_env()
    }
}

fn validate_user_lib(lib_name: &str, unallowed_names: &Vec<&str>) -> AocLanguageResult<()> {
    if lib_name.trim().is_empty() {
        return Err(AocLanguageError::LibInvalid(
            "Library name cannot be empty".to_string(),
        ));
    }

    if !lib_name
        .chars()
        .all(|c| c.is_alphabetic() || c == '_' || c == '-')
    {
        return Err(AocLanguageError::LibInvalid(
            "Library name can only contain letters, underscores, and hyphens".to_string(),
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{python::PythonRunner, rust::RustRunner, traits::LanguageHandler, SolveFile};

    fn test_root(language: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "aocsuite-lang-{language}-{}-{unique}",
            process::id()
        ))
    }

    fn assert_requested_solution_is_active(runner: &dyn LanguageHandler) {
        let first_solution = runner
            .ensure_solvefile(&SolveFile::Solution(1, 2024))
            .expect("create first solution");
        fs::write(&first_solution, "first solution").expect("write first solution");
        runner
            .ensure_solvefile(&SolveFile::linked_solution(1, 2024))
            .expect("activate first solution");

        let active_solution = runner.get_solvefile_path(&SolveFile::linked_solution(1, 2024));
        assert_eq!(
            fs::read_to_string(&active_solution).expect("read active first solution"),
            "first solution"
        );

        let second_solution = runner
            .ensure_solvefile(&SolveFile::Solution(2, 2024))
            .expect("create second solution");
        fs::write(&second_solution, "second solution").expect("write second solution");
        runner
            .ensure_solvefile(&SolveFile::linked_solution(2, 2024))
            .expect("activate second solution");

        assert_eq!(
            fs::read_to_string(&active_solution).expect("read active second solution"),
            "second solution"
        );
    }

    #[test]
    fn rust_activation_selects_the_requested_solution() {
        let root = test_root("rust");
        let runner = RustRunner::new(root.clone());

        assert_requested_solution_is_active(&runner);

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn python_activation_selects_the_requested_solution() {
        let root = test_root("python");
        fs::create_dir_all(&root).expect("create test runtime");
        let runner = PythonRunner::new(root.clone());

        assert_requested_solution_is_active(&runner);

        fs::remove_dir_all(root).expect("remove test runtime");
    }
}
