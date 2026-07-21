# Implementation Audit

Review snapshot: 2026-07-18. This is an issue and test inventory; resolved entries record completed work.

## Agreed Product Decisions

- TUI feature parity covers every current CLI command leaf, not only the narrower TODO scope.
- Preserve the current destructive-confirmation default: an empty line proceeds, while EOF cancels. Treat this as intentional behavior in future tests.
- Redact session values by default; no normal configuration read should print the raw token.
- Migrate existing runtime data as part of the planned storage redesign, using a recoverable backup-and-migration process.
- Serialize TUI solver runs. The stale-result defect still requires clearing and validating the run result, but concurrent TUI runs are out of scope.
- Advent of Code 2025 has twelve puzzles; release days 1-12 daily in December and reject days 13-25.

## Confirmed Issues

### P0: Safety, Security, and Wrong Results

- **I-01 Resolved: destructive prompts approve empty input but reject EOF.** `user_confirm` retains the agreed empty-line-as-yes behavior while treating closed stdin as cancellation. Injected-I/O regression tests cover EOF, Enter, yes, and no (`aocsuite-cli/src/app.rs`).
- **I-02 Resolved: a requested day/year selects its solution.** `Language::compile` and `Language::run` share `setup_solution`, which initializes the solver, materializes the requested Rust/Python source and active link, and then initializes the environment before execution (`aocsuite-lang/src/lib.rs:32-51`, `122-127`). Rust and Python regression tests cover switching the active source between distinct day solutions. Fresh-run and template-edit coverage remains part of Phase 3.8.
- **I-03 Resolved: solver results use unique transient files.** Each run allocates a unique path under `<runtime-root>/runs/`; result files are removed after successful, failed, or malformed consumption. Generated Rust and Python harnesses publish JSON through a same-directory rename/replace, so readers do not observe partial results. Regression tests cover stale legacy output, malformed output, failure cleanup, and generated atomic publication (`aocsuite-lang/src/lib.rs`, `aocsuite-lang/src/utils.rs`).
- **I-04 Partially resolved: Python setup creates its fresh-runtime harness.** `PythonRunner::setup_solver` now creates `main.py` without overwriting an existing harness, allowing normal open/compile/run setup to materialize the required entrypoint (`aocsuite-lang/src/python/solver.rs`). Regression tests cover creation and preservation; an end-to-end fresh Python execution test remains.
- **I-05 Resolved: fetched inputs are marked cache-valid.** Fetches write cache metadata for every downloaded content file, including `input.txt`, while submission invalidation remains limited to puzzle and calendar files (`aocsuite-fs/src/file.rs`). A regression test verifies that a saved input with fetch metadata is cache-valid; fake-downloader coverage remains part of the required test seam work.
- **I-06 Resolved: HTTP failures are rejected before parsing or caching.** Client GET and POST paths classify non-success responses as typed status, authentication, or rate-limit errors; successful login pages are also rejected as authentication failures. The configurable-base request seam supports local-server tests for redirect, 400, 401, 429, 500, and 200 login responses. Puzzle responses without articles fail before the cache write, so failed fetches preserve existing cached content (`aocsuite-client/src/lib.rs`, `aocsuite-fs/src/file.rs`).
- **I-07 Resolved: session tokens and private inputs are protected.** `config get session` returns a typed not-allowed error without reading or printing the token, and session updates use a non-echoing terminal password prompt without displaying the current value. Atomic writes create owner-only Unix files, and existing `config.json` plus cached inputs are tightened to mode `0600` when accessed (`aocsuite-cli/src/app.rs`, `aocsuite-config/src/lib.rs`, `aocsuite-fs/src/file.rs`, `aocsuite-utils/src/lib.rs`). Regression tests cover denied session reads and Unix permission creation/tightening without logging a fixture token.
- **I-08 Resolved: malformed config is preserved.** Configuration construction returns typed I/O and JSON errors instead of replacing invalid data with an empty map. Reads and writes fail before prompting or modifying malformed bytes; editor fallback to `$EDITOR` only occurs when no editor setting exists (`aocsuite-config/src/lib.rs`, `aocsuite-editor/src/lib.rs`). Regression coverage verifies malformed configuration remains byte-for-byte unchanged.

