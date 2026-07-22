use std::path::Path;

use rusqlite::{Connection, Transaction};
use thiserror::Error;

use crate::RuntimeLayout;

const SCHEMA_VERSION: u32 = 1;

pub struct StateDatabase {
    connection: Connection,
}

impl StateDatabase {
    pub fn open(layout: &RuntimeLayout) -> DatabaseResult<Self> {
        let path = layout.database_path();
        Self::open_database(&path)
    }

    fn open_database(path: &Path) -> DatabaseResult<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        verify_integrity(&connection)?;
        migrate(&connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> DatabaseResult<u32> {
        Ok(self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }
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
pub enum DatabaseError {
    #[error("state database schema {found} is newer than supported schema {supported}")]
    NewerSchema { found: u32, supported: u32 },
    #[error("state database is corrupt: {result}")]
    CorruptDatabase { result: String },
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
