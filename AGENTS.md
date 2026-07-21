# AoC Suite Agent Notes

## Product Direction

- The shipped application is CLI-only. Current development refactors shared crates before adding a Ratatui binary crate, `aocsuite-tui`, with full CLI command-leaf parity.
- Keep `aocsuite-cli` working throughout. Treat `aocsuite-cli/src/commands.rs` as the parity inventory and `aocsuite-cli/src/app.rs` only as the current behavior reference.
- Do not drive the TUI through `aocsuite_cli::run_aocsuite` or parse CLI output. CLI and TUI use the same typed domain services and duplicate only presentation, confirmation, terminal handoff, and job scheduling.
- Existing APIs are synchronous. Keep HTTP, storage mutation, language setup/runs, editors/browsers, Git, and other subprocess waits outside Ratatui update/render. The initial TUI uses serialized background jobs with job IDs.
- Do not add a general operations crate. Put policy in the owning service: content/submission/cache/Git/cleanup in storage, execution/package policy in language, configuration precedence in config, and launching in launcher.

## Target Crate Boundaries

- `aocsuite-utils` owns validated UI-neutral puzzle/language values, release calculations, atomic filesystem primitives, environment/clock seams where needed, and the shared synchronous process executor. Shared values must not derive Clap traits.
- Add broad `aocsuite-storage`, replacing `aocsuite-fs`. It owns `RuntimeLayout`, bootstrap/versioning, SQLite, AoC fetch/parse/cache lifecycle, shared examples, workspace Git, run allocation, submission counts, timing retention, cleanup, and uninstall safety.
- Keep storage internally layered. Layout/database modules do not call HTTP or parser code; only the content module depends on the configuration-independent client and semantic parser. Storage never depends on config, language, launcher, CLI, or TUI.
- `aocsuite-config` owns typed non-secret preferences, environment precedence, and session persistence. It receives explicit paths, performs no prompting, and must not create files during reads. Remove the unused `template_dir`/`AOC_TEMPLATE_DIR` setting.
- `aocsuite-client` owns blocking AoC HTTP transport, URL/auth/status behavior, timeout/retry policy, and HTTP validators. It receives an optional session explicitly and must not read config, storage, or environment state.
- `aocsuite-parser` owns pure fallible puzzle/calendar/submission transformations. Return semantic calendar cells/stars and submission outcomes; ANSI, emoji, and frontend prose do not belong here.
- `aocsuite-lang` owns complete tracked Rust/Python projects, versioned generated harnesses, solutions/templates/libraries, active links, Cargo/pip dependencies, compile/run, structured reports, and language cleanup. It receives explicit paths/settings/executor and must not read config or discover the runtime root.
- Rename `aocsuite-editor` to `aocsuite-launcher`. It owns editor/browser resolution and process requests, but not config lookup, terminal suspend/restore, storage, or rendering.
- Clap, `rpassword`, prompts, empty-line/EOF confirmation behavior, output formatting, and terminal lifecycle remain frontend concerns.

## Storage And Workspace Decisions

- The target root contains `.aocsuite-layout.json`, `config.json`, `secrets/session`, `state.sqlite`, `cache/aoc`, `runs`, and a lazily created `workspace` Git root. See `docs/STORAGE.md` for the authoritative layout.
- Bootstrap storage on every application invocation before reading config or constructing services. Root bootstrap must not create `workspace/`; clone targets an absent workspace and other workspace operations initialize it lazily.
- There are no active users requiring import of the current unversioned layout. Reject nonempty unversioned roots without mutation and provide manual-removal guidance. Future versioned migrations use retained backups and resumable phases.
- `workspace/rust` and `workspace/python` are complete portable projects. Track harnesses, `.aocsuite-runtime.json`, `Cargo.toml`, `Cargo.lock`, `requirements.txt`, solutions, templates, libraries, and shared `workspace/examples`.
- Generated harnesses are strictly AoC Suite-owned. Version-only migrations atomically overwrite them and then update the tracked manifest; no hashes or manual-edit detection are required.
- Ignore only disposable state such as Rust `target/`, Python `venv`/bytecode caches, and active solution links. The workspace `.gitignore` is AoC Suite-owned and regenerated completely.
- Persist Python package changes by atomically replacing tracked `requirements.txt` with `pip freeze` after successful pip mutation. Python environment cleanup preserves requirements. Rust cleanup preserves tracked Cargo files.
- Cache raw puzzle HTML canonically and materialize disposable Markdown for editor/CLI use. Normal cache cleaning never deletes examples; example/comprehensive cleanup is explicit and confirmed by the frontend.
- SQLite stores cache metadata, calendar-derived stars, correct/incorrect submission counts, and the latest configurable per-part runtimes (default 10). Store no answers, answer hashes, cooldowns, private leaderboard data, or detailed submission events.

