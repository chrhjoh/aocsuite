use std::{io::Write, path::PathBuf};

use crate::{
    commands::{CleanAction, EnvAction, LibAction},
    git::{get_gitignore_path, run_git_command},
    AocCliResult, AocCommand, ConfigCommand,
};
use aocsuite_client::{open_page, post_answer, AocPage};
use aocsuite_config::{get_config_val, set_config_val};
use aocsuite_editor::open_solution_files;
use aocsuite_fs::{update_cache_status, AocContentFile};
use aocsuite_lang::{Language, SolverFile};
use aocsuite_parser::{parse, parse_submission_result, ParserType};
use aocsuite_utils::{
    get_aocsuite_dir, valid_puzzle_release, valid_year_release, Exercise, PuzzleDay, PuzzleYear,
};

pub fn run_aocsuite(command: AocCommand, day: PuzzleDay, year: PuzzleYear) -> AocCliResult<()> {
    match command {
        AocCommand::Config { command } => match command {
            ConfigCommand::Get { key } => {
                let val: String = get_config_val(&key, None, None)?;
                println!("{}: {val}", key.to_string());
            }
            ConfigCommand::Set { key } => set_config_val(&key)?,
        },

        AocCommand::Calendar => {
            valid_year_release(day, year)?;
            let calendar = AocContentFile::calendar(year).load()?;
            let parsed_calendar = parse(&calendar, ParserType::Colored);
            println!("{parsed_calendar}");
        }

        AocCommand::View => {
            valid_puzzle_release(day, year)?;
            open_page(&AocPage::Puzzle(day, year))?;
        }

        AocCommand::Submit { part, answer } => {
            valid_puzzle_release(day, year)?;
            let output = post_answer(&answer, &part, day, year)?;
            let result = parse_submission_result(&output);
            update_cache_status(&result, day, year, part == Exercise::One);
            println!("{result}");
        }

        AocCommand::Run {
            language,
            part,
            test,
        } => {
            valid_puzzle_release(day, year)?;
            let part = part.map_or("both".to_string(), |exercise| exercise.to_string());
            let path = match test {
                Some(file) => {
                    if file == "" {
                        AocContentFile::example(day, year).to_path()?
                    } else {
                        PathBuf::from(file)
                    }
                }
                None => AocContentFile::input(day, year).to_path()?,
            };

            let language = Language::resolve(&language)?;
            language.compile(day, year)?;
            let result = language.run(day, year, &part, path.as_ref())?;
            println!("{result}");
        }

        AocCommand::Open { language } => {
            valid_puzzle_release(day, year)?;
            let language = Language::resolve(&language)?;
            let solve_path = language.prepare_solver_file(&SolverFile::ActiveSolution(day, year))?;
            let env_vars = language.editor_environment_vars()?;

            open_solution_files(
                &AocContentFile::puzzle(day, year).to_path()?,
                &AocContentFile::example(day, year).to_path()?,
                &solve_path,
                &AocContentFile::input(day, year).to_path()?,
                Some(env_vars),
            )?;
        }
        AocCommand::Template { language, reset } => {
            let language = Language::resolve(&language)?;
            if reset {
                let template_path = language.prepare_solver_file(&SolverFile::SolutionTemplate)?;
                if user_confirm("Are you sure you want to delete template file? (Y/n):")? {
                    std::fs::remove_file(template_path)?;
                }
            }
            let path = language.prepare_solver_file(&SolverFile::SolutionTemplate)?;
            let env_vars = language.editor_environment_vars()?;
            aocsuite_editor::open(&path, Some(env_vars))?;
        }
        AocCommand::Git { args } => {
            let output = run_git_command(&args)?;
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        AocCommand::GitIgnore {} => {
            let path = get_gitignore_path()?;
            aocsuite_editor::open(&path, None)?;
        }
        AocCommand::Env { action, language } => {
            let language = Language::resolve(&language)?;
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
            let language = Language::resolve(&language)?;
            match action {
            LibAction::Edit { lib } => {
                let path = language.get_lib_filepath(&lib)?;
                let env_vars = language.editor_environment_vars()?;
                aocsuite_editor::open(&path, Some(env_vars))?;
            }
            LibAction::Remove { lib, all, force } => {
                let language_name = language.name();
                if all {
                    let files = language.list_lib_files()?;
                    if files.len() == 0 {
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
                            &lib, language_name
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
            open_page(&AocPage::Leaderboard(year, id))?;
        }

        AocCommand::Clean { action } => match action {
            CleanAction::Cache {
                all,
                year_all,
                force,
            } => {
                let clean_day: Option<PuzzleDay>;
                let clean_year_opt: Option<PuzzleYear>;
                let file_prompt: String;

                if all {
                    clean_day = None;
                    clean_year_opt = None;
                    file_prompt = "all cached AoC files".to_string()
                } else if year_all {
                    clean_day = None;
                    clean_year_opt = Some(year);
                    file_prompt = format!("all cached AoC files for {year}").to_string()
                } else {
                    clean_day = Some(day);
                    clean_year_opt = Some(year);
                    file_prompt =
                        format!("all cached AoC files for day {day} in {year}").to_string()
                }
                if 
            user_confirm_or_force(&format!(
                "Do you want to delete {file_prompt} (puzzles, inputs, examples and calendar) (Y/n) : ",
            ), force)?
        {
        aocsuite_fs::clean_cache(clean_year_opt, clean_day)?;
        }
            }

            CleanAction::Lang { language, force } => {
                let language = Language::resolve(&language)?;
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
            let aocsuite_dir = get_aocsuite_dir();
            println!(
                "Ensure you have backed up any solutions. Files can be found at {:?}",
                aocsuite_dir
            );
            if user_confirm("Are you sure you want to delete everything in AoCSuite.\nThis includes any solutions you may have made (Y/n) : ")?{

                std::fs::remove_dir_all(aocsuite_dir)?;
                println!("Removed the AoCSuite directory")
                
            }
        }
    }
    Ok(())
}
fn user_confirm(prompt: &str) -> std::io::Result<bool> {
    print!("{prompt}");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y" || trimmed == "yes")
}

fn user_confirm_or_force(prompt: &str, force: bool) -> std::io::Result<bool> {
    if force {
        return Ok(true);
    }
    return user_confirm(prompt);
}
