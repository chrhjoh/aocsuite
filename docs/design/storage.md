# AoC Suite Storage Design

## Status and purpose

This document records settled persistent-state design: runtime-root resolution,
layout ownership, canonical data, initialization, migrations, cleanup, and
uninstall safety.

Implementation status belongs in `plans/pre-tui-refactor.md`. Crate boundaries
belong in `architecture.md`.

## Goals

- Keep persistent user state recoverable and understandable.
- Distinguish canonical state from disposable derived artifacts.
- Make initialization and migration safe and idempotent.
- Avoid implicit config or environment discovery inside storage.
- Never claim ownership of unknown files.
- Keep generated language projects usable outside a running AoC Suite process.

## Runtime-root resolution

The frontend composition root resolves the complete runtime root in this order:

1. `AOCSUITE_DATA_DIR`, interpreted as the complete runtime root.
2. `$XDG_DATA_HOME/aocsuite`.
3. `$HOME/.local/share/aocsuite`.

The resolved root must be absolute and nonempty.

Storage receives the root explicitly through `RuntimeLayout`. Tests construct
layouts from explicit temporary roots and do not mutate process-global
environment state.

## Layout

```text
<runtime-root>/
├── .aocsuite-layout.json
├── config/
│   ├── config.json
│   └── session
├── cache/
│   ├── state.sqlite
│   ├── puzzles/
│   ├── inputs/
│   └── calendars/
└── workspace/
    ├── .git/
    ├── .gitignore
    ├── .aocsuite-runs/
    ├── examples/
    ├── rust/
    └── python/
```

`workspace/.aocsuite-runs/` is created on demand and ignored by Git.

## Ownership classes

### AoC Suite-owned

AoC Suite may regenerate these files completely:

- `.aocsuite-layout.json`;
- the workspace `.gitignore`;
- versioned language harnesses;
- `.aocsuite-runtime.json` manifests;
- derived puzzle Markdown;
- active solution links;
- transient run files.

Manual edits to generated harnesses or `.gitignore` are not preserved under the
current design.

### User-visible persistent state

These must not be silently overwritten outside a documented operation:

- solutions;
- templates;
- libraries;
- shared examples;
- tracked project manifests and dependency files.

### Unknown files

Unknown files are not assumed to belong to AoC Suite and are not removed by
normal cleanup or migration. they may be removed during full uninstall, as long as user explicitly agrees.

## Canonical and derived state

### Canonical

- raw puzzle HTML;
- puzzle inputs;
- SQLite metadata;
- tracked language-project files;
- solutions, templates, and libraries;
- shared examples;
- persisted non-secret configuration;
- the separate session credential.

### Derived

- puzzle Markdown;
- build artifacts;
- Python virtual environments;
- bytecode caches;
- active links;
- transient run files;
- other explicitly regenerable presentation artifacts.

Derived state may be regenerated or deleted without losing canonical data.

## Configuration and credentials

`config/config.json` stores non-secret settings only.

`config/session` stores the AoC session separately and uses mode `0600` on Unix.

The session must never be printed, logged, snapshotted, included in errors, or
returned by ordinary configuration-display operations.

Configuration reads do not create files.

## Initialization

Only commands requiring persistent runtime state validate or bootstrap the
layout. Pure informational behavior such as `--help` must not touch the runtime
root.

Bootstrap may create:

- the runtime root;
- the layout manifest;
- required directories;
- `workspace/`;
- the workspace Git repository;
- the generated workspace `.gitignore`.

Bootstrap is idempotent and distinguishes:

- absent root;
- empty unversioned root;
- nonempty unversioned root;
- recognized versioned root;
- newer or unknown versioned root;
- incomplete recognized layout.

Under the current policy, every existing unversioned root, including an empty
one, is rejected without mutation. A user must remove or relocate it before AoC
Suite initializes a versioned root. This strict rule avoids silently claiming
ownership of pre-existing directories.

Newer layout versions are rejected without mutation through a typed error.

## Versioning and migration

`.aocsuite-layout.json` records the runtime layout version.

Migrations must:

