use std::{
    env,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::{Tz, US::Eastern};
use clap::ValueEnum;
use thiserror::Error;

pub type PuzzleDay = u32;
pub type PuzzleYear = i32;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum Exercise {
    #[clap(alias = "1")]
    One,
    #[clap(alias = "2")]
    Two,
}
impl ToString for Exercise {
    fn to_string(&self) -> String {
        match self {
            Exercise::One => "1".to_string(),
            Exercise::Two => "2".to_string(),
        }
    }
}

type AocReleaseResult<T> = Result<T, ReleaseError>;

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("Puzzle for {0} {1} has not been released yet.")]
    Puzzle(PuzzleDay, PuzzleYear),
    #[error("Advent of code has not started yet for {0}")]
    Year(PuzzleYear),
}

#[derive(Debug, Error)]
pub enum RuntimeDirError {
    #[error("XDG_DATA_HOME must be an absolute, non-empty path")]
    InvalidXdgDataHome,
    #[error("HOME must be an absolute, non-empty path")]
    InvalidHome,
    #[error("Neither XDG_DATA_HOME nor HOME environment variables are set")]
    MissingHome,
}

pub type RuntimeDirResult<T> = Result<T, RuntimeDirError>;

static ATOMIC_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn atomic_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no parent directory")
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
    if !valid_puzzle_day(day, year) || year < 2015 {
        return Err(ReleaseError::Puzzle(day, year));
    }
    let now_eastern = now_utc.with_timezone(&Eastern);
    if year > now_eastern.year() {
        return Err(ReleaseError::Puzzle(day, year));
    }
    if year < now_eastern.year() {
        return Ok(());
    }

    let release_date = Eastern
        .with_ymd_and_hms(year, 12, day, 0, 0, 0)
        .single()
        .ok_or(ReleaseError::Puzzle(day, year))?;

    if now_eastern >= release_date {
        Ok(())
    } else {
        Err(ReleaseError::Puzzle(day, year))
    }
}

fn valid_puzzle_day(day: PuzzleDay, year: PuzzleYear) -> bool {
    (1..=if year == 2025 { 12 } else { 25 }).contains(&day)
}

pub fn valid_year_release(_day: PuzzleDay, year: PuzzleYear) -> AocReleaseResult<()> {
    valid_year_release_at(year, Utc::now())
}

fn valid_year_release_at(year: PuzzleYear, now_utc: DateTime<Utc>) -> AocReleaseResult<()> {
    let now_eastern = now_utc.with_timezone(&Eastern);
    let now_year = now_eastern.year();

    if year < 2015 || year > now_year {
        return Err(ReleaseError::Year(year));
    }
    if year == now_year {
        let release_date = Eastern
            .with_ymd_and_hms(year, 12, 1, 0, 0, 0)
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
pub fn today_day() -> PuzzleDay {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern).day()
}
pub fn today_year() -> PuzzleYear {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern).year()
}

pub fn get_aocsuite_dir() -> RuntimeDirResult<PathBuf> {
    get_aocsuite_dir_from(env::var_os("XDG_DATA_HOME"), env::var_os("HOME"))
}

fn get_aocsuite_dir_from(
    xdg_data_home: Option<OsString>,
    home: Option<OsString>,
) -> RuntimeDirResult<PathBuf> {
    if let Some(xdg_data_home) = xdg_data_home {
        let base = PathBuf::from(xdg_data_home);
        return valid_runtime_base(base, RuntimeDirError::InvalidXdgDataHome)
            .map(|base| base.join("aocsuite"));
    }

    let home = home.ok_or(RuntimeDirError::MissingHome)?;
    valid_runtime_base(PathBuf::from(home), RuntimeDirError::InvalidHome)
        .map(|home| home.join(".local").join("share").join("aocsuite"))
}

