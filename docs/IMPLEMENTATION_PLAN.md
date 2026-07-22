# Implementation Plan

## Goal

Refactor the existing CLI and shared crates before creating `aocsuite-tui`. Resolve the documented pre-TUI blockers and preserve ordinary successful CLI workflows. Start TUI work only after this milestone passes full verification.

The application assumes one AoC Suite process per runtime root. Do not add normal cross-process coordination.

## Test Policy

Do not add, expand, or modify tests unless the user explicitly requests test work. Existing tests may be run for verification. Test-related roadmap items below are deferred until explicitly requested.

## Current Status

Reviewed 2026-07-22. The pre-TUI refactor is in progress; do not treat a phase as complete merely because its crate or initial schema exists.

Completed foundations:

- `aocsuite-utils` now owns UI-neutral puzzle/language values and the synchronous `CommandExecutor`.
- Client construction accepts explicit optional sessions and request settings; configuration uses explicit paths and non-mutating reads.
- `aocsuite-storage` has replaced `aocsuite-fs` and contains initial layout, SQLite, content, and workspace services.
- Parser calendar and submission APIs return semantic data, while CLI renders calendar presentation.
- Workspace paths, examples, Git execution, generated `.gitignore`, and transient run-result allocation are storage-owned.
- Language projects use flat solution paths, generated runtime manifests, safe active-link replacement, and unique atomic result files.
- `aocsuite-launcher` owns browser/editor process execution, explicit editor working directories, typed puzzle-open requests, and generic fallback behavior for unrecognized exact editor executables.
- Baseline GitHub Actions runs locked workspace check/test and the CLI help smoke test on Ubuntu.

Remaining pre-TUI blockers:

- Finish storage lifecycle policy: strict unversioned-root rejection, cache recovery/validation, stars, submission counts, timing retention, typed cleanup, and uninstall safety.
- Complete portable language-project behavior: activate and migrate before compilation, one serialized typed run operation, public structured results, tracked Cargo preservation, and Python requirements persistence.
- Reduce CLI orchestration to frontend mapping over injectable domain services.
- Expand CI beyond its baseline only when explicitly requested test work establishes the required deterministic coverage.

Current local verification: `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo run -p aocsuite-cli --locked -- --help`, and `cargo fmt --all -- --check` pass. Strict Clippy is not yet clean: `aocsuite-utils/src/process.rs` triggers `clippy::result_large_err` for `CommandError`.

## Delivery Rules

- Work one issue or tightly coupled boundary at a time. Do not add or modify tests unless explicitly requested.
- Keep `aocsuite-cli` functional throughout. Preserve command names and normal successful workflows unless migration makes a documented change unavoidable.
- Keep source files, templates, libraries, complete language projects, Python environments, examples, and cached AoC bodies as filesystem artifacts. SQLite stores cache indexes, calendar-derived stars, submission counts, and bounded recent run timings.
- Do not begin `aocsuite-tui` until the pre-TUI milestone below is complete.

## Phase 1: Runtime Layout And Configuration (In Progress)

The domain/process foundation is complete. Storage, layout, and configuration work remains incomplete until the lifecycle requirements below are met.

1. Complete: add validated UI-neutral puzzle/language types and the shared process executor under `aocsuite-utils`; remove Clap derives from shared types.
2. Partially complete: client and language receive explicit settings/paths/executors, config is explicit-path based, and launcher migration is complete. Finish the remaining typed configuration values.
3. Partially complete: `aocsuite-storage` replaces `aocsuite-fs` and owns initial `RuntimeLayout`, SQLite, content, workspace Git, and transient run allocation. Add uninstall safety and typed cleanup scopes.
4. Complete for the initial storage modules: layout/database do not call HTTP/parser code, and content is the only module using the configuration-independent client and semantic parser. Preserve this direction as storage grows.
5. Partially complete: `get_aocsuite_dir` and explicit `RuntimeLayout::new(root_dir)` exist with the required environment precedence. Make all remaining storage path getters pure and cover resolver/layout behavior with explicit-root tests.
6. Partially complete: establish layout version 1 with configuration/session directories, SQLite state, disposable cache, transient runs, and a bootstrapped workspace. Complete lifecycle behavior and owner-only/session coverage.
7. Partially complete: CLI bootstraps before configuration/service construction and rejects every unversioned root, including empty roots. Use the typed newer-layout error without mutation.
8. Partially complete: split noninteractive configuration/session access from CLI prompting and remove `template_dir` plus `AOC_*` sources. Finish typed configuration values and wire the run-history limit through timing retention.
9. Defer additional layout and configuration test coverage until explicitly requested.