## API And Process Rules

- Path/status getters are pure. Use explicit verbs such as `ensure`, `load`, `refresh`, `activate`, `regenerate`, and `clean` for mutation. Remove side-effecting patterns such as `AocContentFile::to_path()`.
- Prefer structurally valid request enums over optional day/boolean combinations. Destructive services receive typed already-confirmed scopes and return idempotent reports; they never prompt or accept `force`.
- Libraries do not print subprocess output. Return status, stdout, stderr, semantic data, and contextual errors for CLI/TUI rendering.
- Route Git, Cargo, Rust/Python solvers, pip, editors, and browsers through `aocsuite-utils::ProcessExecutor`. Captured execution is the default; foreground terminal inheritance is explicit.
- Serialize every language job spanning active-link mutation, harness migration, environment setup, build, execution, result consumption, and timing persistence.
- Move Git from the private CLI module into `aocsuite-storage::workspace`. Captured Git disables pagers/prompts; pass-through arguments are not a security sandbox.
- Move browser launching out of `aocsuite-client` and into launcher. The TUI owns terminal suspension/restoration around foreground launches.

## Current Implementation Hazards

- The current root is still `$XDG_DATA_HOME/aocsuite`, falling back to `$HOME/.local/share/aocsuite`. The README's `.local/data` path is stale. Use temporary explicit roots or `XDG_DATA_HOME` for isolated manual tests until injected layout APIs exist.
- Current config precedence is explicit override, then `<runtime-root>/config.json`, then `AOC_*`, then caller default. Rust does not load `.envrc` or dotenv files.
- Never run or log `config get session`, and avoid live submission/download verification. The current implementation still stores the session in `config.json`; the target moves it to `secrets/session`.
- Current `aocsuite-fs::AocContentFile::to_path()` may download, parse, write metadata, and mutate permissions. Current examples also live under disposable cache paths.
- Current client, language, and editor crates discover configuration internally. Current language and Git code independently discover runtime paths. Remove these hidden globals before introducing storage dependencies that could cycle.
- Current parser calendar output embeds ANSI, language result fields are not publicly inspectable, and language helpers may print. Do not scrape these outputs; fix the owning APIs.
- Current Git scope is the whole runtime root. Current target design scopes Git to lazy `workspace/` and regenerates its `.gitignore`.
- Current language runs use unique transient result files and activate the requested day/year, but active links remain shared mutable state. Keep activation/build/run serialized.
- Destructive CLI prompts intentionally accept an empty line as yes and reject EOF. Preserve this frontend behavior.

## Verification

- Workspace build: `cargo check --workspace`
- Workspace tests: `cargo test --workspace`
- Focused crate/test: `cargo test -p <crate> [test-filter]`; focused compile: `cargo check -p <crate>`
- CLI smoke test: `cargo run -p aocsuite-cli -- --help`
- Deterministic tests must use explicit temporary roots and fake clock/environment/process/HTTP seams. Do not launch real Git, Cargo, Python, pip, editors, browsers, or AoC requests in normal test coverage.
- Baseline GitHub CI runs locked workspace check/test and the CLI help smoke test. Formatting and strict workspace Clippy pass locally but are not yet required CI jobs.

## CI And Releases

- Follow `docs/CI.md`. Introduce baseline GitHub CI first, then make the Ubuntu/Windows/macOS matrix required only after deterministic fake process/environment/HTTP seams replace real external commands in normal tests.
- CI and release builds use `--locked`, no AoC session, no live AoC requests, no real editor/browser, and explicit temporary roots. TUI tests use `ratatui::TestBackend`, not a real terminal.
- Add formatting, strict Clippy, and rustdoc as required checks only after their recorded existing failures are fixed.
- Release tags are `vMAJOR.MINOR.PATCH` and match synchronized workspace package versions. Release CLI binaries first; add TUI binaries to the same versioned release after parity.
- Initial release targets are Linux x86-64, Windows x86-64, and macOS ARM64. Build/test jobs are read-only; only the release upload job receives `contents: write`.
- Prefer explicit release workflows initially. Consider `cargo-dist`, attestations, signing/notarization, coverage, fuzzing, and MSRV checks only when their support policies are stable.

## Commits

- Do not commit unless explicitly requested.
- Use Conventional Commits.
- Keep commits focused on one logical change.
- Never amend or force-push without explicit permission.
