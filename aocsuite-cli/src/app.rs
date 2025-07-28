use std::{io::Write, path::PathBuf};

use crate::{AocCliResult, AocCommand, ConfigCommand};
use aocsuite_client::{open_puzzle_page, post_answer};
use aocsuite_config::{get_config_val, set_config_val};
use aocsuite_editor::open_solution_files;
use aocsuite_fs::{update_cache_status, AocContentFile};
use aocsuite_lang::{compile, get_path, run, SolveFile};
use aocsuite_parser::{parse, parse_submission_result, ParserType};
use aocsuite_utils::{valid_puzzle_release, valid_year_release, Exercise, PuzzleDay, PuzzleYear};

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
            open_puzzle_page(day, year)?;
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

            compile(day, year, &language)?;
            let result = run(day, year, &part, path.as_ref(), &language)?;
            println!("{result}");
        }

        AocCommand::Open { language } => {
            valid_puzzle_release(day, year)?;
            let solve_path = get_path(
                &SolveFile::LinkedSolution(Box::new(SolveFile::Solution(day, year))),
                &language,
            )?;

            open_solution_files(
                &AocContentFile::puzzle(day, year).to_path()?,
                &AocContentFile::example(day, year).to_path()?,
                &solve_path,
                &AocContentFile::input(day, year).to_path()?,
            )?;
        }
        AocCommand::Template { language, reset } => {
            if reset {
                let template_path = get_path(&SolveFile::TemplateSolution, &language)?;
                let prompt = format!("Are you sure you want to delete template file? (Y/n):",);
                if user_confirm(&prompt)? {
                    std::fs::remove_file(template_path)?;
                }
            }
            let edit_file = SolveFile::LinkedSolution(Box::new(SolveFile::TemplateSolution));
            let path = get_path(&edit_file, &language)?;
            aocsuite_editor::open(&path)?;
        }
        _ => unimplemented!(),
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