## Storage Implementation Sequence (In Progress)

1. Complete: add shared validated domain types and a synchronous captured/foreground `CommandExecutor` to `aocsuite-utils`.
2. Complete: refactor `aocsuite-client` to accept an optional session and request options explicitly, then remove its config dependency.
3. Partially complete: add storage, bundled SQLite, layout manifests, database schema versioning, and initial bootstrap. Reject existing unversioned roots and newer-version layouts without mutation.
4. Mostly complete: refactor config around an explicit directory, non-mutating reads, explicit writes, an owner-only session, and frontend prompting. Replace the remaining string-backed settings map with typed settings as configuration grows.
5. Partially complete: remove `aocsuite-fs` and add initial typed cache keys. Expose pure path/status/read operations separately from load/refresh/invalidate/clean behavior.
6. Partially complete: use flat cache paths, raw puzzle HTML, derived Markdown, and semantic parser output. Validate puzzle bodies before replacing cached data; add cache recovery and public lifecycle APIs.
7. Partially complete: use workspace Rust/Python roots, flat solutions, shared examples, and workspace-scoped Git. Ensure bootstrap also regenerates the workspace `.gitignore`.
8. Partially complete: generate the owned `.gitignore` with the intended ignored paths. Complete tracked portable-project and dependency-file behavior in Phase 3.
9. Complete: move captured/foreground Git operations into `aocsuite-storage::workspace`; preserve clone behavior rooted in the bootstrapped workspace.
10. Partially complete: generated harnesses and runtime manifests exist and migrate atomically. Stop overwriting or deleting user Cargo/dependency files during migration or cleanup.
11. Make Rust package operations edit tracked `Cargo.toml` semantically and track `Cargo.lock`. Add tracked Python `requirements.txt`; atomically persist `pip freeze` after successful package mutations.
12. Complete: rename `aocsuite-editor` to `aocsuite-launcher`, keep browser launching out of the HTTP client, and route editor/browser processes through the shared executor without config discovery or printing. Editor launches receive explicit project or workspace roots, and launcher resolves selected executables exactly without alias translation.
13. Add SQLite bootstrap, integrity checks, typed corrupt-database errors, transactional schema upgrades, and newer-schema rejection.
14. Replace `.aoccache.json` with SQLite cache metadata. Rebuild recognized entries as stale and reparse cached calendar HTML to restore stars.
15. Add submission counts for correct/incorrect outcomes only and retain the latest configurable per-part runtimes, defaulting to 10.
16. Expose a high-level structured language run API that owns activation through result consumption, records timings, and serializes all active-link-changing jobs.
17. Add typed idempotent cleanup and uninstall plans/reports. Normal cache clean preserves examples; explicit example or comprehensive clean may remove them.
18. Defer fixture-driven storage coverage until explicitly requested.

## Phase 2: HTTP, Cache, And Parser Boundaries (In Progress)

Explicit sessions, status/auth errors, request timeouts, cache metadata, and semantic calendar/submission models are in place. Retry/validator policy, cache recovery, and invalid-body preservation remain.

1. Partially complete: use an explicit optional session, finite timeout, AoC user agent, and local configurable-base tests. Add bounded retry/backoff for transient GET failures; never retry submissions.
2. Complete: require sessions for inputs, submissions, and private leaderboards while permitting public requests without one.
3. Partially complete: return typed HTTP/status/auth errors before caching failed responses. Validate successful puzzle bodies before replacing a valid cached body.
4. Partially complete: cache invalidation and idempotent cleaning exist. Expose pure cache-path/status queries and separate explicit fetch/refresh actions.
5. Complete: parser APIs are separate, fallible, and semantic; CLI owns calendar ANSI rendering.
6. Partially complete: recognize rate-limit variants and preserve sanitized unknown submission text.
7. Partially complete: client has local HTTP tests. Defer storage and parser fixtures until explicitly requested.

## Phase 3: Language Execution And Workspaces (In Progress)

