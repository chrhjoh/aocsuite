use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::{Tz, US::Eastern};
use thiserror::Error;

pub mod domain;
pub mod process;

pub use domain::{
    DomainError, LanguageId, PartSelection, PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear,
    RunHistoryLimit,
};
pub use process::{
    execute_command, CommandError, CommandExecutor, CommandRequest, ProcessMode,
    SystemCommandExecutor,
};

type AocReleaseResult<T> = Result<T, ReleaseError>;

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("Puzzle for {0} {1} has not been released yet.")]
    Puzzle(PuzzleDay, PuzzleYear),
    #[error("Advent of code has not started yet for {0}")]
    Year(PuzzleYear),
}

static ATOMIC_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn atomic_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path has no parent directory",
        )
    })?;
    let filename = path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
    })?;

    for _ in 0..16 {
        let sequence = ATOMIC_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{}.{}.{}.tmp",
            filename.to_string_lossy(),
            std::process::id(),
            sequence
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            options.mode(0o600);
        }
        let mut file = match options.open(&temporary) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };

        let result = file.write_all(contents).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = result {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }

        return match fs::rename(&temporary, path) {
            Ok(()) => Ok(()),
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                Err(error)
            }
        };
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate a temporary file for an atomic write",
    ))
}

pub fn set_owner_only_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn valid_puzzle_release(day: PuzzleDay, year: PuzzleYear) -> AocReleaseResult<()> {
    valid_puzzle_release_at(day, year, Utc::now())
}

fn valid_puzzle_release_at(
    day: PuzzleDay,
    year: PuzzleYear,
    now_utc: DateTime<Utc>,
) -> AocReleaseResult<()> {
    if !valid_puzzle_day(day, year) {
        return Err(ReleaseError::Puzzle(day, year));
    }
    let now_eastern = now_utc.with_timezone(&Eastern);
    if year.get() > now_eastern.year() {
        return Err(ReleaseError::Puzzle(day, year));
    }
    if year.get() < now_eastern.year() {
        return Ok(());
    }

    let release_date = Eastern
        .with_ymd_and_hms(year.get(), 12, u32::from(day.get()), 0, 0, 0)
        .single()
        .ok_or(ReleaseError::Puzzle(day, year))?;

    if now_eastern >= release_date {
        Ok(())
    } else {
        Err(ReleaseError::Puzzle(day, year))
    }
}

fn valid_puzzle_day(day: PuzzleDay, year: PuzzleYear) -> bool {
    year.get() != 2025 || day.get() <= 12
}

pub fn valid_year_release(_day: PuzzleDay, year: PuzzleYear) -> AocReleaseResult<()> {
    valid_year_release_at(year, Utc::now())
}

fn valid_year_release_at(year: PuzzleYear, now_utc: DateTime<Utc>) -> AocReleaseResult<()> {
    let now_eastern = now_utc.with_timezone(&Eastern);
    let now_year = now_eastern.year();

    if year.get() > now_year {
        return Err(ReleaseError::Year(year));
    }
    if year.get() == now_year {
        let release_date = Eastern
            .with_ymd_and_hms(year.get(), 12, 1, 0, 0, 0)
            .single()
            .ok_or(ReleaseError::Year(year))?;
        if now_eastern < release_date {
            return Err(ReleaseError::Year(year));
        }
    }
    Ok(())
}

pub fn today() -> DateTime<Tz> {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern)
}

/// Returns the most recently released puzzle date in US/Eastern time.
pub fn default_puzzle_date() -> (PuzzleDay, PuzzleYear) {
    default_puzzle_date_at(Utc::now())
}

pub fn default_puzzle_date_at(now_utc: DateTime<Utc>) -> (PuzzleDay, PuzzleYear) {
    let now = now_utc.with_timezone(&Eastern);
    if now.month() == 12 {
        let year = now.year();
        return (
            PuzzleDay::new(now.day().min(if year == 2025 { 12 } else { 25 }))
                .expect("December day is a valid puzzle day"),
            PuzzleYear::new(year).expect("current year supports Advent of Code"),
        );
    }

    let year = now.year() - 1;
    (
        PuzzleDay::new(if year == 2025 { 12 } else { 25 })
            .expect("default day is a valid puzzle day"),
        PuzzleYear::new(year).expect("previous year supports Advent of Code"),
    )
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        default_puzzle_date_at, valid_puzzle_release_at, valid_year_release_at, PuzzleDay,
        PuzzleYear, ReleaseError,
    };

    fn puzzle(day: u32, year: i32) -> (PuzzleDay, PuzzleYear) {
        (
            PuzzleDay::new(day).expect("valid test puzzle day"),
            PuzzleYear::new(year).expect("valid test puzzle year"),
        )
    }

    fn year(year: i32) -> PuzzleYear {
        PuzzleYear::new(year).expect("valid test puzzle year")
    }

    fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .expect("valid UTC test timestamp")
    }

    #[test]
    fn releases_in_2025_are_limited_to_twelve_days() {
        let (day12, year2025) = puzzle(12, 2025);
        let (day13, _) = puzzle(13, 2025);
        assert!(valid_puzzle_release_at(day12, year2025, utc(2025, 12, 12, 5, 0)).is_ok());
        assert!(matches!(
            valid_puzzle_release_at(day12, year2025, utc(2025, 12, 12, 4, 59)),
            Err(ReleaseError::Puzzle(_, _))
        ));
        assert!(matches!(
            valid_puzzle_release_at(day13, year2025, utc(2025, 12, 13, 5, 0)),
            Err(ReleaseError::Puzzle(_, _))
        ));
    }

    #[test]
    fn default_puzzle_date_uses_the_latest_released_event() {
        assert_eq!(
            default_puzzle_date_at(utc(2026, 7, 20, 12, 0)),
            puzzle(12, 2025)
        );
        assert_eq!(
            default_puzzle_date_at(utc(2026, 12, 2, 5, 0)),
            puzzle(2, 2026)
        );
    }

    #[test]
    fn calendar_release_is_independent_of_selected_day() {
        let year = year(2026);
        assert!(matches!(
            valid_year_release_at(year, utc(2026, 12, 1, 4, 59)),
            Err(ReleaseError::Year(_))
        ));
        assert!(valid_year_release_at(year, utc(2026, 12, 1, 5, 0)).is_ok());
    }

    #[test]
    fn atomic_write_replaces_a_file_without_leaving_a_temporary_file() {
        let dir = std::env::temp_dir().join(format!("aocsuite-utils-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        std::fs::create_dir_all(&dir).expect("create test directory");
        std::fs::write(&path, "old").expect("write existing file");

        super::atomic_write(&path, b"new").expect("atomically replace file");

        assert_eq!(std::fs::read(&path).expect("read replacement"), b"new");
        assert_eq!(
            std::fs::read_dir(&dir)
                .expect("read test directory")
                .count(),
            1
        );
        std::fs::remove_dir_all(dir).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_creates_owner_only_files() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!(
            "aocsuite-utils-test-permissions-{}",
            std::process::id()
        ));
        let path = dir.join("private");
        std::fs::create_dir_all(&dir).expect("create test directory");

        super::atomic_write(&path, b"secret").expect("write private file");

        assert_eq!(
            std::fs::metadata(&path)
                .expect("read private file metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        std::fs::remove_dir_all(dir).expect("remove test directory");
    }
}
