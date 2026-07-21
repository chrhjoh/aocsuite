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
use utils::{
    handle_command_output, new_result_file_path, read_result, with_result_file, ExerciseOutput,
    LanguageRunner,
};
pub use utils::{AocLanguageError, AocLanguageResult, SolverFile};

pub struct Language {
    name: String,
    language_type: LanguageType,
    runner: LanguageRunner,
}

impl Language {
    pub fn resolve(language: &Option<LanguageType>) -> AocLanguageResult<Self> {
        let language = get_config_val(&ConfigOpt::Language, None, language.clone())?;
        Ok(Self {
            name: language.to_string(),
            language_type: language.clone(),
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
        let output_file = new_result_file_path(&get_aocsuite_dir()?.join("runs"))?;
        with_result_file(&output_file, |output_file| {
            let output = self.runner.run(day, year, part, input, output_file)?;
            handle_command_output(output)?;
            read_result(output_file)
        })
    }

    pub fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<()> {
        self.setup_solution(day, year)?;
        match self.runner.compile(day, year)? {
            Some(output) => handle_command_output(output),
            None => Ok(()),
        }
    }

    pub fn prepare_solver_file(&self, file: &SolverFile) -> AocLanguageResult<PathBuf> {
        self.runner.setup_solver()?;
        self.runner.ensure_solver_file(file)
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
        validate_user_lib(lib_name, &self.language_type)?;
        let lib_path = self.runner.get_lib_path(lib_name);
        ensure_no_case_collision(
            &self.runner.lib_dir(),
            &self.runner.file_extention(),
            lib_name,
        )?;

        if !lib_path.exists() {
            std::fs::create_dir_all(lib_path.parent().expect("is not root"))?;
        }

        Ok(lib_path)
    }

    pub fn remove_lib_file(&self, lib_name: &str) -> AocLanguageResult<()> {
        validate_user_lib(lib_name, &self.language_type)?;
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
        Ok(files
            .into_iter()
            .filter(|file| validate_user_lib(file, &self.language_type).is_ok())
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
            .ensure_solver_file(&SolverFile::ActiveSolution(day, year))?;
        self.runner.setup_env()
    }
}

fn validate_user_lib(lib_name: &str, language: &LanguageType) -> AocLanguageResult<()> {
    if lib_name.is_empty() {
        return Err(AocLanguageError::LibInvalid(
            "Library name cannot be empty".to_string(),
        ));
    }

    let mut chars = lib_name.bytes();
    let first = chars.next().expect("checked non-empty library name");
    if !(first.is_ascii_alphabetic() || first == b'_')
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(AocLanguageError::LibInvalid(
            "Library name must be an ASCII identifier".to_string(),
        ));
    }

    let reserved_names = match language {
        LanguageType::Rust => RUST_RESERVED_NAMES,
        LanguageType::Python => PYTHON_RESERVED_NAMES,
    };
    if reserved_names
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(lib_name))
    {
        return Err(AocLanguageError::LibInvalid(format!(
            "'{}' is a reserved name for this language",
            lib_name
        )));
    }

    Ok(())
}

fn ensure_no_case_collision(dir: &Path, extension: &str, lib_name: &str) -> AocLanguageResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for existing in scan_lib_directory(dir, extension)? {
        if existing != lib_name && existing.eq_ignore_ascii_case(lib_name) {
            return Err(AocLanguageError::LibInvalid(format!(
                "'{}' conflicts with existing library '{}'",
                lib_name, existing
            )));
        }
    }
    Ok(())
}

const RUST_RESERVED_NAMES: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "try", "type",
    "typeof", "union", "unsafe", "use", "virtual", "where", "while", "yield", "main", "solution",
    "template",
];

