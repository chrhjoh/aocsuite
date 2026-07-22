mod dir;
mod file;

use std::path::PathBuf;

use aocsuite_client::AocClientError;
use aocsuite_utils::{PuzzleDay, PuzzleId, PuzzleYear};
pub use dir::AocCacheDir;
use file::remove_cached_file;
pub use file::{update_cache_status, AocContentFile, AocFileType};
use thiserror::Error;

type AocFileResult<T> = Result<T, AocFileError>;

#[derive(Error, Debug)]
pub enum AocFileError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Client(#[from] AocClientError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("invalid file error: {0}")]
    InvalidFile(String),

    #[error("error cleaning cache: {0}")]
    CleanError(String),
}

pub fn clean_cache(
    cache_dir: PathBuf,
    year: Option<PuzzleYear>,
    day: Option<PuzzleDay>,
) -> AocFileResult<()> {
    match (year, day) {
        (None, Some(_)) => Err(AocFileError::CleanError(
            "Year was not specified but day was".to_string(),
        )),
        (Some(year), Some(day)) => {
            let puzzle = PuzzleId::new(day, year);
            remove_puzzle_cache_files(&cache_dir, puzzle)?;
            Ok(())
        }
        (Some(year), None) => {
            for day in PuzzleDay::MIN..=PuzzleDay::MAX {
                let day = PuzzleDay::new(u32::from(day)).expect("valid puzzle day");
                remove_puzzle_cache_files(&cache_dir, PuzzleId::new(day, year))?;
            }
            let calendar = AocCacheDir::new(cache_dir.clone())
                .calendars_dir()
                .join(format!("year{year}.html"));
            remove_cached_file(&calendar)?;

            Ok(())
        }
        (None, None) => {
            if !std::fs::exists(&cache_dir)? {
                return Ok(());
            }
            std::fs::remove_dir_all(cache_dir)?;
            Ok(())
        }
    }
}

fn remove_puzzle_cache_files(cache_dir: &std::path::Path, puzzle: PuzzleId) -> AocFileResult<()> {
    let cache = AocCacheDir::new(cache_dir.to_path_buf());
    let paths = [
        cache.puzzles_dir().join(format!("{puzzle}.html")),
        cache.puzzles_dir().join(format!("{puzzle}.md")),
        cache.inputs_dir().join(format!("{puzzle}.txt")),
    ];
    for path in paths {
        remove_cached_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use aocsuite_utils::{PuzzleDay, PuzzleYear};

    use super::clean_cache;

    static TEST_ROOT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_dir() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_nanos();
        let sequence = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "aocsuite-fs-clean-test-{timestamp}-{}-{sequence}",
            process::id()
        ))
    }

    #[test]
    fn day_cleanup_removes_only_recognized_flat_files_and_metadata() {
        let cache = test_dir();
        let puzzles = cache.join("puzzles");
        let inputs = cache.join("inputs");
        fs::create_dir_all(&puzzles).unwrap();
        fs::create_dir_all(&inputs).unwrap();
        for path in [
            puzzles.join("year2024_day4.html"),
            puzzles.join("year2024_day4.md"),
            puzzles.join("year2024_day5.md"),
        ] {
            fs::write(path, "cached").unwrap();
        }
        fs::write(
            puzzles.join(".aoccache.json"),
            r#"{"year2024_day4.md":true,"year2024_day5.md":true}"#,
        )
        .unwrap();
        fs::write(
            inputs.join(".aoccache.json"),
            r#"{"year2024_day4.txt":true}"#,
        )
        .unwrap();

        clean_cache(
            cache.clone(),
            Some(PuzzleYear::new(2024).unwrap()),
            Some(PuzzleDay::new(4).unwrap()),
        )
        .unwrap();

        assert!(!puzzles.join("year2024_day4.html").exists());
        assert!(!puzzles.join("year2024_day4.md").exists());
        assert!(!inputs.join("year2024_day4.txt").exists());
        assert!(puzzles.join("year2024_day5.md").exists());
        assert!(!inputs.join(".aoccache.json").exists());
        let metadata = fs::read_to_string(puzzles.join(".aoccache.json")).unwrap();
        assert!(!metadata.contains("year2024_day4"));
        assert!(metadata.contains("year2024_day5.md"));

        clean_cache(
            cache.clone(),
            Some(PuzzleYear::new(2024).unwrap()),
            Some(PuzzleDay::new(4).unwrap()),
        )
        .unwrap();

        fs::remove_dir_all(cache).unwrap();
    }

    #[test]
    fn year_cleanup_removes_daily_files_and_the_year_calendar() {
        let cache = test_dir();
        let puzzles = cache.join("puzzles");
        let calendars = cache.join("calendars");
        fs::create_dir_all(&puzzles).unwrap();
        fs::create_dir_all(&calendars).unwrap();
        fs::write(puzzles.join("year2024_day1.md"), "cached").unwrap();
        fs::write(puzzles.join("year2023_day1.md"), "cached").unwrap();
        fs::write(calendars.join("year2024.html"), "cached").unwrap();

        clean_cache(cache.clone(), Some(PuzzleYear::new(2024).unwrap()), None).unwrap();

        assert!(!puzzles.join("year2024_day1.md").exists());
        assert!(!calendars.join("year2024.html").exists());
        assert!(puzzles.join("year2023_day1.md").exists());

        fs::remove_dir_all(cache).unwrap();
    }
}
