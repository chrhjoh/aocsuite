# Storage Design

## Operating Assumption

AoC Suite is a simple single-user application. Only one AoC Suite process is expected to use a runtime root at a time. Do not add cross-process locking, coordination, or concurrency handling for normal cache, workspace, language, or metadata operations.

The TUI still serializes its own blocking jobs so it remains responsive and does not start two solver runs at once.

## Ownership

Add a broad `aocsuite-storage` service crate. It replaces `aocsuite-fs` and owns the physical layout, layout bootstrap and versioning, future layout migrations, SQLite lifecycle, AoC content fetch/parse/cache policy, run-file allocation, workspace Git, uninstall, and typed cleanup scopes.

Keep strict internal layers even though they share one crate:

- Layout and database modules depend only on `aocsuite-utils` and persistence packages.
- The content module depends on layout/database plus `aocsuite-client` and `aocsuite-parser`.
- The workspace module depends on layout/database plus the shared process executor.
- Content, workspace, and database modules do not depend on one another through frontend types.

Other crates consume an injected `RuntimeLayout` or storage handle instead of independently appending paths to `get_aocsuite_dir()`:

- `aocsuite-utils` owns validated domain values and the injectable process executor.
- `aocsuite-config` owns typed non-secret configuration values, and session access.
- `aocsuite-client` owns configuration-independent blocking AoC HTTP transport.
- `aocsuite-parser` owns pure semantic parsing and Markdown conversion.
- `aocsuite-lang` owns language projects, generated harness contents, package commands, compilation, and execution.
- `aocsuite-launcher` owns editor/browser command resolution and external application launching.
- CLI and TUI frontends own prompts, confirmation, rendering, terminal handoff, and job scheduling.

`aocsuite-storage` may depend on client and parser APIs for its content service. It must not depend on configuration, language, launcher, CLI, or TUI crates. The client must not depend on configuration or storage; callers inject the optional resolved session.

No general operations crate is planned. CLI and TUI call the same typed domain services and duplicate only frontend concerns. Domain policy such as fetch validation, submission counting/invalidation, run timing, cleanup scopes, and Git root enforcement stays inside its owning service.

## Domain And Process Foundations

Add UI-neutral validated values under `aocsuite_utils::domain`, including `PuzzleDay`, `PuzzleYear`, `PuzzleId`, `PuzzlePart`, `PartSelection`, and `LanguageId`. Structural validation belongs in constructors; release availability remains a separate check. Remove Clap derives from shared values and convert CLI types at the frontend boundary.

`aocsuite-utils` also owns a synchronous injectable process executor with OS-native program, argument, path, and environment values. Process results retain status, stdout, and stderr. Captured execution is the default; foreground terminal inheritance is explicit. Storage Git, language, Cargo, Python, pip, launcher, and solver processes use this seam. Nonzero status is a completed result that the owning service contextualizes.

## Persistence Model

Use one hardened runtime root. Keep user work and complete language projects as Git-managed files, downloaded AoC content as disposable cache files, and transient results outside the Git workspace. Use bundled SQLite for cache metadata and the small amount of application state defined below.

```text
aocsuite/
  .aocsuite-layout.json      # Physical layout version
  config/
    config.json              # Typed non-secret configuration values
    session                  # Owner-only AoC session token
  cache/
    state.sqlite             # Metadata and bounded application state
    puzzles/                 # Disposable downloaded AoC content
    inputs/
    calendars/
  workspace/                 # The only Git-managed area
    .git/
    .gitignore
    .aocsuite-runs/          # Created by language result allocation; ignored by Git
    examples/                # Shared user-authored puzzle examples
    rust/                    # Complete portable Rust project
    python/                  # Complete portable Python project
```

Do not store source files, templates, libraries, Cargo files, Python environments, examples, cache body content, plaintext answers, or the AoC session in SQLite.

## Runtime Layout API