### P1: Correctness and Recoverability

- **I-09 Resolved: release validation is day-safe and calendar-aware.** Puzzle validation rejects invalid days before constructing dates, including 2025 days 13-25 because that event had twelve daily releases. Calendar and leaderboard validation ignores the selected puzzle day and becomes available at December 1 midnight Eastern. Injected-clock regression tests cover invalid/extreme days, 2025 releases, Eastern-time boundaries, and invalid/future years (`aocsuite-utils/src/lib.rs`).
- **I-10 Resolved: runtime-root resolution rejects unsafe values without panicking.** `get_aocsuite_dir` returns a typed `RuntimeDirError`, requires non-empty absolute `XDG_DATA_HOME` or `HOME` values, and preserves valid non-Unicode paths. The error propagates through configuration, cache, language, Git, and CLI operations, preventing cleanup from targeting a relative working-directory tree. Deterministic resolver tests cover empty, relative, absolute, non-Unicode, and missing environment values (`aocsuite-utils/src/lib.rs`).
- **I-11 Resolved: Git empty-argument, clone, and interactive failures are handled safely.** Empty arguments pass to Git without indexing a missing element. Simple clone creates only the runtime parent and runs from there without pre-creating the destination or `.gitignore`; normal Git commands initialize the runtime root and ignore file. Interactive command failures now return `CommandFailed`. Unit tests cover empty and malformed clone argument handling (`aocsuite-cli/src/git.rs`). Git pass-through options such as `-C` are not sandboxed and must not be represented as confined operations in the TUI.
- **I-12 Resolved: custom input paths resolve from the invocation directory.** `run --test FILE` canonicalizes custom input paths before the Rust/Python runners change their working directory. Regression tests cover an input relative to a separate invocation directory and missing-file errors (`aocsuite-cli/src/app.rs`).
- **I-13 Resolved: config and session failures return typed errors.** Configuration creation, reads, writes, and prompts propagate `AocConfigError`; invalid HTTP-header characters in sessions return `AocClientError::Session` rather than panicking. Regression coverage verifies malformed configuration preservation and invalid/valid session header construction (`aocsuite-config/src/lib.rs`, `aocsuite-client/src/lib.rs`). Broader read-only and malformed-path coverage remains required by the Phase 1 test map.
- **I-14 Resolved: settings, content, and cache metadata writes are atomic and fallible.** The shared `atomic_write` helper publishes synchronized temporary files through same-directory rename/replace. Configuration initialization/saves, fetched content, and cache metadata use it; metadata read, parse, serialization, and write failures now propagate as `AocFileError` instead of being ignored. Regression tests cover atomic replacement without temporary-file residue and a cache-metadata failure (`aocsuite-utils/src/lib.rs`, `aocsuite-config/src/lib.rs`, `aocsuite-fs/src/file.rs`).
- **I-15 Partially resolved: query and cleanup APIs no longer perform language setup.** `Language::resolve` only selects a runner; only execution and package mutation set up an environment, while `prepare_solver_file` explicitly migrates generated runtime files. Rust `cargo tree`, Python `pip list`, Rust `cargo clean`, and interactive Git now surface nonzero subprocess status. Browser launch still ignores nonzero status (`aocsuite-client/src/lib.rs`). Test command failures with fake executors and assert read-only/cleanup calls do not create environments.
- **I-16 Resolved: AoC rate-limit messages parse seconds and minutes.** Submission parsing accepts case-insensitive singular/plural seconds, minute-only waits, and compact or word-based minute/second combinations. Table-driven tests cover punctuation, capitalization, and unrecognized messages (`aocsuite-parser/src/http_submission.rs`).
- **I-17 Resolved: CLI defaults and documented command shapes agree with behavior.** Day/year flags are optional and resolve to the latest released puzzle, with explicit or configured years taking precedence. `run` and `submit` use the documented `--part` option, while `submit` prompts when its optional answer is omitted. Regression coverage verifies parsing and configured-year/default resolution (`aocsuite-cli/src/main.rs`, `aocsuite-cli/src/commands.rs`).
- **I-18 Resolved: editor resolution accepts aliases, command lines, and safe paths.** Known aliases are normalized before executable lookup, while configured and `$EDITOR` command lines support executable paths plus arguments. Generic editors receive `OsString` paths directly; Vim split commands escape Ex-special paths and return a typed error for non-Unicode paths rather than panicking. Regression tests cover fake-PATH aliases, command arguments, special paths, and non-Unicode Vim paths (`aocsuite-editor/src/editor_types.rs`, `aocsuite-editor/src/arg_builder.rs`).
- **I-19 Resolved: `run --test` rejects missing example files.** The built-in example path is checked as a regular file before language setup, compilation, or solver execution; missing examples return `NotFound`, while `open` retains its ability to create an example through the editor (`aocsuite-cli/src/app.rs`). Regression tests cover missing, directory, and existing example paths.
- **I-20 Resolved: active solutions and fetched content use safe typed targets.** `SolverFile::ActiveSolution` stores a day/year directly, preventing nested and nonsolution active-link targets. Active-link replacement creates the new link before replacing an existing symlink and refuses to delete a regular file. Invalid puzzle/input content descriptors without a day now return `InvalidFile` instead of panicking; regression tests cover link failures, regular-file preservation, and missing-day descriptors (`aocsuite-lang/src/utils.rs`, `aocsuite-fs/src/file.rs`).

