use std::{
    fs,
    path::{Path, PathBuf},
};

use aocsuite_client::{AocClient, AocPage};
use aocsuite_parser::{parse, AocSubmissionResult, ParserType};
use aocsuite_utils::{atomic_write, set_owner_only_permissions, PuzzleDay, PuzzleId, PuzzleYear};
use serde_json::{Map, Value};

use crate::{AocCacheDir, AocFileError, AocFileResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AocFileType {
    Puzzle,
    Calendar,
    Input,
}

#[derive(Debug, Clone)]
pub struct AocContentFile {
    cache_dir: PathBuf,
    pub file_type: AocFileType,
    pub day: Option<PuzzleDay>,
    pub year: PuzzleYear,
}

impl AocContentFile {
    pub fn puzzle(cache_dir: PathBuf, day: PuzzleDay, year: PuzzleYear) -> Self {
        Self {
            cache_dir,
            file_type: AocFileType::Puzzle,
            day: Some(day),
            year,
        }
    }

    pub fn calendar(cache_dir: PathBuf, year: PuzzleYear) -> Self {
        Self {
            cache_dir,
            file_type: AocFileType::Calendar,
            day: None,
            year,
        }
    }

    pub fn input(cache_dir: PathBuf, day: PuzzleDay, year: PuzzleYear) -> Self {
        Self {
            cache_dir,
            file_type: AocFileType::Input,
            day: Some(day),
            year,
        }
    }

    fn updateable(&self) -> bool {
        matches!(self.file_type, AocFileType::Puzzle | AocFileType::Calendar)
    }
    fn fetchable(&self) -> bool {
        matches!(
            self.file_type,
            AocFileType::Puzzle | AocFileType::Calendar | AocFileType::Input
        )
    }

    pub fn materialize(&self, client: &AocClient) -> AocFileResult<PathBuf> {
        let path = self.path()?;
        if !is_cache_valid(&path) && self.fetchable() {
            fetch_aocfile(self, client)?;
        }
        if self.file_type == AocFileType::Input && path.exists() {
            set_owner_only_permissions(&path)?;
        }

        Ok(path)
    }
    pub fn path(&self) -> AocFileResult<PathBuf> {
        let dir = AocCacheDir::new(self.cache_dir.clone());
        match self.file_type {
            AocFileType::Puzzle => {
                let puzzle = self.puzzle_id()?;
                Ok(dir.puzzles_dir().join(format!("{puzzle}.md")))
            }
            AocFileType::Calendar => {
                Ok(dir.calendars_dir().join(format!("year{}.html", self.year)))
            }
            AocFileType::Input => {
                let puzzle = self.puzzle_id()?;
                Ok(dir.inputs_dir().join(format!("{puzzle}.txt")))
            }
        }
    }

    fn puzzle_id(&self) -> AocFileResult<PuzzleId> {
        self.day
            .map(|day| PuzzleId::new(day, self.year))
            .ok_or_else(|| {
                AocFileError::InvalidFile(format!("{} files require a puzzle day", self))
            })
    }

    pub fn set_cache_status(&self, val: bool) -> AocFileResult<()> {
        if self.updateable() {
            update_cache(&self.path()?, val)?;
        }
        Ok(())
    }

    fn save(&self, contents: &str) -> AocFileResult<()> {
        let path = self.path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        atomic_write(&path, contents.as_bytes())?;

        Ok(())
    }
    pub fn load(&self, client: &AocClient) -> AocFileResult<String> {
        let path = self.materialize(client)?;
        let contents = fs::read_to_string(&path)?;
        Ok(contents)
    }
}
fn fetch_aocfile(file: &AocContentFile, client: &AocClient) -> AocFileResult<()> {
    let page = page_from_file(file)?;
    let content = client.download(&page)?;
    let content = if file.file_type == AocFileType::Puzzle {
        parse_puzzle_content(&content)?
    } else {
        content
    };

    file.save(&content)?;
    update_cache(&file.path()?, true)?;
    Ok(())
}

fn parse_puzzle_content(content: &str) -> AocFileResult<String> {
    let content = parse(content, ParserType::MarkdownArticle);
    if content.trim().is_empty() {
        return Err(AocFileError::InvalidFile(
            "puzzle response did not contain an article".to_string(),
        ));
    }
    Ok(content)
}

impl std::fmt::Display for AocContentFile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.file_type {
            AocFileType::Puzzle => "puzzle",
            AocFileType::Calendar => "calendar",
            AocFileType::Input => "input",
        })
    }
}

const CACHE_FILE: &str = ".aoccache.json";

fn is_cache_valid(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let cache_path = path.parent().unwrap().join(CACHE_FILE);

    if !cache_path.exists() {
        return false;
    }

    let cache_contents = match fs::read_to_string(&cache_path) {
        Ok(contents) => contents,
        Err(_) => return false,
    };

    let cache_json: Map<String, Value> = match serde_json::from_str(&cache_contents) {
        Ok(json) => json,
        Err(_) => return false,
    };
    let filename = path.file_name().unwrap().to_str().unwrap();

    matches!(cache_json.get(filename), Some(Value::Bool(true)))
}

fn update_cache(path: &Path, val: bool) -> AocFileResult<()> {
    let cache_path = path.parent().unwrap().join(CACHE_FILE);

    let mut cache_json: Map<String, Value> = match fs::read_to_string(&cache_path) {
        Ok(cache_contents) => serde_json::from_str(&cache_contents)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(error) => return Err(error.into()),
    };
    let filename = path.file_name().unwrap().to_str().unwrap();
    cache_json.insert(filename.to_owned(), Value::Bool(val));

    let json_string = serde_json::to_string_pretty(&cache_json)?;
    atomic_write(&cache_path, json_string.as_bytes())?;
    Ok(())
}

