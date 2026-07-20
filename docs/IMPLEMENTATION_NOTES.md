# Implementation Audit

Review snapshot: 2026-07-18. This is an issue and test inventory; resolved entries record completed work.

## Agreed Product Decisions

- TUI feature parity covers every current CLI command leaf, not only the narrower TODO scope.
- Preserve the current destructive-confirmation default: empty input and EOF proceed. Treat this as intentional behavior in future tests.
- Redact session values by default; no normal configuration read should print the raw token.
- Migrate existing runtime data as part of the planned storage redesign, using a recoverable backup-and-migration process.
- Serialize TUI solver runs. The stale-result defect still requires clearing and validating the run result, but concurrent TUI runs are out of scope.
- Remove the special rule that blocks 2025 days 13-25; use the normal Advent of Code release rules.

## Confirmed Issues

### P0: Safety, Security, and Wrong Results

- **I-01 Destructive prompts approve empty input and EOF.** `user_confirm` treats an empty string as yes, including `read_line` returning zero bytes on closed stdin. This can approve `uninstall`, which removes the entire runtime root, as well as template reset and clean/remove operations (`aocsuite-cli/src/app.rs:240-272`). This behavior is intentionally retained. Test EOF, Enter, yes, and no with injected input; destructive integration tests must use an isolated runtime with inside/outside sentinel files.
- **I-02 Resolved: a requested day/year selects its solution.** `Language::compile` and `Language::run` share `setup_solution`, which initializes the solver, materializes the requested Rust/Python source and active link, and then initializes the environment before execution (`aocsuite-lang/src/lib.rs:32-51`, `122-127`). Rust and Python regression tests cover switching the active source between distinct day solutions. Fresh-run and template-edit coverage remains part of Phase 3.8.
- **I-03 Resolved: solver results use unique transient files.** Each run allocates a unique path under `<runtime-root>/runs/`; result files are removed after successful, failed, or malformed consumption. Generated Rust and Python harnesses publish JSON through a same-directory rename/replace, so readers do not observe partial results. Regression tests cover stale legacy output, malformed output, failure cleanup, and generated atomic publication (`aocsuite-lang/src/lib.rs`, `aocsuite-lang/src/utils.rs`).
- **I-04 Partially resolved: Python setup creates its fresh-runtime harness.** `PythonRunner::setup_solver` now creates `main.py` without overwriting an existing harness, allowing normal open/compile/run setup to materialize the required entrypoint (`aocsuite-lang/src/python/solver.rs`). Regression tests cover creation and preservation; an end-to-end fresh Python execution test remains.
- **I-05 Inputs are downloaded on every access.** Input is fetchable but not updateable, so a successful fetch never marks `input.txt` cache-valid (`aocsuite-fs/src/file.rs:61-75`, `98-130`). Test two loads against a fake downloader and assert one request, one write, and offline reuse of the cached input.
- **I-06 HTTP failures are accepted as content.** GET and POST paths do not check HTTP status; login/error bodies can overwrite puzzle/input/calendar caches or become an `Unknown` successful submission result (`aocsuite-client/src/lib.rs:52-88`, `aocsuite-fs/src/file.rs:121-130`). Puzzle parsing also returns an empty string when no article exists, which is then cacheable (`aocsuite-parser/src/http_markdown.rs:8-23`). Test local-server 200/redirect/400/401/429/500 responses and 200 login pages; failed fetches must preserve an existing good cache.
- **I-07 Session tokens and private inputs are insufficiently protected.** `config get session` prints the token; setting a session prints the previous value and reads the new value with terminal echo. `config.json` and downloaded input can be created as world-readable under a typical Unix umask (`aocsuite-cli/src/app.rs:20-25`, `aocsuite-config/src/lib.rs:22-31`, `63-89`, `aocsuite-fs/src/file.rs:104-111`). Test output redaction, secret-input behavior, and owner-only Unix permissions without ever logging a fixture token.
- **I-08 Malformed config can be silently destroyed.** Invalid JSON becomes an empty map via `unwrap_or_default`; a later set overwrites the original file (`aocsuite-config/src/lib.rs:34-44`, `86-89`). Test malformed, truncated, wrong-shape, and unreadable files; reads must return an error and failed writes must preserve original bytes.

### P1: Correctness and Recoverability

