use crate::{AocCliResult, AocCommand, ConfigCommand};
use aocsuite_client::{AocHttp, AocPage, DownloadMode, ParserType, open_puzzle_page};
use aocsuite_config::{AocConfig, AocConfigError, ConfigOpt};
use aocsuite_editor::edit_files;
use aocsuite_fs::{AocDataDir, AocDataFile, write_with_confirmation};
use aocsuite_lang::{Language, LanguageFile, compile, get_language_file, run, scaffold};
use aocsuite_utils::{Exercise, PuzzleDay, PuzzleYear, valid_puzzle_release, valid_year_release};

pub fn run_aocsuite(command: AocCommand, day: PuzzleDay, year: PuzzleYear) -> AocCliResult<()> {
    let config = AocConfig::new();
    match command {
        AocCommand::New {
            template_dir,
            language,
            overwrite,
        } => {
            valid_puzzle_release(day, year)?;
            run_template(day, year, language, template_dir, overwrite, &config)?;
            download_files(day, year, DownloadMode::All, overwrite)?
        }

        AocCommand::Template {
            template_dir,
            language,
            overwrite,
        } => {
            valid_puzzle_release(day, year)?;
            run_template(day, year, language, template_dir, overwrite, &config)?;
        }

        AocCommand::Download { mode, overwrite } => {
            valid_puzzle_release(day, year)?;
            download_files(day, year, mode, overwrite)?
        }

        AocCommand::Config { command } => run_config(command)?,

        AocCommand::Calendar => {
            valid_year_release(day, year)?;
            let client = get_http_client()?;
            let calendar = client.get_cleaned(AocPage::Calendar(year), ParserType::Colored)?;
            println!("{calendar}");
        }

        AocCommand::Open => {
            valid_puzzle_release(day, year)?;
            open_puzzle_page(day, year)?;
        }

        AocCommand::Submit { part, answer } => {
            valid_puzzle_release(day, year)?;
            let client = get_http_client()?;
            let output = client.post_answer(&answer, part, day, year)?;
            println!("{output}");
        }

        AocCommand::Run { language, part } => {
            valid_puzzle_release(day, year)?;
            let language = resolve_language(language, &config)?;
            run_wrapped(day, year, language, part, None)?
        }

        AocCommand::Test {
            language,
            input_file,
            part,
        } => {
            let language = resolve_language(language, &config)?;
            let input = input_file.or_else(|| Some(AocDataFile::Example(day, year).to_string()));
            run_wrapped(day, year, language, part, input)?
        }

        AocCommand::Edit { language } => {
            let language = resolve_language(language, &config)?;
            let config = AocConfig::new();
            let editor_type = config.get_ok(ConfigOpt::Editor)?;
            let lib_file = get_language_file(day, year, &language, LanguageFile::Lib);

            edit_files(
                &editor_type,
                &AocDataFile::Puzzle(day, year).to_string(),
                &AocDataFile::Example(day, year).to_string(),
                lib_file.to_str().expect("Valid UTF-8"),
                &AocDataFile::Input(day, year).to_string(),
            )?;
        }
    }
    Ok(())
}

fn download_files(
    day: PuzzleDay,
    year: PuzzleYear,
    mode: DownloadMode,
    overwrite: bool,
) -> AocCliResult<()> {
    valid_puzzle_release(day, year)?;
    let client = get_http_client()?;
    std::fs::create_dir_all(AocDataDir::new(day, year).to_string())?;

    if matches!(mode, DownloadMode::All | DownloadMode::Input) {
        let input = client.get(AocPage::Input(day, year))?;
        write_with_confirmation(AocDataFile::Input(day, year).to_string(), input, overwrite)?;
    }

    if matches!(mode, DownloadMode::All | DownloadMode::Puzzle) {
        let puzzle = client.get_cleaned(AocPage::Puzzle(day, year), ParserType::Markdown)?;
        write_with_confirmation(
            AocDataFile::Puzzle(day, year).to_string(),
            puzzle,
            overwrite,
        )?;
    }

    Ok(())
}

fn run_template(
    day: PuzzleDay,
    year: PuzzleYear,
    language: Option<Language>,
    template_dir: Option<String>,
    overwrite: bool,
    config: &AocConfig,
) -> AocCliResult<()> {
    let language = resolve_language(language, config)?;
    let template_dir = template_dir.or(config.get(ConfigOpt::TemplateDir));
    scaffold(day, year, &language, template_dir.as_deref(), overwrite)?;
    Ok(())
}

fn run_config(command: ConfigCommand) -> AocCliResult<()> {
    let mut config = AocConfig::new();
    match command {
        ConfigCommand::Get(opts) => {
            let val = config.get_ok(opts.key)?;
            println!("{val}");
        }
        ConfigCommand::Set(opts) => config.set(opts.key, opts.value)?,
    }
    Ok(())
}

fn get_http_client() -> AocCliResult<AocHttp> {
    let config = AocConfig::new();
    let session = config.get(ConfigOpt::Session).ok_or(AocConfigError::Get {
        key: ConfigOpt::Session,
    })?;
    AocHttp::new(&session).map_err(Into::into)
}

fn run_wrapped(
    day: PuzzleDay,
    year: PuzzleYear,
    language: Language,
    part: Option<Exercise>,
    input_path: Option<String>,
) -> AocCliResult<()> {
    let part = part.map_or("both".to_string(), |exercise| exercise.to_string());
    let path = input_path.unwrap_or_else(|| AocDataFile::Input(day, year).to_string());

    compile(day, year, &language)?;
    let result = run(day, year, &part, &language, path.as_ref())?;
    if let Some((res, duration_ms)) = result {
        println!("{res}");
        println!("Took {duration_ms} ms");
    }
    Ok(())
}
fn resolve_language(language: Option<Language>, config: &AocConfig) -> AocCliResult<Language> {
    Ok(language.unwrap_or(config.get_ok(ConfigOpt::Language)?.parse()?))
}
