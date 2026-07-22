# Implementation Plan

## Goal

Refactor the existing CLI and shared crates before creating `aocsuite-tui`. Resolve every issue in `IMPLEMENTATION_NOTES.md`, preserve ordinary successful CLI workflows, and add behavior-focused tests to every existing crate. Start TUI work only after this milestone passes full verification.

The application assumes one AoC Suite process per runtime root. Do not add normal cross-process coordination.

## Delivery Rules

- Work one issue or tightly coupled boundary at a time. Add focused regression and behavior tests with each change.
- Keep `aocsuite-cli` functional throughout. Preserve command names and normal successful workflows unless migration makes a documented change unavoidable.
- Keep source files, templates, libraries, complete language projects, Python environments, examples, and cached AoC bodies as filesystem artifacts. SQLite stores cache indexes, calendar-derived stars, submission counts, and bounded recent run timings.
- Do not begin `aocsuite-tui` until the pre-TUI milestone below is complete.

## Phase 1: Runtime Layout And Configuration

1. Add validated UI-neutral puzzle/language types and the shared process executor under `aocsuite-utils`; remove Clap derives from shared types.
2. Remove configuration discovery from `aocsuite-client`, `aocsuite-lang`, and the launcher boundary. Construct services with explicit settings, paths, environment snapshots, and executors.
3. Add a broad `aocsuite-storage` crate replacing `aocsuite-fs`. It owns `RuntimeLayout`, bootstrap/versioning, SQLite, AoC content lifecycle, workspace Git, transient run allocation, uninstall, and typed cleanup scopes.
4. Keep storage internally layered: layout/database modules do not call HTTP/parser code; only the content module depends on the configuration-independent client and semantic parser.
5. Implement public storage-owned `get_aocsuite_dir` resolution and pass its result into explicit `RuntimeLayout::new(root_dir)`. Treat `AOCSUITE_DATA_DIR` as the complete-root override before XDG/HOME defaults. Path getters must not create directories, migrate state, fetch content, or launch commands.
6. Establish layout version 1 with non-secret configuration values, owner-only session storage, SQLite state, disposable AoC cache, transient runs, and a bootstrapped Git workspace.
7. Bootstrap every application invocation before configuration/service construction. Reject nonempty unversioned roots and newer unsupported layouts without mutation; legacy import is out of scope.
8. Split noninteractive typed configuration and session reads/writes from CLI prompting. Remove `template_dir` and all `AOC_*` configuration sources, and add a configurable run-history limit defaulting to 10.
9. Add explicit-temporary-root and fake-executor behavior tests for layout validation, initialization, unsupported roots, permissions, failed writes, and malformed manifests.

## Storage Implementation Sequence

1. Add shared validated domain types and a synchronous captured/foreground `CommandExecutor` to `aocsuite-utils`.
2. Refactor `aocsuite-client` to accept an optional session and request options explicitly, then remove its config dependency so storage can depend on it without a cycle.
3. Add `aocsuite-storage` with bundled `rusqlite` and `walkdir`; add `tempfile` for tests. Implement `RuntimeLayout`, `.aocsuite-layout.json`, fresh bootstrap, and unversioned-root rejection before SQLite/content behavior. `ContentStore` owns `cache/state.sqlite` and cache paths; the manifest owns physical layout compatibility, while SQLite `user_version` owns only database schema migrations.
4. Refactor `aocsuite-config` around a layout-provided configuration directory, non-mutating reads, explicit writes, an owner-only session file, and frontend-owned prompts. Remove Clap, `rpassword`, `template_dir`, and environment configuration sources from the library.
5. Absorb `aocsuite-fs` into storage. Replace invalid `AocContentFile` states and side-effecting `to_path()` with typed cache keys, pure paths/status/reads, and explicit load/refresh/invalidate/clean methods; then remove the old crate.
6. Store raw puzzle HTML canonically and derived Markdown as a disposable editor artifact. Use flat content-specific cache directories keyed by `year{year}_day{day}`, with calendars keyed by year. Return semantic calendar/submission models from parser APIs and keep terminal formatting in frontends.
7. Move language roots to `workspace/rust` and `workspace/python`, flat solution files to each project's `solutions/`, shared flat examples to `workspace/examples`, and Git scope to the bootstrapped `workspace/`.
8. Track complete portable language projects. Regenerate the AoC Suite-owned `.gitignore`; ignore only Rust build output, Python virtual environments/caches, and active solution links.
9. Move typed captured/foreground Git operations into `aocsuite-storage::workspace`. Clone runs into the bootstrapped workspace.
10. Keep generated harnesses and `.aocsuite-runtime.json` tracked. Version-only migrations overwrite AoC Suite-owned harnesses atomically and preserve solutions, templates, libraries, examples, and dependency files.
11. Make Rust package operations edit tracked `Cargo.toml` semantically and track `Cargo.lock`. Add tracked Python `requirements.txt`; atomically persist `pip freeze` after successful package mutations.
12. Rename `aocsuite-editor` to `aocsuite-launcher`, move browser launching out of the HTTP client, and route editor/browser processes through the shared executor without config discovery or printing.
13. Add SQLite bootstrap, integrity checks, typed corrupt-database errors, transactional schema upgrades, and newer-schema rejection.
14. Replace `.aoccache.json` with SQLite cache metadata. Rebuild recognized entries as stale and reparse cached calendar HTML to restore stars.
15. Add submission counts for correct/incorrect outcomes only and retain the latest configurable per-part runtimes, defaulting to 10.
16. Expose a high-level structured language run API that owns activation through result consumption, records timings, and serializes all active-link-changing jobs.
17. Add typed idempotent cleanup and uninstall plans/reports. Normal cache clean preserves examples; explicit example or comprehensive clean may remove them.
18. Add fixture-driven tests for workspace portability, harness migration, Cargo preservation, requirements regeneration, content recovery, Git modes, cleanup, database corruption, and timing pruning.

