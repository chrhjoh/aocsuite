use crate::{AocCliResult, AocCommand, ConfigCommand};
use aocsuite_client::{AocHttp, AocPage};
use aocsuite_config::{AocConfig, ConfigOpt};
use aocsuite_lang::scaffold;
use aocsuite_utils::{PuzzleDay, PuzzleYear, valid_puzzle_release};

pub fn run_aocsuite(command: AocCommand, day: PuzzleDay, year: PuzzleYear) -> AocCliResult<()> {
    match command {
        AocCommand::Template {
            directory,
            language,
        } => {
            valid_puzzle_release(day, year)?;
            scaffold(day, year, &language, directory.as_deref())?;
            Ok(())
        }
        AocCommand::Download { mode } => {
            let config = AocConfig::new();
            let session = config.get(ConfigOpt::Session)?;
            let http_client = AocHttp::new(session)?;
            let page = AocPage::Puzzle(day, year);
            println!("{}", http_client.get(page)?);
            Ok(())
        }
        AocCommand::Config { command } => {
            let mut config = AocConfig::new();
            match command {
                ConfigCommand::Get(opts) => {
                    let val = config.get(opts.key)?;
                    println!("{val}");
                    Ok(())
                }
                ConfigCommand::Set(opts) => {
                    config.set(opts.key, opts.value)?;
                    Ok(())
                }
            }
        }
        _ => unimplemented!(),
    }
}
