use aocsuite_cli::{run_aocsuite, AocCliError, AocCommand};
use aocsuite_config::{ConfigKey, Configuration};
use aocsuite_storage::RuntimeLayout;
use aocsuite_utils::{default_puzzle_date, PuzzleDay, PuzzleYear};

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
    let parsed = AocCli::try_parse();
    let layout = RuntimeLayout::new().unwrap_or_else(|error| terminate_with_error(error.into()));
    layout
        .bootstrap()
        .unwrap_or_else(|error| terminate_with_error(error.into()));
    let args = parsed.unwrap_or_else(|error| error.exit());
    let mut config = Configuration::load(layout.config_path(), layout.session_path())
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
    if let Err(err) = run_aocsuite(args.command, day, year, &layout, &mut config) {
        terminate_with_error(err);
    }
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
    use clap::Parser;

    use super::{resolve_puzzle_date, AocCli};
    use aocsuite_cli::{AocCommand, ConfigCommand, ConfigCommandKey};
    use aocsuite_utils::{PuzzleDay, PuzzlePart, PuzzleYear};

    fn puzzle(day: u32, year: i32) -> (PuzzleDay, PuzzleYear) {
        (
            PuzzleDay::new(day).expect("valid test day"),
            PuzzleYear::new(year).expect("valid test year"),
        )
    }

    #[test]
    fn run_part_uses_the_documented_long_option() {
        let cli =
            AocCli::try_parse_from(["aocsuite-cli", "run", "--part", "1"]).expect("parse run part");

        assert!(matches!(
            cli.command,
            AocCommand::Run {
                part: Some(PuzzlePart::One),
                ..
            }
        ));
    }

    #[test]
    fn submit_prompts_when_answer_is_omitted() {
        let cli = AocCli::try_parse_from(["aocsuite-cli", "submit", "--part", "2"])
            .expect("parse submit without answer");

        assert!(matches!(
            cli.command,
            AocCommand::Submit {
                part: PuzzlePart::Two,
                answer: None,
            }
        ));
    }

    #[test]
    fn config_keys_are_frontend_owned() {
        let cli = AocCli::try_parse_from(["aocsuite-cli", "config", "set", "session"])
            .expect("parse session configuration");
        assert!(matches!(
            cli.command,
            AocCommand::Config {
                command: ConfigCommand::Set {
                    key: ConfigCommandKey::Session
                }
            }
        ));
        assert!(AocCli::try_parse_from(["aocsuite-cli", "config", "set", "template-dir"]).is_err());
    }

    #[test]
    fn explicit_day_and_year_are_parsed() {
        let cli =
            AocCli::try_parse_from(["aocsuite-cli", "--day", "4", "--year", "2024", "calendar"])
                .expect("parse explicit puzzle date");

        let (day, year) = puzzle(4, 2024);
        assert_eq!(cli.day, Some(day));
        assert_eq!(cli.year, Some(year));
    }

    #[test]
    fn invalid_puzzle_values_are_rejected_during_parsing() {
        assert!(AocCli::try_parse_from(["aocsuite-cli", "--day", "0", "calendar"]).is_err());
        assert!(AocCli::try_parse_from(["aocsuite-cli", "--year", "2014", "calendar"]).is_err());
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