`aocsuite-storage` exposes public `get_aocsuite_dir()` and cloneable `RuntimeLayout::new(root_dir)`. The resolver treats `AOCSUITE_DATA_DIR` as a complete-root override, then falls back to `$XDG_DATA_HOME/aocsuite` and `$HOME/.local/share/aocsuite`. Application code resolves once and passes the path explicitly; storage tests construct layouts from temporary roots without changing process-global environment state.

Path getters are pure. They never create directories, fetch content, migrate data, initialize projects, or launch commands. Bootstrap and mutation are separate explicit operations.

The layout provides typed paths for at least:

- Layout manifest, configuration, session, cache root, and workspace paths.

`ContentStore` owns its cache keys, cache paths, and `cache/state.sqlite`. `Workspace` owns paths and creation under `workspace/examples`, language project paths, and lazy allocation under `workspace/.aocsuite-runs`. Database contents must never direct reads or deletion outside the cache root.

Every application invocation bootstraps and validates storage before reading configuration or constructing services, including creation of `workspace/`. Git clone runs into that bootstrapped directory. If CLI help/version must also bootstrap literally, use a non-exiting Clap parse flow rather than relying on `Parser::parse` to terminate first.

## Git Workspace

`workspace/` is one Git repository containing two independent, portable language projects and shared examples. Generated execution harnesses are tracked because a cloned repository should preserve source, dependencies, and project scaffolding. AoC Suite may document manual standalone execution, but only AoC Suite-managed activation and execution are supported.

```text
workspace/
  examples/
    year2024_day1.txt
  rust/
    .aocsuite-runtime.json
    Cargo.toml
    Cargo.lock
    template.rs
    solutions/
      year2024_day1.rs
    src/
      main.rs
      solution.rs            # Generated active link, ignored by Git
      helpers.rs
    target/                  # Ignored by Git
  python/
    .aocsuite-runtime.json
    requirements.txt
    main.py
    template.py
    solution.py              # Generated active link, ignored by Git
    solutions/
      year2024_day1.py
    helpers.py
    venv/                    # Ignored by Git
    __pycache__/             # Ignored by Git
```

The generated workspace `.gitignore` is strictly AoC Suite-owned and may be replaced during workspace setup or migration. It excludes at least:

```gitignore
rust/target/
rust/src/solution.rs
python/venv/
python/solution.py
/.aocsuite-runs/
**/__pycache__/
*.pyc
```

Track Rust `Cargo.toml`, Rust `Cargo.lock`, Python `requirements.txt`, generated harnesses, language runtime manifests, solutions, templates, libraries, and examples.

Storage owns Git command execution rooted at `workspace/`. Captured Git disables pagers and terminal prompts; foreground commands require explicit frontend terminal handoff. Bootstrap creates the workspace, and clone runs into it. Cloning into a nonempty workspace fails without modifying its contents.

## Generated Harnesses

Generated `main.rs` and `main.py` files are strictly AoC Suite-owned even though they are tracked in Git. Manual edits may be overwritten.

Each language project tracks `.aocsuite-runtime.json` with only its infrastructure version:

```json
{
  "infrastructure_version": 1
}
```

When the recorded version is older, the language-owned migration:

1. Atomically replaces all owned generated harness files.
2. Applies required scaffold changes without replacing user dependency declarations.
3. Writes the new manifest version only after the file updates succeed.

Migrations never replace solutions, templates, libraries, examples, `Cargo.toml`, `Cargo.lock`, or `requirements.txt`. No generated-file hashes or manual-modification detection are required.

The active solution link is disposable selection state. It is recreated before compilation or execution and is not tracked in Git.

All language jobs that may change the active link are serialized across activation, harness migration, setup, build, execution, result consumption, and timing persistence. The initial TUI uses one language-effect queue; direct callers must use the same high-level language job API instead of composing activation and execution primitives independently.

## Dependency Management

### Rust