- **I-09 Year-level release validation can panic or reject a valid calendar.** `valid_year_release` uses the CLI puzzle day without range validation and unwraps date construction. Day 0/32 can panic for the current year, while day 25 can delay calendar/leaderboard access until December 25 (`aocsuite-cli/src/app.rs:28-30`, `190-192`, `aocsuite-utils/src/lib.rs:56-69`). The special 2025 days 13-25 restriction must be removed (`aocsuite-utils/src/lib.rs:41-43`). Test with an injected clock at Eastern-time boundaries and invalid/extreme inputs.
- **I-10 Runtime-root resolution accepts unsafe values and can panic.** Empty or relative `XDG_DATA_HOME` produces a relative `aocsuite` directory; missing both `XDG_DATA_HOME` and `HOME` panics (`aocsuite-utils/src/lib.rs:86-94`). Cleanup can consequently target an unexpected working-directory tree. Test empty, relative, absolute, non-Unicode, and missing environment values in isolated child processes.
- **I-11 Git no-argument and clone flows are broken.** Empty args reach `args[0]` and panic. Clone first creates `<runtime-root>/.gitignore`: it fails if the root is absent, and makes the destination non-empty if present, so simple clone cannot succeed (`aocsuite-cli/src/git.rs:22-33`, `53-72`, `147-154`). Test empty args and clone using local temporary repositories and a fake process runner. Git pass-through options such as `-C` are not sandboxed and must not be represented as confined operations in the TUI.
- **I-12 Resolved: custom input paths resolve from the invocation directory.** `run --test FILE` canonicalizes custom input paths before the Rust/Python runners change their working directory. Regression tests cover an input relative to a separate invocation directory and missing-file errors (`aocsuite-cli/src/app.rs`).
- **I-13 Config and session errors can panic instead of returning typed failures.** Config create/open/read/write and prompt I/O use `expect`; invalid header characters in a configured session also use `expect` (`aocsuite-config/src/lib.rs:18-39`, `63-89`, `aocsuite-client/src/lib.rs:39-48`). Test read-only paths, malformed path types, I/O failures, missing environment, and control characters without unwinding.
- **I-14 Cache metadata failures are ignored and writes are not atomic.** Metadata writes discard errors; config, content, and metadata use direct read-modify-write operations (`aocsuite-fs/src/file.rs:98-102`, `168-181`, `201-217`, `aocsuite-config/src/lib.rs:34-44`, `86-89`). Test unwritable metadata and interrupted writes; successful submission invalidation must be observable and reliable.
- **I-15 Partially resolved: query and cleanup APIs no longer perform language setup.** `Language::resolve` only selects a runner; only execution and package mutation set up an environment, while `prepare_solver_file` explicitly sets up solver files (`aocsuite-lang/src/lib.rs:23-75`, `114-127`). Package-list failures still become empty lists, and Rust cache clean, browser launch, and interactive Git ignore nonzero status (`aocsuite-lang/src/rust/dependencies.rs:47-76`, `aocsuite-lang/src/python/dependencies.rs:52-82`, `aocsuite-lang/src/rust/solver.rs:52-58`, `aocsuite-client/src/lib.rs:62-75`, `aocsuite-cli/src/git.rs:103-112`). Test command failures with fake executors and assert read-only/cleanup calls do not create environments.
- **I-16 Common AoC rate-limit messages are not parsed.** The regex only accepts lowercase integer seconds and misses singular seconds and minute/second forms (`aocsuite-parser/src/http_submission.rs:47-78`). Test a fixture table covering seconds, singular units, minutes, mixed units, punctuation, capitalization, and unknown responses.
- **I-17 CLI defaults and documented command shapes disagree with behavior.** `AOC_YEAR`/stored year does not affect Clap's current-year default; current calendar day is used even outside December. Submit help says an omitted answer will prompt, but the argument is required, and README examples use `--part` while `part` is positional (`aocsuite-cli/src/main.rs:13-19`, `aocsuite-cli/src/commands.rs:44-65`, `README.md:79-83`). Test `try_parse_from` for every command leaf and fixed-date/config default resolution.
- **I-18 Editor resolution rejects valid configurations and some paths panic or misparse.** Alias lookup calls `which` before translating `neovim`/`helix`/`sublime`; `$EDITOR` values containing a path or arguments are not supported. Paths are converted with UTF-8 `unwrap`, and Vim split commands do not escape spaces or Ex metacharacters (`aocsuite-editor/src/editor_types.rs:49-66`, `aocsuite-editor/src/lib.rs:42-64`, `aocsuite-editor/src/arg_builder.rs:14-27`). Test fake-PATH aliases, executable paths/arguments, non-UTF-8 paths, and Vim-special characters.
- **I-19 `run --test` can pass a nonexistent example file.** Example paths are neither downloaded nor created by `to_path`; a fresh test run passes the missing file to the solver (`aocsuite-cli/src/app.rs:55-64`, `aocsuite-fs/src/file.rs:53-77`, `184-198`). Test fresh and existing example-file flows with no editor involved.
- **I-20 Partially resolved: active solutions have typed and safe link targets.** `SolverFile::ActiveSolution` stores a day/year directly, preventing nested and nonsolution active-link targets. Active-link replacement now creates the new link before replacing an existing symlink and refuses to delete a regular file; regression tests cover failed creation and regular-file preservation. `AocContentFile` can still panic for missing days (`aocsuite-fs/src/file.rs:21-26`, `184-198`).

### P2: Lower-Severity and Portability

- **I-21 Resolved: Generated Python stubs interpolate input length.** The generated f-strings now use single braces, and a regression test verifies the emitted source (`aocsuite-lang/src/python/solver.rs`).
- **I-22 Library-name validation does not match Rust or Python identifiers.** It accepts hyphens but rejects digits anywhere, creating unusable names and rejecting valid ones (`aocsuite-lang/src/lib.rs:165-196`). Add language-specific table tests for identifiers, reserved names, keywords, Unicode policy, and case-insensitive collisions.
- **I-23 Windows command construction is incorrect.** Rust run omits the executable suffix and Python prepends `PATH` with `:` rather than platform-aware joining (`aocsuite-lang/src/rust/solver.rs:28-34`, `aocsuite-lang/src/python/dependencies.rs:102-110`). Cover with Windows CI and path round-trips via `split_paths`/`join_paths`.
- **I-24 Generated runtime infrastructure has no migration/version boundary.** Existing `main.*` and generated Cargo files are never updated after first creation, so harness fixes do not reach existing users (`aocsuite-lang/src/lib.rs:58-68`, `aocsuite-lang/src/rust/solver.rs:44-49`, `aocsuite-lang/src/rust/dependencies.rs:8-22`). The storage migration must update owned generated harnesses while preserving user solutions, templates, and libraries; add upgrade fixtures for that boundary.

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
