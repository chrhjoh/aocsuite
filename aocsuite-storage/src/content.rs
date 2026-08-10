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

    pub fn refresh_calendar(&self, year: PuzzleYear) -> ContentResult<String> {
        let key = CacheKey::Calendar(year);
        let path = self.cache_path(key);
        self.ensure_replaceable(key)?;

        let body = self.client.download(&AocPage::Calendar(year))?;
        let entry = self.cache_entry(key, body.len());
        replace_with_rollback(&path, body.as_bytes(), || {
            self.database
                .upsert_cache_entry(&entry)
                .map_err(ContentError::from_database)
        })?;
        Ok(body)
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

    pub fn load_puzzle_markdown(&self, puzzle: PuzzleId) -> ContentResult<String> {
        Ok(fs::read_to_string(self.ensure_puzzle_markdown(puzzle)?)?)
    }

    pub fn load_cached_puzzle_markdown(&self, puzzle: PuzzleId) -> ContentResult<Option<String>> {
        let key = CacheKey::PuzzleMarkdown(puzzle);
        if !self.is_cached(key)? {
            return Ok(None);
        }
        Ok(Some(fs::read_to_string(self.cache_path(key))?))
    }

    pub fn download_puzzle_markdown(&self, puzzle: PuzzleId) -> ContentResult<String> {
        let html_key = CacheKey::PuzzleHtml(puzzle);
        let markdown_key = CacheKey::PuzzleMarkdown(puzzle);
        self.ensure_replaceable(html_key)?;
        self.ensure_replaceable(markdown_key)?;

        let html = self.client.download(&AocPage::Puzzle(puzzle))?;
        let markdown = parse_puzzle_markdown(&html)?;
        let html_path = self.cache_path(html_key);
        let markdown_path = self.cache_path(markdown_key);
        let entries = [
            self.cache_entry(html_key, html.len()),
            self.cache_entry(markdown_key, markdown.len()),
        ];
        replace_with_rollback(&html_path, html.as_bytes(), || {
            replace_with_rollback(&markdown_path, markdown.as_bytes(), || {
                self.database
                    .upsert_cache_entries(&entries)
                    .map_err(ContentError::from_database)
            })
        })?;
        Ok(markdown)
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

    fn ensure_replaceable(&self, key: CacheKey) -> ContentResult<()> {
        let path = self.cache_path(key);
        let indexed_at_path = self
            .database
            .cache_entry(key)
            .map_err(ContentError::from_database)?
            .is_some_and(|entry| entry.relative_path == self.cache_relative_path(key));
        match fs::symlink_metadata(&path) {
            Ok(_) if !indexed_at_path => Err(ContentError::UnmanagedCacheFile { path }),
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn save(&self, key: CacheKey, contents: &[u8]) -> ContentResult<PathBuf> {
        let path = self.cache_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        atomic_write(&path, contents)?;
        self.database
            .upsert_cache_entry(&self.cache_entry(key, contents.len()))
            .map_err(ContentError::from_database)?;
        Ok(path)
    }

    fn cache_entry(&self, key: CacheKey, byte_size: usize) -> CacheEntry {
        CacheEntry {
            key,
            relative_path: self.cache_relative_path(key),
            byte_size: byte_size as u64,
            fetched_at: Some(current_unix_timestamp()),
            etag: None,
            last_modified: None,
            is_valid: true,
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

fn replace_with_rollback(
    path: &std::path::Path,
    contents: &[u8],
    persist: impl FnOnce() -> ContentResult<()>,
) -> ContentResult<()> {
    let previous = match fs::read(path) {
        Ok(contents) => Some(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic_write(path, contents)?;
    if let Err(error) = persist() {
        let rollback = match previous {
            Some(previous) => atomic_write(path, &previous),
            None => match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            },
        };
        if let Err(rollback) = rollback {
            return Err(ContentError::RefreshRollback {
                path: path.to_path_buf(),
                persistence: error.to_string(),
                source: rollback,
            });
        }
        return Err(error);
    }
    Ok(())
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
    #[error("refusing to replace unmanaged cache file {path}")]
    UnmanagedCacheFile { path: PathBuf },
    #[error("content refresh persistence failed ({persistence}) and restoring {path} also failed")]
    RefreshRollback {
        path: PathBuf,
        persistence: String,
        #[source]
        source: std::io::Error,
    },
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
    use std::{
        fs,
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc::{self, Receiver},
        thread,
    };

    use aocsuite_client::{AocClient, AocClientOptions};
    use aocsuite_parser::{AocSubmissionResult, ParserError};
    use aocsuite_utils::{PuzzleDay, PuzzlePart, PuzzleYear};
    use tempfile::tempdir;

    use super::{CacheCleanScope, CacheKey, ContentError, ContentStore};

    fn puzzle(day: u32, year: i32) -> aocsuite_utils::PuzzleId {
        aocsuite_utils::PuzzleId::new(
            PuzzleDay::new(day).expect("valid puzzle day"),
            PuzzleYear::new(year).expect("valid puzzle year"),
        )
    }

    fn client() -> AocClient {
        AocClient::new(None, AocClientOptions::default()).expect("create test client")
    }

    fn serve_responses(responses: Vec<(u16, &'static str)>) -> (AocClient, Receiver<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut requests = Vec::new();
            for (status, body) in responses {
                let (mut stream, _) = listener.accept().expect("accept test request");
                let mut request = [0; 4096];
                let bytes = stream.read(&mut request).expect("read test request");
                requests.push(String::from_utf8_lossy(&request[..bytes]).into_owned());
                write!(
                    stream,
                    "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .expect("write test response");
            }
            sender.send(requests).expect("send test requests");
        });
        let client = AocClient::new(
            None,
            AocClientOptions {
                base_url: format!("http://{address}"),
                user_agent: "aocsuite-storage-test/1".to_owned(),
                ..AocClientOptions::default()
            },
        )
        .expect("create test client");
        (client, receiver)
    }

    #[test]
    fn puzzle_markdown_text_is_loaded_cache_first() {
        let temp = tempdir().expect("create temporary cache root");
        let (client, requests) = serve_responses(vec![(
            200,
            "<main><article><h2>Test Puzzle</h2><p>Description.</p></article></main>",
        )]);
        let store =
            ContentStore::open(temp.path().join("cache"), &client).expect("open content store");

        let first = store
            .load_puzzle_markdown(puzzle(1, 2024))
            .expect("load puzzle markdown");
        let second = store
            .load_puzzle_markdown(puzzle(1, 2024))
            .expect("load cached puzzle markdown");

        assert_eq!(second, first);
        assert!(first.contains("Test Puzzle"));
        assert_eq!(requests.recv().expect("receive requests").len(), 1);
    }

    #[test]
    fn puzzle_markdown_download_replaces_an_existing_preview() {
        let temp = tempdir().expect("create temporary cache root");
        let (client, requests) = serve_responses(vec![
            (
                200,
                "<main><article><h2>Original Puzzle</h2><p>Part one.</p></article></main>",
            ),
            (
                200,
                "<main><article><h2>Updated Puzzle</h2><p>Part one.</p></article><article><h2>Part Two</h2><p>New content.</p></article></main>",
            ),
        ]);
        let store =
            ContentStore::open(temp.path().join("cache"), &client).expect("open content store");
        let puzzle = puzzle(1, 2024);

        let original = store
            .load_puzzle_markdown(puzzle)
            .expect("load original markdown");
        let updated = store
            .download_puzzle_markdown(puzzle)
            .expect("download updated markdown");

        assert!(original.contains("Original Puzzle"));
        assert!(updated.contains("Updated Puzzle"));
        assert!(updated.contains("Part Two"));
        assert_eq!(
            store.load_cached_puzzle_markdown(puzzle).unwrap(),
            Some(updated)
        );
        assert_eq!(requests.recv().expect("receive requests").len(), 2);
    }

    #[test]
    fn invalid_puzzle_download_preserves_existing_files_and_metadata() {
        let temp = tempdir().expect("create temporary cache root");
        let (client, requests) = serve_responses(vec![
            (
                200,
                "<main><article><h2>Original Puzzle</h2><p>Part one.</p></article></main>",
            ),
            (200, "<main>missing puzzle article</main>"),
        ]);
        let store =
            ContentStore::open(temp.path().join("cache"), &client).expect("open content store");
        let puzzle = puzzle(1, 2024);
        let original = store
            .load_puzzle_markdown(puzzle)
            .expect("load original markdown");
        let html_key = CacheKey::PuzzleHtml(puzzle);
        let markdown_key = CacheKey::PuzzleMarkdown(puzzle);
        let original_html = fs::read_to_string(store.cache_path(html_key)).unwrap();
        let original_entries = [html_key, markdown_key].map(|key| {
            store
                .database
                .cache_entry(key)
                .expect("read cache metadata")
        });

        assert!(matches!(
            store.download_puzzle_markdown(puzzle),
            Err(ContentError::Parser(ParserError::MissingPuzzleArticle))
        ));
        assert_eq!(
            fs::read_to_string(store.cache_path(html_key)).unwrap(),
            original_html
        );
        assert_eq!(
            store.load_cached_puzzle_markdown(puzzle).unwrap(),
            Some(original)
        );
        assert_eq!(
            [html_key, markdown_key].map(|key| {
                store
                    .database
                    .cache_entry(key)
                    .expect("read cache metadata")
            }),
            original_entries
        );
        assert_eq!(requests.recv().expect("receive requests").len(), 2);
    }

    #[test]
    fn puzzle_markdown_download_does_not_overwrite_an_unindexed_file() {
        let temp = tempdir().expect("create temporary cache root");
        let cache_dir = temp.path().join("cache");
        let client = client();
        let store = ContentStore::open(cache_dir.clone(), &client).expect("open content store");
        let puzzle = puzzle(1, 2024);
        let path = store.cache_path(CacheKey::PuzzleMarkdown(puzzle));
        fs::create_dir_all(path.parent().expect("puzzle has parent"))
            .expect("create puzzle directory");
        fs::write(&path, "unmanaged markdown").expect("write unmanaged markdown");

        assert!(matches!(
            store.download_puzzle_markdown(puzzle),
            Err(ContentError::UnmanagedCacheFile { path: error_path }) if error_path == path
        ));
        assert_eq!(
            fs::read_to_string(path).expect("read unmanaged markdown"),
            "unmanaged markdown"
        );
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
