use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use crate::{
    commands::{CleanAction, ConfigCommandKey, EnvAction, LibAction},
    AocCliResult, AocCommand, ConfigCommand,
};
use aocsuite_client::{AocClient, AocClientOptions, AocPage};
use aocsuite_config::{AocConfigError, ConfigKey, Configuration};
use aocsuite_lang::{Language, LanguageRunRequest, SolverFile};
use aocsuite_launcher::{Launcher, OpenPuzzleRequest};
use aocsuite_parser::{parse_calendar, parse_submission, AocSubmissionResult, Calendar};
use aocsuite_storage::{get_aocsuite_dir, CacheCleanScope, ContentStore, GitMode, Workspace};
use aocsuite_utils::{
    valid_puzzle_release, valid_year_release, CommandExecutor, LanguageId, PartSelection,
    PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear, SystemCommandExecutor,
};
use colored::Colorize;

pub fn run_aocsuite(
    command: AocCommand,
    day: PuzzleDay,
    year: PuzzleYear,
    content: &ContentStore,
    workspace: &Workspace,
    config: &mut Configuration,
) -> AocCliResult<()> {
    let executor = SystemCommandExecutor;
    let launcher = Launcher::new(&executor);
    match command {
        AocCommand::Config { command } => match command {
            ConfigCommand::Get { key } => {
                ensure_config_read_allowed(&key)?;
                let val = config.get::<String>(key.config_key().expect("session rejected"))?;
                println!("{key}: {val}");
            }
            ConfigCommand::Set { key } => set_config_value(config, key)?,
        },

        AocCommand::Calendar => {
            valid_year_release(day, year)?;
            let calendar = content.load_calendar(year, &resolve_aoc_client(config)?)?;
            println!("{}", render_calendar(&parse_calendar(&calendar)?));
        }

        AocCommand::View => {
            valid_puzzle_release(day, year)?;
            launcher.open_browser(&AocPage::Puzzle(day, year).to_string())?;
        }

        AocCommand::Submit { part, answer } => {
            valid_puzzle_release(day, year)?;
            let answer = match answer {
                Some(answer) => answer,
                None => prompt_answer()?,
            };
            let output =
                resolve_aoc_client(config)?.submit(PuzzleId::new(day, year), part, &answer)?;
            let result = parse_submission(&output)?;
            content.record_submission(PuzzleId::new(day, year), part, &result)?;
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
                None => {
                    content.ensure_input(PuzzleId::new(day, year), &resolve_aoc_client(config)?)?
                }
            };

            let language = resolve_language(config, language, workspace, &executor)?;
            let run = language.execute(LanguageRunRequest {
                puzzle: PuzzleId::new(day, year),
                part,
                input: path.as_ref(),
            })?;
            let run_history_limit = config.get::<usize>(ConfigKey::RunHistoryLimit)?;
            for part in [PuzzlePart::One, PuzzlePart::Two] {
                if let Some(result) = run.run.result.part(part) {
                    content.record_run_timing(
                        PuzzleId::new(day, year),
                        language.language_id(),
                        part,
                        result.runtime_ms(),
                        run_history_limit,
                    )?;
                }
            }
            print!("{run}");
        }

        AocCommand::Open { language } => {
            valid_puzzle_release(day, year)?;
            let client = resolve_aoc_client(config)?;
            let language = resolve_language(config, language, workspace, &executor)?;
            let editor_program = config.get::<String>(ConfigKey::Editor)?;
            let puzzle = PuzzleId::new(day, year);
            let request = OpenPuzzleRequest {
                puzzle: content.ensure_puzzle_markdown(puzzle, &client)?,
                example: workspace.ensure_example(puzzle)?,
                solution: language.ensure_solver_file(&SolverFile::ActiveSolution(puzzle))?,
                input: content.ensure_input(puzzle, &client)?,
                working_directory: language.project_dir().to_path_buf(),
            };
            launcher.open_puzzle(editor_program, request)?;
        }
        AocCommand::Template { language, reset } => {
            let language = resolve_language(config, language, workspace, &executor)?;
            let editor_program = config.get::<String>(ConfigKey::Editor)?;
            if reset {
                let template_path = language.ensure_solver_file(&SolverFile::SolutionTemplate)?;
                if user_confirm(
                    &mut std::io::stdin().lock(),
                    &mut std::io::stdout().lock(),
                    "Are you sure you want to delete template file? (Y/n):",
                )? {
                    std::fs::remove_file(template_path)?;
                }
            }
            // Ensure the template exists before opening it; recreate it after a confirmed reset.
            let path = language.ensure_solver_file(&SolverFile::SolutionTemplate)?;
            launcher.open_file(editor_program, &path, language.project_dir())?;
        }
        AocCommand::Git { args } => {
            let mode = if is_interactive_git_command(&args) {
                GitMode::Foreground
            } else {
                GitMode::Captured
            };
            let output = workspace.run_git(&args, mode, &executor)?;
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
            let language = resolve_language(config, language, workspace, &executor)?;
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
            let language = resolve_language(config, language, workspace, &executor)?;
            match action {
                LibAction::Edit { lib } => {
                    let editor_program = config.get::<String>(ConfigKey::Editor)?;
                    let path = language.get_lib_filepath(&lib)?;
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
                        let file = language.get_lib_filepath(&lib)?;
                        if !file.exists() {
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
                year_all,
                force,
            } => {
                let clean_scope: CacheCleanScope;
                let file_prompt: String;
                let content_prompt: &str;

                if all {
                    clean_scope = CacheCleanScope::All;
                    file_prompt = "all cached AoC files".to_string();
                    content_prompt = "puzzles, inputs and calendars";
                } else if year_all {
                    clean_scope = CacheCleanScope::Year(year);
                    file_prompt = format!("all cached AoC files for {year}");
                    content_prompt = "puzzles, inputs and calendar";
                } else {
                    clean_scope = CacheCleanScope::Puzzle(PuzzleId::new(day, year));
                    file_prompt = format!("all cached AoC files for day {day} in {year}");
                    content_prompt = "puzzle and input";
                }
                if user_confirm_or_force(
                    &format!("Do you want to delete {file_prompt} ({content_prompt}) (Y/n) : ",),
                    force,
                )? {
                    content.clean(clean_scope)?;
                }
            }

            CleanAction::Lang { language, force } => {
                let language = resolve_language(config, language, workspace, &executor)?;
                let language_name = language.name();
                if user_confirm_or_force(
                    &format!(
                        "Do you want to delete caches for {}  (Y/n) : ",
                        language_name
                    ),
                    force,
                )? {
                    language.clean_cache()?;
                }
            }
        },

        AocCommand::Uninstall => {
            let aocsuite_dir = get_aocsuite_dir()?;
            println!(
                "Ensure you have backed up any solutions. Files can be found at {:?}",
                aocsuite_dir
            );
            if user_confirm(
                &mut std::io::stdin().lock(),
                &mut std::io::stdout().lock(),
                "Are you sure you want to delete everything in AoCSuite.\nThis includes any solutions you may have made (Y/n) : ",
            )?{
                std::fs::remove_dir_all(aocsuite_dir)?;
                println!("Removed the AoCSuite directory")
            }
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

fn ensure_config_read_allowed(key: &ConfigCommandKey) -> AocCliResult<()> {
    if matches!(key, ConfigCommandKey::Session) {
        return Err(crate::AocCliError::NotAllowed(
            "reading the session configuration value",
        ));
    }
    Ok(())
}

fn resolve_aoc_client(config: &Configuration) -> AocCliResult<AocClient> {
    let session = match config.session() {
        Ok(session) => Some(session),
        Err(AocConfigError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    Ok(AocClient::new(
        session.as_deref(),
        AocClientOptions::default(),
    )?)
}

fn resolve_language<'workspace>(
    config: &Configuration,
    cli_arg: Option<LanguageId>,
    workspace: &'workspace Workspace,
    executor: &'workspace dyn CommandExecutor,
) -> AocCliResult<Language<'workspace, 'workspace>> {
    let language_id = cli_arg
        .map(Ok)
        .unwrap_or_else(|| config.get(ConfigKey::Language))?;
    Ok(Language::new(language_id, workspace, executor))
}

fn set_config_value(config: &mut Configuration, key: ConfigCommandKey) -> AocCliResult<()> {
    if matches!(key, ConfigCommandKey::Session) {
        let value = rpassword::prompt_password("Enter value for session: ")?;
        config.set_session((!value.trim().is_empty()).then_some(value.as_str()))?;
        return Ok(());
    }

    let config_key = key.config_key().expect("session handled separately");
    match config.get::<String>(config_key) {
        Ok(value) => print!("Enter value for {key} [{value}]: "),
        Err(AocConfigError::NotFound { .. }) => print!("Enter value for {key}: "),
        Err(error) => return Err(error.into()),
    }
    std::io::stdout().flush()?;
    let mut value = String::new();
    std::io::stdin().read_line(&mut value)?;
    config.set(
        config_key,
        (!value.trim().is_empty()).then_some(value.as_str()),
    )?;
    Ok(())
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
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        ensure_config_read_allowed, resolve_custom_input_path, user_confirm, ConfigCommandKey,
    };

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
        assert!(matches!(
            ensure_config_read_allowed(&ConfigCommandKey::Session),
            Err(crate::AocCliError::NotAllowed(_))
        ));
        assert!(ensure_config_read_allowed(&ConfigCommandKey::Language).is_ok());
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
}
