# AoC Suite Architecture

## Status and purpose

This document records settled target architecture. It describes ownership,
dependency direction, and cross-crate contracts.

It does not describe migration progress and does not authorize unrelated
refactoring. Pre-TUI migration status is tracked in
`../plans/pre-tui-refactor.md`; initial TUI scope and sequencing are tracked in
`../plans/tui-implementation.md`.

## Architectural principles

- The CLI and TUI are separate frontends over shared typed domain services.
- Frontends must not invoke one another or parse one another's output.
- Domain services receive runtime inputs and dependencies explicitly.
- Domain services do not prompt, print, render, or discover global state.
- Blocking work remains outside Ratatui update and render code.
- Policy belongs to the domain that owns the affected state or behavior.
- A thin frontend composition root may construct and coordinate services.
- The application currently assumes one process per runtime root.

## Terminology

### Frontend

CLI or TUI code responsible for:

- arguments, events, and interaction;
- prompts and confirmation;
- rendering and output formatting;
- terminal suspension and restoration;
- frontend-specific background job scheduling;
- serialization of language jobs within a runtime-root workspace;
- constructing and invoking domain services.

### Domain crate

A UI-neutral crate that owns a coherent area of policy. Domain crates receive
dependencies explicitly and return semantic values or typed errors.

### Composition root

Frontend startup code that resolves the runtime root and configuration,
constructs dependencies, and maps frontend requests to domain services.

## Crate boundaries

### `aocsuite-utils`

Owns small, dependency-light primitives used by multiple domains:

- validated UI-neutral puzzle and language values;
- release-date calculations;
- atomic filesystem primitives;
- clock, environment, and process seams where genuinely cross-domain;
- the shared synchronous `CommandExecutor`.

Code belongs here only when no domain crate can reasonably own it. Do not move
code here merely to avoid choosing an owner or to break a dependency cycle.

Shared values must not derive frontend-specific traits such as Clap traits.

### `aocsuite-storage`

Owns the lifecycle and integrity of persistent local state:

- runtime layout, bootstrap, versioning, and migration;
- SQLite-backed metadata;
- cached AoC content and cache lifecycle;
- the generated workspace;
- workspace Git operations;
- run allocation and retained runtime metadata;
- locally retained submission counts;
- cleanup and uninstall safety.

Storage is the local-state domain, not merely a filesystem adapter.

Storage does not own:

- configuration discovery;
- HTTP transport;
- HTML parsing;
- language compilation or execution;
- editor or browser launching;
- frontend interaction.

`ContentStore` receives an injected client and may coordinate it with parser
interfaces when an operation combines remote retrieval with persistent-state
updates.

Internal boundaries:

- layout and database modules do not perform HTTP requests or parsing;
- only the content module may use client and parser abstractions;
- storage does not depend on config, language, launcher, CLI, or TUI.

### `aocsuite-config`

Owns:

- typed non-secret settings;
- configuration precedence;
- persisted editor selection;
- the separate persisted session credential.

The session is never serialized into `config.json`.

Configuration receives explicit paths, performs no prompting, and does not
create files during reads.

Effective configuration precedence:

1. applicable frontend overrides;
2. persisted values in `<runtime-root>/config/config.json`;
3. effective defaults owned by `aocsuite-config`.

Dynamic environment values such as `$EDITOR` participate only where documented.
Other configuration sources are not supported.

### `aocsuite-client`

Owns blocking Advent of Code HTTP transport:

- URL construction;
- authentication;
- HTTP status and authorization behavior;
- timeouts and bounded retry policy;
- HTTP validators.

The client receives all runtime inputs explicitly, including an optional
session and request policy. It does not read config, storage, or environment
state.

Submissions are never retried automatically.

### `aocsuite-parser`

Owns pure, fallible transformations of AoC responses:

- puzzle content;
- calendar content;
- submission responses.

It returns semantic values such as calendar cells, stars, and submission
outcomes. ANSI, emoji, and frontend prose do not belong here.

### `aocsuite-lang`

Owns language-project and execution policy:

- complete tracked Rust and Python projects;
- versioned generated harnesses and runtime manifests;
- solutions, templates, libraries, and active links;
- Cargo and Python dependency mutation;
- compilation and execution;
- structured execution reports;
- language-specific cleanup.

It receives an explicit `Workspace`, settings, and executor. It does not read
configuration or discover the runtime root.

A language operation spanning migration, activation, environment setup,
compilation, execution, result consumption, or timing persistence is one
serialized job.

### `aocsuite-launcher`

Owns:

- editor and browser executable resolution;
- typed editor and browser process requests;
- process execution through the shared executor.

Editor launches receive an explicit project working directory and inherit the
normal environment unless requested otherwise.

Launcher does not perform configuration lookup, terminal suspension or
restoration, storage access, or rendering.

Exact configured executables are preserved. Do not introduce editor alias
translation unless the product behavior is explicitly changed.

### `aocsuite-cli` and `aocsuite-tui`

Own frontend concerns:

- Clap or event handling;
- prompts and password input;
- output and rendering;
- destructive confirmation;
- terminal lifecycle;
- frontend-specific scheduling.

The TUI should eventually expose the same user-facing AoC workflows as the CLI
unless a workflow is explicitly documented as CLI-only. Phased TUI plans may
defer workflows without making them CLI-only. Parity means equivalent domain
operations and outcomes, not identical interaction or presentation.

Until command metadata is centralized,
`aocsuite-cli/src/commands.rs` is the working inventory of user-facing command
leaves. Do not treat `aocsuite-cli/src/app.rs` as a reusable service API.

## Process execution

Git, Cargo, Rust and Python solvers, pip, editors, and browsers use
`aocsuite-utils::CommandExecutor`.

Captured execution is the default for library-facing operations. Foreground
terminal inheritance must be requested explicitly through a frontend-owned
path.

Libraries do not print subprocess output directly. They return structured
results or typed process requests. Git pass-through may request inherited
streams, but it is not a security sandbox.

## Mutation APIs

Path and status getters are pure.

Use explicit mutation verbs such as:

- `ensure`;
- `load`;
- `refresh`;
- `activate`;
- `regenerate`;
- `clean`;
- `uninstall`.

Prefer validated inputs over optional values or boolean combinations that
require validation within the function.

Destructive services receive typed, already-confirmed scopes and return
idempotent plans or reports. They do not prompt and do not accept a generic
`force` flag.

## Errors

Domain errors preserve:

- the attempted operation;
- relevant paths or commands;
- underlying source errors where applicable.

Frontends convert domain errors into user-facing prose. Sanitized external
output may be included where it is necessary for diagnosis, but credentials and
sensitive headers must never be exposed.

## Dependency direction

```text
aocsuite-cli ─┐
              ├──> config
aocsuite-tui ─┤
              ├──> storage ──> client
              │       └──────> parser
              ├──> language ─> utils
              └──> launcher ─> utils

config  ─────> utils, where needed
client  ─────> utils, where needed
storage ─────> utils
```

Do not add reverse dependencies from domain crates into frontends, from storage
into config or language, or from lower-level crates into runtime-root discovery.
