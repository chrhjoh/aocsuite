use std::io::{BufRead, Write};

use aocsuite_cli::{run_aocsuite, AocCliError, AocCommand, ConfigCommand, ConfigCommandKey};
use aocsuite_client::{AocClient, AocClientOptions};
use aocsuite_config::{AocConfigError, ConfigKey, Configuration};
use aocsuite_storage::{get_aocsuite_dir, ContentStore, RuntimeLayout, Workspace};
use aocsuite_utils::{default_puzzle_date, PuzzleDay, PuzzleYear, SystemCommandExecutor};

use clap::Parser;

/// Advent of Code tool for downloading, executing, submitting, etc...
#[derive(Parser, Debug)]
struct AocCli {
    #[command(subcommand)]
    /// Command to execute
    command: AocCommand,

    /// Specify day for exercises etc. (default: latest released)
    #[arg(long)]
    day: Option<PuzzleDay>,

    /// Specify year for calendar, exercises, etc (default: latest released or configured)
    #[arg(long)]
    year: Option<PuzzleYear>,
}

fn terminate_with_error(err: AocCliError) -> ! {
    eprintln!("encountered error: {err}");
    std::process::exit(1);
}

fn main() {
    let args = AocCli::parse();
    let root = get_aocsuite_dir().unwrap_or_else(|error| terminate_with_error(error.into()));
    let layout =
        RuntimeLayout::new(&root).unwrap_or_else(|error| terminate_with_error(error.into()));
    if matches!(&args.command, AocCommand::Uninstall) {
        if confirm_uninstall().unwrap_or_else(|error| terminate_with_error(error.into())) {
            layout
                .uninstall()
                .unwrap_or_else(|error| terminate_with_error(error.into()));
        }
        return;
    }
    layout
        .bootstrap()
        .unwrap_or_else(|error| terminate_with_error(error.into()));
    let workspace = Workspace::new(layout.workspace_dir());
    workspace
        .ensure()
        .unwrap_or_else(|error| terminate_with_error(error.into()));
    let mut config = Configuration::load(layout.config_dir())
        .unwrap_or_else(|error| terminate_with_error(error.into()));
    if let AocCommand::Config { command } = args.command {
        run_config_command(command, &mut config)
            .unwrap_or_else(|error| terminate_with_error(error));
        return;
    }
    let session = match config.session() {
        Ok(session) => Some(session),
        Err(AocConfigError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => terminate_with_error(error.into()),
    };
    let client = AocClient::new(session.as_deref(), AocClientOptions::default())
        .unwrap_or_else(|error| terminate_with_error(error.into()));
    let content = ContentStore::open(layout.cache_dir(), &client)
        .unwrap_or_else(|error| terminate_with_error(error.into()));
    let configured_year = match args.year {
        Some(year) => Some(year),
        None => match config.get::<PuzzleYear>(ConfigKey::Year) {
            Ok(year) => Some(year),
            Err(aocsuite_config::AocConfigError::NotFound { .. }) => None,
            Err(error) => terminate_with_error(error.into()),
        },
    };
    let (day, year) = resolve_puzzle_date(args.day, configured_year, default_puzzle_date());
    let executor = SystemCommandExecutor;
    if let Err(err) = run_aocsuite(
        args.command,
        day,
        year,
        &client,
        &content,
        &workspace,
        &mut config,
        &executor,
    ) {
        terminate_with_error(err);
    }
}

fn confirm_uninstall() -> std::io::Result<bool> {
    let mut output = std::io::stdout().lock();
    write!(
        output,
        "Are you sure you want to delete everything in AoCSuite.\nThis includes any solutions you may have made (Y/n) : "
    )?;
    output.flush()?;

    let mut response = String::new();
    if std::io::stdin().lock().read_line(&mut response)? == 0 {
        return Ok(false);
    }
    let response = response.trim().to_ascii_lowercase();
    Ok(response.is_empty() || response == "y" || response == "yes")
}

fn run_config_command(
    command: ConfigCommand,
    config: &mut Configuration,
) -> Result<(), AocCliError> {
    match command {
        ConfigCommand::Get { key } => {
            let value = config.get::<String>(key.into())?;
            println!("{key}: {value}");
        }
        ConfigCommand::Set { key } => set_config_value(config, key)?,
    }
    Ok(())
}

fn set_config_value(config: &mut Configuration, key: ConfigCommandKey) -> Result<(), AocCliError> {
    if matches!(key, ConfigCommandKey::Session) {
        let value = rpassword::prompt_password("Enter value for session: ")?;
        config.set(
            key.into(),
            (!value.trim().is_empty()).then_some(value.as_str()),
        )?;
        return Ok(());
    }

    let config_key = key.into();
    match config.get::<String>(config_key) {
        Ok(value) => print!("Enter value for {key} [{value}]: "),
        Err(AocConfigError::NotFound { .. }) => print!("Enter value for {key}: "),
        Err(error) => return Err(error.into()),
    }
    std::io::stdout().flush()?;
    let mut value = String::new();
    std::io::stdin().read_line(&mut value)?;
    let value = (!value.trim().is_empty()).then_some(value.as_str());
    config.set(config_key, value)?;
    Ok(())
}

fn resolve_puzzle_date(
    requested_day: Option<PuzzleDay>,
    configured_year: Option<PuzzleYear>,
    default: (PuzzleDay, PuzzleYear),
) -> (PuzzleDay, PuzzleYear) {
    let year = configured_year.unwrap_or(default.1);
    let day = requested_day.unwrap_or_else(|| {
        if year == default.1 {
            default.0
        } else if year.get() == 2025 {
            PuzzleDay::new(12).expect("valid final puzzle day for 2025")
        } else {
            PuzzleDay::new(25).expect("valid final puzzle day")
        }
    });
    (day, year)
}

#[cfg(test)]
mod tests {
    use super::resolve_puzzle_date;
    use aocsuite_utils::{PuzzleDay, PuzzleYear};

    fn puzzle(day: u32, year: i32) -> (PuzzleDay, PuzzleYear) {
        (
            PuzzleDay::new(day).expect("valid test day"),
            PuzzleYear::new(year).expect("valid test year"),
        )
    }

    #[test]
    fn configured_year_overrides_the_default_year() {
        assert_eq!(
            resolve_puzzle_date(None, Some(puzzle(1, 2024).1), puzzle(20, 2026)),
            puzzle(25, 2024)
        );
        assert_eq!(
            resolve_puzzle_date(None, Some(puzzle(1, 2025).1), puzzle(20, 2026)),
            puzzle(12, 2025)
        );
    }
}
