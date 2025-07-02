use crate::{AocCliResult, AocCommand, ConfigCommand};
use aocsuite_client::{AocHttp, AocPage, DownloadMode, ParserType, open_puzzle_page};
use aocsuite_config::{AocConfig, ConfigOpt};
use aocsuite_fs::{AocDataDir, AocDataFile};
use aocsuite_lang::{compile, run, scaffold};
use aocsuite_utils::{PuzzleDay, PuzzleYear, valid_puzzle_release, valid_year_release};

pub fn run_aocsuite(command: AocCommand, day: PuzzleDay, year: PuzzleYear) -> AocCliResult<()> {
    match command {
        AocCommand::New {
            template_directory,
            language,
        } => {
            valid_puzzle_release(day, year)?;
            let language = match language {
                Some(lang) => lang,
                None => AocConfig::new().get(ConfigOpt::Language)?.parse()?,
            };
            scaffold(day, year, &language, template_directory.as_deref())?;
            run_download(day, year, DownloadMode::All)?;
            Ok(())
        }

        AocCommand::Template {
            template_directory,
            language,
        } => {
            valid_puzzle_release(day, year)?;
            let language = match language {
                Some(lang) => lang,
                None => AocConfig::new().get(ConfigOpt::Language)?.parse()?,
            };
            scaffold(day, year, &language, template_directory.as_deref())?;
            Ok(())
        }

        AocCommand::Download { mode } => {
            run_download(day, year, mode)?;
            Ok(())
        }

        AocCommand::Config { command } => run_config(command),

        AocCommand::Calendar => {
            valid_year_release(day, year)?;
            let client = get_http_client()?;
            let page = AocPage::Calendar(year);
            let calendar = client.get_cleaned(page, ParserType::Colored)?;
            println!("{}", calendar);
            Ok(())
        }

        AocCommand::Open => {
            valid_puzzle_release(day, year)?;
            open_puzzle_page(day, year)?;
            Ok(())
        }

        AocCommand::Submit { part, answer } => {
            valid_puzzle_release(day, year)?;
            let client = get_http_client()?;
            let output = client.post_answer(&answer, part, day, year)?;
            println!("{}", output);
            Ok(())
        }

        AocCommand::Run { language, part } => {
            let path = AocDataFile::Input(day, year).to_string();
            let language = match language {
                Some(lang) => lang,
                None => AocConfig::new().get(ConfigOpt::Language)?.parse()?,
            };
            let part = match part {
                Some(e) => e.to_string(),
                None => "both".to_string(),
            };
            compile(day, year, &language)?;
            run(day, year, &part, &language, path.as_ref())?;
            Ok(())
        }

        AocCommand::Test {
            language,
            input_file,
            part,
        } => {
            let path = input_file.unwrap_or_else(|| AocDataFile::Example(day, year).to_string());
            let language = match language {
                Some(lang) => lang,
                None => AocConfig::new().get(ConfigOpt::Language)?.parse()?,
            };
            let part = match part {
                Some(e) => e.to_string(),
                None => "both".to_string(),
            };
            compile(day, year, &language)?;
            run(day, year, &part, &language, path.as_ref())?;
            Ok(())
        }
    }
}

fn run_download(day: PuzzleDay, year: PuzzleYear, mode: DownloadMode) -> AocCliResult<()> {
    valid_puzzle_release(day, year)?;
    let client = get_http_client()?;
    let dir = AocDataDir::new(day, year);
    std::fs::create_dir_all(dir.to_string())?;

    if matches!(mode, DownloadMode::All | DownloadMode::Input) {
        let input = client.get(AocPage::Input(day, year))?;
        std::fs::write(AocDataFile::Input(day, year).to_string(), input)?;
    }

    if matches!(mode, DownloadMode::All | DownloadMode::Puzzle) {
        let puzzle = client.get(AocPage::Puzzle(day, year))?;
        std::fs::write(AocDataFile::Puzzle(day, year).to_string(), puzzle)?;
    }

    Ok(())
}

fn run_config(command: ConfigCommand) -> AocCliResult<()> {
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

fn get_http_client() -> AocCliResult<AocHttp> {
    let config = AocConfig::new();
    let session = config.get(ConfigOpt::Session)?;
    let http = AocHttp::new(session)?;
    Ok(http)
}