## Phase 2: HTTP, Cache, And Parser Boundaries

1. Replace fixed global request helpers with a configurable, testable client using an optional session, finite timeout, AoC user agent, and bounded retry/backoff for transient GET failures only. Never retry answer submissions.
2. Require sessions for inputs, submissions, and private leaderboards. Permit public puzzle, calendar, and global leaderboard requests without one, attaching a session if available.
3. Return typed HTTP/status/auth errors and reject invalid response bodies before writing cache files. Preserve valid cached files on failed requests.
4. Complete the split between pure cache-path/status queries and explicit fetch/refresh actions. Fix input validity, cache invalidation, and idempotent clean behavior.
5. Replace string parser dispatch with separate fallible typed puzzle, calendar, and submission APIs. Calendar parsing returns semantic cells, styles, and validated puzzle dates; CLI ANSI and future Ratatui rendering are separate adapters.
6. Preserve sanitized server text for unknown submission responses, recognize rate-limit variants, and fail visibly on missing/changed puzzle/calendar structure.
7. Add local HTTP mock-server and parser-fixture coverage in `aocsuite-client`, `aocsuite-storage`, and `aocsuite-parser`.

## Phase 3: Language Execution And Workspaces

1. Complete: selected day/year activation occurs before compile/run and missing day sources are created from the template for Rust and Python.
2. Complete the split between pure path/list operations and explicit workspace, environment, compile, and run setup. Query and clean operations do not create a virtual environment or compile projects.
3. Generate Python `main.py` in fresh workspaces and correct generated Python placeholder behavior.
4. Replace root-level `result.json` with a unique per-run temporary JSON path, atomically written and cleared after validated consumption. Initial TUI behavior waits for one background job; cancellation is deferred.
5. Expose public structured run requests/results and command diagnostics. Libraries must not print subprocess output directly or accept stringly typed part selections.
6. Track versioned generated harnesses, Rust Cargo files, and Python `requirements.txt` in the Git workspace. Harness migrations overwrite versioned AoC Suite-owned files while preserving user files and dependency declarations.
7. Fix safe active-link replacement, library-name validation, process failure propagation, and workspace cleanup semantics. Serialize all jobs spanning active-link mutation, build, and execution.
8. Add focused Rust/Python behavior tests in `aocsuite-lang` for fresh setup, source selection, results, command errors, templates, dependencies, migrations, and cleanup.

## Phase 4: Remaining CLI, Editor, And Git Behavior

1. Refactor `run_aocsuite` into thin command handlers over shared typed domain services; keep Clap, prompts, confirmation, and rendering in the CLI without introducing a general operations crate.
2. Correct release/default-date handling, and cover Eastern-time boundaries without wall-clock tests.
3. Make command parsing match documented submit/run behavior and preserve compatible CLI syntax where practical.
4. Move Git operations into storage workspace services and scope their normal working directory to `workspace/`; do not claim pass-through Git arguments are sandboxed.
5. Rename editor support to `aocsuite-launcher`, combine editor/browser launching, and preserve explicit foreground terminal handoff in frontends.
6. Retain empty-line destructive confirmation behavior while treating EOF as cancellation, with explicit regression coverage.
7. Add behavior tests to `aocsuite-cli` and `aocsuite-launcher`, using fake processes rather than real Git, editors, Cargo, Python, pip, or browsers.

## Pre-TUI Milestone

Before adding `aocsuite-tui`:

- Every issue in `IMPLEMENTATION_NOTES.md` is resolved or has an explicit accepted product decision recorded there.
- Complete refactor of other workspace crates including HTTP, storage and parser changes.
- Every existing workspace crate has behavior-focused test coverage for its public responsibilities and each corrected defect.
- `cargo check --workspace`, `cargo test --workspace`, and `cargo run -p aocsuite-cli -- --help` pass.
- The CLI operates through the same typed storage, language, parser, client, config, and launcher services intended for the TUI; frontends duplicate only presentation, confirmation, and job scheduling.
- Required GitHub CI checks pass on supported platforms without real AoC, subprocess, editor/browser, or developer-runtime dependencies.

## Phase 5: CI And Release Automation

1. Add baseline GitHub Actions for locked workspace check/test and the CLI help smoke test on pull requests and default-branch pushes.
2. After deterministic process/environment seams land, expand required tests to Ubuntu, Windows, and macOS. Add formatting, strict Clippy, and rustdoc gates only after their existing baselines are fixed.
3. Add weekly and dependency-PR `cargo-deny` checks plus Dependabot updates for Cargo and GitHub Actions.
4. Add a tag-driven release workflow that validates synchronized versions, builds native CLI artifacts, smoke-tests them, publishes checksums, and creates a GitHub Release.
5. Add `aocsuite-tui` to the same release version and artifacts after it reaches parity. Keep crates.io publication, code signing/notarization, coverage gates, fuzzing, and MSRV checks optional until their policies are defined.

See `docs/CI.md` for workflow structure, targets, permissions, and staged rollout.

## Phase 6: TUI

1. Add `aocsuite-tui` to the workspace with Ratatui and terminal lifecycle handling.
2. Implement a pure state reducer plus serialized background-effect runner; rendering and event handling perform no blocking I/O.
3. Build full command parity against the shared domain services, not through `run_aocsuite` or parsed CLI output.
4. Add reducer, render, event, terminal-restoration, and CLI/TUI parity tests using `ratatui::TestBackend` and fake services.