`workspace/rust/Cargo.toml` is the actual project manifest. Uses `cargo` package mutation and scaffold updates so unknown sections, comments, profiles, and user dependencies survive. Required harness dependencies may be inserted or repaired semantically, but the entire manifest is never regenerated during a harness migration.

`Cargo.lock` is tracked because the solver is an executable project. Rust environment cleanup runs `cargo clean`; it does not remove either Cargo file.

### Python

`workspace/python/requirements.txt` is tracked and records the complete output of `pip freeze`.

Package addition and removal run pip first. After a successful mutation, `pip freeze` atomically replaces `requirements.txt`. If freezing fails, return an error rather than reporting the persisted dependency state as current.

Python setup creates an empty `requirements.txt` when absent, creates `venv/` when absent, and installs `-r requirements.txt`. Python environment cleanup removes only the virtual environment and generated Python caches. It preserves `requirements.txt`.

Migration of dependencies from the current unversioned layout is out of scope.

## Examples And Cleanup

One example file is shared between Rust and Python for each puzzle. Examples use the flat `workspace/examples/year{year}_day{day}.txt` shape, are user-owned and Git-managed, and are not cache content. Explicit `ensure_example` creation creates an empty file only when absent and never overwrites user content.

Cleanup scopes are explicit and idempotent:

- Normal cache cleaning removes only `cache/puzzles`, `cache/inputs`, and `cache/calendars`; it preserves `cache/state.sqlite`.
- Example cleaning removes examples only when explicitly requested.
- A comprehensive confirmed clean may include both cache and examples.
- Language cache cleaning removes Rust build output or Python bytecode caches.
- Environment cleaning removes disposable environment state but preserves dependency declarations.
- Missing cleanup targets are successful no-ops.

## Cache Content And Metadata

`cache/state.sqlite` indexes cache entries by content type, year, day, validated relative path, size, fetch time, HTTP validation data, and validity/error state. Cache files are canonical; their database rows are rebuildable indexes.

Puzzle HTML is the canonical downloaded body. Cache paths are flat within content-specific directories:

```text
cache/
  state.sqlite
  puzzles/
    year2024_day1.html
    year2024_day1.md
  inputs/
    year2024_day1.txt
  calendars/
    year2024.html
```

Store raw puzzle HTML plus disposable derived Markdown for editor and CLI workflows. Validate and convert the raw body before replacing a previously valid entry. If Markdown is missing after an interrupted write or parser upgrade, regenerate it from the cached HTML without another request. Calendar HTML and input text remain canonical raw bodies.

Cache writes follow this order:

1. Fetch and validate the response.
2. Atomically replace the cache body using a same-directory temporary file.
3. Record its size and fetch metadata.
4. Commit the SQLite metadata row.

Cache files without a valid SQLite metadata row are unmanaged disposable files. Do not scan or reconstruct metadata for them; fetch and validate a new body when content is needed, then publish its metadata row. If `cache/state.sqlite` is corrupt, return a typed error without modifying it.

Path lookup, status lookup, cached reads, fetch, refresh, invalidation, and cleanup are separate APIs. Asking for a cache path must not perform HTTP or filesystem mutation.

Downloaded AoC content is disposable. Cache cleaning does not remove configuration, secrets, examples, workspaces, or run history.

## SQLite State

The initial database stores only:

- Cache metadata.
- Submission attempt count by year, day, and part.
- The most recent solver runtimes by year, day, language, and part.

Calendar HTML remains the source of truth for current CLI completion rendering. Defer a typed cached calendar and persisted stars until TUI calendar update and rendering behavior is defined; it may then avoid reparsing HTML during TUI startup. Do not invent or store a duration from puzzle download to completion. A run requesting both parts records each reported part runtime independently.

Retain the latest 10 runtimes per year/day/language/part by default. The limit is a typed non-secret preference. Prune older entries in the same transaction that inserts a new timing.

Do not store submitted answers, answer hashes, cooldowns, private leaderboard data, or detailed submission events. Cooldown and broader history are deferred until the shared domain services are established.

