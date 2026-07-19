# AoC Suite Agent Notes

## Product Direction

- The shipped application is CLI-only. Current development targets a new Ratatui binary crate, `aocsuite-tui`, with feature parity; keep `aocsuite-cli` working while adding it to the root workspace.
- Treat `aocsuite-cli/src/commands.rs` as the parity inventory and `aocsuite-cli/src/app.rs` as the current behavior reference.
- Do not drive the TUI through `aocsuite_cli::run_aocsuite`: it mixes orchestration with stdout/stderr, blocking stdin prompts, network calls, filesystem mutation, and subprocess waits. Move reusable operations into the owning library crates with structured, non-interactive inputs/results.
- Existing APIs are synchronous. Keep HTTP, cache loads, language setup/runs, editors, and Git subprocesses outside the Ratatui event/render path.
- Several parity APIs need extraction rather than output parsing: Git and confirmation helpers are private to the CLI; config writes prompt on stdin; calendar parsing returns ANSI text; language run results are not publicly inspectable. Fix the shared boundary instead of scraping CLI output.

## Workspace Boundaries

- The root is a virtual Cargo workspace. `aocsuite-cli/src/main.rs` only parses Clap arguments; `run_aocsuite` in `app.rs` is the real synchronous dispatcher.
- `aocsuite-utils` owns puzzle dates/release checks and the runtime root; `aocsuite-config` owns persisted/env config; `aocsuite-client` is blocking AoC HTTP/browser I/O; `aocsuite-fs` owns cached AoC files; `aocsuite-parser` transforms AoC HTML; `aocsuite-lang` owns Rust/Python environments and runners; `aocsuite-editor` launches editors.
- `AocContentFile::to_path()` is not a pure path getter: for puzzle, calendar, and input files it can download and write the cache.
- Resolving an `aocsuite-lang` runner calls both `setup_solver` and `setup_env`, even for list, clean, and path-oriented operations; these calls can create files or invoke Cargo/Python.

## Runtime State

- State lives at `$XDG_DATA_HOME/aocsuite`, falling back to `$HOME/.local/share/aocsuite` (the README's `.local/data` path is stale). Set `XDG_DATA_HOME` to a temporary directory for isolated manual tests.
- Config precedence is explicit function override, then `<runtime-root>/config.json`, then `AOC_*` environment variable, then caller default. Rust does not load `.envrc` or dotenv files.
- Never run or log `config get session`: it prints the raw AoC session token. Avoid live submission/download commands as verification.
- CLI day/year defaults use the current US/Eastern calendar date, not the latest released puzzle; pass explicit values in behavior checks outside December.
- `compile` and `run` activate the requested day/year solution before execution. Keep this selection behavior covered for both Rust and Python runners.
- All language runs share `<runtime-root>/result.json`; concurrent runs can race and failed runs can leave stale results.
- Destructive CLI prompts accept empty input as yes, and `uninstall` recursively removes the entire runtime root.

## Verification

- Workspace build: `cargo check --workspace`
- Workspace tests: `cargo test --workspace` (the current workspace has no tests, so a green run provides compilation coverage only).
- Focused crate/test: `cargo test -p <crate> [test-filter]`; focused compile: `cargo check -p <crate>`.
- CLI smoke test: `cargo run -p aocsuite-cli -- --help`.
- There is no CI or repository lint/format configuration. The current baseline already fails `cargo fmt --all -- --check` in `aocsuite-cli/src/app.rs` and strict Clippy in `aocsuite-utils`; do not fold unrelated cleanup into TUI work.

## Commits

- Do not commit unless explicitly requested.
- Use Conventional Commits.
- Keep commits focused on one logical change.
- Never amend or force-push without explicit permission.
