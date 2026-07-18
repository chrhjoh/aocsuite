# Storage Design

## Operating Assumption

AoC Suite is a simple single-user application. Only one AoC Suite process is expected to use a runtime root at a time. Do not add cross-process locking, coordination, or concurrency handling for normal cache, workspace, language, or metadata operations.

The TUI still serializes its own blocking jobs so it remains responsive and does not start two solver runs at once.

## Persistence Model

Use one hardened runtime root. Keep files as the canonical representation of user work and cached AoC content; use bundled SQLite for cache metadata, storage schema versions, and durable application history.

Do not store source files, templates, libraries, Cargo projects, Python environments, editor-opened files, cache content blobs, or the AoC session in SQLite.

The database stores cache metadata, puzzle progress, local run history, submission history, and schema versions. It does not persist private leaderboard data.

```text
aocsuite/
  config.json                # Typed non-secret preferences
  secrets/session            # Owner-only AoC session token
  state.sqlite               # Cache metadata and schema versions
  cache/aoc/                 # Disposable downloaded puzzle/input/calendar files
  workspace/                 # Git root for Rust/Python solutions, templates, and libraries
  runs/                      # Unique transient solver result files
```

`workspace/` is the only Git-managed area. Cache, generated build products, environments, transient runs, and secrets are excluded from Git.

## Cache Metadata

`state.sqlite` indexes cache entries by content type, year, day, file path, hash, size, fetch time, HTTP validation data, and validity/error state. Cache files are canonical; the database is an index.

Write cache files to a same-directory temporary path and atomically replace the destination before updating the SQLite entry. If the database is missing or corrupt, rebuild its index from the cache files and mark entries without verifiable metadata stale.

Downloaded AoC content is disposable. `clean cache` removes only cached AoC files and their metadata; it does not remove preferences, secrets, or workspaces.

## Puzzle Progress And History

SQLite records puzzle progress by year, day, and part, including completion status and any observed completion time. This supports calendar and progress views without treating cached AoC pages as the source of user state.

Local run history records the selected puzzle, runner and language, source revision when available, execution time, duration, exit status, and answer hashes. Run result files remain transient; the history is durable and must not contain puzzle inputs or plaintext answers.

Submission history records the selected puzzle and part, submission time, hash of the submitted answer, AoC outcome, and any cooldown expiry. It prevents accidental duplicate submissions while keeping submitted answers out of SQLite. It must never include the session token.

Do not persist private leaderboard membership, names, rankings, or completion data. Leaderboard data is opt-in on AoC and must not be retained locally.

## Configuration And Secrets

`config.json` contains non-secret typed preferences and is written atomically. The session token is stored separately in `secrets/session` with owner-only permissions and is redacted in all normal output. `AOC_SESSION` remains a nonpersistent configuration source.

## Migration

The storage redesign migrates existing runtime data automatically:

1. Create an exclusive short-lived migration marker.
2. Create a timestamped backup beside the existing runtime root.
3. Move legacy source and Git state into `workspace/`.
4. Move downloaded data into `cache/aoc/`.
5. Move the session from legacy configuration into `secrets/session`.
6. Initialize typed preferences and `state.sqlite`, then index cache files.
7. Record the completed layout version and remove the migration marker.

Retain the backup for recovery. The migration also updates owned generated Rust/Python harness files and dependency scaffolding while preserving user solutions, templates, libraries, and dependency declarations.

## Deferred Features

SQLite does not yet store search indexes. Reconsider expanding its role only when that feature is scheduled.
