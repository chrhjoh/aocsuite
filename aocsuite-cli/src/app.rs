use std::{
    fmt::Write as _,
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use crate::{
    commands::{CleanAction, EnvAction, LibAction},
    AocCliError, AocCliResult, AocCommand,
};
use aocsuite_client::{AocClient, AocPage};
use aocsuite_config::{ConfigKey, Configuration};
use aocsuite_lang::{ConfirmedTemplateReset, Language, LanguageRunOutput, PartResult, SolverFile};
use aocsuite_launcher::{Launcher, OpenPuzzleRequest};
use aocsuite_parser::{parse_calendar, parse_submission, AocSubmissionResult, Calendar};
use aocsuite_storage::{CacheCleanScope, ContentStore, GitMode, Workspace};
use aocsuite_utils::{
    valid_puzzle_release, valid_year_release, CommandExecutor, LanguageId, PartSelection,
    PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear, RunHistoryLimit,
};
use colored::Colorize;

#[allow(clippy::too_many_arguments)]
pub fn run_aocsuite(
    command: AocCommand,
    day: PuzzleDay,
    year: PuzzleYear,
    client: &AocClient,
    content: &ContentStore,
    workspace: &Workspace,
    config: &mut Configuration,
    executor: &dyn CommandExecutor,
) -> AocCliResult<()> {
    let launcher = Launcher::new(executor);
    match command {
        AocCommand::Config { .. } => {
            return Err(AocCliError::NotAllowed(
                "config must be handled before content service construction",
            ));
        }

        AocCommand::Calendar => {
            valid_year_release(day, year)?;
            let calendar = content.load_calendar(year)?;
            println!("{}", render_calendar(&parse_calendar(&calendar)?));
        }

        AocCommand::View => {
            valid_puzzle_release(day, year)?;
            launcher.open_browser(&AocPage::Puzzle(PuzzleId::new(day, year)).to_string())?;
        }

        AocCommand::Submit { part, answer } => {
            valid_puzzle_release(day, year)?;
            let answer = match answer {
                Some(answer) => answer,
                None => prompt_answer()?,
            };
            let puzzle = PuzzleId::new(day, year);
            let result = parse_submission(&client.submit(puzzle, part, &answer)?)?;
            content.record_submission(puzzle, part, &result)?;
            println!("{}", format_submission_result(&result));
        }

        AocCommand::Run {
            language,
            part,
            test,
        } => {
            valid_puzzle_release(day, year)?;
            let part = part.map_or(PartSelection::Both, PartSelection::from);
            let path = match test {
                Some(file) => {
                    if file.is_empty() {
                        workspace.ensure_example(PuzzleId::new(day, year))?
                    } else {
                        resolve_custom_input_path(&file, &std::env::current_dir()?)?
                    }
                }
                None => content.ensure_input(PuzzleId::new(day, year))?,
            };

            let language = resolve_language(config, language, workspace, executor)?;
            let run_history_limit = config.get::<RunHistoryLimit>(ConfigKey::RunHistoryLimit)?;
            let puzzle = PuzzleId::new(day, year);
            let run = language.execute(puzzle, part, path.as_ref())?;
            for part in [PuzzlePart::One, PuzzlePart::Two] {
                if let Some(result) = run.run.result.part(part) {
                    content.record_run_timing(
                        puzzle,
                        language.language_id(),
                        part,
                        result.runtime_ms(),
                        run_history_limit,
                    )?;
                }
            }
            print!("{}", render_language_run(&run));
        }

        AocCommand::Open { language } => {
            valid_puzzle_release(day, year)?;
            let language = resolve_language(config, language, workspace, executor)?;
            let editor_program = config.get::<String>(ConfigKey::Editor)?;
            let puzzle = PuzzleId::new(day, year);
            let request = OpenPuzzleRequest {
                puzzle: content.ensure_puzzle_markdown(puzzle)?,
                example: workspace.ensure_example(puzzle)?,
                solution: language.ensure_solver_file(&SolverFile::ActiveSolution(puzzle))?,
                input: content.ensure_input(puzzle)?,
                working_directory: language.project_dir().to_path_buf(),
            };
            launcher.open_puzzle(editor_program, request)?;
        }
        AocCommand::Template { language, reset } => {
            let language = resolve_language(config, language, workspace, executor)?;
            let editor_program = config.get::<String>(ConfigKey::Editor)?;
            let template_path = language.ensure_solver_file(&SolverFile::SolutionTemplate)?;
            let path = if reset
                && user_confirm(
                    &mut std::io::stdin().lock(),
                    &mut std::io::stdout().lock(),
                    "Are you sure you want to delete template file? (Y/n):",
                )? {
                language.reset_template(ConfirmedTemplateReset::Confirmed)?
            } else {
                template_path
            };
            launcher.open_file(editor_program, &path, language.project_dir())?;
        }
        AocCommand::Git { args } => {
            let mode = if is_interactive_git_command(&args) {
                GitMode::Foreground
            } else {
                GitMode::Captured
            };
            let output = workspace.run_git(&args, mode, executor)?;
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        AocCommand::GitIgnore => {
            let editor_program = config.get::<String>(ConfigKey::Editor)?;
            workspace.ensure()?;
            let path = workspace.gitignore_path();
            launcher.open_file(editor_program, &path, workspace.root_dir())?;
        }
        AocCommand::Env { action, language } => {
            let language = resolve_language(config, language, workspace, executor)?;
            match action {
                EnvAction::Add { package } => {
                    language.add_package(&package)?;
                    println!("Added package: {}", package);
                }
                EnvAction::Remove { package } => {
                    language.remove_package(&package)?;
                    println!("Removed package: {}", package);
                }
                EnvAction::List => {
                    let packages = language.list_packages()?;
                    if packages.is_empty() {
                        println!("No packages installed");
                    } else {
                        println!("Installed packages:");
                        for package in packages {
                            println!("  {}", package);
                        }
                    }
                }
                EnvAction::Clean { force } => {
                    if user_confirm_or_force(
                        "Are you sure you want to delete your current environment (Y/n): ",
                        force,
                    )? {
                        language.clean_env()?;
                    }
                }
            }
        }
        AocCommand::Lib { action, language } => {
            let language = resolve_language(config, language, workspace, executor)?;
            match action {
                LibAction::Edit { lib } => {
                    let editor_program = config.get::<String>(ConfigKey::Editor)?;
                    let path = language.ensure_lib_path(&lib)?;
                    launcher.open_file(editor_program, &path, language.project_dir())?;
                }
                LibAction::Remove { lib, all, force } => {
                    let language_name = language.name();
                    if all {
                        let files = language.list_lib_files()?;
                        if files.is_empty() {
                            println!("No library files found");
                            return Ok(());
                        }
                        if user_confirm_or_force(
                            &format!(
                                "Do you want to delete {} libary files for {} (Y/n) : ",
                                files.len(),
                                language_name
                            ),
                            force,
                        )? {
                            for lib in files.iter() {
                                language.remove_lib_file(lib)?
                            }
                        }
                    } else {
                        let lib = lib.expect("Lib only none when all is false");
                        if !language.library_exists(&lib)? {
                            println!("Library file {lib} was not found");
                            return Ok(());
                        }
                        if user_confirm_or_force(
                            &format!(
                                "Do you want to delete the library {} for {} (Y/n) : ",
                                lib, language_name
                            ),
                            force,
                        )? {
                            language.remove_lib_file(&lib)?;
                            println!("Removed library: {} for {}", lib, language_name);
                        }
                    }
                }
                LibAction::List => {
                    let files = language.list_lib_files()?;
                    if files.is_empty() {
                        println!("No library files found");
                    } else {
                        println!("Current library names:");
                        for package in files {
                            println!("  {}", package);
                        }
                    }
                }
            }
        }
        AocCommand::Leaderboard { id } => {
            valid_year_release(day, year)?;
            launcher.open_browser(&AocPage::Leaderboard(year, id).to_string())?;
        }

        AocCommand::Clean { action } => match action {
            CleanAction::Cache {
                all,
                year: clean_year,
                force,
            } => {
                let clean_scope: CacheCleanScope;
                let prompt: String;

                if all {
                    clean_scope = CacheCleanScope::All;
                    prompt = "all cached AoC files".to_string();
                } else if clean_year {
                    clean_scope = CacheCleanScope::Year(year);
                    prompt = format!("all cached AoC files for {year}");
                } else {
                    clean_scope = CacheCleanScope::Date(PuzzleId::new(day, year));
                    prompt = format!("all cached AoC files for day {day} in {year}");
                }
                if user_confirm_or_force(
                    &format!("Do you want to delete {prompt} (Y/n) : ",),
                    force,
                )? {
                    content.clean(clean_scope)?;
                }
            }

            CleanAction::Lang { language, force } => {
                let language = resolve_language(config, language, workspace, executor)?;
                let language_name = language.name();
                if user_confirm_or_force(
                    &format!(
                        "Do you want to remove generated runtime files for {}  (Y/n) : ",
                        language_name
                    ),
                    force,
                )? {
                    language.clean_runtime()?;
                }
            }
        },

        AocCommand::Uninstall => {
            return Err(AocCliError::NotAllowed(
                "uninstall must be handled before runtime bootstrap",
            ));
        }
    }
    Ok(())
}

fn render_calendar(calendar: &Calendar) -> String {
    calendar
        .rows
        .iter()
        .map(|row| {
            row.cells
                .iter()
                .map(|cell| {
                    cell.text
                        .truecolor(cell.color.red, cell.color.green, cell.color.blue)
                        .to_string()
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_language_run(output: &LanguageRunOutput) -> String {
    let mut rendered = String::new();
    render_stream(&mut rendered, "Compiler output", &output.compile.stdout);
    render_stream(&mut rendered, "Compiler errors", &output.compile.stderr);
    render_stream(&mut rendered, "Solver output", &output.run.stdout);
    render_stream(&mut rendered, "Solver errors", &output.run.stderr);

    let part1 = output.run.result.part(PuzzlePart::One);
    let part2 = output.run.result.part(PuzzlePart::Two);
    if let Some(part) = part1 {
        render_part(&mut rendered, "Part 1", part);
    }
    if part1.is_some() && part2.is_some() {
        rendered.push('\n');
    }
    if let Some(part) = part2 {
        render_part(&mut rendered, "Part 2", part);
    }
    rendered
}

fn render_stream(rendered: &mut String, label: &str, stream: &str) {
    if !stream.is_empty() {
        writeln!(rendered, "{label}:").expect("write to string");
        writeln!(rendered, "{}", stream.trim_end()).expect("write to string");
    }
}

fn render_part(rendered: &mut String, label: &str, part: &PartResult) {
    writeln!(rendered, "\n┌──────────────┐").expect("write to string");
    writeln!(rendered, "│   {label:<6}     │").expect("write to string");
    writeln!(rendered, "└──────────────┘").expect("write to string");
    writeln!(rendered, "Answer: {}", part.answer()).expect("write to string");
    writeln!(rendered, "Runtime: {} ms", part.runtime_ms()).expect("write to string");
}

fn is_interactive_git_command(args: &[String]) -> bool {
    match args.first().map(String::as_str) {
        Some("commit") => !args
            .iter()
            .any(|arg| arg == "-m" || arg.starts_with("--message")),
        Some("rebase") => args.iter().any(|arg| arg == "-i" || arg == "--interactive"),
        Some("add" | "checkout" | "reset") => {
            args.iter().any(|arg| arg == "-p" || arg == "--patch")
        }
        Some("difftool" | "mergetool") => true,
        _ => false,
    }
}

fn format_submission_result(result: &AocSubmissionResult) -> String {
    match result {
        AocSubmissionResult::Correct => "✅ Correct! That's the right answer!".to_owned(),
        AocSubmissionResult::AlreadyCompleted => {
            "ℹ️  You've already completed this puzzle.".to_owned()
        }
        AocSubmissionResult::IncorrectTooHigh => "❌ Your answer is too high.".to_owned(),
        AocSubmissionResult::IncorrectTooLow => "❌ Your answer is too low.".to_owned(),
        AocSubmissionResult::Incorrect => "❌ That's not the right answer.".to_owned(),
        AocSubmissionResult::RateLimited(seconds) => {
            format!("⏳ Rate limited. Please wait {seconds} seconds before submitting again.")
        }
        AocSubmissionResult::Locked => "🔒 This part of the puzzle is not yet unlocked.".to_owned(),
        AocSubmissionResult::EmptySubmission => "⚠️  You didn't provide an answer.".to_owned(),
        AocSubmissionResult::InvalidFormat => {
            "⚠️  Your answer isn't in the expected format.".to_owned()
        }
        AocSubmissionResult::Unknown(message) => format!("❓ Unknown response: {message}"),
    }
}

fn resolve_language<'workspace>(
    config: &Configuration,
    cli_arg: Option<LanguageId>,
    workspace: &'workspace Workspace,
    executor: &'workspace dyn CommandExecutor,
) -> AocCliResult<Language<'workspace, 'workspace>> {
    let language_id = match cli_arg {
        Some(language_id) => language_id,
        None => config.get::<LanguageId>(ConfigKey::Language)?,
    };
    Ok(Language::new(language_id, workspace, executor))
}

fn user_confirm(
    input: &mut impl BufRead,
    output: &mut impl Write,
    prompt: &str,
) -> std::io::Result<bool> {
    write!(output, "{prompt}")?;
    output.flush()?;

    let mut response = String::new();
    if input.read_line(&mut response)? == 0 {
        return Ok(false);
    }

    let trimmed = response.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y" || trimmed == "yes")
}

fn user_confirm_or_force(prompt: &str, force: bool) -> std::io::Result<bool> {
    if force {
        return Ok(true);
    }
    user_confirm(
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        prompt,
    )
}

fn prompt_answer() -> std::io::Result<String> {
    print!("Enter answer: ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    Ok(answer.trim().to_string())
}

fn resolve_custom_input_path(file: &str, invocation_dir: &Path) -> std::io::Result<PathBuf> {
    let path = PathBuf::from(file);
    let path = if path.is_absolute() {
        path
    } else {
        invocation_dir.join(path)
    };
    path.canonicalize()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Cursor,
        path::PathBuf,
        process,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Mutex,
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    use aocsuite_client::{AocClient, AocClientOptions};
    use aocsuite_config::{AocConfigError, ConfigKey, Configuration};
    use aocsuite_storage::{ContentStore, Workspace};
    use aocsuite_utils::{CommandExecutor, CommandRequest, ProcessMode, PuzzleDay, PuzzleYear};

    use super::{resolve_custom_input_path, run_aocsuite, user_confirm};
    use crate::AocCommand;

    static TEST_ROOT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("aocsuite-cli-{unique}-{}-{counter}", process::id()))
    }

    #[test]
    fn custom_input_is_resolved_from_the_invocation_directory() {
        let root = test_root();
        let invocation_dir = root.join("invocation");
        let language_dir = root.join("language");
        let input = invocation_dir.join("fixtures/input.txt");
        fs::create_dir_all(input.parent().expect("input has parent"))
            .expect("create input directory");
        fs::create_dir_all(&language_dir).expect("create language directory");
        fs::write(&input, "example input").expect("write input");

        let resolved = resolve_custom_input_path("fixtures/input.txt", &invocation_dir)
            .expect("resolve custom input");

        assert_eq!(resolved, input.canonicalize().expect("canonicalize input"));
        assert_ne!(resolved, language_dir.join("fixtures/input.txt"));

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn missing_custom_input_returns_an_io_error() {
        let root = test_root();
        fs::create_dir_all(&root).expect("create test runtime");

        let result = resolve_custom_input_path("missing.txt", &root);

        assert_eq!(
            result.expect_err("missing input fails").kind(),
            std::io::ErrorKind::NotFound
        );
        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn session_config_reads_are_not_allowed() {
        let root = test_root();
        fs::create_dir(&root).expect("create test runtime");
        let config = Configuration::load(&root).expect("load configuration");

        assert!(matches!(
            config.get::<String>(ConfigKey::Session),
            Err(AocConfigError::SessionReadNotAllowed)
        ));
        assert_eq!(
            config
                .get::<String>(ConfigKey::Language)
                .expect("read language"),
            "rust"
        );

        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[test]
    fn confirmations_reject_eof_but_accept_empty_and_yes() {
        for response in [b"\n".as_slice(), b"yes\n", b"Y\n"] {
            let mut output = Vec::new();
            assert!(
                user_confirm(&mut Cursor::new(response), &mut output, "Confirm? ")
                    .expect("read confirmation")
            );
            assert_eq!(output, b"Confirm? ");
        }

        let mut output = Vec::new();
        assert!(!user_confirm(&mut Cursor::new(b""), &mut output, "Confirm? ").expect("read EOF"));

        let mut output = Vec::new();
        assert!(
            !user_confirm(&mut Cursor::new(b"no\n"), &mut output, "Confirm? ")
                .expect("read rejection")
        );
    }

    #[test]
    fn run_aocsuite_uses_the_injected_executor_for_git() {
        struct FakeExecutor {
            requests: Mutex<Vec<CommandRequest>>,
        }

        impl CommandExecutor for FakeExecutor {
            fn execute(&self, request: &CommandRequest) -> std::io::Result<process::Output> {
                self.requests.lock().unwrap().push(request.clone());
                Ok(successful_output())
            }
        }

        let root = test_root();
        fs::create_dir_all(&root).expect("create test runtime");
        let client = AocClient::new(None, AocClientOptions::default()).expect("create client");
        let content = ContentStore::open(root.join("cache"), &client).expect("open content store");
        let workspace = Workspace::new(root.join("workspace"));
        let mut config = Configuration::load(root.join("config")).expect("load configuration");
        let executor = FakeExecutor {
            requests: Mutex::new(Vec::new()),
        };

        run_aocsuite(
            AocCommand::Git {
                args: vec!["status".to_string()],
            },
            PuzzleDay::new(1).expect("valid test day"),
            PuzzleYear::new(2024).expect("valid test year"),
            &client,
            &content,
            &workspace,
            &mut config,
            &executor,
        )
        .expect("run git command");

        let requests = executor.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].program, "git");
        assert_eq!(requests[0].args[0], "status");
        assert_eq!(requests[0].mode, ProcessMode::Captured);

        drop(requests);
        drop(content);
        fs::remove_dir_all(root).expect("remove test runtime");
    }

    #[cfg(unix)]
    fn successful_output() -> process::Output {
        use std::os::unix::process::ExitStatusExt;

        process::Output {
            status: process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[cfg(windows)]
    fn successful_output() -> process::Output {
        use std::os::windows::process::ExitStatusExt;

        process::Output {
            status: process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}
