use std::path::{Component, Path, PathBuf};

use aocsuite_utils::{LanguageId, PuzzleId, PuzzlePart, PuzzleYear};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use thiserror::Error;

use crate::content::CacheKey;

const SCHEMA_VERSION: u32 = 1;

pub(crate) struct StateDatabase {
    connection: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheEntry {
    pub key: CacheKey,
    pub relative_path: PathBuf,
    pub byte_size: u64,
    pub fetched_at: Option<i64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub is_valid: bool,
}

impl StateDatabase {
    pub(crate) fn open(path: &Path) -> DatabaseResult<Self> {
        Self::open_database(path)
    }

    fn open_database(path: &Path) -> DatabaseResult<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        verify_integrity(&connection)?;
        migrate(&connection)?;
        Ok(Self { connection })
    }

    pub(crate) fn cache_entry(&self, key: CacheKey) -> DatabaseResult<Option<CacheEntry>> {
        let (content_type, year, day) = cache_key_parts(key);
        let entry = self
            .connection
            .query_row(
                "
                SELECT relative_path, byte_size, fetched_at, etag, last_modified, is_valid
                FROM cache_entries
                WHERE content_type = ?1 AND year = ?2 AND day = ?3
                ",
                params![content_type, year, day],
                |row| {
                    let relative_path = PathBuf::from(row.get::<_, String>(0)?);
                    let byte_size = row.get::<_, i64>(1)?;
                    let is_valid = row.get::<_, i64>(5)?;
                    Ok((
                        relative_path,
                        byte_size,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        is_valid,
                    ))
                },
            )
            .optional()?;

        entry
            .map(
                |(relative_path, byte_size, fetched_at, etag, last_modified, is_valid)| {
                    let relative_path = validated_relative_path(relative_path)?;
                    let byte_size = u64::try_from(byte_size)
                        .map_err(|_| DatabaseError::InvalidCacheEntry("negative byte size"))?;
                    let is_valid = match is_valid {
                        0 => false,
                        1 => true,
                        _ => return Err(DatabaseError::InvalidCacheEntry("invalid validity flag")),
                    };
                    Ok(CacheEntry {
                        key,
                        relative_path,
                        byte_size,
                        fetched_at,
                        etag,
                        last_modified,
                        is_valid,
                    })
                },
            )
            .transpose()
    }