### P2: Lower-Severity and Portability

- **I-21 Resolved: Generated Python stubs interpolate input length.** The generated f-strings now use single braces, and a regression test verifies the emitted source (`aocsuite-lang/src/python/solver.rs`).
- **I-22 Resolved: library names follow Rust and Python identifier rules.** Names use an ASCII identifier policy, allowing digits after the first character while rejecting hyphens and Unicode. Rust/Python keywords and runtime-owned files are rejected case-insensitively, and new paths refuse to collide with an existing library that differs only by case. Table-driven tests cover valid identifiers, keywords, runtime names, Unicode, and collisions (`aocsuite-lang/src/lib.rs`).
- **I-23 Resolved: runner command paths are platform-aware.** Rust release binaries use `EXE_SUFFIX`, while Python editor setup builds `PATH` with `split_paths`/`join_paths` and preserves OS-native environment strings. Regression tests cover executable suffix construction, platform-path round-trips, and non-Unicode PATH entries on Unix (`aocsuite-lang/src/rust/solver.rs`, `aocsuite-lang/src/python/dependencies.rs`). Windows execution coverage remains dependent on Windows CI.
- **I-24 Resolved: generated runtime infrastructure has a migration boundary.** Per-language `.aocsuite-runtime.json` manifests track infrastructure versions. Versioned migrations atomically update owned Rust (`Cargo.toml`, `src/main.rs`) and Python (`main.py`) harnesses, while preserving solution, template, library, and day files. Legacy-runtime fixtures verify these upgrade boundaries (`aocsuite-lang/src/runtime.rs`, `aocsuite-lang/src/rust/mod.rs`, `aocsuite-lang/src/python/mod.rs`).

## TUI Boundary Work

These are prerequisites or design risks, not separate confirmed defects:

- Extract typed, non-interactive operations from the CLI dispatcher. The TUI must not call `run_aocsuite` or parse its output (`aocsuite-cli/src/app.rs`).
- Separate pure path queries from fetch/setup/mutation. `AocContentFile::to_path` can download, and language lookup/list/clean can create environments.
- Return semantic calendar data and public structured solver results. Current calendar output embeds ANSI and `ExerciseOutput` fields are inaccessible outside `aocsuite-lang` (`aocsuite-parser/src/http_ansicalendar.rs`, `aocsuite-lang/src/utils.rs:18-58`).
- Represent browser/editor/Git/Cargo/Python/pip/solver launches as structured command requests behind an injectable executor. Current calls block and some print directly to the host terminal.
- Move Git operations out of the private CLI module or introduce a shared owning crate. Confirmation remains frontend state; shared destructive APIs should receive already-confirmed typed requests.
- Keep network, filesystem mutation, environment setup, subprocess waits, and editor terminal handoff outside Ratatui update/render. Serialize solver jobs and use job IDs so stale asynchronous completions cannot update the wrong selection.

