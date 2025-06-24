use chrono::Datelike;

use crate::{
    AocCommand, AocResult,
    scaffold::scaffold,
    utils::{PuzzleDay, PuzzleYear, today, valid_puzzle_release},
};
pub fn run_aocsuite(
    command: AocCommand,
    day: Option<PuzzleDay>,
    year: Option<PuzzleYear>,
) -> AocResult<()> {
    let day = day.unwrap_or_else(|| today().day());
    let year = year.unwrap_or_else(|| today().year());
    run_command(command, day, year)
}

fn run_command(command: AocCommand, day: PuzzleDay, year: PuzzleYear) -> AocResult<()> {
    match command {
        AocCommand::New { template, language } => {
            valid_puzzle_release(day, year)?;
            scaffold(day, year, template, &language)
        }
        _ => unimplemented!(),
    }
}