Increment submission counts only for parsed correct and incorrect AoC outcomes. Rate limits, already-completed responses, authentication failures, transport failures, and unknown responses do not increment the count. Correct outcomes invalidate calendar content; correct part-one outcomes also invalidate puzzle content. This policy is implemented once in the shared storage content service.

If `cache/state.sqlite` fails its integrity check, return a typed corrupt-database error without modifying it. Never silently delete, replace, or quarantine a corrupt database.

## SQLite Versioning

Use `PRAGMA user_version`, foreign keys, and ordered embedded schema migrations. Run each schema upgrade in a transaction and reject databases newer than the binary supports.

`PRAGMA user_version` records only the SQLite schema version. `.aocsuite-layout.json` is authoritative for the physical runtime layout because the database is replaceable and a corrupt database must not change the filesystem compatibility contract:

```json
{
  "layout_version": 1,
  "created_by": "0.4.0"
}
```

Do not use the database schema version to select runtime-layout migrations.

## Configuration And Secrets

`config/config.json` contains typed non-secret configuration values and is written atomically. It includes the run-history limit with a default of 10. Configuration reads do not create files; initialization and writes are explicit.

The session token is stored separately at `<runtime-root>/config/session` and explicitly set to mode `0600` on Unix. Session reads do not create or modify the file. `AOC_SESSION` and other `AOC_*` configuration sources are not supported. Prompting belongs to the frontend, not `aocsuite-config`.

Remove the unused `template_dir` configuration and `AOC_TEMPLATE_DIR`; templates are tracked inside each language project. Configuration, language, client, and launcher services receive explicit paths/settings and do not independently discover global configuration.

## Launcher And Frontend Boundaries

`aocsuite-launcher` owns browser platform commands, exact configured editor executable resolution, typed launch requests, argument construction, and process result handling. Editors launch from supplied language project or workspace roots and inherit the normal environment. It does not read configuration, inspect storage, print output, or suspend terminals. CLI/TUI resolve the effective launcher setting and own terminal suspend/restore around foreground launches.

Libraries return presentation-neutral values. Parser output contains semantic calendar cells, stars, validated puzzle dates, and submission outcomes, while language execution returns public part results and command diagnostics. ANSI, emoji, box drawing, prompts, and user-facing prose remain frontend adapters.

Destructive service APIs accept typed, already-confirmed scopes and return idempotent reports. Storage owns cache/example/workspace/uninstall deletion safety; language owns template/library/build/environment cleanup. No service accepts a `force` flag or prompts.

## Initial Layout And Future Migration

There are no active users requiring migration from the current unversioned layout. Do not implement best-effort import, Git relocation, session extraction, or dependency extraction from it.

Bootstrap behavior is:

- Missing root: create layout version 1.
- Existing root without a manifest, including an empty root: reject without mutation.
- Current supported layout: open normally.
- Nonempty unversioned root: reject without mutation and provide manual-removal guidance.
- Newer layout version: reject without mutation.

Do not add an automatic or destructive legacy reset command.

Future migrations between versioned layouts use a short-lived sibling marker and an untouched timestamped sibling backup. Migration phases are idempotent and resumable from the backup after interruption. Retain the backup for manual recovery and never silently merge conflicting files.

## Packages

Use `rusqlite` with bundled SQLite for the synchronous database layer and `walkdir` for deterministic cache indexing. Continue using workspace `chrono`, `serde`, `serde_json`, and `thiserror`; `toml_edit` belongs in `aocsuite-lang` for tracked Cargo manifest updates. Use `tempfile` for isolated storage tests.

Do not add an async SQL client, a separate SQL migration framework, normal-operation file locks, answer-HMAC dependencies, or an archive format for migration backups.

## Deferred Features

Defer cooldown tracking, detailed run/submission history, answer retention, search indexes, private leaderboard persistence, and automatic legacy-root import. Reconsider them only after shared noninteractive operations and TUI requirements justify them.
