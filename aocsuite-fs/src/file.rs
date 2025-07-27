use std::{
    fs,
    path::{Path, PathBuf},
};

use aocsuite_client::{download_file, AocPage};
use aocsuite_parser::{parse, AocSubmissionResult, ParserType};
use aocsuite_utils::{PuzzleDay, PuzzleYear};
use serde_json::{Map, Value};

use crate::{AocContentDir, AocFileError, AocFileResult};

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
        let path = self._to_path();
        if !is_cache_valid(&path) & self.fetchable() {
            fetch_aocfile(self)?;
        }

        Ok(path)
    }
    fn _to_path(&self) -> PathBuf {
        let dir = AocContentDir::new();
        let filename = self.filename();

        match self.day {
            Some(day) => dir.daily_data_dir(day, self.year).join(filename),
            None => dir.yearly_data_dir(self.year).join(filename),
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

    pub fn set_cache_status(&self, val: bool) {
        if self.updateable() {
            update_cache(&self._to_path(), val)
        }
    }

    fn save(&self, contents: &str) -> AocFileResult<()> {
        let path = self._to_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, contents)?;

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
    file.set_cache_status(true);
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

fn update_cache(path: &Path, val: bool) {
    let cache_path = path.parent().unwrap().join(CACHE_FILE);

    let mut cache_json: Map<String, Value> = if cache_path.exists() {
        let cache_contents = fs::read_to_string(&cache_path).unwrap_or_default();
        serde_json::from_str(&cache_contents).unwrap_or_default()
    } else {
        Map::new()
    };
    let filename = path.file_name().unwrap().to_str().unwrap();
    cache_json.insert(filename.to_owned(), Value::Bool(val));

    let json_string = serde_json::to_string_pretty(&cache_json).unwrap();
    fs::write(&cache_path, json_string).ok();
}

fn page_from_file(file: &AocContentFile) -> AocFileResult<AocPage> {
    match file.file_type {
        AocFileType::Puzzle => {
            let day = file.day.expect("cannot be created without day");
            Ok(AocPage::Puzzle(day, file.year))
        }
        AocFileType::Calendar => Ok(AocPage::Calendar(file.year)),
        AocFileType::Input => {
            let day = file.day.expect("cannot be created without day");
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
) -> () {
    if result == &AocSubmissionResult::Correct {
        // set the calendar cache to false for year
        let calendar_file = AocContentFile::calendar(year);
        calendar_file.set_cache_status(false);

        if update_puzzle {
            // set the puzzle cache to false for day
            let puzzle_file = AocContentFile::puzzle(day, year);
            puzzle_file.set_cache_status(false);
        }
    }
}
