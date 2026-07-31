mod languages;
mod python;
mod runtime;
mod rust;
mod traits;
mod utils;

use std::path::{Path, PathBuf};

use aocsuite_storage::Workspace;
use aocsuite_utils::{CommandExecutor, LanguageId, PartSelection, PuzzleId};
use utils::{read_result, with_result_file, LanguageRunner};
pub use utils::{
    AocLanguageError, AocLanguageResult, CompileOutput, PartResult, PuzzleResult, RunOutput,
    SolverFile,
};

#[derive(Debug)]
pub struct LanguageRunOutput {
    pub compile: CompileOutput,
    pub run: RunOutput,
}

pub struct Language<'workspace, 'executor> {
    language_type: LanguageId,
    project_dir: PathBuf,
    runner: LanguageRunner<'executor>,
    workspace: &'workspace Workspace,
}

impl<'workspace, 'executor> Language<'workspace, 'executor> {
    pub fn new(
        language: LanguageId,
        workspace: &'workspace Workspace,
        executor: &'executor dyn CommandExecutor,
    ) -> Self {
        let project_dir = workspace.language_project_dir(language);
        Self {
            language_type: language,
            runner: languages::to_runner(language, project_dir.clone(), executor),
            project_dir,
            workspace,
        }
    }

    pub fn execute(
        &self,
        puzzle: PuzzleId,
        part: PartSelection,
        input: &Path,
    ) -> AocLanguageResult<LanguageRunOutput> {
        self.setup_solution(puzzle)?;
        Ok(LanguageRunOutput {
            compile: self.compile()?,
            run: self.run_active(part, input)?,
        })
    }

    fn run_active(&self, part: PartSelection, input: &Path) -> AocLanguageResult<RunOutput> {
        let output_file = self.workspace.allocate_run_result_file()?;
        with_result_file(&output_file, |output_file| {
            let output = self.runner.run(part, input, output_file)?;
            Ok(RunOutput::from_output(read_result(output_file)?, output))
        })
    }

    fn compile(&self) -> AocLanguageResult<CompileOutput> {
        Ok(self
            .runner
            .compile()?
            .map(CompileOutput::from_output)
            .unwrap_or_default())
    }

    pub fn ensure_solver_file(&self, file: &SolverFile) -> AocLanguageResult<PathBuf> {
        self.runner.migrate_runtime()?;
        self.runner.ensure_solver_file(file)
    }

