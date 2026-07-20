# Implementation Plan

## Goal

Refactor the existing CLI and shared crates before creating `aocsuite-tui`. Resolve every issue in `IMPLEMENTATION_NOTES.md`, preserve ordinary successful CLI workflows, and add behavior-focused tests to every existing crate. Start TUI work only after this milestone passes full verification.

The application assumes one AoC Suite process per runtime root. Do not add normal cross-process coordination.

## Delivery Rules

- Work one issue or tightly coupled boundary at a time. Add focused regression and behavior tests with each change.
- Keep `aocsuite-cli` functional throughout. Preserve command names and normal successful workflows unless migration makes a documented change unavoidable.
- Keep source files, templates, libraries, Cargo projects, Python environments, and cached AoC bodies as filesystem artifacts. SQLite stores only rebuildable metadata.
- Do not begin `aocsuite-tui` until the pre-TUI milestone below is complete.

## Phase 1: Runtime Storage And Configuration

1. Define the versioned single-root layout in `STORAGE.md`: non-secret typed preferences, owner-only session file, cache, workspace, metadata database, and transient run files.
2. Add bundled SQLite metadata for cache validity, hashes, fetch state, and layout/schema versions. Cache files remain canonical and rebuild the index if the database is missing or corrupt.
3. Add atomic writes for settings, secrets, cache files, generated result files, and metadata updates. Remove panic-based routine I/O/config paths.
4. Implement recoverable automatic migration: migration marker, timestamped backup, workspace/Git move, cache move, session extraction, metadata indexing, and generated Rust/Python harness migration. Preserve user solutions, templates, libraries, and dependency declarations.
5. Split noninteractive configuration reads/writes from CLI prompting. Redact session reads and preserve `AOC_*` configuration behavior.
6. Add behavior tests in `aocsuite-utils`, `aocsuite-config`, and `aocsuite-fs` for paths, migration, permissions, malformed state, recovery, cache deletion, and configuration precedence.

## Phase 2: HTTP, Cache, And Parser Boundaries

1. Replace fixed global request helpers with a configurable, testable client using an optional session, finite timeout, AoC user agent, and bounded retry/backoff for transient GET failures only. Never retry answer submissions.
2. Require sessions for inputs, submissions, and private leaderboards. Permit public puzzle, calendar, and global leaderboard requests without one, attaching a session if available.
3. Return typed HTTP/status/auth errors and reject invalid response bodies before writing cache files. Preserve valid cached files on failed requests.
4. Split pure cache-path/status queries from explicit fetch/refresh actions. Fix input validity, cache invalidation, and idempotent clean behavior.
5. Replace string parser dispatch with separate fallible typed puzzle, calendar, and submission APIs. Calendar parsing returns semantic cells/styles; CLI ANSI and future Ratatui rendering are separate adapters.
6. Preserve sanitized server text for unknown submission responses, recognize rate-limit variants, and fail visibly on missing/changed puzzle/calendar structure.
7. Add local HTTP mock-server and parser-fixture coverage in `aocsuite-client`, `aocsuite-fs`, and `aocsuite-parser`.

## Phase 3: Language Execution And Workspaces

1. Complete: selected day/year activation occurs before compile/run and missing day sources are created from the template for Rust and Python.
2. Complete: pure path/list operations are split from explicit workspace, environment, compile, and run setup. Query and clean operations do not create a virtual environment or compile project.
3. Generate Python `main.py` in fresh workspaces and correct generated Python placeholder behavior.
4. Replace root-level `result.json` with a unique per-run temporary JSON path, atomically written and cleared after validated consumption. Initial TUI behavior waits for one background job; cancellation is deferred.
5. Expose public structured run results and command diagnostics. Libraries must not print subprocess output directly.
6. Make generated harnesses and dependency scaffolding versioned/migratable, add a persistent Python dependency manifest, and correct Windows path/executable behavior.
7. Fix safe active-link replacement, library-name validation, process failure propagation, and workspace cleanup semantics.
8. Add focused Rust/Python behavior tests in `aocsuite-lang` for fresh setup, source selection, results, command errors, templates, dependencies, migrations, and cleanup.

## Phase 4: Remaining CLI, Editor, And Git Behavior

1. Extract typed noninteractive operations from `run_aocsuite`; keep it as the CLI formatting/prompt adapter.
2. Correct release/default-date handling, remove the 2025 day 13-25 restriction, and cover Eastern-time boundaries without wall-clock tests.
3. Make command parsing match documented submit/run behavior and preserve compatible CLI syntax where practical.
4. Fix Git empty-argument/clone behavior and scope Git to `workspace/`; do not claim Git arguments are sandboxed.
5. Fix editor resolution, path handling, argument escaping, exit-status propagation, and platform support.
6. Retain empty-line destructive confirmation behavior while treating EOF as cancellation, with explicit regression coverage.
7. Add behavior tests to `aocsuite-cli` and `aocsuite-editor`, using fake process launchers rather than real Git, editors, Cargo, Python, pip, or browsers.

## Pre-TUI Milestone

Before adding `aocsuite-tui`:

- Every issue in `IMPLEMENTATION_NOTES.md` is resolved or has an explicit accepted product decision recorded there.
- Every existing workspace crate has behavior-focused test coverage for its public responsibilities and each corrected defect.
- `cargo check --workspace`, `cargo test --workspace`, and `cargo run -p aocsuite-cli -- --help` pass.
- The CLI operates through the same structured, noninteractive shared boundaries intended for the TUI.

## Phase 5: TUI

1. Add `aocsuite-tui` to the workspace with Ratatui and terminal lifecycle handling.
2. Implement a pure state reducer plus serialized background-effect runner; rendering and event handling perform no blocking I/O.
3. Build full command parity against the shared operations, not through `run_aocsuite` or parsed CLI output.
4. Add reducer, render, event, terminal-restoration, and CLI/TUI parity tests using `ratatui::TestBackend` and fake services.