fn valid_runtime_base(base: PathBuf, error: RuntimeDirError) -> RuntimeDirResult<PathBuf> {
    if base.as_os_str().is_empty() || !base.is_absolute() {
        return Err(error);
    }
    Ok(base)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use std::{ffi::OsString, path::PathBuf};

    use super::{
        get_aocsuite_dir_from, valid_puzzle_release_at, valid_year_release_at, ReleaseError,
        RuntimeDirError,
    };

    fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .expect("valid UTC test timestamp")
    }

    #[test]
    fn invalid_puzzle_days_return_errors_without_panicking() {
        let now = utc(2026, 12, 26, 5, 0);

        for day in [0, 26, u32::MAX] {
            assert!(matches!(
                valid_puzzle_release_at(day, 2026, now),
                Err(ReleaseError::Puzzle(_, _))
            ));
        }
    }

    #[test]
    fn releases_in_2025_are_limited_to_twelve_days() {
        assert!(valid_puzzle_release_at(12, 2025, utc(2025, 12, 12, 5, 0)).is_ok());
        assert!(matches!(
            valid_puzzle_release_at(12, 2025, utc(2025, 12, 12, 4, 59)),
            Err(ReleaseError::Puzzle(12, 2025))
        ));
        assert!(matches!(
            valid_puzzle_release_at(13, 2025, utc(2025, 12, 13, 5, 0)),
            Err(ReleaseError::Puzzle(13, 2025))
        ));
    }

    #[test]
    fn puzzle_releases_at_eastern_midnight() {
        assert!(matches!(
            valid_puzzle_release_at(2, 2026, utc(2026, 12, 2, 4, 59)),
            Err(ReleaseError::Puzzle(2, 2026))
        ));
        assert!(valid_puzzle_release_at(2, 2026, utc(2026, 12, 2, 5, 0)).is_ok());
    }

    #[test]
    fn calendar_release_is_independent_of_selected_day() {
        assert!(matches!(
            valid_year_release_at(2026, utc(2026, 12, 1, 4, 59)),
            Err(ReleaseError::Year(2026))
        ));
        assert!(valid_year_release_at(2026, utc(2026, 12, 1, 5, 0)).is_ok());
    }

    #[test]
    fn invalid_and_future_years_return_errors() {
        let now = utc(2026, 12, 1, 5, 0);

        assert!(matches!(
            valid_year_release_at(2014, now),
            Err(ReleaseError::Year(2014))
        ));
        assert!(matches!(
            valid_year_release_at(2027, now),
            Err(ReleaseError::Year(2027))
        ));
    }

    #[test]
    fn runtime_dir_prefers_an_absolute_xdg_data_home() {
        assert_eq!(
            get_aocsuite_dir_from(Some(OsString::from("/var/aoc-data")), None).unwrap(),
            PathBuf::from("/var/aoc-data/aocsuite")
        );
    }

    #[test]
    fn runtime_dir_uses_home_when_xdg_data_home_is_absent() {
        assert_eq!(
            get_aocsuite_dir_from(None, Some(OsString::from("/Users/tester"))).unwrap(),
            PathBuf::from("/Users/tester/.local/share/aocsuite")
        );
    }

    #[test]
    fn runtime_dir_rejects_empty_and_relative_environment_values() {
        assert!(matches!(
            get_aocsuite_dir_from(Some(OsString::new()), Some(OsString::from("/Users/tester"))),
            Err(RuntimeDirError::InvalidXdgDataHome)
        ));
        assert!(matches!(
            get_aocsuite_dir_from(None, Some(OsString::from("relative-home"))),
            Err(RuntimeDirError::InvalidHome)
        ));
    }

    #[test]
    fn runtime_dir_handles_missing_and_non_unicode_environment_values() {
        assert!(matches!(
            get_aocsuite_dir_from(None, None),
            Err(RuntimeDirError::MissingHome)
        ));

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            let home = OsString::from_vec(vec![b'/', b't', 0x80]);
            assert_eq!(
                get_aocsuite_dir_from(None, Some(home)).unwrap(),
                PathBuf::from(OsString::from_vec(vec![b'/', b't', 0x80]))
                    .join(".local/share/aocsuite")
            );
        }
    }

    #[test]
    fn atomic_write_replaces_a_file_without_leaving_a_temporary_file() {
        let dir = std::env::temp_dir().join(format!(
            "aocsuite-utils-test-{}",
            std::process::id()
        ));
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

    #[cfg(unix)]
    #[test]
    fn owner_only_permissions_tightens_existing_files() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!(
            "aocsuite-utils-test-existing-permissions-{}",
            std::process::id()
        ));
        let path = dir.join("private");
        std::fs::create_dir_all(&dir).expect("create test directory");
        std::fs::write(&path, "secret").expect("write test file");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("make test file public");

        super::set_owner_only_permissions(&path).expect("tighten file permissions");

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