## Required Test Coverage

### Test Seams and Isolation

- Use an explicit temporary runtime root per test. Never fall back to the developer's home; destructive tests need sentinel files immediately inside and outside the target.
- Inject a fixed clock for release/default tests; cover UTC/US-Eastern date disagreement and midnight release boundaries without relying on the execution date.
- Inject environment lookup where possible. Transitional tests that mutate process environment must run in isolated child processes because parallel tests otherwise race.
- Inject an HTTP base URL/client and use a local mock server. Never contact Advent of Code or use a real session token.
- Introduce a process-executor seam returning status/stdout/stderr. Deterministic tests must not launch real Git, Cargo, Python, pip, editors, or browsers.
- Do not add cross-process concurrency coverage: the project assumes a single AoCSuite process at a time. Serialize solver runs in the TUI and give each queued job an ID for stale-completion handling.
- TUI tests must use `ratatui::TestBackend`, synthetic events, and fake terminal operations; they must not alter the test runner's raw mode, alternate screen, cursor, or stdin.

### Crate Test Map

- **`aocsuite-utils`:** release boundaries, invalid inputs without panic, Eastern defaults, 2025 twelve-day policy, and runtime-root precedence/validation.
- **`aocsuite-config`:** source precedence, typed parsing, non-interactive set/remove, secret handling, Unix permissions, malformed JSON, and failed writes.
- **`aocsuite-client`:** every URL shape, cookie/form construction, status handling, redirects, timeout behavior, invalid sessions, and browser-launch status through fakes.
- **`aocsuite-fs`:** pure paths; cache miss/hit/offline/refresh; input persistence; parser/fetch failure preserving good data; metadata corruption; submission invalidation; clean day/year/all scope and idempotence.
- **`aocsuite-parser`:** fixture-driven markdown with zero/one/two articles, structured calendar cells/stars/styles, and the complete submission/rate-limit response table.
- **`aocsuite-lang`:** shared Rust/Python contract for setup, selected solution, compile/run, custom/example input, part selection, public result fields, unique result files, command failures, package list/add/remove/clean, library names, symlink safety, generated templates, and migrations.
- **`aocsuite-editor`:** editor resolution, argument ordering/escaping, environment forwarding, non-UTF-8 paths, and child exit status using command specifications.
- **`aocsuite-cli`:** expose parser construction for table-driven coverage of all command leaves; test formatting/error mapping, Git classification, confirmations, and every destructive scope against fake services.
- **`aocsuite-tui`:** pure state/reducer tests; loading/success/error/confirmation transitions; selection-to-request correctness; `TestBackend` rendering at normal/narrow sizes; no ANSI or session leakage; synthetic key/resize/tick handling; terminal restoration; responsive background jobs; stale-result rejection.

### Cross-Frontend and Workflow Tests

- Maintain one parity case table for all CLI command leaves. CLI parsing and TUI actions must produce the same typed operation request, including selected day/year/language/part/input and clean target.
- Add three deterministic workflow integrations after the seams exist: calendar cache miss then hit; select/open/run an example using the exact selected solution; submit a correct answer and verify cache invalidation.
- Keep live AoC, real editor/browser, and installed Cargo/Python environment smoke tests optional and outside the deterministic suite.

## Current Verification Baseline

- `cargo check --workspace` passes.
- `cargo test --workspace` passes; `aocsuite-lang` currently covers Rust/Python active-source selection.
- `cargo run -p aocsuite-cli -- --help` passes.
- `cargo fmt --all -- --check` currently fails on existing formatting in `aocsuite-cli/src/app.rs`.
- `cargo clippy --workspace --all-targets --all-features` currently fails on `clippy::never_loop` in `aocsuite-parser/src/http_ansicalendar.rs:17-24` and also reports existing warnings in `aocsuite-utils` and the calendar parser.
