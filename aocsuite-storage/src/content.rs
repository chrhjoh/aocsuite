use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use aocsuite_client::{AocClient, AocClientError, AocPage};
use aocsuite_parser::{parse_puzzle_markdown, AocSubmissionResult, ParserError};
use aocsuite_utils::{
    atomic_write, set_owner_only_permissions, LanguageId, PuzzleId, PuzzlePart, PuzzleYear,
    RunHistoryLimit,
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

impl CacheKey {
    fn source_page(self) -> Option<AocPage> {
        match self {
            Self::PuzzleHtml(puzzle) => Some(AocPage::Puzzle(puzzle)),
            Self::Input(puzzle) => Some(AocPage::Input(puzzle)),
            Self::Calendar(year) => Some(AocPage::Calendar(year)),
            Self::PuzzleMarkdown(_) => None,
        }
    }
}

pub struct ContentStore<'client> {
    cache_dir: PathBuf,
    database: StateDatabase,
    client: &'client AocClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheCleanScope {
    All,
    Year(PuzzleYear),
    Date(PuzzleId),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheCleanReport {
    pub removed_files: usize,
    pub already_absent: usize,
}

impl<'client> ContentStore<'client> {
    pub fn open(cache_dir: PathBuf, client: &'client AocClient) -> ContentResult<Self> {
        fs::create_dir_all(&cache_dir)?;
        let database = StateDatabase::open(&cache_dir.join("state.sqlite"))
            .map_err(ContentError::from_database)?;
        Ok(Self {
            cache_dir,
            database,
            client,
        })
    }

    pub fn load_calendar(&self, year: PuzzleYear) -> ContentResult<String> {
        let path = self.load_or_fetch(CacheKey::Calendar(year))?;
        Ok(fs::read_to_string(path)?)
    }

    pub fn ensure_input(&self, puzzle: PuzzleId) -> ContentResult<PathBuf> {
        let path = self.load_or_fetch(CacheKey::Input(puzzle))?;
        set_owner_only_permissions(&path)?;
        Ok(path)
    }

    pub fn ensure_puzzle_markdown(&self, puzzle: PuzzleId) -> ContentResult<PathBuf> {
        let html_path = self.load_or_fetch(CacheKey::PuzzleHtml(puzzle))?;
        let markdown_key = CacheKey::PuzzleMarkdown(puzzle);
        if self.is_cached(markdown_key)? {
            return Ok(self.cache_path(markdown_key));
        }

        let markdown = parse_puzzle_markdown(&fs::read_to_string(html_path)?)?;
        self.save(markdown_key, markdown.as_bytes())
    }

    pub fn record_submission(
        &self,
        puzzle: PuzzleId,
        part: PuzzlePart,
        result: &AocSubmissionResult,
    ) -> ContentResult<()> {
        match result {
            AocSubmissionResult::Correct => {
                self.database
                    .increment_submission_count(puzzle, part, true)
                    .map_err(ContentError::from_database)?;
            }
            AocSubmissionResult::Incorrect
            | AocSubmissionResult::IncorrectTooHigh
            | AocSubmissionResult::IncorrectTooLow => {
                self.database
                    .increment_submission_count(puzzle, part, false)
                    .map_err(ContentError::from_database)?;
                return Ok(());
            }
            _ => return Ok(()),
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

    pub fn record_run_timing(
        &self,
        puzzle: PuzzleId,
        language: LanguageId,
        part: PuzzlePart,
        runtime_ms: u128,
        retention_limit: RunHistoryLimit,
    ) -> ContentResult<()> {
        let duration_nanos = runtime_ms
            .checked_mul(1_000_000)
            .and_then(|duration| u64::try_from(duration).ok())
            .ok_or(ContentError::InvalidRuntime)?;
        self.database
            .record_run_timing(
                puzzle,
                language,
                part,
                duration_nanos,
                retention_limit.get(),
                current_unix_timestamp(),
            )
            .map_err(ContentError::from_database)
    }

    pub fn clean(&self, scope: CacheCleanScope) -> ContentResult<CacheCleanReport> {
        let mut report = CacheCleanReport::default();
        for entry in self
            .database
            .cache_entries()
            .map_err(ContentError::from_database)?
            .into_iter()
            .filter(|entry| scope.includes(entry.key))
        {
            if entry.relative_path != self.cache_relative_path(entry.key) {
                self.database
                    .remove_cache_entry(entry.key)
                    .map_err(ContentError::from_database)?;
                continue;
            }
            match fs::remove_file(self.cache_dir.join(&entry.relative_path)) {
                Ok(()) => report.removed_files += 1,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    report.already_absent += 1;
                }
                Err(error) => return Err(error.into()),
            }
            self.database
                .remove_cache_entry(entry.key)
                .map_err(ContentError::from_database)?;
        }
        Ok(report)
    }

    fn load_or_fetch(&self, key: CacheKey) -> ContentResult<PathBuf> {
        if self.is_cached(key)? {
            return Ok(self.cache_path(key));
        }

        let page = key.source_page().expect("cache only fetch source content");
        let body = self.client.download(&page)?;
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

impl CacheCleanScope {
    fn includes(self, key: CacheKey) -> bool {
        match self {
            Self::All => true,
            Self::Year(year) => match key {
                CacheKey::PuzzleHtml(puzzle)
                | CacheKey::PuzzleMarkdown(puzzle)
                | CacheKey::Input(puzzle) => puzzle.year == year,
                CacheKey::Calendar(calendar_year) => calendar_year == year,
            },
            Self::Date(puzzle) => matches!(
                key,
                CacheKey::PuzzleHtml(entry_puzzle)
                    | CacheKey::PuzzleMarkdown(entry_puzzle)
                    | CacheKey::Input(entry_puzzle)
                    if entry_puzzle == puzzle
            ),
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
    #[error("solver runtime is too large to store")]
    InvalidRuntime,
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

#[cfg(test)]
mod tests {
    use std::fs;

    use aocsuite_client::{AocClient, AocClientOptions, AocPage};
    use aocsuite_parser::AocSubmissionResult;
    use aocsuite_utils::{PuzzleDay, PuzzlePart, PuzzleYear};
    use tempfile::tempdir;

    use super::{CacheCleanScope, CacheKey, ContentStore};

    fn puzzle(day: u32, year: i32) -> aocsuite_utils::PuzzleId {
        aocsuite_utils::PuzzleId::new(
            PuzzleDay::new(day).expect("valid puzzle day"),
            PuzzleYear::new(year).expect("valid puzzle year"),
        )
    }

    fn client() -> AocClient {
        AocClient::new(None, AocClientOptions::default()).expect("create test client")
    }

    #[test]
    fn cleanup_removes_only_indexed_files_for_each_target() {
        let temp = tempdir().expect("create temporary cache root");
        let cache_dir = temp.path().join("cache");
        let client = client();
        let store = ContentStore::open(cache_dir.clone(), &client).expect("open content store");
        let date = puzzle(1, 2024);
        let other_date = puzzle(2, 2025);
        let calendar_2024 = PuzzleYear::new(2024).expect("valid year");
        let calendar_2025 = PuzzleYear::new(2025).expect("valid year");

        for key in [
            CacheKey::PuzzleHtml(date),
            CacheKey::PuzzleMarkdown(date),
            CacheKey::Input(date),
            CacheKey::Calendar(calendar_2024),
            CacheKey::PuzzleHtml(other_date),
            CacheKey::PuzzleMarkdown(other_date),
            CacheKey::Input(other_date),
            CacheKey::Calendar(calendar_2025),
        ] {
            store
                .save(key, b"indexed")
                .expect("save indexed cache file");
        }
        let unindexed = [
            cache_dir.join("puzzles/manual.html"),
            cache_dir.join("inputs/manual.txt"),
            cache_dir.join("calendars/manual.html"),
        ];
        for path in &unindexed {
            fs::write(path, "unindexed").expect("write unmanaged cache file");
        }

        let date_report = store
            .clean(CacheCleanScope::Date(date))
            .expect("clean date cache");
        assert_eq!(date_report.removed_files, 3);
        assert!(!store.cache_path(CacheKey::PuzzleHtml(date)).exists());
        assert!(store.cache_path(CacheKey::Calendar(calendar_2024)).exists());

        let year_report = store
            .clean(CacheCleanScope::Year(calendar_2024))
            .expect("clean year cache");
        assert_eq!(year_report.removed_files, 1);
        assert!(!store.cache_path(CacheKey::Calendar(calendar_2024)).exists());

        let all_report = store.clean(CacheCleanScope::All).expect("clean all cache");
        assert_eq!(all_report.removed_files, 4);
        assert!(unindexed.iter().all(|path| path.exists()));
        assert!(cache_dir.join("state.sqlite").is_file());
        assert_eq!(
            store.clean(CacheCleanScope::All).expect("repeat cleanup"),
            Default::default()
        );
    }

    #[test]
    fn cache_keys_map_only_source_content_to_aoc_pages() {
        let puzzle = puzzle(1, 2024);
        let year = PuzzleYear::new(2024).expect("valid year");

        assert!(matches!(
            CacheKey::PuzzleHtml(puzzle).source_page(),
            Some(AocPage::Puzzle(page)) if page == puzzle
        ));
        assert!(matches!(
            CacheKey::Input(puzzle).source_page(),
            Some(AocPage::Input(page)) if page == puzzle
        ));
        assert!(matches!(
            CacheKey::Calendar(year).source_page(),
            Some(AocPage::Calendar(page_year)) if page_year == year
        ));
        assert!(CacheKey::PuzzleMarkdown(puzzle).source_page().is_none());
    }

    #[test]
    fn recording_a_correct_submission_invalidates_affected_cache_entries() {
        let temp = tempdir().expect("create temporary cache root");
        let cache_dir = temp.path().join("cache");
        let puzzle = puzzle(1, 2024);
        let year = PuzzleYear::new(2024).expect("valid year");
        let client = client();
        let store = ContentStore::open(cache_dir, &client).expect("open content store");

        for key in [
            CacheKey::Calendar(year),
            CacheKey::PuzzleHtml(puzzle),
            CacheKey::PuzzleMarkdown(puzzle),
        ] {
            store.save(key, b"indexed").expect("save indexed content");
        }

        store
            .record_submission(puzzle, PuzzlePart::One, &AocSubmissionResult::Correct)
            .expect("record submission");

        assert!(!store
            .is_cached(CacheKey::Calendar(year))
            .expect("check calendar cache"));
        assert!(!store
            .is_cached(CacheKey::PuzzleHtml(puzzle))
            .expect("check puzzle html cache"));
        assert!(!store
            .is_cached(CacheKey::PuzzleMarkdown(puzzle))
            .expect("check puzzle markdown cache"));
    }
}
