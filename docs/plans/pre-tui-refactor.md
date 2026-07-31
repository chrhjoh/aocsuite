# Pre-TUI Refactor Plan

## Purpose

This file tracks current implementation status and sequencing before
`aocsuite-tui` begins.

Stable decisions belong in:

- `../ARCHITECTURE.md`;
- `../STORAGE.md`;
- `../CI.md`.

This plan does not restate every design detail and does not authorize unrelated
work.

## Goal

Refactor the existing CLI and shared crates so CLI and future TUI frontends can
use the same typed domain services.

Preserve ordinary successful CLI workflows. Begin TUI implementation only after
the pre-TUI milestone is met.

The application assumes one AoC Suite process per runtime root. Cross-process
coordination is outside this plan.

## Status

Last reviewed: 2026-07-29.

Overall status: **in progress**.

A crate, type, or initial schema existing does not make its phase complete.

### Completed foundations

- `aocsuite-utils` owns UI-neutral puzzle and language values plus the
  synchronous `CommandExecutor`.
- Client construction accepts explicit optional sessions and request settings.
- Config uses explicit paths and non-mutating reads.
- `aocsuite-storage` has replaced `aocsuite-fs` and contains initial layout,
  SQLite, content, and workspace services.
- Parser calendar and submission APIs return semantic data; CLI renders them.
- Workspace paths, examples, Git execution, generated `.gitignore`, and
  transient run allocation are storage-owned.
- Language projects use flat solution paths, generated runtime manifests, safe
  active-link replacement, and unique atomic result files.
- `aocsuite-launcher` owns browser and editor process execution, explicit
  working directories, typed puzzle-open requests, and generic fallback for
  unrecognized exact executables.
- Baseline GitHub Actions runs locked workspace check, tests, and CLI help on
  Ubuntu.

### Current verification baseline

Passing:
On next pass add cargo clippy to this baseline.

```text
cargo check --workspace --locked
cargo test --workspace --locked
cargo run -p aocsuite-cli --locked -- --help
cargo fmt --all -- --check
```

## Remaining pre-TUI blockers

1. Complete storage lifecycle policy:
   - strict unversioned-root handling;
   - database and cache validation;
   - typed cleanup;
   - uninstall safety.
2. Complete portable language execution:
   - activate and migrate before compilation;
   - expose one serialized typed run operation;
   - return public structured results.
3. Reduce CLI orchestration to frontend mapping over injectable domain services.
4. Reach the verification and required-CI milestone described below.

Typed persisted calendar state remains deferred until TUI calendar behavior is
defined.

## Delivery rules

- Work on one issue or tightly coupled boundary at a time.
- Keep `aocsuite-cli` functional.
- Preserve command names and normal successful workflows unless an approved
  migration requires a documented change.
- Do not begin TUI implementation before the milestone is met.
- tests may be added or updated when behavior changes, following
  `../../AGENTS.md`.
- Do not opportunistically complete later items while working on an earlier task.
- Update this plan to reflect current progress.

## Workstreams

Statuses use:

- **done**: implemented and verified;
- **partial**: foundations exist, but acceptance criteria remain;
- **open**: not implemented;
- **deferred**: intentionally postponed pending another design decision.

### 1. Runtime layout and configuration — done

Done:

- Shared validated types and process executor exist.
- Client, launcher, and most language APIs receive explicit inputs.
- Config uses explicit paths and non-mutating reads.
- Configuration values are parsed into typed values during load and provide
  effective defaults.
- Configurable run-history retention is passed to storage when recording run
  timings.
- `RuntimeLayout::new(root)` exists.
- Layout version 1 and initial bootstrap exist.
- CLI bootstraps before config and service construction.
- Every existing unversioned root, including an empty root, is rejected.
- Newer layout versions are rejected without mutation.
- Session storage is separate from non-secret config.
- Session creation and rewrites use owner-only permissions on Unix.
- Path and status getters are pure.
- Typed persisted-cache cleanup and confirmed `RuntimeLayout` root uninstall
  are storage-owned.
- Workspace initialization creates a missing `.gitignore` without overwriting
  an existing user-managed file.

Acceptance:

- Runtime-root and config behavior match `../STORAGE.md`.
- Reads are non-mutating.
- No domain input discovers config or the runtime root globally.
- Focused resolver and layout tests use explicit roots and deterministic seams.

### 2. Storage lifecycle and content — done

Done:

- `aocsuite-storage` replaced `aocsuite-fs`.
- Initial layout, SQLite, content, workspace Git, and run allocation exist.
- Layout and database modules do not call HTTP or parser code.
- Content is the only storage module coordinating client and parser behavior.
- Flat cache paths, raw HTML, derived Markdown, and semantic parser output exist.
- `.aoccache.json` has been replaced by SQLite metadata.
- Correct and incorrect submission counts and bounded recent timings are stored.
- Cache invalidation and basic idempotent cleaning exist.
- Indexed persisted-cache cleanup preserves unindexed files and `state.sqlite`.
- ContentStore uses its injected client for cache misses and records submission
  invalidation.
