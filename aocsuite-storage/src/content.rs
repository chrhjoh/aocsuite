use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use aocsuite_client::{AocClient, AocClientError, AocPage};
use aocsuite_parser::{parse_puzzle_markdown, AocSubmissionResult, ParserError};
use aocsuite_utils::{
    atomic_write, set_owner_only_permissions, PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear,
};
use thiserror::Error;

use crate::database::{CacheEntry, DatabaseError, StateDatabase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheKey {
    PuzzleHtml(PuzzleId),
    PuzzleMarkdown(PuzzleId),
    Input(PuzzleId),
    Calendar(PuzzleYear),
}

pub struct ContentStore {
    cache_dir: PathBuf,
    database: StateDatabase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheCleanScope {
    All,
    Year(PuzzleYear),
    Puzzle(PuzzleId),
}

impl ContentStore {
    pub fn open(cache_dir: PathBuf) -> ContentResult<Self> {
        fs::create_dir_all(&cache_dir)?;
        let database = StateDatabase::open(&cache_dir.join("state.sqlite"))
            .map_err(ContentError::from_database)?;
        Ok(Self {
            cache_dir,
            database,
        })
    }

    pub fn load_calendar(&self, year: PuzzleYear, client: &AocClient) -> ContentResult<String> {
        let path = self.load_or_fetch(CacheKey::Calendar(year), AocPage::Calendar(year), client)?;
        Ok(fs::read_to_string(path)?)
    }

    pub fn ensure_input(&self, puzzle: PuzzleId, client: &AocClient) -> ContentResult<PathBuf> {
        let path = self.load_or_fetch(
            CacheKey::Input(puzzle),
            AocPage::Input(puzzle.day, puzzle.year),
            client,
        )?;
        set_owner_only_permissions(&path)?;
        Ok(path)
    }

    pub fn ensure_puzzle_markdown(
        &self,
        puzzle: PuzzleId,
        client: &AocClient,
    ) -> ContentResult<PathBuf> {
        let html_path = self.load_or_fetch(
            CacheKey::PuzzleHtml(puzzle),
            AocPage::Puzzle(puzzle.day, puzzle.year),
            client,
        )?;
        let markdown_key = CacheKey::PuzzleMarkdown(puzzle);
        if self.is_cached(markdown_key)? {
            return Ok(self.cache_path(markdown_key));
        }

        let markdown = parse_puzzle_markdown(&fs::read_to_string(html_path)?)?;
        self.save(markdown_key, markdown.as_bytes())
    }

    pub fn invalidate_after_submission(
        &self,
        puzzle: PuzzleId,
        part: PuzzlePart,
        result: &AocSubmissionResult,
    ) -> ContentResult<()> {
        if !matches!(result, AocSubmissionResult::Correct) {
            return Ok(());
        }

        self.database
            .invalidate_cache_entry(CacheKey::Calendar(puzzle.year))
            .map_err(ContentError::from_database)?;
        if part == PuzzlePart::One {
            self.database
                .invalidate_cache_entry(CacheKey::PuzzleHtml(puzzle))
                .map_err(ContentError::from_database)?;
            self.database
                .invalidate_cache_entry(CacheKey::PuzzleMarkdown(puzzle))
                .map_err(ContentError::from_database)?;
        }
        Ok(())
    }

    pub fn clean(&self, scope: CacheCleanScope) -> ContentResult<()> {
        match scope {
            CacheCleanScope::All => {
                for directory in ["puzzles", "inputs", "calendars"] {
                    self.remove_directory(directory)?;
                }
                self.database
                    .clear_cache_entries()
                    .map_err(ContentError::from_database)?;
            }
            CacheCleanScope::Year(year) => {
                for day in PuzzleDay::MIN..=PuzzleDay::MAX {
                    let day = PuzzleDay::new(u32::from(day)).expect("valid puzzle day");
                    self.remove_puzzle(PuzzleId::new(day, year))?;
                }
                self.remove_file(CacheKey::Calendar(year))?;
                self.database
                    .clear_cache_entries_for_year(year)
                    .map_err(ContentError::from_database)?;
            }
            CacheCleanScope::Puzzle(puzzle) => self.remove_puzzle(puzzle)?,
        }
        Ok(())
    }

    fn load_or_fetch(
        &self,
        key: CacheKey,
        page: AocPage,
        client: &AocClient,
    ) -> ContentResult<PathBuf> {
        if self.is_cached(key)? {
            return Ok(self.cache_path(key));
        }

        let body = client.download(&page)?;
        self.save(key, body.as_bytes())
    }

    fn is_cached(&self, key: CacheKey) -> ContentResult<bool> {
        Ok(self
            .database
            .cache_entry(key)
            .map_err(ContentError::from_database)?
            .is_some_and(|entry| {
                entry.is_valid
                    && entry.relative_path == self.cache_relative_path(key)
                    && self.cache_path(key).is_file()
            }))
    }

    fn save(&self, key: CacheKey, contents: &[u8]) -> ContentResult<PathBuf> {
        let path = self.cache_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        atomic_write(&path, contents)?;
        self.database
            .upsert_cache_entry(&CacheEntry {
                key,
                relative_path: self.cache_relative_path(key),
                byte_size: contents.len() as u64,
                fetched_at: Some(current_unix_timestamp()),
                etag: None,
                last_modified: None,
                is_valid: true,
            })
            .map_err(ContentError::from_database)?;
        Ok(path)
    }

    fn remove_puzzle(&self, puzzle: PuzzleId) -> ContentResult<()> {
        for key in [
            CacheKey::PuzzleHtml(puzzle),
            CacheKey::PuzzleMarkdown(puzzle),
            CacheKey::Input(puzzle),
        ] {
            self.remove_file(key)?;
        }
        Ok(())
    }

    fn remove_file(&self, key: CacheKey) -> ContentResult<()> {
        match fs::remove_file(self.cache_path(key)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        self.database
            .remove_cache_entry(key)
            .map_err(ContentError::from_database)?;
        Ok(())
    }

    fn remove_directory(&self, name: &str) -> ContentResult<()> {
        match fs::remove_dir_all(self.cache_dir.join(name)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn cache_path(&self, key: CacheKey) -> PathBuf {
        self.cache_dir.join(self.cache_relative_path(key))
    }

    fn cache_relative_path(&self, key: CacheKey) -> PathBuf {
        match key {
            CacheKey::PuzzleHtml(puzzle) => PathBuf::from("puzzles").join(format!("{puzzle}.html")),
            CacheKey::PuzzleMarkdown(puzzle) => {
                PathBuf::from("puzzles").join(format!("{puzzle}.md"))
            }
            CacheKey::Input(puzzle) => PathBuf::from("inputs").join(format!("{puzzle}.txt")),
            CacheKey::Calendar(year) => PathBuf::from("calendars").join(format!("year{year}.html")),
        }
    }
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX)
}

#[derive(Debug, Error)]
pub enum ContentError {
    #[error(transparent)]
    Client(#[from] AocClientError),
    #[error("content state schema {found} is newer than supported schema {supported}")]
    NewerStateSchema { found: u32, supported: u32 },
    #[error("content state database is corrupt: {detail}")]
    CorruptStateDatabase { detail: String },
    #[error("content state error: {0}")]
    State(String),
    #[error(transparent)]
    Parser(#[from] ParserError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl ContentError {
    fn from_database(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NewerSchema { found, supported } => {
                Self::NewerStateSchema { found, supported }
            }
            DatabaseError::CorruptDatabase { result } => {
                Self::CorruptStateDatabase { detail: result }
            }
            error => Self::State(error.to_string()),
        }
    }
}

pub type ContentResult<T> = Result<T, ContentError>;