    pub fn project_dir(&self) -> &Path {
        &self.project_dir
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

    pub fn ensure_lib_path(&self, lib_name: &str) -> AocLanguageResult<PathBuf> {
        let lib_path = self.lib_path(lib_name)?;
        std::fs::create_dir_all(lib_path.parent().expect("library path is not root"))?;
        Ok(lib_path)
    }

    pub fn library_exists(&self, lib_name: &str) -> AocLanguageResult<bool> {
        Ok(self.lib_path(lib_name)?.is_file())
    }

    fn lib_path(&self, lib_name: &str) -> AocLanguageResult<PathBuf> {
        validate_user_lib(lib_name, &self.language_type)?;
        let lib_path = self.runner.get_lib_path(lib_name);
        ensure_no_case_collision(
            &self.runner.lib_dir(),
            &self.runner.file_extention(),
            lib_name,
        )?;
        Ok(lib_path)
    }

    pub fn remove_lib_file(&self, lib_name: &str) -> AocLanguageResult<()> {
        let lib_path = self.lib_path(lib_name)?;
        if lib_path.exists() {
            std::fs::remove_file(lib_path)?;
        }
        Ok(())
    }

    pub fn name(&self) -> String {
        self.language_type.to_string()
    }

    pub fn language_id(&self) -> LanguageId {
        self.language_type
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

    pub fn clean_runtime(&self) -> AocLanguageResult<()> {
        self.runner.clean_runtime()
    }

    pub fn clean_env(&self) -> AocLanguageResult<()> {
        self.runner.clean_env()
    }

    fn setup_solution(&self, puzzle: PuzzleId) -> AocLanguageResult<()> {
        self.runner.migrate_runtime()?;
        self.runner
            .ensure_solver_file(&SolverFile::ActiveSolution(puzzle))?;
        self.runner.setup_env()?;
        Ok(())
    }
}

fn validate_user_lib(lib_name: &str, language: &LanguageId) -> AocLanguageResult<()> {
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
        LanguageId::Rust => RUST_RESERVED_NAMES,
        LanguageId::Python => PYTHON_RESERVED_NAMES,
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
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(lib_files),
        Err(error) => return Err(error.into()),
    };
    for entry in entries {
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
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{ensure_no_case_collision, validate_user_lib, Language};
    use crate::{
        python::PythonRunner,
        rust::RustRunner,
        traits::{LanguageHandler, Solver},
        utils::{read_result, with_result_file},
        AocLanguageError, SolverFile,
    };
    use aocsuite_storage::Workspace;
    use aocsuite_utils::{
        CommandExecutor, CommandRequest, LanguageId, PartSelection, PuzzleDay, PuzzleId,
        PuzzlePart, PuzzleYear, SystemCommandExecutor,
    };

    static SYSTEM_EXECUTOR: SystemCommandExecutor = SystemCommandExecutor;

    fn puzzle_solution(day: u32) -> SolverFile {
        SolverFile::PuzzleSolution(PuzzleId::new(
            PuzzleDay::new(day).expect("valid test day"),
            PuzzleYear::new(2024).expect("valid test year"),
        ))
    }

    fn active_solution(day: u32) -> SolverFile {
        SolverFile::ActiveSolution(PuzzleId::new(
            PuzzleDay::new(day).expect("valid test day"),
            PuzzleYear::new(2024).expect("valid test year"),
        ))
    }

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
            (LanguageId::Rust, "day2", true),
            (LanguageId::Rust, "snake_case", true),
            (LanguageId::Rust, "match", false),
            (LanguageId::Rust, "Main", false),
            (LanguageId::Rust, "two-words", false),
            (LanguageId::Rust, "2fast", false),
            (LanguageId::Rust, "café", false),
            (LanguageId::Python, "day2", true),
            (LanguageId::Python, "snake_case", true),
            (LanguageId::Python, "class", false),
            (LanguageId::Python, "venv", false),
            (LanguageId::Python, "two-words", false),
            (LanguageId::Python, "2fast", false),
            (LanguageId::Python, "café", false),
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

    #[test]
    fn library_path_queries_do_not_initialize_language_projects() {
        let root = test_root("library-path");
        let workspace = Workspace::new(root.join("workspace"));
        let language = Language::new(LanguageId::Rust, &workspace, &SYSTEM_EXECUTOR);
        let expected = root.join("workspace/rust/src/helpers.rs");

        assert!(!language.library_exists("helpers").expect("check library"));
        assert_eq!(
            language.list_lib_files().expect("list libraries"),
            Vec::<String>::new()
        );
        language
            .remove_lib_file("helpers")
            .expect("remove absent library");
        assert!(!expected.parent().expect("library parent").exists());

        assert_eq!(
            language
                .ensure_lib_path("helpers")
                .expect("ensure library path"),
            expected
        );
        assert!(expected.parent().expect("library parent").is_dir());

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn runtime_cleanup_removes_only_generated_runtime_files() {
        for (language_id, entrypoint, active_solution) in [
            (LanguageId::Rust, "src/main.rs", "src/solution.rs"),
            (LanguageId::Python, "main.py", "solution.py"),
        ] {
            let root = test_root("runtime-cleanup");
            let workspace = Workspace::new(root.join("workspace"));
            let language = Language::new(language_id, &workspace, &SYSTEM_EXECUTOR);
            let project = workspace.language_project_dir(language_id);
            let entrypoint = project.join(entrypoint);
            let active_solution = project.join(active_solution);
            let manifest = project.join(".aocsuite-runtime.json");
            let preserved = project.join("preserved.txt");

            fs::create_dir_all(entrypoint.parent().expect("entrypoint parent"))
                .expect("create project directory");
            fs::write(&entrypoint, "generated entrypoint").expect("write entrypoint");
            fs::write(&manifest, "generated manifest").expect("write runtime manifest");
            fs::write(&active_solution, "user file").expect("write non-link active path");
            fs::write(&preserved, "preserved").expect("write preserved file");

            language.clean_runtime().expect("clean runtime");

            assert!(!entrypoint.exists());
            assert!(!manifest.exists());
            assert!(active_solution.exists());
            assert!(preserved.exists());
            fs::remove_dir_all(root).expect("remove test runtime");
        }
    }

    #[cfg(unix)]
    #[test]
    fn runtime_cleanup_removes_active_solution_links() {
        let root = test_root("runtime-active-link-cleanup");
        let workspace = Workspace::new(root.join("workspace"));
        let language = Language::new(LanguageId::Rust, &workspace, &SYSTEM_EXECUTOR);
        let project = workspace.language_project_dir(LanguageId::Rust);
        let source = project.join("solutions/source.rs");
        let active = project.join("src/solution.rs");

        fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
        fs::create_dir_all(active.parent().expect("active parent")).expect("create active parent");
        fs::write(&source, "solution").expect("write solution source");
        std::os::unix::fs::symlink(&source, &active).expect("create active solution link");

        language.clean_runtime().expect("clean runtime");

        assert!(source.exists());
        assert!(!active.exists());
        fs::remove_dir_all(root).expect("remove test runtime");
    }

    fn assert_requested_solution_is_active(runner: &dyn LanguageHandler) {
        let first_solution = runner
            .ensure_solver_file(&puzzle_solution(1))
            .expect("create first solution");
        fs::write(&first_solution, "first solution").expect("write first solution");
        runner
            .ensure_solver_file(&active_solution(1))
            .expect("activate first solution");

        let active_path = runner.solver_file_path(&active_solution(1));
        assert_eq!(
            fs::read_to_string(&active_path).expect("read active first solution"),
            "first solution"
        );

        let second_solution = runner
            .ensure_solver_file(&puzzle_solution(2))
            .expect("create second solution");
        fs::write(&second_solution, "second solution").expect("write second solution");
        runner
            .ensure_solver_file(&active_solution(2))
            .expect("activate second solution");

        assert_eq!(
            fs::read_to_string(&active_path).expect("read active second solution"),
            "second solution"
        );
    }

    #[test]
    fn puzzle_solutions_use_flat_language_solution_directories() {
        let rust_root = test_root("rust-path");
        let python_root = test_root("python-path");
        let solution = puzzle_solution(4);

        assert_eq!(
            RustRunner::new(rust_root.clone(), &SYSTEM_EXECUTOR).solver_file_path(&solution),
            rust_root.join("solutions/year2024_day4.rs")
        );
        assert_eq!(
            PythonRunner::new(python_root.clone(), &SYSTEM_EXECUTOR).solver_file_path(&solution),
            python_root.join("solutions/year2024_day4.py")
        );
    }

    #[test]
    fn rust_activation_selects_the_requested_solution() {
        let root = test_root("rust");
        let runner = RustRunner::new(root.clone(), &SYSTEM_EXECUTOR);

        assert_requested_solution_is_active(&runner);

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn python_activation_selects_the_requested_solution() {
        let root = test_root("python");
        fs::create_dir_all(&root).expect("create test runtime");
        let runner = PythonRunner::new(root.clone(), &SYSTEM_EXECUTOR);

        assert_requested_solution_is_active(&runner);

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn python_setup_creates_main_without_overwriting_it() {
        let root = test_root("python-main");
        let runner = PythonRunner::new(root.clone(), &SYSTEM_EXECUTOR);
        let main_path = runner.solver_file_path(&SolverFile::Entrypoint);

        runner
            .migrate_runtime()
            .expect("set up fresh Python solver");
        assert_eq!(
            fs::read_to_string(&main_path).expect("read generated Python main"),
            runner.main_contents()
        );

        fs::write(&main_path, "custom main").expect("replace generated Python main");
        runner
            .migrate_runtime()
            .expect("set up existing Python solver");
        assert_eq!(
            fs::read_to_string(&main_path).expect("read preserved Python main"),
            "custom main"
        );

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn language_execute_prepares_runtime_before_running_solver() {
        struct ScriptedExecutor {
            requests: Mutex<Vec<CommandRequest>>,
            project_dir: PathBuf,
            active_solution: PathBuf,
            entrypoint: PathBuf,
        }

        impl CommandExecutor for ScriptedExecutor {
            fn execute(&self, request: &CommandRequest) -> std::io::Result<std::process::Output> {
                self.requests.lock().unwrap().push(request.clone());
                if request
                    .args
                    .last()
                    .is_some_and(|argument| argument == "both")
                {
                    assert!(self.project_dir.join(".aocsuite-runtime.json").is_file());
                    assert!(self.entrypoint.is_file());
                    assert!(self.active_solution.exists());

                    let output_file = if request.args.len() == 4 {
                        &request.args[2]
                    } else {
                        &request.args[1]
                    };
                    std::fs::write(
                        std::path::PathBuf::from(output_file),
                        r#"{"part1":{"answer":"example","runtime_ms":3},"part2":{"answer":"8","runtime_ms":4}}"#,
                    )?;
                }
                Ok(successful_output())
            }
        }

        for (language_id, entrypoint, active_solution) in [
            (LanguageId::Rust, "src/main.rs", "src/solution.rs"),
            (LanguageId::Python, "main.py", "solution.py"),
        ] {
            let root = test_root(&format!("{language_id}-execution"));
            let workspace = Workspace::new(root.clone());
            let input = root.join("input.txt");
            let project_dir = workspace.language_project_dir(language_id);
            fs::create_dir_all(&root).expect("create test workspace");
            fs::write(&input, "example\n").expect("write input");
            let executor = ScriptedExecutor {
                requests: Mutex::new(Vec::new()),
                active_solution: project_dir.join(active_solution),
                entrypoint: project_dir.join(entrypoint),
                project_dir,
            };
            let language = Language::new(language_id, &workspace, &executor);

            let result = language
                .execute(
                    PuzzleId::new(PuzzleDay::new(1).unwrap(), PuzzleYear::new(2024).unwrap()),
                    PartSelection::Both,
                    &input,
                )
                .expect("run solution");

            assert_eq!(
                result
                    .run
                    .result
                    .part(PuzzlePart::One)
                    .expect("part one result")
                    .answer(),
                "example"
            );
            assert_eq!(
                result
                    .run
                    .result
                    .part(PuzzlePart::Two)
                    .expect("part two result")
                    .answer(),
                "8"
            );
            assert_eq!(result.run.stdout, "command output");

            fs::remove_dir_all(root).expect("remove test runtime");
        }
    }

    #[cfg(unix)]
    fn successful_output() -> std::process::Output {
        use std::os::unix::process::ExitStatusExt;

        std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"command output".to_vec(),
            stderr: Vec::new(),
        }
    }

    #[cfg(windows)]
    fn successful_output() -> std::process::Output {
        use std::os::windows::process::ExitStatusExt;

        std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"command output".to_vec(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn rust_runtime_migration_replaces_only_owned_files() {
        let root = test_root("rust-migration");
        let runner = RustRunner::new(root.clone(), &SYSTEM_EXECUTOR);
        let main = root.join("src/main.rs");
        let cargo = root.join("Cargo.toml");
        let solution = root.join("src/solution.rs");
        let library = root.join("src/helpers.rs");
        let template = root.join("template.rs");
        let puzzle = root.join("solutions/year2024_day1.rs");
        fs::create_dir_all(main.parent().expect("main parent")).expect("create source directory");
        fs::create_dir_all(puzzle.parent().expect("puzzle parent"))
            .expect("create puzzle directory");
        for path in [&main, &cargo, &solution, &library, &template, &puzzle] {
            fs::write(path, "legacy or user content").expect("write legacy fixture");
        }

        runner
            .migrate_runtime()
            .expect("migrate legacy Rust runtime");

        assert_eq!(
            fs::read_to_string(&main).expect("read main"),
            runner.main_contents()
        );
        for path in [&cargo, &solution, &library, &template, &puzzle] {
            assert_eq!(
                fs::read_to_string(path).expect("read preserved user file"),
                "legacy or user content"
            );
        }
        assert!(root.join(".aocsuite-runtime.json").exists());

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn python_runtime_migration_replaces_only_owned_files() {
        let root = test_root("python-migration");
        let runner = PythonRunner::new(root.clone(), &SYSTEM_EXECUTOR);
        let main = root.join("main.py");
        let solution = root.join("solution.py");
        let library = root.join("helpers.py");
        let template = root.join("template.py");
        let puzzle = root.join("solutions/year2024_day1.py");
        fs::create_dir_all(puzzle.parent().expect("puzzle parent"))
            .expect("create puzzle directory");
        for path in [&main, &solution, &library, &template, &puzzle] {
            fs::write(path, "legacy or user content").expect("write legacy fixture");
        }

        runner
            .migrate_runtime()
            .expect("migrate legacy Python runtime");

        assert_eq!(
            fs::read_to_string(&main).expect("read main"),
            runner.main_contents()
        );
        for path in [&solution, &library, &template, &puzzle] {
            assert_eq!(
                fs::read_to_string(path).expect("read preserved user file"),
                "legacy or user content"
            );
        }
        assert!(root.join(".aocsuite-runtime.json").exists());

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn python_solution_template_interpolates_input_length() {
        let root = test_root("python-template");
        let runner = PythonRunner::new(root.clone(), &SYSTEM_EXECUTOR);

        let solution = runner
            .ensure_solver_file(&puzzle_solution(1))
            .expect("create Python solution from template");
        let contents = fs::read_to_string(solution).expect("read generated Python solution");

        assert!(contents.contains("Input length: {len(input)}"));
        assert!(!contents.contains("Input length: {{len(input)}}"));

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn result_files_are_cleaned_after_failures() {
        let root = test_root("results");
        let runs_dir = root.join(".aocsuite-runs");
        fs::create_dir_all(&runs_dir).expect("create runs directory");
        let stale_result = root.join("result.json");
        fs::write(&stale_result, "stale result").expect("write stale legacy result");

        let malformed_result = runs_dir.join("malformed.json");
        assert_ne!(malformed_result, stale_result);
        assert!(!malformed_result.exists());
        fs::write(&malformed_result, "not JSON").expect("write malformed result");
        assert!(with_result_file(&malformed_result, read_result).is_err());
        assert!(!malformed_result.exists());
        assert!(stale_result.exists());

        let failed_result = runs_dir.join("failed.json");
        fs::write(&failed_result, "partial result").expect("write partial result");
        let failure: crate::AocLanguageResult<()> = with_result_file(&failed_result, |_| {
            Err(AocLanguageError::Clean("solver failed".to_string()))
        });
        assert!(failure.is_err());
        assert!(!failed_result.exists());

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn generated_harnesses_publish_results_atomically() {
        let root = test_root("atomic-harnesses");
        fs::create_dir_all(&root).expect("create test runtime");
        let rust = RustRunner::new(root.clone(), &SYSTEM_EXECUTOR);
        let python = PythonRunner::new(root.clone(), &SYSTEM_EXECUTOR);

        assert!(rust.main_contents().contains("fs::rename"));
        assert!(python.main_contents().contains("os.replace"));

        fs::remove_dir_all(root).expect("remove test runtime");
    }
}