- validate the source version;
- reject unsupported newer versions;
- retain backups for canonical state that cannot be reconstructed;
- use atomic replacement where practical;
- be resumable or detect incomplete prior execution;
- update the version manifest only after success;
- avoid modifying unknown files.

Generated harness migrations may atomically overwrite AoC Suite-owned harnesses
and then update their runtime manifest. Hashes and manual-edit detection are not
required under the current ownership policy.

## Database

`cache/state.sqlite` stores:

- schema and cache metadata;
- correct and incorrect submission counts;
- the latest configurable number of runtimes per puzzle part.

The default runtime retention count is 10.

It does not store:

- submitted answers;
- answer hashes;
- cooldown state;
- private leaderboard data;
- complete submission events;
- typed calendar state or derived stars until the TUI calendar contract is
  defined.

Database bootstrap must provide:

- integrity checks;
- typed corrupt-database errors;
- transactional schema upgrades;
- rejection of newer unsupported schemas.

Schema changes require an explicit migration.

## Content lifecycle

`ContentStore` owns:

- raw puzzle HTML;
- derived puzzle Markdown;
- puzzle inputs;
- calendar cache files;
- cache metadata;
- submission invalidation;
- input permissions;
- typed cache cleanup.

Files use flat date-keyed paths under:

- `cache/puzzles`;
- `cache/inputs`;
- `cache/calendars`.

Raw puzzle HTML is canonical. Markdown is derived.

Successful HTTP status alone is insufficient to replace valid cached content.
Puzzle bodies are semantically validated before replacement. Invalid or
unexpected bodies preserve existing valid cache entries.

Cache files without valid metadata remain unmanaged. Storage does not
reconstruct ownership merely from their filenames.

Expose pure path, status, and read operations separately from explicit load,
fetch, refresh, invalidate, and clean operations.

## Workspace

`workspace/rust` and `workspace/python` are complete projects that remain usable
outside the running AoC Suite process.

Persist:

- project manifests;
- lock or requirements files;
- generated harnesses and runtime manifests;
- solutions;
- templates;
- libraries;
- shared examples.

Do not persist machine-specific build products or virtual environments as
tracked state.

Solutions use flat paths:

```text
solutions/year{year}_day{day}
```

Examples use:

```text
workspace/examples/year{year}_day{day}.txt
```

## Workspace Git

Storage owns Git operations scoped to `workspace/`.

Captured Git disables pagers and interactive prompts. Explicit pass-through may
inherit terminal streams, but it is not a security sandbox.

The workspace `.gitignore` is AoC Suite-owned and regenerated completely. It
ignores only disposable state, including:

- Rust `target/`;
- Python virtual environments;
- Python bytecode caches;
- active solution links;
- `.aocsuite-runs/`.

## Language-project persistence

Generated harnesses and runtime manifests are versioned and may be migrated
atomically.

Rust package operations preserve and update tracked `Cargo.toml` and
`Cargo.lock`.

Python package operations preserve `requirements.txt`. After a successful
package mutation, the resolved environment state is atomically persisted through
`pip freeze`.

Environment cleanup preserves tracked dependency files. General cleanup must not
delete user solutions, templates, libraries, or project manifests.

## Run allocation and timing

Each language run receives a unique transient result path. Result files are
atomically written, validated, consumed, and cleared.

Active links remain shared mutable workspace state. Operations spanning
activation, harness migration, environment setup, build, execution, result
consumption, and timing persistence must be serialized.

Storage retains only the latest configured number of runtimes per puzzle part.

## Cleanup and uninstall

Cleanup is modeled through typed scopes and produces idempotent plans or reports.
Storage does not prompt and does not accept a generic `force` boolean.

Normal cache cleanup removes disposable content and preserves:

- `state.sqlite`;
- shared examples;
- language projects;
- configuration and credentials.

Example cleanup is an explicit scope. Comprehensive cleanup and uninstall are
separate explicit scopes confirmed by a frontend.

Before removal, uninstall must distinguish AoC Suite-owned paths from unknown or
user-owned files and refuse unsafe deletion.