    pub(crate) fn cache_entries(&self) -> DatabaseResult<Vec<CacheEntry>> {
        let mut statement = self.connection.prepare(
            "
            SELECT content_type, year, day, relative_path, byte_size, fetched_at, etag, last_modified, is_valid
            FROM cache_entries
            ",
        )?;
        let entries = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i64>(2)?,
                    PathBuf::from(row.get::<_, String>(3)?),
                    row.get::<_, i64>(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get::<_, i64>(8)?,
                ))
            })?
            .map(|entry| {
                let (
                    content_type,
                    year,
                    day,
                    relative_path,
                    byte_size,
                    fetched_at,
                    etag,
                    last_modified,
                    is_valid,
                ) = entry?;
                let key = cache_key_from_parts(&content_type, year, day)?;
                let relative_path = validated_relative_path(relative_path)?;
                let byte_size = u64::try_from(byte_size)
                    .map_err(|_| DatabaseError::InvalidCacheEntry("negative byte size"))?;
                let is_valid = match is_valid {
                    0 => false,
                    1 => true,
                    _ => return Err(DatabaseError::InvalidCacheEntry("invalid validity flag")),
                };
                Ok(CacheEntry {
                    key,
                    relative_path,
                    byte_size,
                    fetched_at,
                    etag,
                    last_modified,
                    is_valid,
                })
            })
            .collect::<DatabaseResult<Vec<_>>>()?;
        Ok(entries)
    }

    pub(crate) fn upsert_cache_entry(&self, entry: &CacheEntry) -> DatabaseResult<()> {
        let relative_path = validated_relative_path(entry.relative_path.clone())?;
        let byte_size = i64::try_from(entry.byte_size)
            .map_err(|_| DatabaseError::InvalidCacheEntry("byte size exceeds SQLite range"))?;
        let (content_type, year, day) = cache_key_parts(entry.key);
        self.connection.execute(
            "
            INSERT INTO cache_entries (
                content_type, year, day, relative_path, byte_size, fetched_at, etag, last_modified, is_valid
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT (content_type, year, day) DO UPDATE SET
                relative_path = excluded.relative_path,
                byte_size = excluded.byte_size,
                fetched_at = excluded.fetched_at,
                etag = excluded.etag,
                last_modified = excluded.last_modified,
                is_valid = excluded.is_valid
            ",
            params![
                content_type,
                year,
                day,
                relative_path.to_string_lossy(),
                byte_size,
                entry.fetched_at,
                entry.etag,
                entry.last_modified,
                i64::from(entry.is_valid),
            ],
        )?;
        Ok(())
    }

    pub(crate) fn remove_cache_entry(&self, key: CacheKey) -> DatabaseResult<bool> {
        let (content_type, year, day) = cache_key_parts(key);
        Ok(self.connection.execute(
            "DELETE FROM cache_entries WHERE content_type = ?1 AND year = ?2 AND day = ?3",
            params![content_type, year, day],
        )? > 0)
    }

    pub(crate) fn invalidate_cache_entry(&self, key: CacheKey) -> DatabaseResult<bool> {
        let (content_type, year, day) = cache_key_parts(key);
        Ok(self.connection.execute(
            "
            UPDATE cache_entries
            SET is_valid = 0
            WHERE content_type = ?1 AND year = ?2 AND day = ?3 AND is_valid = 1
            ",
            params![content_type, year, day],
        )? > 0)
    }

    pub(crate) fn increment_submission_count(
        &self,
        puzzle: PuzzleId,
        part: PuzzlePart,
        correct: bool,
    ) -> DatabaseResult<()> {
        let (correct_count, incorrect_count) = if correct { (1, 0) } else { (0, 1) };
        self.connection.execute(
            "
            INSERT INTO submission_counts (year, day, part, correct_count, incorrect_count)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT (year, day, part) DO UPDATE SET
                correct_count = correct_count + excluded.correct_count,
                incorrect_count = incorrect_count + excluded.incorrect_count
            ",
            params![
                puzzle.year.get(),
                puzzle.day.get(),
                puzzle_part_value(part),
                correct_count,
                incorrect_count,
            ],
        )?;
        Ok(())
    }

    pub(crate) fn record_run_timing(
        &self,
        puzzle: PuzzleId,
        language: LanguageId,
        part: PuzzlePart,
        duration_nanos: u64,
        retention_limit: usize,
        recorded_at: i64,
    ) -> DatabaseResult<()> {
        let duration_nanos = i64::try_from(duration_nanos)
            .map_err(|_| DatabaseError::InvalidTiming("duration exceeds SQLite range"))?;
        let retention_limit = i64::try_from(retention_limit)
            .map_err(|_| DatabaseError::InvalidTiming("retention limit exceeds SQLite range"))?;
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "
            INSERT INTO run_timings (year, day, language, part, duration_nanos, recorded_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                puzzle.year.get(),
                puzzle.day.get(),
                language.to_string(),
                puzzle_part_value(part),
                duration_nanos,
                recorded_at,
            ],
        )?;
        transaction.execute(
            "
            DELETE FROM run_timings
            WHERE id IN (
                SELECT id FROM run_timings
                WHERE year = ?1 AND day = ?2 AND language = ?3 AND part = ?4
                ORDER BY recorded_at DESC, id DESC
                LIMIT -1 OFFSET ?5
            )
            ",
            params![
                puzzle.year.get(),
                puzzle.day.get(),
                language.to_string(),
                puzzle_part_value(part),
                retention_limit,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn cache_key_parts(key: CacheKey) -> (&'static str, i32, i64) {
    match key {
        CacheKey::PuzzleHtml(puzzle) => (
            "puzzle_html",
            puzzle.year.get(),
            i64::from(puzzle.day.get()),
        ),
        CacheKey::PuzzleMarkdown(puzzle) => (
            "puzzle_markdown",
            puzzle.year.get(),
            i64::from(puzzle.day.get()),
        ),
        CacheKey::Input(puzzle) => ("input", puzzle.year.get(), i64::from(puzzle.day.get())),
        CacheKey::Calendar(year) => ("calendar", year.get(), 0),
    }
}

fn cache_key_from_parts(content_type: &str, year: i32, day: i64) -> DatabaseResult<CacheKey> {
    let year = PuzzleYear::new(year)
        .map_err(|_| DatabaseError::InvalidCacheEntry("invalid puzzle year"))?;
    let puzzle = || -> DatabaseResult<PuzzleId> {
        let day = u32::try_from(day)
            .ok()
            .and_then(|day| aocsuite_utils::PuzzleDay::new(day).ok())
            .ok_or(DatabaseError::InvalidCacheEntry("invalid puzzle day"))?;
        Ok::<PuzzleId, DatabaseError>(PuzzleId::new(day, year))
    };
    match content_type {
        "puzzle_html" => Ok(CacheKey::PuzzleHtml(puzzle()?)),
        "puzzle_markdown" => Ok(CacheKey::PuzzleMarkdown(puzzle()?)),
        "input" => Ok(CacheKey::Input(puzzle()?)),
        "calendar" if day == 0 => Ok(CacheKey::Calendar(year)),
        "calendar" => Err(DatabaseError::InvalidCacheEntry("invalid calendar day")),
        _ => Err(DatabaseError::InvalidCacheEntry("invalid content type")),
    }
}

fn puzzle_part_value(part: PuzzlePart) -> i64 {
    match part {
        PuzzlePart::One => 1,
        PuzzlePart::Two => 2,
    }
}

fn validated_relative_path(path: PathBuf) -> DatabaseResult<PathBuf> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(DatabaseError::InvalidCacheEntry(
            "cache path must be a non-empty relative path",
        ));
    }
    Ok(path)
}

fn verify_integrity(connection: &Connection) -> DatabaseResult<()> {
    let result: String = connection.query_row("PRAGMA integrity_check;", [], |row| row.get(0))?;
    if result == "ok" {
        Ok(())
    } else {
        Err(DatabaseError::CorruptDatabase { result })
    }
}

fn migrate(connection: &Connection) -> DatabaseResult<()> {
    let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(DatabaseError::NewerSchema {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    let transaction = connection.unchecked_transaction()?;
    if version == 0 {
        migrate_to_version_one(&transaction)?;
    }
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_to_version_one(transaction: &Transaction<'_>) -> DatabaseResult<()> {
    transaction.execute_batch(
        "
        CREATE TABLE cache_entries (
            content_type TEXT NOT NULL,
            year INTEGER NOT NULL,
            day INTEGER NOT NULL DEFAULT 0,
            relative_path TEXT NOT NULL,
            byte_size INTEGER NOT NULL,
            fetched_at INTEGER,
            etag TEXT,
            last_modified TEXT,
            is_valid INTEGER NOT NULL CHECK (is_valid IN (0, 1)),
            PRIMARY KEY (content_type, year, day)
        ) STRICT;

        CREATE TABLE calendar_stars (
            year INTEGER NOT NULL,
            day INTEGER NOT NULL,
            stars INTEGER NOT NULL CHECK (stars BETWEEN 0 AND 2),
            PRIMARY KEY (year, day)
        ) STRICT;

        CREATE TABLE submission_counts (
            year INTEGER NOT NULL,
            day INTEGER NOT NULL,
            part INTEGER NOT NULL CHECK (part IN (1, 2)),
            correct_count INTEGER NOT NULL DEFAULT 0 CHECK (correct_count >= 0),
            incorrect_count INTEGER NOT NULL DEFAULT 0 CHECK (incorrect_count >= 0),
            PRIMARY KEY (year, day, part)
        ) STRICT;

        CREATE TABLE run_timings (
            id INTEGER PRIMARY KEY,
            year INTEGER NOT NULL,
            day INTEGER NOT NULL,
            language TEXT NOT NULL,
            part INTEGER NOT NULL CHECK (part IN (1, 2)),
            duration_nanos INTEGER NOT NULL CHECK (duration_nanos >= 0),
            recorded_at INTEGER NOT NULL
        ) STRICT;

        CREATE INDEX run_timings_retention
            ON run_timings (year, day, language, part, recorded_at DESC, id DESC);
        ",
    )?;
    Ok(())
}

#[derive(Debug, Error)]
pub(crate) enum DatabaseError {
    #[error("state database schema {found} is newer than supported schema {supported}")]
    NewerSchema { found: u32, supported: u32 },
    #[error("state database is corrupt: {result}")]
    CorruptDatabase { result: String },
    #[error("invalid cache entry: {0}")]
    InvalidCacheEntry(&'static str),
    #[error("invalid run timing: {0}")]
    InvalidTiming(&'static str),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
}

pub(crate) type DatabaseResult<T> = Result<T, DatabaseError>;