- SQLite integrity checks, typed corrupt-database errors, transactional schema
  upgrades, and newer-schema rejection exist.
- Focused tests cover corrupt database files, v1 bootstrap, newer-schema
  rejection, and public error mapping.

Deferred:

- Persisted typed calendar state and derived stars.

Acceptance:

- Behavior matches `../STORAGE.md`.
- Unindexed cache files remain unmanaged; full uninstall removes its confirmed
  runtime root.
- Client-rejected remote bodies cannot replace valid cached data.
- Cleanup scopes are explicit, typed, idempotent, and frontend-confirmed.

### 3. HTTP and parser boundaries — done

Done:

- Client receives an explicit optional session.
- Inputs, submissions, and private leaderboard requests require a session.
- Public requests may omit a session.
- Typed status and authorization errors exist.
- Client rejects bodies matching currently known invalid AoC response markers.
- Parser APIs are separate, fallible, and semantic.
- CLI owns calendar presentation.
- GET requests use bounded retries and backoff for transient HTTP and transport
  failures; submissions are never retried.
- Submission parsing recognizes explicit numeric cooldown phrases before generic
  incorrect-answer text.
- Unknown submission responses preserve their extracted article Markdown.

Acceptance:

- Client owns transport only.
- Parser owns transformations only.
- Storage owns cache lifecycle.
- Client-rejected responses do not corrupt valid cache state.

### 4. Language execution and portable projects — done

Done:

- Rust and Python use flat workspace solution paths.
- Generated harnesses and runtime manifests are versioned.
- Activation and template materialization occur before compilation.
- Python `main.py` is generated for fresh workspaces.
- Per-run result files are unique, atomic, validated, and cleared.
- Structured run results and typed part selection exist.
- Rust package operations update `Cargo.toml` and `Cargo.lock`.
- Python package operations retain `requirements.txt` and persist successful
  package mutations.
- Active-link replacement and library validation exist.
- Runtime cleanup removes only generated runtime files.
- Migration and activation occur before public compile-and-run execution.
- One public typed operation covers migration, activation, environment setup,
  compilation, execution, and result consumption.
- Language jobs are frontend-scheduled and serialized per runtime-root workspace
  through timing persistence.
- Query and cleanup operations do not initialize language projects, environments,
  or compilation.

Acceptance:

- A frontend can invoke one typed language job and receive a structured result.
- The full active-link-changing operation is serialized.
- Query and cleanup operations do not create environments or compile projects.
- Projects remain usable outside AoC Suite.

### 5. CLI, launcher, and Git — partial

Done:

- Release and default-date handling is deterministic.
- Submit and run command parsing matches documented shapes.
- Git operations are storage-owned and workspace-scoped.
- Git pass-through is not described as sandboxed.
- Editor and browser launching are combined in `aocsuite-launcher`.
- Foreground terminal handoff remains frontend-owned.
- Destructive CLI confirmation treats an empty line as confirmation and EOF as
  cancellation.
- CLI command execution receives an injected executor from the composition root.

Remaining:

- Remove remaining domain policy from CLI orchestration.
- Preserve exact executable resolution without adding alias translation.

Acceptance:

- CLI behavior is frontend mapping over the same services intended for TUI.
- CLI contains interaction and rendering, not domain policy.
- `aocsuite-cli/src/app.rs` is not used as a service by another frontend.

### 6. CI and releases — partial

Done:

- Baseline Ubuntu CI runs locked workspace check, tests, and CLI help.

Remaining:

- Fix the strict Clippy baseline.
- Add deterministic cross-platform required jobs when supporting tests are
  reliable.
- Add formatting, Clippy, and rustdoc gates after their baselines pass.
- Add `cargo-deny` and Dependabot.
- Add the tag-driven CLI release workflow.
- Add TUI artifacts only after TUI parity.

Acceptance:

- Required checks pass without real AoC, external user applications, terminals,
  or developer-local runtime state.
- Release jobs follow `../CI.md`.
- Build and test jobs remain read-only.

## Pre-TUI milestone

The milestone is met only when:

- every blocker above is done or has an explicit accepted deferral recorded here;
- CLI behavior is implemented through the typed services intended for TUI;
- storage lifecycle and language execution acceptance criteria are met;
- the applicable workspace checks pass;
- required CI passes on the supported baseline without live services or real
  external applications.

## TUI phase — not started

After the milestone:

1. Add `aocsuite-tui` with Ratatui and explicit terminal lifecycle handling.
2. Implement a pure state reducer and serialized background-effect runner.
3. Keep rendering and event handling free of blocking I/O.
4. Build workflow parity through shared services, not `run_aocsuite` or parsed
   CLI output.
5. Use `ratatui::TestBackend` and fake terminal operations for deterministic
   tests.

The precise calendar persistence model should be decided while defining TUI
calendar behavior.
