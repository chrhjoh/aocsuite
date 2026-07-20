use std::{
    fs,
    path::{Path, PathBuf},
};

use aocsuite_client::{download_file, AocPage};
use aocsuite_parser::{parse, AocSubmissionResult, ParserType};
use aocsuite_utils::{atomic_write, set_owner_only_permissions, PuzzleDay, PuzzleYear};
use serde_json::{Map, Value};

use crate::{AocCacheDir, AocFileError, AocFileResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AocFileType {
    Puzzle,
    Calendar,
    Input,
    Example,
}

#[derive(Debug, Clone, Copy)]
pub struct AocContentFile {
    pub file_type: AocFileType,
    pub day: Option<PuzzleDay>,
    pub year: PuzzleYear,
}

impl AocContentFile {
    pub fn puzzle(day: PuzzleDay, year: PuzzleYear) -> Self {
        Self {
            file_type: AocFileType::Puzzle,
            day: Some(day),
            year,
        }
    }

    pub fn calendar(year: PuzzleYear) -> Self {
        Self {
            file_type: AocFileType::Calendar,
            day: None,
            year,
        }
    }

    pub fn input(day: PuzzleDay, year: PuzzleYear) -> Self {
        Self {
            file_type: AocFileType::Input,
            day: Some(day),
            year,
        }
    }

    pub fn example(day: PuzzleDay, year: PuzzleYear) -> Self {
        Self {
            file_type: AocFileType::Example,
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

    pub fn to_path(&self) -> AocFileResult<PathBuf> {
        let path = self._to_path()?;
        if !is_cache_valid(&path) & self.fetchable() {
            fetch_aocfile(self)?;
        }
        if self.file_type == AocFileType::Input && path.exists() {
            set_owner_only_permissions(&path)?;
        }

        Ok(path)
    }
    fn _to_path(&self) -> AocFileResult<PathBuf> {
        let dir = AocCacheDir::new()?;
        let filename = self.filename();

        match self.day {
            Some(day) => Ok(dir.daily_data_dir(day, self.year).join(filename)),
            None => Ok(dir.yearly_data_dir(self.year).join(filename)),
        }
    }

    fn filename(&self) -> &'static str {
        match self.file_type {
            AocFileType::Puzzle => "puzzle.md",
            AocFileType::Calendar => "calendar.html",
            AocFileType::Input => "input.txt",
            AocFileType::Example => "example.txt",
        }
    }

    pub fn set_cache_status(&self, val: bool) -> AocFileResult<()> {
        if self.updateable() {
            update_cache(&self._to_path()?, val)?;
        }
        Ok(())
    }

    fn save(&self, contents: &str) -> AocFileResult<()> {
        let path = self._to_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        atomic_write(&path, contents.as_bytes())?;

        Ok(())
    }
    pub fn load(&self) -> AocFileResult<String> {
        let path = self.to_path()?;
        let contents = fs::read_to_string(&path)?;
        Ok(contents)
    }
}
fn fetch_aocfile(file: &AocContentFile) -> AocFileResult<()> {
    let page = page_from_file(file)?;
    let mut content = download_file(&page)?;
    if file.file_type == AocFileType::Puzzle {
        content = parse(&content, ParserType::MarkdownArticle);
    }

    file.save(&content)?;
    update_cache(&file._to_path()?, true)?;
    Ok(())
}

impl ToString for AocContentFile {
    fn to_string(&self) -> String {
        self.filename().to_owned()
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

    match cache_json.get(filename) {
        Some(Value::Bool(true)) => true,
        _ => false,
    }
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
        AocFileType::Example => Err(AocFileError::InvalidFile(
            "Example files cannot be downloaded".to_string(),
        )),
    }
}

pub fn update_cache_status(
    result: &AocSubmissionResult,
    day: PuzzleDay,
    year: PuzzleYear,
    update_puzzle: bool,
) -> AocFileResult<()> {
    if result == &AocSubmissionResult::Correct {
        // set the calendar cache to false for year
        let calendar_file = AocContentFile::calendar(year);
        calendar_file.set_cache_status(false)?;

        if update_puzzle {
            // set the puzzle cache to false for day
            let puzzle_file = AocContentFile::puzzle(day, year);
            puzzle_file.set_cache_status(false)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        is_cache_valid, page_from_file, update_cache, AocContentFile, AocFileError, AocFileType,
        CACHE_FILE,
    };

    #[test]
    fn fetched_input_is_cache_valid() {
        let temp_dir = std::env::temp_dir().join(format!(
            "aocsuite-fs-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch")
                .as_nanos()
        ));
        let input_path = temp_dir.join("input.txt");
        fs::create_dir_all(&temp_dir).expect("create test cache directory");
        fs::write(&input_path, "cached input").expect("write input cache");

        update_cache(&input_path, true).expect("update input cache");

        assert!(is_cache_valid(&input_path));
        fs::remove_dir_all(temp_dir).expect("remove test cache directory");
    }

    #[test]
    fn cache_metadata_write_failures_are_returned() {
        let temp_dir = std::env::temp_dir().join(format!(
            "aocsuite-fs-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch")
                .as_nanos()
        ));
        let input_path = temp_dir.join("input.txt");
        fs::create_dir_all(temp_dir.join(CACHE_FILE)).expect("create invalid metadata directory");
        fs::write(&input_path, "cached input").expect("write input cache");

        assert!(matches!(update_cache(&input_path, true), Err(AocFileError::Io(_))));

        fs::remove_dir_all(temp_dir).expect("remove test cache directory");
    }

    #[test]
    fn puzzle_and_input_without_a_day_return_errors() {
        for file_type in [AocFileType::Puzzle, AocFileType::Input] {
            let file = AocContentFile {
                file_type,
                day: None,
                year: 2024,
            };

            assert!(matches!(
                page_from_file(&file),
                Err(AocFileError::InvalidFile(_))
            ));
        }
    }
}