Flat workspace paths, runtime manifests, active-link safety, and unique result files are complete foundations. The end-to-end run and project portability milestones remain open.

1. Partially complete: selected day/year activation and template materialization occur before run for Rust and Python. Also migrate and activate before compile; the current `compile` path can run before setup.
2. Continue the split between pure path/list operations and explicit workspace, environment, compile, and run setup. Query and clean operations do not create a virtual environment or compile projects.
3. Complete: generate Python `main.py` in fresh workspaces and correct generated Python placeholder behavior.
4. Complete: use a unique per-run temporary JSON path that is atomically written and cleared after validated consumption. Add serialized TUI job scheduling with the TUI.
5. Partially complete: command diagnostics and typed part selection exist. Expose public structured run requests/results and remove remaining library-owned result presentation.
6. Partially complete: generated harnesses and runtime manifests are versioned. Preserve tracked Rust Cargo files and add tracked Python `requirements.txt` with dependency persistence.
7. Partially complete: active-link replacement, library validation, and process failure propagation are covered. Preserve project files during cleanup and serialize all active-link-changing jobs.
8. Partially complete: existing tests cover path selection, active links, result cleanup, migrations, and templates. Defer further fake-executor coverage until explicitly requested.

## Phase 4: Remaining CLI, Editor, And Git Behavior (In Progress)

Default-date/command parsing, destructive confirmation behavior, storage-owned Git execution, and launcher migration are complete. CLI service wiring remains.

1. Refactor `run_aocsuite` into thin command handlers over shared typed domain services; keep Clap, prompts, confirmation, and rendering in the CLI without introducing a general operations crate.
2. Complete: correct release/default-date handling with deterministic Eastern-time coverage.
3. Complete: make submit/run parsing match the documented command shapes.
4. Complete: move Git operations into storage workspace services rooted in `workspace/`; do not claim pass-through Git arguments are sandboxed.
5. Complete: rename editor support to `aocsuite-launcher`, combine editor/browser launching, and preserve explicit foreground terminal handoff in frontends.
6. Complete: retain empty-line destructive confirmation behavior while treating EOF as cancellation, with regression coverage.
7. Defer CLI and launcher test additions until explicitly requested.

## Pre-TUI Milestone (Not Met)

Before adding `aocsuite-tui`:

- Every documented pre-TUI blocker is resolved or has an explicit accepted product decision recorded in this plan.
- Complete refactor of other workspace crates including HTTP, storage and parser changes.
- `cargo check --workspace`, `cargo test --workspace`, and `cargo run -p aocsuite-cli -- --help` pass.
- The CLI operates through the same typed storage, language, parser, client, config, and launcher services intended for the TUI; frontends duplicate only presentation, confirmation, and job scheduling.
- Required GitHub CI checks pass on supported platforms without real AoC, subprocess, editor/browser, or developer-runtime dependencies.

## Phase 5: CI And Release Automation (In Progress)

1. Complete: baseline GitHub Actions runs locked workspace check/test and the CLI help smoke test on pull requests and default-branch pushes.
2. After explicitly requested deterministic test work lands, expand required tests to Ubuntu, Windows, and macOS. Add formatting, strict Clippy, and rustdoc gates only after their existing baselines are fixed.
3. Add weekly and dependency-PR `cargo-deny` checks plus Dependabot updates for Cargo and GitHub Actions.
4. Add a tag-driven release workflow that validates synchronized versions, builds native CLI artifacts, smoke-tests them, publishes checksums, and creates a GitHub Release.
5. Add `aocsuite-tui` to the same release version and artifacts after it reaches parity. Keep crates.io publication, code signing/notarization, coverage gates, fuzzing, and MSRV checks optional until their policies are defined.

See `docs/CI.md` for workflow structure, targets, permissions, and staged rollout.

## Phase 6: TUI (Not Started)

1. Add `aocsuite-tui` to the workspace with Ratatui and terminal lifecycle handling.
2. Implement a pure state reducer plus serialized background-effect runner; rendering and event handling perform no blocking I/O.
3. Build full command parity against the shared domain services, not through `run_aocsuite` or parsed CLI output.
4. Defer TUI reducer, render, event, terminal-restoration, and CLI/TUI parity test additions until explicitly requested.