const PYTHON_RESERVED_NAMES: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield", "match", "case", "type", "main", "solution", "template", "venv",
];
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

    use super::{ensure_no_case_collision, validate_user_lib};
    use crate::{
        python::PythonRunner,
        rust::RustRunner,
        traits::{LanguageHandler, Solver},
        utils::{new_result_file_path, read_result, with_result_file},
        AocLanguageError, LanguageType, SolverFile,
    };

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

    #[test]
    fn library_names_follow_language_identifier_rules() {
        let cases = [
            (LanguageType::Rust, "day2", true),
            (LanguageType::Rust, "snake_case", true),
            (LanguageType::Rust, "match", false),
            (LanguageType::Rust, "Main", false),
            (LanguageType::Rust, "two-words", false),
            (LanguageType::Rust, "2fast", false),
            (LanguageType::Rust, "café", false),
            (LanguageType::Python, "day2", true),
            (LanguageType::Python, "snake_case", true),
            (LanguageType::Python, "class", false),
            (LanguageType::Python, "venv", false),
            (LanguageType::Python, "two-words", false),
            (LanguageType::Python, "2fast", false),
            (LanguageType::Python, "café", false),
        ];

        for (language, name, valid) in cases {
            assert_eq!(
                validate_user_lib(name, &language).is_ok(),
                valid,
                "{language:?} name {name}"
            );
        }
    }

    #[test]
    fn library_names_reject_case_insensitive_collisions() {
        let dir = test_root("library-collision");
        std::fs::create_dir_all(&dir).expect("create library directory");
        std::fs::write(dir.join("Helpers.rs"), "").expect("create library");

        assert!(ensure_no_case_collision(&dir, "rs", "helpers").is_err());
        assert!(ensure_no_case_collision(&dir, "rs", "Helpers").is_ok());

        std::fs::remove_dir_all(dir).expect("remove library directory");
    }

    fn assert_requested_solution_is_active(runner: &dyn LanguageHandler) {
        let first_solution = runner
            .ensure_solver_file(&SolverFile::PuzzleSolution(1, 2024))
            .expect("create first solution");
        fs::write(&first_solution, "first solution").expect("write first solution");
        runner
            .ensure_solver_file(&SolverFile::ActiveSolution(1, 2024))
            .expect("activate first solution");

        let active_solution = runner.solver_file_path(&SolverFile::ActiveSolution(1, 2024));
        assert_eq!(
            fs::read_to_string(&active_solution).expect("read active first solution"),
            "first solution"
        );

        let second_solution = runner
            .ensure_solver_file(&SolverFile::PuzzleSolution(2, 2024))
            .expect("create second solution");
        fs::write(&second_solution, "second solution").expect("write second solution");
        runner
            .ensure_solver_file(&SolverFile::ActiveSolution(2, 2024))
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

    #[test]
    fn python_setup_creates_main_without_overwriting_it() {
        let root = test_root("python-main");
        let runner = PythonRunner::new(root.clone());
        let main_path = runner.solver_file_path(&SolverFile::Entrypoint);

        runner.setup_solver().expect("set up fresh Python solver");
        assert_eq!(
            fs::read_to_string(&main_path).expect("read generated Python main"),
            runner.main_contents()
        );

        fs::write(&main_path, "custom main").expect("replace generated Python main");
        runner
            .setup_solver()
            .expect("set up existing Python solver");
        assert_eq!(
            fs::read_to_string(&main_path).expect("read preserved Python main"),
            "custom main"
        );

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn python_solution_template_interpolates_input_length() {
        let root = test_root("python-template");
        let runner = PythonRunner::new(root.clone());

        let solution = runner
            .ensure_solver_file(&SolverFile::PuzzleSolution(1, 2024))
            .expect("create Python solution from template");
        let contents = fs::read_to_string(solution).expect("read generated Python solution");

        assert!(contents.contains("Input length: {len(input)}"));
        assert!(!contents.contains("Input length: {{len(input)}}"));

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn result_files_are_unique_and_cleaned_after_failures() {
        let root = test_root("results");
        let runs_dir = root.join("runs");
        fs::create_dir_all(&runs_dir).expect("create runs directory");
        let stale_result = root.join("result.json");
        fs::write(&stale_result, "stale result").expect("write stale legacy result");

        let malformed_result = new_result_file_path(&runs_dir).expect("allocate result path");
        assert_ne!(malformed_result, stale_result);
        assert!(!malformed_result.exists());
        fs::write(&malformed_result, "not JSON").expect("write malformed result");
        assert!(with_result_file(&malformed_result, read_result).is_err());
        assert!(!malformed_result.exists());
        assert!(stale_result.exists());

        let failed_result = new_result_file_path(&runs_dir).expect("allocate result path");
        fs::write(&failed_result, "partial result").expect("write partial result");
        let failure: crate::AocLanguageResult<()> = with_result_file(&failed_result, |_| {
            Err(AocLanguageError::Command("solver failed".to_string()))
        });
        assert!(failure.is_err());
        assert!(!failed_result.exists());

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn generated_harnesses_publish_results_atomically() {
        let root = test_root("atomic-harnesses");
        fs::create_dir_all(&root).expect("create test runtime");
        let rust = RustRunner::new(root.clone());
        let python = PythonRunner::new(root.clone());

        assert!(rust.main_contents().contains("fs::rename"));
        assert!(python.main_contents().contains("os.replace"));

        fs::remove_dir_all(root).expect("remove test runtime");
    }
}