pub(crate) fn remove_cached_file(path: &Path) -> AocFileResult<bool> {
    let cache_path = path.parent().unwrap().join(CACHE_FILE);
    let metadata_removed = match fs::read_to_string(&cache_path) {
        Ok(cache_contents) => {
            let mut cache_json: Map<String, Value> = serde_json::from_str(&cache_contents)?;
            let filename = path.file_name().unwrap().to_str().unwrap();
            if cache_json.remove(filename).is_none() {
                false
            } else {
                if cache_json.is_empty() {
                    match fs::remove_file(&cache_path) {
                        Ok(()) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                } else {
                    let json = serde_json::to_vec_pretty(&cache_json)?;
                    atomic_write(&cache_path, &json)?;
                }
                true
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };

    let file_removed = match fs::remove_file(path) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    Ok(metadata_removed || file_removed)
}

fn page_from_file(file: &AocContentFile) -> AocFileResult<AocPage> {
    match file.file_type {
        AocFileType::Puzzle => {
            let day = file.day.ok_or_else(|| {
                AocFileError::InvalidFile("Puzzle files require a puzzle day".to_string())
            })?;
            Ok(AocPage::Puzzle(day, file.year))
        }
        AocFileType::Calendar => Ok(AocPage::Calendar(file.year)),
        AocFileType::Input => {
            let day = file.day.ok_or_else(|| {
                AocFileError::InvalidFile("Input files require a puzzle day".to_string())
            })?;
            Ok(AocPage::Input(day, file.year))
        }
    }
}

pub fn update_cache_status(
    cache_dir: PathBuf,
    result: &AocSubmissionResult,
    day: PuzzleDay,
    year: PuzzleYear,
    update_puzzle: bool,
) -> AocFileResult<()> {
    if result == &AocSubmissionResult::Correct {
        // set the calendar cache to false for year
        let calendar_file = AocContentFile::calendar(cache_dir.clone(), year);
        calendar_file.set_cache_status(false)?;

        if update_puzzle {
            // set the puzzle cache to false for day
            let puzzle_file = AocContentFile::puzzle(cache_dir, day, year);
            puzzle_file.set_cache_status(false)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs, process,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use aocsuite_utils::PuzzleYear;

    use super::{
        is_cache_valid, page_from_file, parse_puzzle_content, remove_cached_file, update_cache,
        AocContentFile, AocFileError, AocFileType, CACHE_FILE,
    };

    static TEST_ROOT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_dir() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_nanos();
        let sequence = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "aocsuite-fs-test-{timestamp}-{}-{sequence}",
            process::id()
        ))
    }

    #[test]
    fn fetched_input_is_cache_valid() {
        let temp_dir = test_dir();
        let input_path = temp_dir.join("input.txt");
        fs::create_dir_all(&temp_dir).expect("create test cache directory");
        fs::write(&input_path, "cached input").expect("write input cache");

        update_cache(&input_path, true).expect("update input cache");

        assert!(is_cache_valid(&input_path));
        fs::remove_dir_all(temp_dir).expect("remove test cache directory");
    }

    #[test]
    fn cache_metadata_write_failures_are_returned() {
        let temp_dir = test_dir();
        let input_path = temp_dir.join("input.txt");
        fs::create_dir_all(temp_dir.join(CACHE_FILE)).expect("create invalid metadata directory");
        fs::write(&input_path, "cached input").expect("write input cache");

        assert!(matches!(
            update_cache(&input_path, true),
            Err(AocFileError::Io(_))
        ));

        fs::remove_dir_all(temp_dir).expect("remove test cache directory");
    }

    #[test]
    fn cleanup_metadata_failures_preserve_the_cache_body() {
        let temp_dir = test_dir();
        let input_path = temp_dir.join("input.txt");
        fs::create_dir_all(&temp_dir).expect("create test cache directory");
        fs::write(&input_path, "cached input").expect("write input cache");
        fs::write(temp_dir.join(CACHE_FILE), "not json").expect("write invalid metadata");

        assert!(matches!(
            remove_cached_file(&input_path),
            Err(AocFileError::Json(_))
        ));
        assert!(input_path.exists());

        fs::remove_dir_all(temp_dir).expect("remove test cache directory");
    }

    #[test]
    fn puzzle_responses_without_articles_are_rejected_before_caching() {
        assert!(matches!(
            parse_puzzle_content("<html><main>Please log in</main></html>"),
            Err(AocFileError::InvalidFile(_))
        ));
    }

    #[test]
    fn puzzle_and_input_without_a_day_return_errors() {
        for file_type in [AocFileType::Puzzle, AocFileType::Input] {
            let file = AocContentFile {
                cache_dir: test_dir(),
                file_type,
                day: None,
                year: PuzzleYear::new(2024).expect("valid test year"),
            };

            assert!(matches!(
                page_from_file(&file),
                Err(AocFileError::InvalidFile(_))
            ));
        }
    }

    #[test]
    fn content_paths_are_flat_and_grouped_by_type() {
        let cache = test_dir();
        let day = aocsuite_utils::PuzzleDay::new(4).unwrap();
        let year = PuzzleYear::new(2024).unwrap();

        assert_eq!(
            AocContentFile::puzzle(cache.clone(), day, year)
                .path()
                .unwrap(),
            cache.join("puzzles/year2024_day4.md")
        );
        assert_eq!(
            AocContentFile::input(cache.clone(), day, year)
                .path()
                .unwrap(),
            cache.join("inputs/year2024_day4.txt")
        );
        assert_eq!(
            AocContentFile::calendar(cache.clone(), year)
                .path()
                .unwrap(),
            cache.join("calendars/year2024.html")
        );
    }
}
